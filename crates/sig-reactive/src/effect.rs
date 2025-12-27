use generational_box::{AnyStorage, Owner, UnsyncStorage};
use slotmap::{SlotMap, new_key_type};
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::runtime::RUNTIME;
use super::signal::SignalId;

// Define SlotMap keys for subscribers
new_key_type! {
  /// Key for effect subscriptions in the subscriber SlotMap
  pub struct SubscriberKey;
}

type EffectCallback = Rc<RefCell<Box<dyn FnMut()>>>;

// ============================================================================
// ScopeId - Scope identifier
// ============================================================================

/// Global Scope ID counter (starts from 1, 0 is reserved for ROOT)
static SCOPE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Scope unique identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(pub usize);

impl ScopeId {
  pub const ROOT: ScopeId = ScopeId(1);
}

// ============================================================================
// ReactiveState - Reactive state management
// ============================================================================

pub struct Runtime {
  pub scopes: RefCell<HashMap<ScopeId, Option<Scope>>>,
  pub scope_stack: RefCell<Vec<ScopeId>>,
  pub effects: RefCell<Vec<Rc<RefCell<Effect>>>>,

  // Signal ID → SlotMap of Effect subscribers (improves insert/remove performance)
  pub(crate) signal_subscribers:
    RefCell<HashMap<SignalId, SlotMap<SubscriberKey, Weak<RefCell<Effect>>>>>,
  // Effect ID (ScopeId) → Set of (SignalId, SubscriberKey) pairs for fast cleanup
  pub(crate) effect_observers: RefCell<HashMap<ScopeId, Vec<(SignalId, SubscriberKey)>>>,
  
  // Pending effects queue for deferred execution
  pub(crate) pending_effects: RefCell<Vec<Rc<RefCell<Effect>>>>,
  // Effect execution depth (to detect when to flush)
  pub(crate) effect_depth: RefCell<usize>,
  // Track which effects are already in the pending queue (for deduplication)
  pub(crate) pending_effect_ids: RefCell<HashSet<ScopeId>>,
}

impl Drop for Runtime {
  fn drop(&mut self) {
    let scopes = std::mem::take(&mut self.scopes);
    std::mem::forget(scopes);
  }
}

impl Runtime {
  pub fn new() -> Self {
    let state = Runtime {
      scopes: RefCell::new(HashMap::new()),
      scope_stack: RefCell::new(Vec::new()),
      effects: RefCell::new(Vec::new()),
      signal_subscribers: RefCell::new(HashMap::new()),
      effect_observers: RefCell::new(HashMap::new()),
      pending_effects: RefCell::new(Vec::new()),
      effect_depth: RefCell::new(0),
      pending_effect_ids: RefCell::new(HashSet::new()),
    };

    let global = state.create_scope("global", None);
    if global.0 == 1 {
      assert_eq!(global, ScopeId::ROOT);
    }

    state.scope_stack.borrow_mut().push(global);

    state
  }

  /// Create new scope
  pub fn create_scope(&self, name: &'static str, parent_id: Option<ScopeId>) -> ScopeId {
    let mut scopes = self.scopes.borrow_mut();

    let id = ScopeId(SCOPE_ID_COUNTER.fetch_add(1, Ordering::SeqCst));

    let height = parent_id
      .and_then(|pid| scopes.get(&pid).and_then(|s| s.as_ref()))
      .map(|p| p.height + 1)
      .unwrap_or(0);

    let scope = Scope::new(name, id, parent_id, height);

    scopes.insert(id, Some(scope));

    // 🔥 Register this scope as a child of its parent (for efficient cleanup)
    if let Some(parent_id) = parent_id {
      if let Some(Some(parent)) = scopes.get(&parent_id) {
        parent.children.borrow_mut().push(id);
      }
    }

    id
  }

