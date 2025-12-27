//! ViewState - View state management
//!
//! Stores all state information for views

use crate::ViewId;
use crate::event::EventHandlers;
use crate::style::{Style, StyleBuilder, StyleProperty, StylePropertyKey};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// View state
///
/// Stores all runtime state information for a view
pub struct ViewState {
  /// View name
  pub name: String,
  /// Child view ID list
  /// Wrapped in Rc for sharing and efficient cloning
  pub children: Rc<Vec<ViewId>>,
  /// Parent view ID (cached for O(1) lookup)
  /// Updated when parent calls set_children_ids
  pub parent: Option<ViewId>,
  /// Whether mounted to layout tree
  pub mounted: bool,
  /// Whether needs redraw
  pub dirty: bool,
  /// Event handler collection (indexed by TypeId)
  pub event_handlers: EventHandlers,
  /// Render style (final computed style)
  pub style: Style,
  /// Style builder list (for style inheritance and computation)
  pub styles: Vec<Option<StyleBuilder>>,
  /// Style cache (for style inheritance)
  pub style_cache: HashMap<StylePropertyKey, StyleProperty>,
  /// Scroll offset (x, y)
  pub scroll_offset: (f32, f32),
  /// Vertical scrollbar rect (x, y, width, height) - for drag detection
  pub vertical_scrollbar_rect: Option<(f32, f32, f32, f32)>,
  /// Horizontal scrollbar rect (x, y, width, height) - for drag detection
  pub horizontal_scrollbar_rect: Option<(f32, f32, f32, f32)>,
  /// Content size (width, height) - for scrollbar calculation
  pub content_size: Option<(f32, f32)>,
  /// Container size (width, height) - for scrollbar calculation
  pub container_size: Option<(f32, f32)>,
  /// Absolute position (x, y) - cached from render pass for popper positioning
  /// This is the absolute screen position, accounting for all parent transforms and scroll offsets
  pub absolute_position: Option<(f32, f32)>,
  
  // ===== Dirty Bubbling Flags =====
  /// Whether this node's style is dirty (needs recomputation)
  pub style_dirty: bool,
  /// Whether subtree has dirty style nodes
  pub child_style_dirty: bool,
  
  // ===== Focus Management =====
  /// Whether this view can receive focus
  pub focusable: bool,
}

impl Default for ViewState {
  fn default() -> Self {
    Self::new()
  }
}

impl ViewState {
  /// Create new view state
  pub fn new() -> Self {
    ViewState {
      name: String::from("View"),
      children: Rc::new(Vec::new()),
      parent: None,
      mounted: false,
      dirty: false,
      event_handlers: EventHandlers::new(),
      style: Style::default(),
      styles: Vec::new(),
      style_cache: HashMap::new(),
      scroll_offset: (0.0, 0.0),
      vertical_scrollbar_rect: None,
      horizontal_scrollbar_rect: None,
      content_size: None,
      container_size: None,
      absolute_position: None,
      style_dirty: false,
      child_style_dirty: false,
      focusable: false,
    }
  }

  /// Create view state with name
  pub fn with_name(name: impl Into<String>) -> Self {
    let mut state = Self::new();
    state.name = name.into();
    state
  }
}

/// View state reference
///
/// Wraps ViewState in Rc<RefCell>, allowing shared and mutable state access
pub type ViewStateRef = Rc<RefCell<ViewState>>;
