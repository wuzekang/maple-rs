//! sig-reactive - Fine-grained reactive system
//!
//! A facade over the [`samara-signals`](https://github.com/wuzekang/samara)
//! crate (alien-signals propagation: link/version-based dependency tracking,
//! glitch-free updates, scope ownership with explicit disposal). This crate
//! keeps sig's historical API — guard-style `Signal::read`/`write`,
//! `create_effect`, `create_scope`, scope ids and context injection — so the
//! UI framework is insulated from the engine underneath.

pub mod context;
pub mod effect;
pub mod runtime;
pub mod signal;

// Re-export core types
pub use effect::{create_effect, create_scope, as_child_scope};
pub use runtime::{Effect, ScopeId};
pub use signal::*;

// Re-export runtime functions
pub use runtime::{current_effect, current_scope_id, on_cleanup, remove_scope, untrack};

// Re-export context API
pub use context::{consume_context, has_context, provide_context};