  pub fn remove_scope(&self, id: ScopeId) {
    // Get child scopes from the cached children list
    let children: Vec<ScopeId> = self
      .scopes
      .borrow()
      .get(&id)
      .and_then(|s| s.as_ref())
      .map(|scope| scope.children.borrow().clone())
      .unwrap_or_default();

    // Recursively remove all child scopes first
    for child in children {
      self.remove_scope(child);
    }

    // Extract cleanup functions WITHOUT removing the scope yet
    let cleanups = self
      .scopes
      .borrow()
      .get(&id)
      .and_then(|s| s.as_ref())
      .map(|scope| {
        let mut vec = scope.cleanups.take();
        vec.reverse(); // LIFO order
        vec
      })
      .unwrap_or_default();

    // Execute cleanup functions while scope is still in stack
    for cleanup in cleanups {
      cleanup();
    }

    // Remove the scope from the map
    if let Some(scope) = self.scopes.borrow_mut().remove(&id) {
      drop(scope);
    }
  }

  /// Get immutable reference to scope
  pub fn get_scope(&self, id: ScopeId) -> Option<Ref<'_, Scope>> {
    Ref::filter_map(self.scopes.borrow(), |scopes| {
      scopes.get(&id).and_then(|s| s.as_ref())
    })
    .ok()
  }

  /// Clear all subscriptions for an effect (called before effect reruns)
  pub fn clear_effect_subscriptions(&self, effect_id: ScopeId) {
    let observers = {
      let mut effect_observers = self.effect_observers.borrow_mut();
      effect_observers.remove(&effect_id).unwrap_or_default()
    };

    let mut signal_subscribers = self.signal_subscribers.borrow_mut();
    for (signal_id, subscriber_key) in observers {
      if let Some(subs) = signal_subscribers.get_mut(&signal_id) {
        subs.remove(subscriber_key);
      }
    }
  }

  /// Establish subscription relationship (called when Signal is read)
  pub fn subscribe(&self, signal_id: SignalId, effect_rc: Rc<RefCell<Effect>>) {
    let effect_id = effect_rc.borrow().scope_id;

    // Signal → Effect: Insert into SlotMap and get key
    let subscriber_key = self
      .signal_subscribers
      .borrow_mut()
      .entry(signal_id)
      .or_insert_with(|| SlotMap::with_key())
      .insert(Rc::downgrade(&effect_rc));

    // Effect → Signal: Store (SignalId, SubscriberKey) pair for O(1) removal
    self
      .effect_observers
      .borrow_mut()
      .entry(effect_id)
      .or_insert_with(Vec::new)
      .push((signal_id, subscriber_key));
  }

  /// Get all Effects subscribed to a Signal
  pub fn get_signal_subscribers(&self, signal_id: SignalId) -> Vec<Rc<RefCell<Effect>>> {
    self
      .signal_subscribers
      .borrow()
      .get(&signal_id)
      .map(|subs| subs.values().filter_map(|weak| weak.upgrade()).collect())
      .unwrap_or_default()
  }

  /// Get current scope ID
  pub fn current_scope_id(&self) -> Option<ScopeId> {
    self.scope_stack.borrow().last().copied()
  }

  pub fn with_scope<O>(&self, id: ScopeId, f: impl FnOnce() -> O) -> O {
    self.scope_stack.borrow_mut().push(id);
    let result = f();
    self.scope_stack.borrow_mut().pop();
    result
  }

  /// Execute function with current scope reference
  pub(crate) fn with_current_scope<T>(&self, f: impl FnOnce(&Scope) -> T) -> Option<T> {
    self
      .current_scope_id()
      .and_then(|id| self.get_scope(id))
      .map(|scope| f(&scope))
  }

  /// Register a cleanup function for the current scope
  pub fn on_cleanup<F: FnOnce() + 'static>(&self, f: F) -> bool {
    self.with_current_scope(|scope| {
      scope.on_cleanup(f);
    }).is_some()
  }

  /// Flush all pending effects (unified logic)
  pub(crate) fn flush_pending_effects(&self) {
    loop {
      let pending_effects = std::mem::take(&mut *self.pending_effects.borrow_mut());
      if pending_effects.is_empty() {
        break;
      }

      self.pending_effect_ids.borrow_mut().clear();

      for effect in pending_effects {
        Effect::run_with_rc(&effect);
      }
    }
  }

  /// Queue effect with deduplication
  pub(crate) fn queue_effect(&self, effect_rc: Rc<RefCell<Effect>>) {
    let effect_id = effect_rc.borrow().scope_id;
    if self.pending_effect_ids.borrow_mut().insert(effect_id) {
      self.pending_effects.borrow_mut().push(effect_rc);
    }
  }
}

