//! Reactive system runtime - signal and effect management

use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

use generational_box::{AnyStorage, Owner, UnsyncStorage};
use slotmap::{SlotMap, new_key_type};

use super::signal::SignalId;

// ============================================================================
// Thread-local reactive state
// ============================================================================

thread_local! {
    pub(crate) static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

// ============================================================================
// Public API wrappers
// ============================================================================

/// Run a function without tracking signal dependencies
pub fn untrack<F, R>(f: F) -> R
where
  F: FnOnce() -> R,
{
  let saved_effects = RUNTIME.with(|runtime| {
    let mut state = runtime.borrow_mut();
    let saved = state.effects.clone();
    state.effects.clear();
    saved
  });

  let result = f();

  RUNTIME.with(|runtime| {
    runtime.borrow_mut().effects = saved_effects;
  });

  result
}

/// Register a cleanup function for the current scope
pub fn on_cleanup<F: FnOnce() + 'static>(f: F) -> bool {
  RUNTIME.with(|runtime| runtime.borrow_mut().on_cleanup(f))
}

pub fn current_effect() -> Option<Rc<RefCell<Effect>>> {
  RUNTIME.with(|runtime| runtime.borrow().effects.last().cloned())
}

pub fn push_effect(effect: Rc<RefCell<Effect>>) {
  RUNTIME.with(|runtime| runtime.borrow_mut().effects.push(effect));
}

pub fn pop_effect() {
  RUNTIME.with(|runtime| {
    runtime.borrow_mut().effects.pop();
  });
}

pub fn current_scope_id() -> Option<ScopeId> {
  RUNTIME.with(|runtime| runtime.borrow().current_scope_id())
}

pub fn current_owner() -> generational_box::Owner<generational_box::UnsyncStorage> {
  RUNTIME.with(|runtime| {
    let state = runtime.borrow();
    let scope_id = state
      .current_scope_id()
      .expect("No current scope. Use create_scope first.");

    state
      .scopes
      .get(&scope_id)
      .expect("Scope not found")
      .as_ref()
      .expect("Scope is None")
      .owner
      .clone()
  })
}

pub fn remove_scope(scope_id: ScopeId) {
  RUNTIME.with(|runtime| {
    runtime.borrow_mut().remove_scope(scope_id);
  });
}

// ============================================================================
// ScopeId
// ============================================================================

static SCOPE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(pub usize);

impl ScopeId {
  pub const ROOT: ScopeId = ScopeId(1);
}

// ============================================================================
// Subscriber Key
// ============================================================================

new_key_type! {
    pub struct SubscriberKey;
}

// ============================================================================
// Runtime
// ============================================================================

pub struct Runtime {
  pub scopes: HashMap<ScopeId, Option<Scope>>,
  pub scope_stack: Vec<ScopeId>,
  pub effects: Vec<Rc<RefCell<Effect>>>,

  // Signal ID -> SlotMap of Effect subscribers
  pub signal_subscribers: HashMap<SignalId, SlotMap<SubscriberKey, Weak<RefCell<Effect>>>>,
  // Effect ID -> Set of (SignalId, SubscriberKey) pairs
  pub effect_observers: HashMap<ScopeId, Vec<(SignalId, SubscriberKey)>>,

  // Pending effects queue (Strong references to guarantee execution)
  pub pending_effects: Vec<Rc<RefCell<Effect>>>,
  // Deduplication set for pending effects
  pub pending_effect_ids: HashSet<ScopeId>,
  // Effect execution depth
  pub effect_depth: usize,
  // Pending scope disposals
  pub pending_scope_disposals: Vec<ScopeId>,
}

impl Drop for Runtime {
  fn drop(&mut self) {
    let scopes = std::mem::take(&mut self.scopes);
    std::mem::forget(scopes);
  }
}

impl Runtime {
  pub fn new() -> Self {
    let mut state = Runtime {
      scopes: HashMap::new(),
      scope_stack: Vec::new(),
      effects: Vec::new(),
      signal_subscribers: HashMap::new(),
      effect_observers: HashMap::new(),
      pending_effects: Vec::new(),
      pending_effect_ids: HashSet::new(),
      effect_depth: 0,
      pending_scope_disposals: Vec::new(),
    };

    let global = state.create_scope("global", None);
    if global.0 == 1 {
      assert_eq!(global, ScopeId::ROOT);
    }

    state.scope_stack.push(global);
    state
  }

