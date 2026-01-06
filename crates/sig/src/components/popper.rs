//! Popper component module
//!
//! Provides a popper component for positioning content relative to a reference element.
//! Commonly used for tooltips, dropdowns, popovers, and context menus.
//!
//! # Design Philosophy
//!
//! The Popper component follows a portal-based approach:
//! - Uses `portal::child()` to render content in a portal container
//! - Automatically positions content relative to the reference element
//! - Supports multiple placement options (top, bottom, left, right, etc.)
//! - Provides offset and alignment customization
//!
//! # Features
//!
//! - **Automatic positioning**: Calculates position based on reference element's layout
//! - **Multiple placements**: Top, Bottom, Left, Right with Start/Center/End alignment
//! - **Offset support**: Customize distance from reference element
//! - **Portal rendering**: Content rendered in a separate layer for z-index control
//! - **Reactive updates**: Automatically updates position when layout changes
//!
//! # Examples
//!
//! ```rust
//! use sig::{popper, Placement, view, text, Signal};
//!
//! let show_tooltip = Signal::new(false);
//!
//! // ✅ CORRECT: Non-interactive tooltip with pointer_events_none
//! let reference = view()
//!     .on_mouse_enter(move |_| *show_tooltip.write() = true)
//!     .on_mouse_leave(move |_| *show_tooltip.write() = false)
//!     .child(text("Hover me"));
//!
//! let content = view()
//!     .style(|s| s.pointer_events_none())  // ← ESSENTIAL!
//!     .child(text("Tooltip content"));
//!
//! popper()
//!   .reference(reference.id())
//!   .content(content)
//!   .placement(Placement::Top)
//!   .open(show_tooltip)
//!   .offset(8.0);
//!
//! // ❌ WRONG: Missing pointer_events_none() causes event loop
//! let bad_content = view().child(text("This will break!"));
//! ```

use crate::{
  Element, Node, Signal, Styleable, ViewId, ViewTuple, dynamic, fragment, portal,
  style::dimension::percent, view,
};

/// Popper placement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
  /// Top center
  Top,
  /// Top start (left aligned)
  TopStart,
  /// Top end (right aligned)
  TopEnd,
  /// Bottom center
  Bottom,
  /// Bottom start (left aligned)
  BottomStart,
  /// Bottom end (right aligned)
  BottomEnd,
  /// Left center
  Left,
  /// Left start (top aligned)
  LeftStart,
  /// Left end (bottom aligned)
  LeftEnd,
  /// Right center
  Right,
  /// Right start (top aligned)
  RightStart,
  /// Right end (bottom aligned)
  RightEnd,
}

impl Default for Placement {
  fn default() -> Self {
    Placement::Bottom
  }
}

/// Position information for popper content
#[derive(Debug, Clone, Copy)]
struct PopperPosition {
  x: f32,
  y: f32,
}

impl Default for PopperPosition {
  fn default() -> Self {
    Self { x: 0.0, y: 0.0 }
  }
}

/// Calculate position based on reference element layout and absolute position
fn calculate_position(
  ref_layout: taffy::Layout,
  ref_abs_pos: (f32, f32),
  placement: Placement,
  offset: f32,
) -> PopperPosition {
  // Use absolute position instead of relative position
  let ref_x = ref_abs_pos.0;
  let ref_y = ref_abs_pos.1;
  let ref_width = ref_layout.size.width;
  let ref_height = ref_layout.size.height;

  let (x, y) = match placement {
    // Top placements
    Placement::Top => (ref_x + ref_width / 2.0, ref_y - offset),
    Placement::TopStart => (ref_x, ref_y - offset),
    Placement::TopEnd => (ref_x + ref_width, ref_y - offset),

    // Bottom placements
    Placement::Bottom => (ref_x + ref_width / 2.0, ref_y + ref_height + offset),
    Placement::BottomStart => (ref_x, ref_y + ref_height + offset),
    Placement::BottomEnd => (ref_x + ref_width, ref_y + ref_height + offset),

    // Left placements
    Placement::Left => (ref_x - offset, ref_y + ref_height / 2.0),
    Placement::LeftStart => (ref_x - offset, ref_y),
    Placement::LeftEnd => (ref_x - offset, ref_y + ref_height),

    // Right placements
    Placement::Right => (ref_x + ref_width + offset, ref_y + ref_height / 2.0),
    Placement::RightStart => (ref_x + ref_width + offset, ref_y),
    Placement::RightEnd => (ref_x + ref_width + offset, ref_y + ref_height),
  };

  PopperPosition { x, y }
}

