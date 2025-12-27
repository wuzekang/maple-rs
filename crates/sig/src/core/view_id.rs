//! ViewId - View unique identifier
//!
//! Uses taffy NodeId as the underlying implementation

use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use taffy::{NodeId, Style};

/// Global view ID counter
static VIEW_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// ViewId - View unique identifier wrapping taffy NodeId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId {
  pub id: u64,
  pub node_id: NodeId,
}

impl Default for ViewId {
  fn default() -> Self {
    Self::new()
  }
}

impl ViewId {
  /// Create a new ViewId
  pub fn new() -> Self {
    let id = VIEW_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    let view_id = ViewId {
      id,
      node_id: crate::runtime::with_layout_mut(|runtime| {
        runtime.taffy.new_leaf(Style::DEFAULT).unwrap()
      }),
    };

    crate::runtime::with_layout_mut(|runtime| {
      let state = crate::ViewState::new();
      // 🔥 Performance optimization: Don't mark as dirty on creation
      // Only mark dirty when style() is actually called
      // This prevents hundreds of dirty nodes in VirtualList scenarios
      runtime.view_states.insert(view_id, state);
    });

    // 🔑 Register cleanup function to auto-remove resources when scope is destroyed
    // This ensures ViewStates, Taffy nodes, Widgets, and Styles are cleaned up
    crate::runtime::on_cleanup(move || {
      crate::runtime::with_layout_mut(|runtime| {
        // Remove ViewState
        runtime.view_states.remove(&view_id);

        // Remove Taffy node
        let _ = runtime.taffy.remove(view_id.node_id);

        // Remove Widget
        runtime.widgets.remove(view_id.node_id().into());

        // Remove Style
        runtime.styles.remove(&view_id.node_id);

        // Remove from dirty set
        runtime.style_dirty_nodes.remove(&view_id);
      });
    });

    view_id
  }

  pub fn id(&self) -> u64 {
    self.id
  }

  pub fn node_id(&self) -> NodeId {
    self.node_id
  }

  pub fn get_children(&self) -> std::rc::Rc<Vec<ViewId>> {
    crate::runtime::with_layout(|runtime| {
      runtime
        .view_states
        .get(self)
        .map(|state| state.children.clone())
        .unwrap_or_else(|| std::rc::Rc::new(Vec::new()))
    })
  }

  /// Get parent node (O(1) cached lookup)
  ///
  /// Performance: Changed from O(n) iteration to O(1) HashMap lookup
  /// This is critical for performance in batch_bubble_dirty_marks
  pub fn parent(&self) -> Option<ViewId> {
    crate::runtime::with_layout(|runtime| {
      runtime.view_states.get(self).and_then(|state| state.parent)
    })
  }

  /// Set custom Widget
  pub fn widget(self, widget: Rc<dyn crate::widget::Widget>) -> Self {
    let node_id = self.node_id;
    crate::runtime::with_layout_mut(|runtime| {
      runtime.widgets.insert(node_id.into(), widget);
    });
    self
  }