  pub fn create_scope(&mut self, name: &'static str, parent_id: Option<ScopeId>) -> ScopeId {
    let id = ScopeId(SCOPE_ID_COUNTER.fetch_add(1, Ordering::SeqCst));

    let height = parent_id
      .and_then(|pid| self.scopes.get(&pid).and_then(|s| s.as_ref()))
      .map(|p| p.height + 1)
      .unwrap_or(0);

    let scope = Scope::new(name, id, parent_id, height);
    self.scopes.insert(id, Some(scope));

    if let Some(parent_id) = parent_id {
      if let Some(Some(parent)) = self.scopes.get_mut(&parent_id) {
        parent.children.push(id);
      }
    }

    id
  }

  pub fn remove_scope(&mut self, id: ScopeId) {
    self.pending_scope_disposals.push(id);
  }
}

// Since we cannot implement complex disposal logic safely inside `&mut Runtime`,
// we'll move the "logic that runs user code" to free functions or `Runtime` methods that return actions.

impl Runtime {
  /// Collects cleanups and IDs for disposal without removing yet
  pub(crate) fn prepare_disposal(
    &mut self,
    id: ScopeId,
    out_cleanups: &mut Vec<Box<dyn FnOnce()>>,
    out_ids: &mut Vec<ScopeId>,
  ) {
    let children = self
      .scopes
      .get(&id)
      .and_then(|s| s.as_ref())
      .map(|scope| scope.children.clone())
      .unwrap_or_default();

    for child in children {
      self.prepare_disposal(child, out_cleanups, out_ids);
    }

    if let Some(Some(scope)) = self.scopes.get_mut(&id) {
      let mut scope_cleanups = std::mem::take(&mut scope.cleanups);
      scope_cleanups.reverse();
      out_cleanups.extend(scope_cleanups);
    }

    out_ids.push(id);
  }

  pub fn current_scope_id(&self) -> Option<ScopeId> {
    self.scope_stack.last().copied()
  }

  pub fn on_cleanup<F: FnOnce() + 'static>(&mut self, f: F) -> bool {
    if let Some(id) = self.current_scope_id() {
      if let Some(Some(scope)) = self.scopes.get_mut(&id) {
        scope.cleanups.push(Box::new(f));
        return true;
      }
    }
    false
  }

  pub fn subscribe(&mut self, signal_id: SignalId, effect_rc: Rc<RefCell<Effect>>) {
    let effect_id = effect_rc.borrow().scope_id;

    let subscriber_key = self
      .signal_subscribers
      .entry(signal_id)
      .or_insert_with(|| SlotMap::with_key())
      .insert(Rc::downgrade(&effect_rc));

    self
      .effect_observers
      .entry(effect_id)
      .or_insert_with(Vec::new)
      .push((signal_id, subscriber_key));
  }

  pub fn get_signal_subscribers(&self, signal_id: SignalId) -> Vec<Rc<RefCell<Effect>>> {
    self
      .signal_subscribers
      .get(&signal_id)
      .map(|subs| subs.values().filter_map(|weak| weak.upgrade()).collect())
      .unwrap_or_default()
  }

  pub fn queue_effect(&mut self, effect_rc: Rc<RefCell<Effect>>) {
    let effect_id = effect_rc.borrow().scope_id;
    if self.pending_effect_ids.insert(effect_id) {
      self.pending_effects.push(effect_rc);
    }
  }
}

// ============================================================================
// Effect
// ============================================================================

#[derive(Clone)]
pub struct Effect {
  pub(crate) callback: Rc<RefCell<Box<dyn FnMut()>>>,
  pub(crate) scope_id: ScopeId,
}

pub struct Scope {
  pub name: &'static str,
  pub id: ScopeId,
  pub parent_id: Option<ScopeId>,
  pub height: u32,
  pub(crate) owner: Owner<UnsyncStorage>,
  pub(crate) effects: Vec<Rc<RefCell<Effect>>>,
  pub(crate) contexts: Vec<Box<dyn Any>>,
  pub(crate) cleanups: Vec<Box<dyn FnOnce()>>,
  pub(crate) children: Vec<ScopeId>,
}

impl Scope {
  pub fn new(name: &'static str, id: ScopeId, parent_id: Option<ScopeId>, height: u32) -> Self {
    Self {
      name,
      id,
      parent_id,
      height,
      owner: UnsyncStorage::owner(),
      effects: Vec::new(),
      contexts: Vec::new(),
      cleanups: Vec::new(),
      children: Vec::new(),
    }
  }

  pub fn has_context<T: 'static + Clone>(&self) -> Option<T> {
    self
      .contexts
      .iter()
      .find_map(|any| any.downcast_ref::<T>())
      .cloned()
  }
}

