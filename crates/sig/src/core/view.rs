//! View components with unique IDs and child element management

use crate::{Node, Signal, ViewId, ViewTuple, create_effect};
use std::cell::RefCell;
use std::rc::Rc;

/// View - A component with a unique ID and children
///
/// Children are stored in a Signal and subscribed via create_effect
#[derive(Clone, Copy)]
pub struct View {
  pub id: ViewId,
  /// Child node list wrapped in Signal for reactive updates
  pub children: Signal<Vec<Node>>,
}

impl View {
  /// Create a new View
  pub fn new() -> Self {
    let id = ViewId::new();

    let nodes = Signal::new(Vec::new());

    // Reuse the same Vec across effect invocations to avoid repeated allocations
    let children = Rc::new(RefCell::new(Vec::new()));

    create_effect(move || {
      let mut children = children.borrow_mut();
      children.clear(); // Clear previous data
      flatten_all(&nodes.read(), &mut children);

      id.set_children(children.clone());

      // Request redraw since the subtree structure has changed
      crate::runtime::with_window(|window_state| {
        if let Some(window) = &window_state.redraw_requester {
          window.request_redraw();
        }
      });
    });

    View { id, children: nodes }
  }

  /// Set child elements
  pub fn child<VT: ViewTuple>(self, child: VT) -> Self {
    self.children.write().push(child.into_node());
    self
  }

  /// Set view name (chainable)
  pub fn name(self, name: impl Into<String>) -> Self {
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&self.id) {
        state.name = name.into();
      }
    });
    self
  }

  /// Get view name
  pub fn get_name(&self) -> String {
    crate::runtime::with_layout(|runtime| {
      runtime
        .view_states
        .get(&self.id)
        .map(|state| state.name.clone())
        .unwrap_or_else(|| String::from("View"))
    })
  }

  /// Check if this view is currently focused
  pub fn is_focused(&self) -> bool {
    self.id.is_focused()
  }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Flatten a single Node into a list of ViewIds
fn flatten(node: &Node, children: &mut Vec<ViewId>) {
  match node {
    Node::View(v) => children.push(*v),
    Node::Fragment(nodes) => {
      for node in nodes {
        flatten(node, children);
      }
    }
    Node::Dynamic(signal) => {
      // Try to read the signal - it might have been dropped if the scope was destroyed
      if let Some(nodes) = signal.try_read() {
        for (node, _) in nodes.iter() {
          flatten(node, children);
        }
      } else {
        // Signal was dropped - this can happen when a Dynamic is created in a scope
        // that gets destroyed before the View's effect runs
        // This is safe to ignore as the content should be cleaned up
        eprintln!("Warning: Attempted to read a dropped Dynamic signal in View::flatten");
      }
    }
  }
}

/// Flatten all children into a list of ViewIds
fn flatten_all(nodes: &[Node], children: &mut Vec<ViewId>) {
  for node in nodes {
    flatten(node, children);
  }
}

impl Default for View {
  fn default() -> Self {
    Self::new()
  }
}

/// Create a new View
pub fn view() -> View {
  View::new()
}

/// Implement Element trait for View
impl crate::Element for View {
  fn id(&self) -> ViewId {
    self.id
  }

  fn name(&self) -> String {
    self.get_name()
  }

}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::create_scope;

  #[test]
  fn test_view_creation() {
    create_scope(|| {
      let v = view();
      assert_eq!(v.get_name(), "View");
      assert_eq!(v.id.get_children().len(), 0);
    });
  }

  #[test]
  fn test_view_id_unique() {
    create_scope(|| {
      let v1 = view();
      let v2 = view();
      assert_ne!(v1.id.id(), v2.id.id());
    });
  }
}
