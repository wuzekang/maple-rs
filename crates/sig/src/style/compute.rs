//! Style computation module
//!
//! Reference crates/ui/src/style/compute.rs for style computation and inheritance implementation
//!
//! Improvements:
//! 1. ✅ Implement StyleTrigger mechanism for fine-grained repaint control
//! 2. ✅ Use strum::IntoEnumIterator to iterate over property keys
//! 3. ✅ Add dirty checking optimization to avoid unnecessary recursion
//! 4. ✅ Improve error handling strategy
//! 5. ✅ Get styles directly from parent view (through context propagation)
//!
//! Note: Bump allocator is not used yet, as hashbrown requires additional allocator-api2 support

use crate::ViewId;
use crate::style::{
  Style, StyleProperty, StylePropertyKey, StyleTrigger, TaffyStyleProperty, TaffyStylePropertyKey,
};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

/// Batch bubbling: Mark upwards from collected dirty nodes
///
/// # Algorithm
///
/// Iterate through all dirty nodes and bubble upwards to mark ancestors' child_style_dirty
/// At this point, parent-child relationships are stable and parent() queries are reliable
///
/// # Complexity
///
/// O(m × depth), where m = number of dirty nodes
///
pub fn batch_bubble_dirty_marks() {
  let dirty_nodes: Vec<ViewId> = crate::runtime::with_layout(|runtime| {
    runtime
      .style_dirty_nodes
      .iter()
      .copied()
      // 🔥 Filter out deleted/invalid nodes to prevent wasted work
      .filter(|id| runtime.view_states.contains_key(id))
      .collect()
  });

  // 🔍 Performance monitoring (enable with RUST_LOG=debug)
  if !dirty_nodes.is_empty() {
    #[cfg(debug_assertions)]
    {
      if dirty_nodes.len() > 50 {
        eprintln!(
          "⚠️  WARNING: {} dirty nodes (may impact performance)",
          dirty_nodes.len()
        );
      } else if dirty_nodes.len() > 10 {
        eprintln!("🔍 Style dirty nodes: {}", dirty_nodes.len());
      }
    }
  }

  if dirty_nodes.is_empty() {
    return;
  }

  for node in &dirty_nodes {
    // 🔥 Collect parent chain first (avoid borrow conflicts)
    let parent_chain: Vec<ViewId> = {
      let mut chain = Vec::new();
      let mut current = node.parent();
      while let Some(parent_id) = current {
        chain.push(parent_id);
        current = parent_id.parent();
      }
      chain
    };

    crate::runtime::with_layout_mut(|runtime| {
      // Mark this node
      if let Some(state) = runtime.view_states.get_mut(node) {
        if !state.style_dirty {
          state.style_dirty = true;
        }
      }

      // Bubble upwards
      for parent_id in &parent_chain {
        if let Some(parent_state) = runtime.view_states.get_mut(parent_id) {
          if parent_state.child_style_dirty {
            break; // Early exit
          }
          parent_state.child_style_dirty = true;
        }
      }
    });
  }

  // Clear dirty node set
  crate::runtime::with_layout_mut(|runtime| {
    runtime.style_dirty_nodes.clear();
  });
}

/// Style computation context
pub struct StyleComputeContext {
  /// Currently inherited style properties
  pub style: HashMap<StylePropertyKey, StyleProperty>,
  /// Whether to continue propagating downwards (has inheritable property changes)
  pub dirty: bool,
  /// Initial style properties
  pub style_initial: Vec<(StylePropertyKey, StyleProperty)>,
  /// Initial Taffy style properties
  pub taffy_style_initial: Vec<(TaffyStylePropertyKey, TaffyStyleProperty)>,
  /// Visited views (avoid duplicate computation)
  pub visited: HashSet<ViewId>,
}

impl StyleComputeContext {
  /// Create a new style computation context
  pub fn new() -> Self {
    Self {
      style: HashMap::new(),
      dirty: false,
      visited: HashSet::new(),
      style_initial: StyleProperty::initial(),
      taffy_style_initial: TaffyStyleProperty::initial(),
    }
  }
}

impl Default for StyleComputeContext {
  fn default() -> Self {
    Self::new()
  }
}

