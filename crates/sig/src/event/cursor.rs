//! Cursor module - Mouse pointer styles
//!
//! Provides mouse pointer style setting functionality

use winit::window::CursorIcon;

/// Mouse pointer style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cursor {
  /// Default pointer (inherits from parent element)
  Inherit,
  /// Default arrow
  Default,
  /// Hand pointer (for clickable elements)
  Pointer,
  /// Text cursor (I-beam)
  Text,
  /// Crosshair
  Crosshair,
  /// Move cursor
  Move,
  /// Not allowed
  NotAllowed,
  /// Wait/loading
  Wait,
  /// Help
  Help,
  /// Progress (arrow + wait)
  Progress,
  /// Grab (open palm)
  Grab,
  /// Grabbing (closed fist)
  Grabbing,
  /// Resize up
  ResizeNorth,
  /// Resize down
  ResizeSouth,
  /// Resize left
  ResizeWest,
  /// Resize right
  ResizeEast,
  /// Resize northeast
  ResizeNorthEast,
  /// Resize northwest
  ResizeNorthWest,
  /// Resize southeast
  ResizeSouthEast,
  /// Resize southwest
  ResizeSouthWest,
  /// Resize vertical
  ResizeVertical,
  /// Resize horizontal
  ResizeHorizontal,
  /// Bidirectional resize (diagonal)
  ResizeNeSw,
  /// Bidirectional resize (diagonal)
  ResizeNwSe,
  /// Column resize
  ColResize,
  /// Row resize
  RowResize,
  /// All-direction resize
  AllScroll,
  /// Zoom in
  ZoomIn,
  /// Zoom out
  ZoomOut,
  /// Cell selection
  Cell,
  /// Context menu
  ContextMenu,
  /// Copy
  Copy,
  /// Alias/shortcut
  Alias,
  /// No pointer (hidden)
  None,
}

impl Cursor {
  /// Convert to winit's CursorIcon
  pub fn to_winit_cursor(self) -> Option<CursorIcon> {
    match self {
      Cursor::Inherit => None, // Determined by parent element
      Cursor::Default => Some(CursorIcon::Default),
      Cursor::Pointer => Some(CursorIcon::Pointer),
      Cursor::Text => Some(CursorIcon::Text),
      Cursor::Crosshair => Some(CursorIcon::Crosshair),
      Cursor::Move => Some(CursorIcon::Move),
      Cursor::NotAllowed => Some(CursorIcon::NotAllowed),
      Cursor::Wait => Some(CursorIcon::Wait),
      Cursor::Help => Some(CursorIcon::Help),
      Cursor::Progress => Some(CursorIcon::Progress),
      Cursor::Grab => Some(CursorIcon::Grab),
      Cursor::Grabbing => Some(CursorIcon::Grabbing),
      Cursor::ResizeNorth => Some(CursorIcon::NResize),
      Cursor::ResizeSouth => Some(CursorIcon::SResize),
      Cursor::ResizeWest => Some(CursorIcon::WResize),
      Cursor::ResizeEast => Some(CursorIcon::EResize),
      Cursor::ResizeNorthEast => Some(CursorIcon::NeResize),
      Cursor::ResizeNorthWest => Some(CursorIcon::NwResize),
      Cursor::ResizeSouthEast => Some(CursorIcon::SeResize),
      Cursor::ResizeSouthWest => Some(CursorIcon::SwResize),
      Cursor::ResizeVertical => Some(CursorIcon::NsResize),
      Cursor::ResizeHorizontal => Some(CursorIcon::EwResize),
      Cursor::ResizeNeSw => Some(CursorIcon::NeswResize),
      Cursor::ResizeNwSe => Some(CursorIcon::NwseResize),
      Cursor::ColResize => Some(CursorIcon::ColResize),
      Cursor::RowResize => Some(CursorIcon::RowResize),
      Cursor::AllScroll => Some(CursorIcon::Move), // winit doesn't have AllScroll, use Move
      Cursor::ZoomIn => Some(CursorIcon::ZoomIn),
      Cursor::ZoomOut => Some(CursorIcon::ZoomOut),
      Cursor::Cell => Some(CursorIcon::Cell),
      Cursor::ContextMenu => Some(CursorIcon::ContextMenu),
      Cursor::Copy => Some(CursorIcon::Copy),
      Cursor::Alias => Some(CursorIcon::Alias),
      Cursor::None => Some(CursorIcon::Default), // winit doesn't have None, use Default
    }
  }

  /// Check if it's inherit
  pub fn is_inherit(&self) -> bool {
    matches!(self, Cursor::Inherit)
  }
}

impl Default for Cursor {
  fn default() -> Self {
    Cursor::Default
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cursor_default() {
    let cursor = Cursor::default();
    assert_eq!(cursor, Cursor::Default);
  }

  #[test]
  fn test_cursor_is_inherit() {
    assert!(Cursor::Inherit.is_inherit());
    assert!(!Cursor::Pointer.is_inherit());
  }

  #[test]
  fn test_cursor_to_winit() {
    assert!(Cursor::Pointer.to_winit_cursor().is_some());
    assert!(Cursor::Inherit.to_winit_cursor().is_none());
  }
}
