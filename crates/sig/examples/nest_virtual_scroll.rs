//! Optimized nest.rs - supports virtual scrolling (using reusable components)
//!
//! Optimizations:
//! 1. Virtual scrolling: only render nodes in visible area (via virtual list component)
//! 2. Lazy loading: only parse child nodes when expanded
//! 3. Performance monitoring: display rendering statistics
//! 4. Smart diff detection: use `each` to only update changed nodes

use sig::prelude::*;
use sig::render::{run, AppConfig};
use sig::{dynamic, Signal, VirtualList};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use vello::peniko::Color;
use wz_parser::util::{node_util, resolve_base};
use wz_parser::WzNodeArc;

// ============================================================================
// Virtual scrolling configuration
// ============================================================================

/// Fixed height of each node (in pixels)
const ITEM_HEIGHT: f32 = 32.0;

/// Buffer size: additional nodes to render above and below visible area
const BUFFER_SIZE: usize = 10;

// ============================================================================
// Data structures
// ============================================================================

/// Flattened node data
#[derive(Clone, Debug)]
struct FlatNode {
  node: WzNodeArc,
  depth: usize,
  index: usize,
  has_children: bool,
}

// Implement PartialEq for diff detection
// Only need to compare Arc pointers to determine if it's the same node
impl PartialEq for FlatNode {
  fn eq(&self, other: &Self) -> bool {
    std::sync::Arc::ptr_eq(&self.node, &other.node)
  }
}

/// Virtual scrolling state management
struct VirtualScrollState {
  root_node: WzNodeArc,
  flat_nodes: Signal<Vec<FlatNode>>,
  open_states: Signal<HashMap<usize, bool>>,
  total_nodes: Signal<usize>,
  last_operation_time: Signal<f64>,
}

impl VirtualScrollState {
  fn new(root_node: WzNodeArc) -> Rc<Self> {
    let open_states = HashMap::new();
    let flat_nodes = Self::flatten_tree(&root_node, 0, 0, &open_states);
    let total = flat_nodes.len();

    Rc::new(Self {
      root_node: root_node.clone(),
      flat_nodes: Signal::new(flat_nodes),
      open_states: Signal::new(open_states),
      total_nodes: Signal::new(total),
      last_operation_time: Signal::new(0.0),
    })
  }

  fn flatten_tree(
    node: &WzNodeArc,
    depth: usize,
    mut index: usize,
    open_states: &HashMap<usize, bool>,
  ) -> Vec<FlatNode> {
    let node_data = node.read().unwrap();
    let has_children = !node_data.children.is_empty();
    let is_open = open_states.get(&index).copied().unwrap_or(false);

    let mut result = vec![FlatNode {
      node: node.clone(),
      depth,
      index,
      has_children,
    }];

    if is_open && has_children {
      let children = &node_data.children;
      index += 1;

      for (_, child) in children.iter() {
        let child_nodes = Self::flatten_tree(child, depth + 1, index, open_states);
        index += child_nodes.len();
        result.extend(child_nodes);
      }
    }

    result
  }

  fn toggle_node(&self, index: usize) {
    let start = Instant::now();

    // Read and copy values first, then drop the guards
    let is_open = {
      self
        .open_states
        .read_untracked()
        .get(&index)
        .copied()
        .unwrap_or(false)
    };

    // Clone the node in a separate scope to drop the read guard
    let node_to_parse = {
      let flat_nodes = self.flat_nodes.read_untracked();
      if !is_open && index < flat_nodes.len() {
        Some(flat_nodes[index].node.clone())
      } else {
        None
      }
    };

    // Parse node if needed (no borrows held)
    if let Some(node) = node_to_parse {
      if let Err(e) = node_util::parse_node(&node) {
        eprintln!("⚠️  Failed to parse node: {}", e);
        return;
      }
    }

    // Update open states
    {
      let mut states = self.open_states.write();
      states.insert(index, !is_open);
    }

    // Read open_states in a separate scope to drop the guard
    let open_states_snapshot = { self.open_states.read_untracked().clone() };

    let new_flat = Self::flatten_tree(&self.root_node, 0, 0, &open_states_snapshot);
    let new_total = new_flat.len();

    *self.flat_nodes.write() = new_flat;
    *self.total_nodes.write() = new_total;

    let total_time = start.elapsed().as_secs_f64() * 1000.0;
    *self.last_operation_time.write() = total_time;
  }
}