impl Default for Runtime {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// Scope - Scope management
// ============================================================================

pub struct Scope {
  pub name: &'static str,
  pub id: ScopeId,
  pub parent_id: Option<ScopeId>,
  pub height: u32,
  pub(crate) owner: Owner<UnsyncStorage>,
  pub(crate) effects: RefCell<Vec<Rc<RefCell<Effect>>>>,
  pub(crate) contexts: RefCell<Vec<Box<dyn Any>>>,
  pub(crate) cleanups: RefCell<Vec<Box<dyn FnOnce()>>>,
  pub(crate) children: RefCell<Vec<ScopeId>>,
}

impl Scope {
  pub fn new(name: &'static str, id: ScopeId, parent_id: Option<ScopeId>, height: u32) -> Self {
    Self {
      name,
      id,
      parent_id,
      height,
      owner: UnsyncStorage::owner(),
      effects: RefCell::new(Vec::new()),
      contexts: RefCell::new(Vec::new()),
      cleanups: RefCell::new(Vec::new()),
      children: RefCell::new(Vec::new()),
    }
  }

  /// Register a cleanup function to run when this scope is destroyed
  pub fn on_cleanup<F: FnOnce() + 'static>(&self, f: F) {
    self.cleanups.borrow_mut().push(Box::new(f));
  }

  /// Get this scope's owner
  pub fn owner(&self) -> &Owner<UnsyncStorage> {
    &self.owner
  }

  pub fn has_context<T: 'static + Clone>(&self) -> Option<T> {
    self.contexts
      .borrow()
      .iter()
      .find_map(|any| any.downcast_ref::<T>())
      .cloned()
  }

  /// Look up context from parent scopes
  pub fn consume_context<T: 'static + Clone>(&self) -> Option<T> {
    if let Some(ctx) = self.has_context::<T>() {
      return Some(ctx);
    }

    let mut search_parent = self.parent_id;

    RUNTIME.with(|state| {
      while let Some(parent_id) = search_parent {
        let scopes = state.scopes.borrow();
        let parent = scopes.get(&parent_id)?.as_ref()?;

        if let Some(ctx) = parent.has_context::<T>() {
          return Some(ctx);
        }

        search_parent = parent.parent_id;
      }
      None
    })
  }

  pub fn provide_context<T: 'static + Clone>(&self, value: T) -> T {
    let mut contexts = self.contexts.borrow_mut();

    // Replace if exists
    for ctx in contexts.iter_mut() {
      if let Some(existing) = ctx.downcast_mut::<T>() {
        *existing = value.clone();
        return value;
      }
    }

    contexts.push(Box::new(value.clone()));
    value
  }
}

// ============================================================================
// Effect - Side effect management
// ============================================================================

#[derive(Clone)]
pub struct Effect {
  pub(crate) callback: EffectCallback,
  pub(crate) scope_id: ScopeId,
}

