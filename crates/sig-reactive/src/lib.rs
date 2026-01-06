//! sig-reactive - Fine-grained reactive system
//!
//! This crate provides the core reactive primitives for the sig UI framework,
//! including signals, effects, and scopes.

pub mod context;
pub mod effect;
pub mod runtime;
pub mod signal;

// Re-export core types
pub use effect::{create_effect, create_scope, as_child_scope};
pub use runtime::{Effect, ScopeId, Scope, Runtime}; // ScopeId and Effect are useful types
pub use signal::*;

// Re-export runtime functions
pub use runtime::{
  current_effect, current_owner, current_scope_id, on_cleanup, pop_effect, push_effect,
  remove_scope, untrack,
};

// Re-export context API
pub use context::{consume_context, has_context, provide_context};
