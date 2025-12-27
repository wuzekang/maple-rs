//! Runtime subsystem management - domain-separated thread-local state
//!
//! This module re-exports runtime functions from their respective feature modules.
//! Each subsystem manages its own thread-local state:
//!
//! - `reactive` - Signal/Effect dependency tracking (from sig-reactive)
//! - `layout` - Taffy layout computation
//! - `event` - Input focus and IME state
//! - `app` - Window redraw and timing
//! - `render` - Render context

pub use crate::reactive::{current_owner, current_scope_id, on_cleanup, remove_scope, untrack};

// Re-export from feature modules
pub use crate::app::runtime::*;
pub use crate::event::runtime::*;
pub use crate::layout::runtime::*;
pub use crate::render::runtime::*;

// Legacy AppRuntime for backwards compatibility (can be deprecated later)
pub struct AppRuntime {
  _marker: (),
}

impl AppRuntime {
  pub fn new() -> Self {
    Self { _marker: () }
  }
}

impl Default for AppRuntime {
  fn default() -> Self {
    Self::new()
  }
}