/// Calculate the style for a single view
fn compute_style(view: &ViewId, ctx: &mut StyleComputeContext) {
  let mut style_trigger = StyleTrigger::None;

  let mut style_props = HashMap::<StylePropertyKey, StyleProperty>::new();
  let mut taffy_style_props = HashMap::<TaffyStylePropertyKey, TaffyStyleProperty>::new();

  // Get state from runtime and iterate through style builders (backwards, later ones have higher priority)
  let (styles, old_style, old_taffy_style) = crate::runtime::with_layout(|runtime| {
    if let Some(state) = runtime.view_states.get(view) {
      let taffy_style = runtime
        .taffy
        .style(view.node_id())
        .cloned()
        .unwrap_or_default();
      (state.styles.clone(), state.style.clone(), taffy_style)
    } else {
      (Vec::new(), Style::default(), taffy::Style::default())
    }
  });

  // Iterate through style builders backwards (later ones have higher priority)
  for style_builder in styles.iter().rev() {
    if let Some(style_builder) = style_builder.as_ref() {
      // Collect Taffy style properties (deduplicated)
      for (key, value) in style_builder.taffy_style_props.iter().rev() {
        if !taffy_style_props.contains_key(key) {
          taffy_style_props.insert(*key, *value);
        }
      }
      // Collect render style properties (deduplicated)
      for (key, value) in style_builder.style_props.iter().rev() {
        if !style_props.contains_key(key) {
          style_props.insert(*key, value.clone());
        }
      }
    }
  }

  // Apply initial Taffy styles
  for (key, value) in &ctx.taffy_style_initial {
    if !taffy_style_props.contains_key(key) {
      taffy_style_props.insert(*key, *value);
    }
  }

  // Update Taffy styles
  let mut taffy_style = taffy::Style::default();
  for (_, value) in taffy_style_props {
    value.assign_to(&mut taffy_style);
  }

  // Detect Taffy style changes and update trigger
  if old_taffy_style != taffy_style {
    if style_trigger < StyleTrigger::Layout {
      style_trigger = StyleTrigger::Layout;
    }
  }

  // Apply to Taffy tree
  crate::runtime::with_layout_mut(|runtime| {
    runtime
      .taffy
      .set_style(view.node_id(), taffy_style)
      .expect("Failed to set taffy style");
  });

  // Build render style
  let mut style = Style::default();

  // Apply initial styles
  for (key, value) in &ctx.style_initial {
    if !style_props.contains_key(key) {
      value.assign_to(&mut style);
    }
  }

  // Inherit inheritable properties from parent (use iterator to traverse all property keys)
  for key in StylePropertyKey::iter() {
    if !style_props.contains_key(&key) && key.inherited() {
      if let Some(value) = ctx.style.get(&key) {
        value.assign_to(&mut style);
      }
    }
  }

  // Apply current view's style properties
  for (_, value) in style_props.iter() {
    value.assign_to(&mut style);
  }

  // Detect style changes and update trigger and dirty flag
  for key in StylePropertyKey::iter() {
    if key.value(&old_style) != key.value(&style) {
      let trigger = key.trigger();
      if trigger > style_trigger {
        style_trigger = trigger;
      }
      // If inheritable property changes, mark as dirty to propagate to child nodes
      if !ctx.dirty && key.inherited() {
        ctx.dirty = true;
      }
    }
  }

  // Update styles in state
  crate::runtime::with_layout_mut(|runtime| {
    // Update render styles to styles (for measure calls)
    runtime.styles.insert(view.node_id(), style.clone());

    // Update view state
    if let Some(state) = runtime.view_states.get_mut(view) {
      state.style = style.clone();
    }
  });

  // Update inheritable properties in context (for propagation to child nodes)
  for (key, value) in &style_props {
    if key.inherited() {
      ctx.style.remove(key);
      ctx.style.insert(*key, value.clone());
    }
  }

  // Request repaint based on trigger level
  view.request_repaint(style_trigger);
}