impl Effect {
  pub fn run_with_rc(effect_rc: &Rc<RefCell<Effect>>) {
    let scope_id = effect_rc.borrow().scope_id;

    // Run previous cleanup functions and clear them before re-running effect
    let cleanups = RUNTIME.with(|state| {
      state
        .get_scope(scope_id)
        .map(|scope| {
          let mut vec = scope.cleanups.borrow_mut().drain(..).collect::<Vec<_>>();
          vec.reverse(); // LIFO order
          vec
        })
        .unwrap_or_default()
    });

    // Execute previous cleanup functions immediately
    for cleanup in cleanups {
      cleanup();
    }

    RUNTIME.with(|state| {
      state.clear_effect_subscriptions(scope_id);
      state.scope_stack.borrow_mut().push(scope_id);
      state.effects.borrow_mut().push(effect_rc.clone());
      *state.effect_depth.borrow_mut() += 1;
    });

    // Execute callback
    (effect_rc.borrow().callback.borrow_mut())();

    RUNTIME.with(|state| {
      *state.effect_depth.borrow_mut() -= 1;
      state.scope_stack.borrow_mut().pop();
      state.effects.borrow_mut().pop();
    });
  }
}

impl Drop for Effect {
  fn drop(&mut self) {
    // Scope cleanup is managed by owner's lifetime
  }
}

// ============================================================================
// Public API
// ============================================================================

pub fn create_effect<F: FnMut() + 'static>(f: F) -> Effect {
  let callback = Rc::new(RefCell::new(Box::new(f) as Box<dyn FnMut()>));

  let (parent, scope_id) = {
    let parent = RUNTIME.with(|state| state.current_scope_id());
    let id = RUNTIME.with(|state| state.create_scope("effect", parent));
    (parent, id)
  };

  let effect = Effect {
    callback: callback.clone(),
    scope_id,
  };

  let rc = Rc::new(RefCell::new(effect.clone()));

  if let Some(parent) = parent {
    RUNTIME.with(|state| {
      if let Some(scope) = state.get_scope(parent) {
        scope.effects.borrow_mut().push(rc.clone());
      }
    });
  }

  RUNTIME.with(|state| {
    state.effects.borrow_mut().push(rc.clone());
  });

  // Run effect to collect dependencies (initial execution)
  RUNTIME.with(|state| {
    state.clear_effect_subscriptions(scope_id);
    state.scope_stack.borrow_mut().push(scope_id);
    *state.effect_depth.borrow_mut() += 1;
  });

  (rc.borrow().callback.borrow_mut())();

  RUNTIME.with(|state| {
    *state.effect_depth.borrow_mut() -= 1;
    state.scope_stack.borrow_mut().pop();
    state.flush_pending_effects();
  });

  RUNTIME.with(|state| {
    state.effects.borrow_mut().pop();
  });

  effect
}

pub fn create_scope<F: FnOnce() + 'static>(f: F) {
  let id = RUNTIME.with(|state| {
    let parent = state.current_scope_id();
    let id = state.create_scope("scope", parent);
    state.scope_stack.borrow_mut().push(id);
    id
  });

  f();

  RUNTIME.with(|state| {
    state.remove_scope(id);
    state.scope_stack.borrow_mut().pop();
  });
}

/// Create a child scope version of a function.
/// Each call creates a new child scope that persists until manually cleaned up.
/// Returns (result, ScopeId) to control scope lifetime.
/// This is crucial for scenarios like `each` that need fine-grained scope lifetime control.
pub fn as_child_scope<F, Args, R>(f: F) -> impl Fn(Args) -> (R, ScopeId)
where
  F: Fn(Args) -> R + 'static,
  Args: 'static,
  R: 'static,
{
  let current = RUNTIME.with(|state| {
    state
      .current_scope_id()
      .expect("as_child_scope must be called within a scope")
  });

  move |args: Args| -> (R, ScopeId) {
    let id = RUNTIME.with(|state| {
      let id = state.create_scope("child_scope", Some(current));
      state.scope_stack.borrow_mut().push(id);
      id
    });

    let result = f(args);

    RUNTIME.with(|state| {
      state.scope_stack.borrow_mut().pop();
    });

    (result, id)
  }
}
