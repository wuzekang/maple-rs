// Context API for the reactive system

use crate::runtime::RUNTIME;

/// Provide a context value for the current scope and its children
pub fn provide_context<T: 'static + Clone>(value: T) {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().provide_context(value);
    });
}

/// Consume a context value from the current scope or its parents
pub fn consume_context<T: 'static + Clone>() -> Option<T> {
    RUNTIME.with(|runtime| {
        runtime.borrow().consume_context::<T>()
    })
}

/// Check if a context value exists in the current scope (doesn't search parents)
pub fn has_context<T: 'static + Clone>() -> bool {
    RUNTIME.with(|runtime| {
        let state = runtime.borrow();
        state.current_scope_id()
            .and_then(|id| state.scopes.get(&id))
            .and_then(|s| s.as_ref())
            .and_then(|s| s.has_context::<T>())
            .is_some()
    })
}
