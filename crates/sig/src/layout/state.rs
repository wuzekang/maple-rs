//! Layout module - Taffy-based layout system
//!
//! Provides reactive layout functionality integrated with the taffy layout engine

use crate::widget::Widget;
use crate::{ViewId, ViewState, RenderContext};
use parley::{FontContext, LayoutContext};
use slotmap::{DefaultKey, SecondaryMap};
use std::rc::Rc;
pub use taffy;
use taffy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Compute layout (with measure function support)
pub fn compute_layout(root_id: NodeId, available_space: Size<AvailableSpace>, ctx: &RenderContext) {
  crate::runtime::with_layout_mut(|state| {
    state.compute_layout(root_id, available_space, ctx);
  });
}

/// Global layout runtime
pub struct LayoutState {
  pub taffy: TaffyTree,
  pub widgets: SecondaryMap<DefaultKey, Rc<dyn Widget>>,
  pub styles: HashMap<NodeId, crate::style::Style>,
  /// View state storage (indexed by ViewId)
  pub view_states: HashMap<ViewId, ViewState>,
  /// Parley font context
  pub font_context: FontContext,
  /// Parley layout context
  pub layout_context: LayoutContext<()>,
  /// Style dirty node set (collected via Dirty Bubbling)
  pub style_dirty_nodes: HashSet<ViewId>,
}

impl LayoutState {
  pub fn new() -> Self {
    Self {
      taffy: TaffyTree::new(),
      widgets: SecondaryMap::<DefaultKey, Rc<dyn Widget>>::new(),
      styles: HashMap::new(),
      view_states: HashMap::new(),
      font_context: FontContext::new(),
      layout_context: LayoutContext::new(),
      style_dirty_nodes: HashSet::new(),
    }
  }

  pub fn compute_layout(&mut self, root_id: NodeId, available_space: Size<AvailableSpace>, ctx: &RenderContext) {
    let widgets = &self.widgets;
    let styles = &self.styles;
    let font_context = &mut self.font_context;
    let layout_context = &mut self.layout_context;
    
    let _ = self.taffy.compute_layout_with_measure(
      root_id,
      available_space,
      |known, available, node_id, _ctx, _style| {
        if let Some(widget) = widgets.get(node_id.into()) {
          let style = styles.get(&node_id).cloned().unwrap_or_default();
          widget.measure(known, available, &style, ctx, font_context, layout_context)
        } else {
          Size::ZERO
        }
      },
    );
  }
}

impl Default for LayoutState {
  fn default() -> Self {
    Self::new()
  }
}
