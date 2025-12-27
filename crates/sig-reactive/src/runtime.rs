//! Reactive system runtime - signal and effect management

pub use super::{Effect, Runtime, SignalId};
use std::cell::RefCell;
use std::rc::Rc;

use super::ScopeId;

// ============================================================================
// Thread-local reactive state
// ============================================================================

thread_local! {
    pub(crate) static RUNTIME: Runtime = Runtime::new();
}

// ============================================================================
// Public API - Untrack dependencies
// ============================================================================

/// Run a function without tracking signal dependencies
///
/// This is useful when you want to read signals inside an effect but don't
/// want those reads to create new dependencies.
///
/// # Example
/// ```ignore
/// use sig::prelude::*;
///
/// create_effect(|| {
///     let value = untrack(|| *signal.read()); // Won't create dependency
///     // ...
/// });
/// ```
pub fn untrack<F, R>(f: F) -> R
where
  F: FnOnce() -> R,
{
  // Save current effect stack
  let saved_effects = RUNTIME.with(|state| {
    let mut effects = state.effects.borrow_mut();
    let saved = effects.clone();
    effects.clear();
    saved
  });

  // Run function without effects (no dependency tracking)
  let result = f();

  // Restore effect stack
  RUNTIME.with(|state| {
    *state.effects.borrow_mut() = saved_effects;
  });

  result
}

// ============================================================================
// Public API - Cleanup utilities
// ============================================================================

/// Register a cleanup function for the current scope
///
/// # Example
/// ```ignore
/// use sig::{create_scope, on_cleanup};
///
/// create_scope(|| {
///   let resource = acquire_resource();
///     on_cleanup(move || {
///         release_resource(resource);
///     });
/// });
/// ```
pub fn on_cleanup<F: FnOnce() + 'static>(f: F) -> bool {
  RUNTIME.with(|state| state.on_cleanup(f))
}

// ============================================================================
// Public API - Effect context management (for sig-async)
// ============================================================================

/// Get the current effect Rc (for async context tracking)
///
/// This is used by sig-async to maintain effect context during Future polling.
/// Returns None if not currently inside an effect.
pub fn current_effect() -> Option<Rc<RefCell<Effect>>> {
  RUNTIME.with(|state| state.effects.borrow().last().cloned())
}

/// Push an effect onto the effect stack
///
/// This is used by sig-async's ReactiveContextFuture to maintain effect context
/// during Future polling. Must be balanced with a call to pop_effect().
///
/// # Safety
/// Caller must ensure pop_effect() is called to restore the stack.
pub fn push_effect(effect: Rc<RefCell<Effect>>) {
  RUNTIME.with(|state| {
    state.effects.borrow_mut().push(effect);
  });
}

/// Pop an effect from the effect stack
///
/// This is used by sig-async's ReactiveContextFuture to restore effect context
/// after Future polling.
pub fn pop_effect() {
  RUNTIME.with(|state| {
    state.effects.borrow_mut().pop();
  });
}

/// Get the current scope ID
///
/// This is used by sig-async to determine which scope a task should belong to.
/// Returns None if not currently inside a scope (should not happen in normal usage).
pub fn current_scope_id() -> Option<ScopeId> {
  RUNTIME.with(|state| state.current_scope_id())
}

// ============================================================================
// Public API - Owner access (for generational-box storage)
// ============================================================================

/// Get the current scope's owner for allocating generational boxes
///
/// This allows you to store values in the current scope's generational-box storage.
/// The values will be automatically cleaned up when the scope is dropped.
///
/// # Example
/// ```ignore
/// use sig_reactive::current_owner;
/// use generational_box::GenerationalBox;
///
/// let owner = current_owner().expect("No current scope");
/// let my_box: GenerationalBox<MyData> = owner.insert(MyData::new());
/// // my_box is Copy and will be cleaned up when the scope ends
/// ```
///
/// # Panics
/// Panics if called outside of a scope.
pub fn current_owner() -> generational_box::Owner<generational_box::UnsyncStorage> {
  RUNTIME.with(|state| {
    let scope_id = state
      .current_scope_id()
      .expect("No current scope. Use create_scope first.");
    
    state
      .scopes
      .borrow()
      .get(&scope_id)
      .expect("Scope not found")
      .as_ref()
      .expect("Scope is None")
      .owner
      .clone()
  })
}

// ============================================================================
// Public API - Scope management
// ============================================================================

/// Remove a scope and all its children
///
/// This is useful for manually managing scope lifetimes, such as in dynamic
/// UI components that need to clean up old scopes when content changes.
///
/// # Warning
///
/// This function should be used carefully. In most cases, scopes are automatically
/// cleaned up when they go out of scope or when their parent is removed.
///
/// # Example
/// ```ignore
/// use sig_reactive::{as_child_scope, remove_scope};
///
/// let view_fn = as_child_scope(|()| {
///     // Create view...
/// });
///
/// let (view, scope_id) = view_fn(());
/// 
/// // Later, manually clean up the scope
/// remove_scope(scope_id);
/// ```
pub fn remove_scope(scope_id: ScopeId) {
  RUNTIME.with(|state| {
    state.remove_scope(scope_id);
  });
}