/// Recursively compute styles for a view and its subviews (Dirty Bubbling with Stack-based Context)
///
/// # Algorithm: Dirty Bubbling with Bailout + Context Stack
///
/// Works with batch_bubble_dirty_marks():
/// 1. First call batch_bubble_dirty_marks() to batch bubble marks
/// 2. Then call this function to recurse from root node with automatic pruning
///
/// # Context Stack Mechanism
///
/// To ensure correct property inheritance, we use a stack-based approach:
/// 1. **Push**: Save current context state before processing children
/// 2. **Process**: Children inherit from current context and may modify it
/// 3. **Pop**: Restore context state after children, ensuring siblings don't cross-contaminate
///
/// Example:
/// ```
/// Parent (cursor: Default)
///   ├─ Button (cursor: Pointer) ← modifies context
///   └─ Sibling ← should still inherit Default, not Pointer
/// ```
///
/// # Advantages
///
/// - ✅ Does not rely on parent() queries during request_style() (avoid timing issues)
/// - ✅ Parent-child relationships are stable during batch bubbling
/// - ✅ Complete Dirty Bubbling pruning optimization
/// - ✅ Correct property inheritance with context isolation between siblings
///
pub fn compute_style_recursive(view: &ViewId, ctx: &mut StyleComputeContext) {
  // Check if already visited (avoid duplicate computation)
  if ctx.visited.contains(view) {
    return;
  }

  // Bailout check
  let (style_dirty, child_style_dirty) = crate::runtime::with_layout(|runtime| {
    runtime
      .view_states
      .get(view)
      .map(|state| (state.style_dirty, state.child_style_dirty))
      .unwrap_or((false, false))
  });

  if !style_dirty && !child_style_dirty {
    return; // Bailout
  }

  let dirty = ctx.dirty;
  let saved_style = ctx.style.clone();

  // If this node is dirty, compute style
  if style_dirty {
    compute_style(view, ctx);

    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(view) {
        state.style_dirty = false;
      }
    });
  }

  // Mark as visited and update cache
  ctx.visited.insert(*view);
  crate::runtime::with_layout_mut(|runtime| {
    if let Some(state) = runtime.view_states.get_mut(view) {
      state.style_cache.clear();
      for (key, value) in &ctx.style {
        state.style_cache.insert(*key, value.clone());
      }
    }
  });

  // If subtree is dirty or has inheritable property changes, recurse into child nodes
  if child_style_dirty || ctx.dirty {
    let child_ids = view.get_children();

    for child_id in child_ids.iter() {
      compute_style_recursive(&child_id, ctx);
    }

    // Clear child_style_dirty flag
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(view) {
        state.child_style_dirty = false;
      }
    });
  }

  ctx.dirty = dirty;
  ctx.style = saved_style;
}

/// Get the computed cursor style for a view
pub fn get_computed_cursor(view: &ViewId) -> crate::Cursor {
  crate::runtime::with_layout(|runtime| {
    if let Some(style) = runtime.styles.get(&view.node_id()) {
      style.cursor
    } else {
      crate::Cursor::Default
    }
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_style_compute_context() {
    let ctx = StyleComputeContext::new();
    assert!(!ctx.dirty);
    assert!(ctx.visited.is_empty());
  }

  #[test]
  fn test_style_trigger_ordering() {
    assert!(StyleTrigger::None < StyleTrigger::Composite);
    assert!(StyleTrigger::Composite < StyleTrigger::Paint);
    assert!(StyleTrigger::Paint < StyleTrigger::Layout);
  }

  #[test]
  fn test_style_property_key_trigger() {
    assert_eq!(StylePropertyKey::Cursor.trigger(), StyleTrigger::None);
    assert_eq!(StylePropertyKey::Opacity.trigger(), StyleTrigger::Composite);
    assert_eq!(StylePropertyKey::Background.trigger(), StyleTrigger::Paint);
    assert_eq!(StylePropertyKey::FontSize.trigger(), StyleTrigger::Layout);
  }

  #[test]
  fn test_style_property_key_inherited() {
    assert!(StylePropertyKey::Color.inherited());
    assert!(StylePropertyKey::FontSize.inherited());
    assert!(!StylePropertyKey::Background.inherited());
    assert!(!StylePropertyKey::Opacity.inherited());
  }

  #[test]
  fn test_style_property_key_iteration() {
    let mut count = 0;
    for _key in StylePropertyKey::iter() {
      count += 1;
    }
    // Ensure all property keys can be iterated
    assert!(count > 0);
  }
}
