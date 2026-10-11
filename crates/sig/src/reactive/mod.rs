//! Reactive system for sig
//!
//! This module re-exports the sig-reactive and sig-async crates,
//! providing a unified reactive API for the sig UI framework.

// ============================================================================
// Re-export sig-reactive (core reactive primitives)
// ============================================================================

pub use sig_reactive::{
  Effect,
  ReadGuard,

  ScopeId,
  // Core types
  Signal,
  SignalId,
  WriteGuard,
  as_child_scope,

  consume_context,
  // Effect functions
  create_effect,
  create_scope,
  current_effect,
  current_scope_id,
  has_context,
  // Runtime functions
  on_cleanup,
  // Context API
  provide_context,
  remove_scope,

  untrack,
};

// ============================================================================
// Re-export sig-async (async integration)
// ============================================================================

pub use sig_async::{
  EventLoopWaker,
  // Reactive context
  ReactiveContextFuture,
  // Resource types
  Resource,
  ResourceState,

  // Task types
  Task,
  TaskMessage,

  TaskRuntime,

  current_task,
  parent_task,
  poll_tasks,
  set_event_waker,
  // Task functions
  spawn,
  // Resource functions
  use_resource,
  use_resource_with_tracking,

  wait_for_task_message,

  // Task runtime
  with_task_runtime,
  with_task_runtime_mut,
};

// ============================================================================
// Type aliases for compatibility
// ============================================================================