  pub fn name(&self) -> String {
    crate::runtime::with_layout(|runtime| {
      runtime
        .view_states
        .get(self)
        .map(|state| state.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
    })
  }

  pub fn print_tree(&self) {
    let id = self.node_id;
    crate::runtime::with_layout_mut(|runtime| runtime.taffy.print_tree(id));
  }

  /// Set child view IDs (updates both ViewState and Taffy tree)
  pub fn set_children(&self, children: Vec<ViewId>) {
    crate::runtime::with_layout_mut(|runtime| {
      for child_id in &children {
        if let Some(child_state) = runtime.view_states.get_mut(child_id) {
          child_state.parent = Some(*self);
        }
      }

      let mut taffy_children: Vec<NodeId> = Vec::with_capacity(children.len());
      for child in &children {
        taffy_children.push(child.node_id);
      }
      let _ = runtime.taffy.set_children(self.node_id, &taffy_children);

      if let Some(state) = runtime.view_states.get_mut(self) {
        state.children = std::rc::Rc::new(children);
      }
    });
  }

  /// Append child view IDs to existing children (for Portal support)
  pub fn append_children(&self, new_children: Vec<ViewId>) {
    crate::runtime::with_layout_mut(|runtime| {
      // Update parent references for new children
      for child_id in &new_children {
        if let Some(child_state) = runtime.view_states.get_mut(child_id) {
          child_state.parent = Some(*self);
        }
      }

      // Get existing children and append new ones
      if let Some(state) = runtime.view_states.get_mut(self) {
        let mut all_children = (*state.children).clone();
        all_children.extend(new_children);

        // Update taffy tree
        let taffy_children: Vec<NodeId> = all_children.iter().map(|id| id.node_id).collect();
        let _ = runtime.taffy.set_children(self.node_id, &taffy_children);

        // Update ViewState
        state.children = std::rc::Rc::new(all_children);
      }
    });
  }

  /// Compute layout
  pub fn compute_layout(
    &self,
    available_space: taffy::Size<taffy::AvailableSpace>,
    ctx: &crate::RenderContext,
  ) -> Option<taffy::Layout> {
    crate::layout::compute_layout(self.node_id, available_space, ctx);
    self.layout()
  }

  /// Get current layout without recomputing
  pub fn layout(&self) -> Option<taffy::Layout> {
    crate::runtime::with_layout(|runtime| runtime.taffy.layout(self.node_id).ok().cloned())
  }


  /// Get absolute position (cached from render pass)
  /// Returns None if the view hasn't been rendered yet
  pub fn absolute_position(&self) -> Option<(f32, f32)> {
    crate::runtime::with_layout(|runtime| {
      runtime
        .view_states
        .get(self)
        .and_then(|state| state.absolute_position)
    })
  }

  /// Mark as needing redraw and re-layout
  ///
  /// Note: This method is safe to call even if the node has been cleaned up.
  /// It will silently ignore the call if the node no longer exists.
  pub fn mark_dirty(&self) {
    crate::runtime::with_layout_mut(|layout| {
      // Check if view state still exists (node may have been cleaned up)
      if layout.view_states.contains_key(self) {
        if let Some(state) = layout.view_states.get_mut(self) {
          state.dirty = true;
        }

        // Only mark taffy node dirty if it still exists
        // This prevents panic when marking dirty on cleaned-up nodes
        // Ignore errors (node may have been removed)
        let _ = layout.taffy.mark_dirty(self.node_id);
      }
      // If view state doesn't exist, the node has been cleaned up, silently ignore
    });

    crate::runtime::with_window(|window_state| {
      if let Some(window) = &window_state.redraw_requester {
        window.request_redraw();
      }
    });
  }

  /// Set style
  pub fn style<F>(self, f: F) -> Self
  where
    F: Fn(crate::style::StyleBuilder) -> crate::style::StyleBuilder + 'static,
  {
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&self) {
        let builder = f(crate::style::StyleBuilder::new());
        state.styles.push(Some(builder));

        // 🔥 Mark as dirty only when style is set
        // This is the key optimization to prevent dirty node explosion
        if !state.style_dirty {
          state.style_dirty = true;
          runtime.style_dirty_nodes.insert(self);
        }
      }
    });
    self
  }

  /// Add click event handler
  pub fn on_click<F>(self, handler: F) -> Self
  where
    F: Fn(&mut crate::event::MouseEvent) + 'static,
  {
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&self) {
        state.event_handlers.add_handler(handler);
      }
    });
    self
  }

  /// Request repaint based on StyleTrigger level
  pub fn request_repaint(&self, trigger: crate::style::StyleTrigger) {
    use crate::style::StyleTrigger;

    match trigger {
      StyleTrigger::None => {}
      StyleTrigger::Composite | StyleTrigger::Paint | StyleTrigger::Layout => {
        crate::runtime::with_window(|window_state| {
          if let Some(window) = &window_state.redraw_requester {
            window.request_redraw();
          }
        });
      }
    }
  }

  /// Request style update (collect dirty nodes)
  ///
  /// Only marks this node as dirty and collects it to the global dirty set.
  /// Bubbling is deferred to batch processing time to avoid timing issues.
  pub fn request_style(&self) {
    crate::runtime::with_layout_mut(|runtime| {
      runtime.style_dirty_nodes.insert(*self);
    });

    crate::runtime::with_window(|window_state| {
      if let Some(window) = &window_state.redraw_requester {
        window.request_redraw();
      }
    });
  }

  // ============================================================================
  // Focus Management
  // ============================================================================

  /// Check if this view is currently focused
  pub fn is_focused(&self) -> bool {
    crate::runtime::with_window(|state| state.focused_view == Some(*self))
  }
}
