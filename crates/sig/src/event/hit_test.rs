use glam::Vec2;
use crate::ViewId;

/// Hit test the view tree and return the path from root to the target view
pub fn hit_test(view: &ViewId, point: Vec2, current_offset: Vec2) -> Option<Vec<ViewId>> {
  let mut path = Vec::new();
  if hit_test_recursive(view, point, current_offset, &mut path) {
    // path is currently constructed in reverse visit order (view is pushed AFTER children checked).
    // Since we return true if child hits, and push view *after* returning true,
    // the order in path is [child_deepest, ..., child, view].
    // This is exactly Target -> Root order.
    Some(path)
  } else {
    None
  }
}

fn hit_test_recursive(
  view: &ViewId,
  point: Vec2,
  current_offset: Vec2,
  path: &mut Vec<ViewId>,
) -> bool {
  // 🎯 Check pointer_events first - skip if set to None
  let has_pointer_events = crate::runtime::with_layout(|runtime| {
    runtime
      .view_states
      .get(view)
      .map(|state| state.style.pointer_events == crate::style::PointerEvents::Auto)
      .unwrap_or(true) // Default to Auto if not found
  });

  if !has_pointer_events {
    return false; // Skip this view and its children
  }

  // Check if point is within bounds
  if let Some(layout) = view.layout() {
    let x = layout.location.x + current_offset.x;
    let y = layout.location.y + current_offset.y;
    let width = layout.size.width;
    let height = layout.size.height;

    // Simple AABB check
    if point.x >= x && point.x < x + width && point.y >= y && point.y < y + height {
      // Get scroll offset for this view
      let scroll_offset = crate::runtime::with_layout(|runtime| {
        runtime
          .view_states
          .get(view)
          .map(|state| state.scroll_offset)
          .unwrap_or((0.0, 0.0))
      });

      // Hit! Now check children (reverse order for z-order)
      let child_ids = view.get_children();

      // Apply scroll offset to children's coordinate space
      let child_offset = Vec2::new(x - scroll_offset.0, y - scroll_offset.1);

      // We need to iterate backward
      for child_id in child_ids.iter().rev() {
        if hit_test_recursive(&child_id, point, child_offset, path) {
          path.push(*view);
          return true;
        }
      }

      // If no child hit, but we hit this view
      path.push(*view);
      return true;
    }
  }
  false
}

/// Find a view by its ID in the tree
pub fn find_view_by_id(view: &ViewId, target_id: ViewId) -> Option<ViewId> {
  if *view == target_id {
    return Some(*view);
  }

  let child_ids = view.get_children();
  for child_id in child_ids.iter() {
    if let Some(found) = find_view_by_id(&child_id, target_id) {
      return Some(found);
    }
  }

  None
}