// ============================================================================
// UI Components
// ============================================================================

fn node_row(flat_node: &FlatNode, _index: usize, state: &Rc<VirtualScrollState>) -> View {
  let depth = flat_node.depth;
  let index = flat_node.index;
  let has_children = flat_node.has_children;
  let open_states = state.open_states.clone();

  let node_data = flat_node.node.read().unwrap();
  let name = node_data.name.to_string();
  let child_count = node_data.children.len();

  view()
    .style(move |s| {
      s.flex()
        .height(ITEM_HEIGHT)
        .w_full()
        .padding_left((depth * 16) as f32 + 8.0)
        .items_center()
        .gap(8.0)
        .border_bottom(1.0)
        .cursor(sig::cursor::Cursor::Pointer)
    })
    .child((
      view()
        .style(|s| s.width(16.0).justify_center())
        .child(dynamic(move || {
          let is_open = open_states.read().get(&index).copied().unwrap_or(false);
          text(if has_children {
            if is_open { "▼" } else { "▶" }
          } else {
            " "
          })
          .style(|s| s.font_size(10.0).color(Color::from_rgb8(120, 120, 120)))
        })),
      view()
        .style(|s| s.flex_grow(1.0))
        .child(
          text(if child_count > 0 {
            format!("{} ({})", name, child_count)
          } else {
            name
          })
          .style(move |s| {
            s.font_size(13.0).color(if has_children {
              Color::from_rgb8(40, 40, 40)
            } else {
              Color::from_rgb8(100, 100, 100)
            })
          }),
        ),
    ))
    .on_click({
      let state = state.clone();
      move |_| {
        if has_children {
          state.toggle_node(index);
        }
      }
    })
}

fn stats_panel(state: &Rc<VirtualScrollState>) -> View {
  let total_nodes = state.total_nodes.clone();
  let last_op_time = state.last_operation_time.clone();

  view()
    .style(|s| {
      s.flex()
        .padding(12.0)
        .gap(20.0)
        .background(Color::from_rgb8(245, 247, 250))
        .border_bottom(1.0)
        .w_full()
    })
    .child((
      view().style(|s| s.flex().flex_col().gap(2.0)).child((
        text("WZ Node Browser (Virtual Scrolling Component)").style(|s| {
          s.font_size(16.0)
            .font_weight(600)
            .color(Color::from_rgb8(30, 30, 30))
        }),
        text("💡 Using reusable VirtualList component")
          .style(|s| s.font_size(11.0).color(Color::from_rgb8(100, 100, 100))),
      )),
      view().style(|s| s.flex_grow(1.0)),
      view().style(|s| s.flex().gap(16.0).items_center()).child((
        dynamic(move || {
          text(format!("📊 Total nodes: {}", *total_nodes.read()))
            .style(|s| s.font_size(12.0).color(Color::from_rgb8(60, 60, 60)))
        }),
        dynamic(move || {
          let time = *last_op_time.read();
          if time > 0.0 {
            text(format!("⏱️  {:.2}ms", time))
              .style(|s| s.font_size(12.0).color(Color::from_rgb8(200, 80, 0)))
          } else {
            text("")
          }
        }),
      )),
    ))
}

fn main() -> anyhow::Result<()> {
  run(
    AppConfig {
      title: "WZ Node Browser - Virtual Scrolling".to_string(),
      width: 1000.0,
      height: 700.0,
    },
    || {
      let node = resolve_base("./data/Base.wz", None).unwrap();
      let state = VirtualScrollState::new(node);
      let state_for_render = state.clone();

      // Use virtual list component
      let list = VirtualList::new(state.flat_nodes.clone())
        .height(600.0)
        .item_height(ITEM_HEIGHT)
        .buffer_size(BUFFER_SIZE)
        .build(move |flat_node, index| node_row(flat_node, index, &state_for_render));

      view()
        .style(|s| {
          s.size_full()
            .flex_col()
            .background(Color::from_rgb8(250, 250, 250))
        })
        .child((stats_panel(&state), list))
    },
  )
}
