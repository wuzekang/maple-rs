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
  Style, StyleProperty, StylePropertyKey, StylePropertyMap, StyleTrigger, TaffyStyleProperty,
  TaffyStylePropertyKey, TaffyStylePropertyMap,
};
use std::collections::{HashMap, HashSet};
use strum::{EnumCount, IntoEnumIterator};

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
  /// Currently inherited style properties, indexed by
  /// [`StylePropertyKey`] discriminant (no hashing, no allocation).
  pub style: StylePropertyMap,
  /// Whether to continue propagating downwards (has inheritable property changes)
  pub dirty: bool,
  /// Initial style properties
  pub style_initial: Vec<(StylePropertyKey, StyleProperty)>,
  /// Initial Taffy style properties
  pub taffy_style_initial: Vec<(TaffyStylePropertyKey, TaffyStyleProperty)>,
  /// Visited views (avoid duplicate computation)
  pub visited: HashSet<ViewId>,
  /// Undo journal for `style`: each inherited-property overwrite records the
  /// previous entry so subtree walks restore the context with a bounded
  /// replay instead of cloning the whole map per node. Indices, not keys.
  style_undo: Vec<(usize, Option<StyleProperty>)>,
}

impl StyleComputeContext {
  /// Create a new style computation context
  pub fn new() -> Self {
    Self {
      style: [None; StylePropertyKey::COUNT],
      dirty: false,
      visited: HashSet::new(),
      style_initial: StyleProperty::initial(),
      taffy_style_initial: TaffyStyleProperty::initial(),
      style_undo: Vec::new(),
    }
  }

  /// Mark the current undo length; restore with [`Self::undo_style_to`].
  fn undo_mark(&self) -> usize {
    self.style_undo.len()
  }

  /// Replay the journal back to a mark, restoring `style` entries.
  fn undo_style_to(&mut self, mark: usize) {
    while self.style_undo.len() > mark {
      let (idx, prev) = self.style_undo.pop().unwrap();
      self.style[idx] = prev;
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

  // Merge style builders (backwards, later ones have higher priority) under
  // the read borrow. Building the prop maps in place avoids cloning every
  // StyleBuilder on the hot path.
  let (style_props, mut taffy_style_props, old_style, old_taffy_style) =
    crate::runtime::with_layout(|runtime| {
      // Later builders win; a Some() slot is only filled by the first
      // (highest-priority) builder that sets the key.
      let mut style_props: StylePropertyMap = [None; StylePropertyKey::COUNT];
      let mut taffy_style_props: TaffyStylePropertyMap = [None; TaffyStylePropertyKey::COUNT];
      let (old_style, old_taffy_style) = if let Some(state) = runtime.view_states.get(view) {
        for style_builder in state.styles.iter().rev() {
          if let Some(style_builder) = style_builder.as_ref() {
            for (key, value) in style_builder.taffy_style_props.iter().rev() {
              let slot = &mut taffy_style_props[*key as usize];
              if slot.is_none() {
                *slot = Some(*value);
              }
            }
            for (key, value) in style_builder.style_props.iter().rev() {
              let slot = &mut style_props[*key as usize];
              if slot.is_none() {
                *slot = Some(*value);
              }
            }
          }
        }
        (
          state.style.clone(),
          runtime
            .taffy
            .style(view.node_id())
            .cloned()
            .unwrap_or_default(),
        )
      } else {
        (Style::default(), taffy::Style::default())
      };
      (style_props, taffy_style_props, old_style, old_taffy_style)
    });

  // Apply initial Taffy styles
  for (key, value) in &ctx.taffy_style_initial {
    let slot = &mut taffy_style_props[*key as usize];
    if slot.is_none() {
      *slot = Some(*value);
    }
  }

  // Build the taffy style from whichever slots are set
  let mut taffy_style = taffy::Style::default();
  for value in taffy_style_props.iter().flatten() {
    value.assign_to(&mut taffy_style);
  }

  // Apply to the taffy tree ONLY when the taffy-relevant style actually
  // changed. taffy's set_style does not compare old and new values — it
  // marks the node layout-dirty unconditionally, so an equal-style write
  // (e.g. a paint-only property changed) costs a full subtree relayout.
  if old_taffy_style != taffy_style {
    if style_trigger < StyleTrigger::Layout {
      style_trigger = StyleTrigger::Layout;
    }

    crate::runtime::with_layout_mut(|runtime| {
      runtime
        .taffy
        .set_style(view.node_id(), taffy_style)
        .expect("Failed to set taffy style");
    });
  }

  // Build render style
  let mut style = Style::default();

  // Apply initial styles
  for (key, value) in &ctx.style_initial {
    if style_props[*key as usize].is_none() {
      value.assign_to(&mut style);
    }
  }

  // Inherit inheritable properties from parent
  for key in StylePropertyKey::iter() {
    let idx = key as usize;
    if style_props[idx].is_none() && key.inherited() {
      if let Some(value) = ctx.style[idx] {
        value.assign_to(&mut style);
      }
    }
  }

  // Apply current view's style properties
  for value in style_props.iter().flatten() {
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

  // Update styles in state (single clone: view state keeps a copy, the
  // measure-call map takes the original)
  crate::runtime::with_layout_mut(|runtime| {
    if let Some(state) = runtime.view_states.get_mut(view) {
      state.style = style.clone();
    }
    runtime.styles.insert(view.node_id(), style);
  });

  // Update inheritable properties in context (for propagation to child
  // nodes). Each overwrite journals the previous entry; the subtree walk
  // restores via the journal instead of cloning the map.
  for (idx, slot) in style_props.iter().enumerate() {
    if let Some(value) = slot
      && StylePropertyKey::iter().nth(idx).is_some_and(|k| k.inherited())
    {
      let prev = ctx.style[idx];
      ctx.style[idx] = Some(*value);
      ctx.style_undo.push((idx, prev));
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
  let undo_mark = ctx.undo_mark();

  // If this node is dirty, compute style
  if style_dirty {
    compute_style(view, ctx);

    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(view) {
        state.style_dirty = false;
      }
    });
  }

  // Mark as visited and update cache (plain array copy)
  ctx.visited.insert(*view);
  crate::runtime::with_layout_mut(|runtime| {
    if let Some(state) = runtime.view_states.get_mut(view) {
      state.style_cache = ctx.style;
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
  ctx.undo_style_to(undo_mark);
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
