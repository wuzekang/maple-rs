//! sig-async - Async integration for sig-reactive
//!
//! This crate provides async runtime integration for the sig-reactive system,
//! including task spawning and resource management.

pub mod task_runtime;
pub mod tasks;
pub mod use_resource;
pub mod reactive_context;

// Re-export core types
pub use task_runtime::{
    EventLoopWaker, TaskRuntime, TaskMessage, 
    with_task_runtime, with_task_runtime_mut, 
    set_event_waker, wait_for_task_message,
};
pub use tasks::*;
pub use use_resource::*;
pub use reactive_context::*;

// Re-export ScopeId from sig-reactive for convenience
pub use sig_reactive::ScopeId;
