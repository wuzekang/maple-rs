use std::cell::RefCell;
use std::rc::Rc;

use super::runtime::{self, Effect, RUNTIME};
pub use super::runtime::ScopeId;

/// Create a new reactive effect
pub fn create_effect<F: FnMut() + 'static>(f: F) -> Effect {
    let callback = Rc::new(RefCell::new(Box::new(f) as Box<dyn FnMut()>));

    // 1. Create scope
    let (parent, scope_id) = RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        let parent = state.current_scope_id();
        let id = state.create_scope("effect", parent);
        (parent, id)
    });

    let effect = Effect {
        callback: callback.clone(),
        scope_id,
    };

    let rc = Rc::new(RefCell::new(effect.clone()));

    // 2. Register effect in runtime and parent
    RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        
        if let Some(parent) = parent {
            if let Some(Some(scope)) = state.scopes.get_mut(&parent) {
                scope.effects.push(rc.clone());
            }
        }
        
        state.effects.push(rc.clone());
        
        // Prepare for initial run
        state.effect_depth += 1;
        state.scope_stack.push(scope_id);
    });

    // 3. Initial run
    (callback.borrow_mut())();

    // 4. Cleanup after run
    RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        state.effect_depth -= 1;
        state.scope_stack.pop();
        state.effects.pop();
    });
    
    // 5. Flush any effects triggered by initial run
    runtime::flush_pending_effects();

    effect
}

/// Create a new scope
pub fn create_scope<F: FnOnce() + 'static>(f: F) {
    let id = RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        let parent = state.current_scope_id();
        let id = state.create_scope("scope", parent);
        state.scope_stack.push(id);
        id
    });

    f();

    RUNTIME.with(|runtime| {
        let mut state = runtime.borrow_mut();
        state.remove_scope(id);
        state.scope_stack.pop();
    });
    
    // Process the removal
    runtime::flush_pending_effects();
}

/// Create a child scope wrapper
pub fn as_child_scope<F, Args, R>(f: F) -> impl Fn(Args) -> (R, ScopeId)
where
    F: Fn(Args) -> R + 'static,
    Args: 'static,
    R: 'static,
{
    let current = RUNTIME.with(|runtime| {
        runtime.borrow().current_scope_id().expect("as_child_scope must be called within a scope")
    });

    move |args: Args| -> (R, ScopeId) {
        let id = RUNTIME.with(|runtime| {
            let mut state = runtime.borrow_mut();
            let id = state.create_scope("child_scope", Some(current));
            state.scope_stack.push(id);
            id
        });

        let result = f(args);

        RUNTIME.with(|runtime| {
            runtime.borrow_mut().scope_stack.pop();
        });

        (result, id)
    }
}