impl Runtime {
  pub fn provide_context<T: 'static + Clone>(&mut self, value: T) -> T {
    if let Some(id) = self.current_scope_id() {
      if let Some(Some(scope)) = self.scopes.get_mut(&id) {
        // Replace if exists
        for ctx in scope.contexts.iter_mut() {
          if let Some(existing) = ctx.downcast_mut::<T>() {
            *existing = value.clone();
            return value;
          }
        }
        scope.contexts.push(Box::new(value.clone()));
        return value;
      }
    }
    value
  }

  pub fn consume_context<T: 'static + Clone>(&self) -> Option<T> {
    let mut search_id = self.current_scope_id();

    while let Some(id) = search_id {
      let scope = self.scopes.get(&id)?.as_ref()?;
      if let Some(ctx) = scope.has_context::<T>() {
        return Some(ctx);
      }
      search_id = scope.parent_id;
    }
    None
  }
}

// ============================================================================
// Internal Logic (Flush)
// ============================================================================

pub(crate) fn flush_pending_effects() {
  loop {
    let mut work_effects = Vec::new();
    let mut work_disposals = Vec::new();

    // 1. Extract work (holding lock)
    let has_work = RUNTIME.with(|runtime| {
      let mut state = runtime.borrow_mut();

      if !state.pending_effects.is_empty() {
        work_effects = std::mem::take(&mut state.pending_effects);
        state.pending_effect_ids.clear();
      }

      if !state.pending_scope_disposals.is_empty() {
        work_disposals = std::mem::take(&mut state.pending_scope_disposals);
      }

      !work_effects.is_empty() || !work_disposals.is_empty()
    });

    if !has_work {
      break;
    }

    // 2. Execute effects (NO lock)
    for effect in work_effects {
      Effect::run_with_rc(&effect);
    }

    // 3. Execute disposals (NO lock during execution)
    if !work_disposals.is_empty() {
      let mut all_cleanups = Vec::new();
      let mut ids_to_remove = Vec::new();

      // Collect cleanups and IDs (Holding lock)
      RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        for id in work_disposals {
          state.prepare_disposal(id, &mut all_cleanups, &mut ids_to_remove);
        }
      });

      // Run cleanups (NO lock, Scopes are still alive)
      for cleanup in all_cleanups {
        cleanup();
      }

      // Remove scopes (Holding lock)
      RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        for id in ids_to_remove {
          state.scopes.remove(&id);
        }
      });
    }
  }
}

impl Effect {
  pub fn run_with_rc(effect_rc: &Rc<RefCell<Effect>>) {
    let scope_id = effect_rc.borrow().scope_id;

    // 1. Run cleanups (must be done without holding RuntimeState lock if possible,
    // or we need to extract them first)
    let mut cleanups = Vec::new();

    RUNTIME.with(|runtime| {
      let mut state = runtime.borrow_mut();
      if let Some(Some(scope)) = state.scopes.get_mut(&scope_id) {
        // For effects, we often treat them as scopes.
        // But wait, Effect has a ScopeId? Yes.
        // The Effect IS a Scope (conceptually), or rather it HAS a Scope.
        // existing logic: `state.get_scope(scope_id).map(...)`
        // So yes, we extract cleanups from the scope associated with the effect.

        // NOTE: We only want to run cleanups that were registered IN the effect run.
        // The Scope associated with the Effect is reused?
        // `create_effect` -> `create_scope("effect")`
        // Yes, the scope persists across runs.

        cleanups = std::mem::take(&mut scope.cleanups);
        cleanups.reverse();
      }
    });

    for cleanup in cleanups {
      cleanup();
    }

    // 2. Prepare for run
    RUNTIME.with(|runtime| {
      let mut state = runtime.borrow_mut();

      // Clear old subscriptions
      if let Some(observers) = state.effect_observers.remove(&scope_id) {
        for (sig_id, key) in observers {
          if let Some(subs) = state.signal_subscribers.get_mut(&sig_id) {
            subs.remove(key);
          }
        }
      }

      state.scope_stack.push(scope_id);
      state.effects.push(effect_rc.clone());
      state.effect_depth += 1;
    });

    // 3. Execute callback
    (effect_rc.borrow().callback.borrow_mut())();

    // 4. Restore state
    RUNTIME.with(|runtime| {
      let mut state = runtime.borrow_mut();
      state.effect_depth -= 1;
      state.effects.pop();
      state.scope_stack.pop();
    });
  }
}
