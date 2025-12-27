use glam::Vec2;
use crate::ViewId;

/// Drag state management
#[derive(Debug, Clone)]
pub struct DragState {
  /// Whether dragging is in progress
  pub is_dragging: bool,
  /// Drag start position
  pub start_position: Vec2,
  /// Previous frame position (for calculating delta)
  pub last_position: Vec2,
  /// Drag target view
  pub target: Option<ViewId>,
  /// Drag trigger threshold (pixels)
  pub threshold: f32,
  /// Whether scrollbar is being dragged
  pub is_scrollbar_drag: bool,
  /// Scrollbar drag direction (true = vertical, false = horizontal)
  pub scrollbar_direction_vertical: bool,
}

impl Default for DragState {
  fn default() -> Self {
    Self {
      is_dragging: false,
      start_position: Vec2::ZERO,
      last_position: Vec2::ZERO,
      target: None,
      threshold: 5.0, // 5 pixel threshold, prevents accidental triggering
      is_scrollbar_drag: false,
      scrollbar_direction_vertical: false,
    }
  }
}