/// Popper component builder
///
/// A positioning utility that renders content relative to a reference element.
///
/// # Examples
///
/// ```rust
/// use sig::{popper, Placement, view, text, button, Signal};
///
/// let open = Signal::new(false);
/// let reference = button("Click me")
///   .on_click(move |_| *open.write() = !*open.read());
///
/// let content = view()
///   .style(|s| s.padding(10.0).background(Color::WHITE))
///   .child(text("Dropdown content"));
///
/// popper()
///   .reference(reference)
///   .content(content)
///   .placement(Placement::BottomStart)
///   .open(open)
///   .offset(4.0);
/// ```
#[derive(Clone)]
pub struct Popper<T: Element> {
  /// Reference element
  reference: T,
  /// Popper content
  content: Signal<Option<crate::Node>>,
  /// Placement
  placement: Signal<Placement>,
  /// Offset from reference element
  offset: Signal<f32>,
  /// Whether popper is open
  open: Signal<bool>,
}

impl<T: Element> Popper<T> {
  /// Create a new popper
  pub fn new(reference: T) -> Self {
    Self {
      reference: reference,
      content: Signal::new(None),
      placement: Signal::new(Placement::default()),
      offset: Signal::new(8.0),
      open: Signal::new(true),
    }
  }

  /// Set popper content
  pub fn content<VT: ViewTuple>(self, content: VT) -> Self {
    *self.content.write() = Some(content.into_node());
    self
  }

  /// Set placement
  pub fn placement(self, placement: Placement) -> Self {
    *self.placement.write() = placement;
    self
  }

  /// Set offset
  pub fn offset(self, offset: f32) -> Self {
    *self.offset.write() = offset;
    self
  }

  /// Set open state
  pub fn open(mut self, open: Signal<bool>) -> Self {
    self.open = open;
    self
  }

  /// Build the popper and render to portal
  pub fn build_to_portal(self) -> Node {
    let Self {
      reference,
      content,
      placement,
      offset,
      open,
    } = self;

    let ref_id = reference.id();

    // Render popper content to portal
    portal::child(dynamic(move || {
      if !*open.read() {
        return fragment(());
      }

      let content_node = (*content.read()).clone();

      if content_node.is_none() {
        return fragment(());
      }

      let content_node = content_node.unwrap();

      // Get reference element layout
      let ref_layout = ref_id.layout();
      if ref_layout.is_none() {
        return fragment(());
      }

      let absolute_position = ref_id.absolute_position();
      if absolute_position.is_none() {
        return fragment(());
      }

      let absolute_position = absolute_position.unwrap();

      let ref_layout = ref_layout.unwrap();
      let placement_val = *placement.read();
      let offset_val = *offset.read();

      // Calculate position
      let pos = calculate_position(
        ref_layout,
        (absolute_position.0, absolute_position.1),
        placement_val,
        offset_val,
      );

      println!(
        "[Popper] Reference: x={}, y={}, w={}, h={}",
        ref_layout.location.x, ref_layout.location.y, ref_layout.size.width, ref_layout.size.height
      );
      println!("[Popper] Calculated position: ({}, {})", pos.x, pos.y);

      // Calculate alignment transform (as decimal: 0.5 = 50%, 1.0 = 100%)
      let align_x = match placement_val {
        Placement::TopStart | Placement::BottomStart => 0.0,
        Placement::Top | Placement::Bottom => -0.5, // -50%
        Placement::TopEnd | Placement::BottomEnd => -1.0, // -100%
        Placement::Left | Placement::LeftStart | Placement::LeftEnd => -1.0, // -100%
        _ => 0.0,
      };

      let align_y = match placement_val {
        Placement::LeftStart | Placement::RightStart => 0.0,
        Placement::Left | Placement::Right => -0.5, // -50%
        Placement::LeftEnd | Placement::RightEnd => -1.0, // -100%
        Placement::Top | Placement::TopStart | Placement::TopEnd => -1.0, // -100%
        _ => 0.0,
      };

      println!(
        "[Popper] Alignment: translate_x={}, translate_y={}",
        align_x, align_y
      );

      fragment(
        view()
          .style(move |s| {
            s.position(taffy::Position::Absolute)
              .left(pos.x)
              .top(pos.y)
              .translate_x(percent(align_x))
              .translate_y(percent(align_y))
              .pointer_events_none()
              .flex()
              .flex_col()
          })
          .child(content_node),
      )
    }));

    reference.build()
  }
}

impl<T: Element> Element for Popper<T>
where
  T: Element,
{
  fn id(&self) -> ViewId {
    self.reference.id()
  }

  fn build(self) -> crate::Node {
    self.build_to_portal()
  }
}

/// Convenience function for creating a popper
///
/// # Examples
/// ```rust
/// use sig::{popper, Placement};
///
/// let pop = popper()
///   .placement(Placement::Top)
///   .offset(10.0);
/// ```
pub fn popper<T: Element>(reference: T) -> Popper<T> {
  Popper::new(reference)
}
