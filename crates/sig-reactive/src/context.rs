// Context API for the reactive system

use crate::runtime::RUNTIME;
use crate::effect::Scope;

/// Helper function to execute a closure with the current scope
fn with_current_scope<T, F>(f: F) -> Option<T>
where
  F: FnOnce(&Scope) -> Option<T>
{
  RUNTIME.with(|state| {
    state
      .current_scope_id()
      .and_then(|scope_id| state.get_scope(scope_id))
      .and_then(|scope| f(&scope))
  })
}

/// Provide a context value for the current scope and its children
/// 
/// # Example
/// ```ignore
/// provide_context(MyConfig { value: 42 });
/// ```
pub fn provide_context<T: 'static + Clone>(value: T) {
  with_current_scope(|scope| {
    scope.provide_context(value.clone());
    Some(())
  });
}

/// Consume a context value from the current scope or its parents
/// 
/// # Example
/// ```ignore
/// if let Some(config) = consume_context::<MyConfig>() {
///     println!("Config value: {}", config.value);
/// }
/// ```
pub fn consume_context<T: 'static + Clone>() -> Option<T> {
  with_current_scope(|scope| scope.consume_context::<T>())
}

/// Check if a context value exists in the current scope (doesn't search parents)
/// 
/// # Example
/// ```ignore
/// if has_context::<MyConfig>() {
///     println!("Config is available!");
/// }
/// ```
pub fn has_context<T: 'static + Clone>() -> bool {
  with_current_scope(|scope| scope.has_context::<T>()).is_some()
}

