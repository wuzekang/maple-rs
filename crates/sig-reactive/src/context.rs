//! Typed, scope-aware dependency injection — facade over `samara-signals`.

use samara_signals as samara;

/// Provide a context value for the current scope and its children.
pub fn provide_context<T: 'static>(value: T) {
  samara::provide_context(value);
}

/// Consume a context value from the current scope or any of its parents.
pub fn consume_context<T: 'static + Clone>() -> Option<T> {
  samara::use_context::<T>()
}

/// Check whether the current scope itself provides a context of this type
/// (does not search parents — sig's historical semantics).
pub fn has_context<T: 'static + Clone>() -> bool {
  samara::has_local_context::<T>()
}
