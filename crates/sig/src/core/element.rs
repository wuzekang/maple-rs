//! Element trait - Base interface for all components
//!
//! All components automatically gain Styleable and Interactive capabilities by implementing this trait

use crate::{Node, ViewId};

/// Base interface that all components must implement
///
/// By implementing this trait, components automatically gain:
/// 1. Styleable trait (style setting capabilities)
/// 2. Interactive trait (event handling capabilities)
///
/// Requires only two methods:
/// - `id()`: Returns the component's unique ViewId
/// - `name()`: Returns component name for debugging
pub trait Element: Sized {
  /// Get the component's unique ViewId
  fn id(&self) -> ViewId;

  /// Get component name for debugging (default returns "Element")
  fn name(&self) -> String {
    "Element".to_string()
  }

  fn build(self) -> Node {
    Node::View(self.id())
  }
}