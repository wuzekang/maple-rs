//! Resource hook for async data fetching
//!
//! Automatically tracks reactive dependencies in the future factory function.
//! When any accessed Signal changes, the resource automatically restarts.
//!
//! ## Two Implementation Modes
//!
//! This module provides two ways to create resources:
//!
//! 1. **Standard mode** (`use_resource`): Requires reading signals before the async block
//! 2. **Reactive context mode** (`use_resource_with_tracking`): Allows reading signals inside async blocks
//!
//! The reactive context mode uses a polling wrapper to maintain reactive context during
//! Future execution, similar to Dioxus's implementation.

use sig_reactive::{Signal, create_effect, runtime::untrack, current_owner};
use crate::{spawn, Task};
use crate::reactive_context::ReactiveContextFuture;
use generational_box::GenerationalBox;
use std::pin::Pin;
use std::future::Future;

/// State of a resource
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceState {
    Pending,
    Ready,
    Paused,
    Stopped,
}

/// A resource that manages an async operation
pub struct Resource<T: 'static> {
    value: Signal<Option<T>>,
    state: Signal<ResourceState>,
    task: Signal<Option<Task>>,
    future_fn: GenerationalBox<Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T>>>>>,
}

// Now Resource is Copy!
impl<T: 'static> Copy for Resource<T> {}

impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Resource<T> {
    #[allow(dead_code)]
    fn new<F>(mut future_fn: impl FnMut() -> F + 'static) -> Self
    where
        F: std::future::Future<Output = T> + 'static,
    {
        let value = Signal::new(None);
        let state = Signal::new(ResourceState::Pending);
        let task_signal = Signal::new(None);
        
        // Store the future factory in current scope's owner
        let future_fn_box = current_owner().insert(
            Box::new(move || Box::pin(future_fn()) as Pin<Box<dyn Future<Output = T>>>)
                as Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T>>>>
        );
        
        let resource = Resource {
            value,
            state,
            task: task_signal,
            future_fn: future_fn_box,
        };
        
        // Don't start initial task here - let use_resource handle it via create_effect
        
        resource
    }
    
    /// Internal: run the task
    fn run_task(&self) {
        // Cancel old task (untracked - this is internal state)
        untrack(|| {
            if let Some(old_task) = *self.task.read() {
                old_task.cancel();
            }
        });
        
        // Set state to pending (untracked - this is internal state)
        untrack(|| *self.state.write() = ResourceState::Pending);
        
        // Generate future - this SHOULD track dependencies!
        let fut = (self.future_fn.write())();
        
        let value = self.value;
        let state = self.state;
        
        // Spawn task (untracked - this is internal state)
        let new_task = untrack(|| {
            spawn(async move {
                let result = fut.await;
                *value.write() = Some(result);
                *state.write() = ResourceState::Ready;
            })
        });
        
        untrack(|| *self.task.write() = Some(new_task));
    }
    
    pub fn value(&self) -> Option<T>
    where
        T: Clone,
    {
        (*self.value.read()).clone()
    }
    
    pub fn state(&self) -> ResourceState {
        *self.state.read()
    }
    
    pub fn pending(&self) -> bool {
        self.state() == ResourceState::Pending
    }
    
    pub fn ready(&self) -> bool {
        self.state() == ResourceState::Ready
    }
    
    pub fn restart(&self) {
        self.run_task();
    }
    
    pub fn cancel(&self) {
        if let Some(task) = *self.task.read() {
            task.cancel();
        }
        *self.task.write() = None;
        *self.state.write() = ResourceState::Stopped;
    }
    
    pub fn pause(&self) {
        if let Some(task) = *self.task.read() {
            task.pause();
            *self.state.write() = ResourceState::Paused;
        }
    }
    
    pub fn resume(&self) {
        if let Some(task) = *self.task.read() {
            task.resume();
            *self.state.write() = ResourceState::Pending;
        }
    }
    
    pub fn clear(&self) {
        *self.value.write() = None;
    }
}

/// Create a resource that runs an async operation
///
/// Automatically tracks reactive dependencies. When any Signal accessed
/// in the future factory function changes, the resource automatically restarts.
///
/// **Note**: With this implementation, you must read signals BEFORE the async block,
/// not inside it. If you need to read signals inside async blocks, use
/// `use_resource_with_tracking` instead.
///
/// # Example
/// ```
/// use sig_async::{use_resource, poll_tasks};
/// use sig_reactive::{create_scope, Signal};
///
/// create_scope(move || {
///     let user_id = Signal::new(1);
///     
///     // Automatically tracks user_id and restarts when it changes
///     let user_data = use_resource(move || {
///         let id = *user_id.read();  // Automatically tracked!
///         async move {
///             format!("User {}", id)
///         }
///     });
///     
///     poll_tasks();
///     assert_eq!(user_data.value(), Some("User 1".to_string()));
///     
///     // When user_id changes, the resource automatically restarts
///     *user_id.write() = 2;
///     poll_tasks();
///     assert_eq!(user_data.value(), Some("User 2".to_string()));
/// });
/// ```
pub fn use_resource<T, F>(mut future_fn: impl FnMut() -> F + 'static) -> Resource<T>
where
    T: 'static,
    F: std::future::Future<Output = T> + 'static,
{
    let value = Signal::new(None);
    let state = Signal::new(ResourceState::Pending);
    let task_signal = Signal::new(None);
    
    // Store the future factory in current scope's owner
    let future_fn_box = current_owner().insert(
        Box::new(move || Box::pin(future_fn()) as Pin<Box<dyn Future<Output = T>>>)
            as Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T>>>>
    );
    
    // Create the resource
    let resource = Resource {
        value,
        state,
        task: task_signal,
        future_fn: future_fn_box,
    };
    
    // Use create_effect to automatically track dependencies
    // The effect will call future_fn() which reads signals and establishes dependencies
    create_effect(move || {
        // Cancel old task (untracked - we don't want to subscribe to task_signal)
        untrack(|| {
            if let Some(old_task) = *task_signal.read() {
                old_task.cancel();
            }
        });
        
        // Set state (untracked - we don't want to subscribe to state)
        untrack(|| {
            *state.write() = ResourceState::Pending;
        });
        
        // Call future_fn HERE to establish dependencies
        // This is critical: future_fn() must run INSIDE the effect to track signals
        let fut = (future_fn_box.write())();
        
        // Spawn task and write to signals (untracked)
        untrack(|| {
            let new_task = crate::spawn(async move {
                let result = fut.await;
                *value.write() = Some(result);
                *state.write() = ResourceState::Ready;
            });
            
            *task_signal.write() = Some(new_task);
        });
    });
    
    resource
}

/// Create a resource with reactive context tracking (allows reading signals in async blocks)
///
/// This version allows you to read signals directly inside the async block.
/// It wraps the future to maintain effect context during polling.
///
/// # Example
/// ```
/// use sig_async::{use_resource_with_tracking, poll_tasks};
/// use sig_reactive::{create_scope, Signal};
///
/// create_scope(move || {
///     let user_id = Signal::new(1);
///     
///     // ✅ Can read signals inside the async block!
///     let user_data = use_resource_with_tracking(move || async move {
///         let id = *user_id.read();  // This works!
///         format!("User {}", id)
///     });
///     
///     poll_tasks();
///     assert_eq!(user_data.value(), Some("User 1".to_string()));
///     
///     *user_id.write() = 2;
///     poll_tasks();
///     assert_eq!(user_data.value(), Some("User 2".to_string()));
/// });
/// ```
pub fn use_resource_with_tracking<T, F>(mut future_fn: impl FnMut() -> F + 'static) -> Resource<T>
where
    T: 'static,
    F: std::future::Future<Output = T> + 'static,
{
    let value = Signal::new(None);
    let state = Signal::new(ResourceState::Pending);
    let task_signal = Signal::new(None);
    
    // We need to create a shared effect_rc that will be used during polling
    // This will be set up inside create_effect
    let effect_rc: std::rc::Rc<std::cell::RefCell<Option<std::rc::Rc<std::cell::RefCell<sig_reactive::Effect>>>>> = 
        std::rc::Rc::new(std::cell::RefCell::new(None));
    
    let effect_rc_for_future = effect_rc.clone();
    
    // Store the future factory in current scope's owner
    let future_fn_box = current_owner().insert(
        Box::new(move || {
            let fut = future_fn();
            
            // Wrap with ReactiveContextFuture if we have an effect_rc
            if let Some(ref eff_rc) = *effect_rc_for_future.borrow() {
                let ctx_fut = ReactiveContextFuture::new(fut, eff_rc.clone());
                Box::pin(ctx_fut) as Pin<Box<dyn Future<Output = T>>>
            } else {
                // Fallback: just box the future normally
                Box::pin(fut) as Pin<Box<dyn Future<Output = T>>>
            }
        }) as Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T>>>>
    );
    
    // Create the resource
    let resource = Resource {
        value,
        state,
        task: task_signal,
        future_fn: future_fn_box,
    };
    
    // Use create_effect to automatically track dependencies
    let _eff = create_effect(move || {
        // Get the current effect_rc from the runtime
        // This is the effect we're running inside right now!
        let current_effect_rc = sig_reactive::current_effect();
        
        // Store it so the future can use it during polling
        if let Some(ref eff_rc) = current_effect_rc {
            *effect_rc.borrow_mut() = Some(eff_rc.clone());
        }
        
        // Cancel old task (untracked)
        untrack(|| {
            if let Some(old_task) = *task_signal.read() {
                old_task.cancel();
            }
        });
        
        // Set state (untracked)
        untrack(|| {
            *state.write() = ResourceState::Pending;
        });
        
        // Call future_fn to create the future
        // The future is now wrapped with ReactiveContextFuture
        let fut = (future_fn_box.write())();
        
        // Spawn task (untracked)
        untrack(|| {
            let new_task = crate::spawn(async move {
                let result = fut.await;
                *value.write() = Some(result);
                *state.write() = ResourceState::Ready;
            });
            
            *task_signal.write() = Some(new_task);
        });
    });
    
    resource
}
