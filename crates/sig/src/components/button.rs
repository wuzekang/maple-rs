//! Button component module
//!
//! Provides an interactive button component with support for click events, hover states, and custom styling.
//!
//! # Design Philosophy
//!
//! The Button component follows Component Design Pattern A (Component as Wrapper Pattern):
//! - Holds an internal View
//! - Implements the Element trait
//! - Automatically gains Styleable and Interactive capabilities
//!
//! # Design Specification
//!
//! ## Tokens Used
//! - Height: `Size::SM/MD/LG` (24px/32px/40px) - Compact for desktop UIs
//! - Padding-X: `Spacing::SM/MD/LG` (8px/12px/16px)
//! - Border-Radius: `Radius::SM/MD` (2px/4px, adapts to size)
//! - Font-Size: `FontSize::SM/MD/LG` (12px/14px/16px)
//! - Font-Weight: 400 (Normal)
//!
//! ## Variants
//! - Size: Small (24px) | Medium (32px, default) | Large (40px)
//! - Style: Primary (default) | Secondary | Outline | Danger | Text
//!
//! ## States
//! - hover: background darken 10-15%, translate -2px
//! - press: background darken 20-25%, translate +1px
//! - disabled: opacity = 0.5, pointer-events none
//!
//! ## Deviations from Standard
//! - None (fully compliant with design guidelines)
//! - Optimized for code editors and desktop applications
//!
//! # Examples
//!
//! ```rust
//! use sig::{create_scope, button, ButtonVariant};
//!
//! create_scope(|| {
//!     // Can directly call .style() and .on_click(), no need for .build()
//!     let btn = button("Click Me")
//!         .variant(ButtonVariant::Primary)
//!         .style(|s| s.margin_top(10.0))  // Direct style methods!
//!         .on_click(|_| println!("Clicked"));  // Direct event methods!
//! });
//! ```

use crate::{
  Element, Interactive, Signal, Styleable, View, ViewId, ViewTuple, text,
  theme::{Border, FontSize, Opacity, Radius, Size, Spacing},
  view,
};
use vello::peniko::Color;

/// Button variant
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
  /// Primary button
  Primary,
  /// Secondary button
  Secondary,
  /// Outline button
  Outline,
  /// Danger button
  Danger,
  /// Text button
  Text,
}

impl Default for ButtonVariant {
  fn default() -> Self {
    ButtonVariant::Primary
  }
}

/// Button size
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
  /// Small
  Small,
  /// Medium (default)
  Medium,
  /// Large
  Large,
}

impl Default for ButtonSize {
  fn default() -> Self {
    ButtonSize::Medium
  }
}

impl ButtonSize {
  /// Get height using design tokens
  pub fn height(&self) -> f32 {
    match self {
      ButtonSize::Small => Size::SM,
      ButtonSize::Medium => Size::MD,
      ButtonSize::Large => Size::LG,
    }
  }

  /// Get horizontal padding using design tokens
  pub fn padding_x(&self) -> f32 {
    match self {
      ButtonSize::Small => Spacing::SM,
      ButtonSize::Medium => Spacing::MD,
      ButtonSize::Large => Spacing::LG,
    }
  }

  /// Get vertical padding (auto-calculated for vertical centering)
  pub fn padding_y(&self) -> f32 {
    match self {
      ButtonSize::Small => 4.0,
      ButtonSize::Medium => 6.0,
      ButtonSize::Large => 8.0,
    }
  }

  /// Get font size using design tokens
  pub fn font_size(&self) -> f32 {
    match self {
      ButtonSize::Small => FontSize::SM,
      ButtonSize::Medium => FontSize::MD,
      ButtonSize::Large => FontSize::LG,
    }
  }

  /// Get border radius using design tokens (adapts to size)
  pub fn border_radius(&self) -> f32 {
    match self {
      ButtonSize::Small => Radius::SM,
      ButtonSize::Medium => Radius::MD,
      ButtonSize::Large => Radius::MD,
    }
  }
}

/// Button color theme
pub struct ButtonColors {
  /// Background color
  pub background: Color,
  /// Hover background color
  pub hover_background: Color,
  /// Pressed background color
  pub pressed_background: Color,
  /// Text color
  pub text: Color,
  /// Border color
  pub border: Color,
}

impl ButtonVariant {
  /// Get button colors
  pub fn colors(&self) -> ButtonColors {
    match self {
      ButtonVariant::Primary => ButtonColors {
        background: Color::from_rgb8(59, 130, 246), // blue-500
        hover_background: Color::from_rgb8(37, 99, 235), // blue-600
        pressed_background: Color::from_rgb8(29, 78, 216), // blue-700
        text: Color::WHITE,
        border: Color::from_rgb8(59, 130, 246),
      },
      ButtonVariant::Secondary => ButtonColors {
        background: Color::from_rgb8(107, 114, 128), // gray-500
        hover_background: Color::from_rgb8(75, 85, 99), // gray-600
        pressed_background: Color::from_rgb8(55, 65, 81), // gray-700
        text: Color::WHITE,
        border: Color::from_rgb8(107, 114, 128),
      },
      ButtonVariant::Outline => ButtonColors {
        background: Color::TRANSPARENT,
        hover_background: Color::from_rgba8(59, 130, 246, 26), // blue-500 with low opacity
        pressed_background: Color::from_rgba8(59, 130, 246, 51), // blue-500 with medium opacity
        text: Color::from_rgb8(59, 130, 246),
        border: Color::from_rgb8(59, 130, 246),
      },
      ButtonVariant::Danger => ButtonColors {
        background: Color::from_rgb8(239, 68, 68),       // red-500
        hover_background: Color::from_rgb8(220, 38, 38), // red-600
        pressed_background: Color::from_rgb8(185, 28, 28), // red-700
        text: Color::WHITE,
        border: Color::from_rgb8(239, 68, 68),
      },
      ButtonVariant::Text => ButtonColors {
        background: Color::TRANSPARENT,
        hover_background: Color::from_rgba8(107, 114, 128, 26), // gray-500 with low opacity
        pressed_background: Color::from_rgba8(107, 114, 128, 51), // gray-500 with medium opacity
        text: Color::from_rgb8(107, 114, 128),
        border: Color::TRANSPARENT,
      },
    }
  }
}

/// Button state
#[derive(Clone, Copy)]
struct ButtonState {
  /// Is hovered
  hovered: Signal<bool>,
  /// Is pressed
  pressed: Signal<bool>,
  /// Is disabled
  disabled: Signal<bool>,
}

impl ButtonState {
  fn new() -> Self {
    let hovered = Signal::new(false);
    let pressed = Signal::new(false);
    let disabled = Signal::new(false);
    Self {
      hovered,
      pressed,
      disabled,
    }
  }
}

/// Button component
///
/// Button implements the Element trait, automatically providing:
/// - Styleable: All styling methods (40+)
/// - Interactive: All event handling methods (10+)
///
/// # Examples
///
/// ```rust
/// use sig::{button, create_scope, ButtonVariant, ButtonSize};
///
/// create_scope(|| {
///     // Component methods and View methods can be freely combined
///     let btn = button("Submit")
///         .variant(ButtonVariant::Primary)  // Component-specific method
///         .size(ButtonSize::Large)          // Component-specific method
///         .style(|s| s                      // View generic method
///             .width(200.0)
///             .margin_top(20.0))
///         .on_click(|_| {                   // View generic method
///             println!("Submitted!");
///         });
/// });
/// ```
#[derive(Clone)]
pub struct Button {
  view: View,
  state: ButtonState,
  variant: Signal<ButtonVariant>,
  size: Signal<ButtonSize>,
  label: String,
}

impl Button {
  /// Create a new button
  ///
  /// # Parameters
  /// - `label`: Button text
  ///
  /// # Examples
  /// ```
  /// use sig::{create_scope, button::Button};
  ///
  /// create_scope(|| {
  ///     let btn = Button::new("Click Me");
  /// });
  /// ```
  pub fn new<S: ToString>(label: S) -> Self {
    Self {
      view: view().name("Button"),
      state: ButtonState::new(),
      variant: Signal::new(ButtonVariant::default()),
      size: Signal::new(ButtonSize::default()),
      label: label.to_string(),
    }
  }

  /// Set button variant
  ///
  /// # Examples
  /// ```
  /// use sig::{button, ButtonVariant, create_scope};
  ///
  /// create_scope(|| {
  ///     let btn = button("Click").variant(ButtonVariant::Primary);
  /// });
  /// ```
  pub fn variant(self, variant: ButtonVariant) -> Self {
    *self.variant.write() = variant;
    self
  }

  /// Set button size
  ///
  /// # Examples
  /// ```
  /// use sig::{button, ButtonSize, create_scope};
  ///
  /// create_scope(|| {
  ///     let btn = button("Click").size(ButtonSize::Large);
  /// });
  /// ```
  pub fn size(self, size: ButtonSize) -> Self {
    *self.size.write() = size;
    self
  }

  /// Set disabled state
  ///
  /// # Examples
  /// ```
  /// use sig::{button, create_scope};
  ///
  /// create_scope(|| {
  ///     let btn = button("Click").disabled(true);
  /// });
  /// ```
  pub fn disabled(self, disabled: bool) -> Self {
    *self.state.disabled.write() = disabled;
    self
  }
}

/// Implement Element trait for Button
///
/// This provides Button with:
/// - Styleable: All styling methods (40+)
/// - Interactive: All event handling methods (10+)  
/// - ViewTuple: Can be added as a child element to View (via blanket impl)
impl Element for Button {
  fn id(&self) -> ViewId {
    self.view.id
  }

  fn name(&self) -> String {
    "Button".to_string()
  }

  fn build(self) -> crate::Node {
    let Self {
      view,
      state: ButtonState {
        hovered,
        pressed,
        disabled,
      },
      variant,
      size,
      label,
    } = self;

    view
      .style(move |s| {
        let variant_val = *variant.read();

        let size_val = *size.read();
        let colors = variant_val.colors();
        let is_hovered = *hovered.read();
        let is_pressed = *pressed.read();
        let is_disabled = *disabled.read();

        // Select background color based on state
        let bg = if is_disabled {
          colors.background
        } else if is_pressed {
          colors.pressed_background
        } else if is_hovered {
          colors.hover_background
        } else {
          colors.background
        };

        let mut style = s
          .height(size_val.height())
          .padding_left(size_val.padding_x())
          .padding_right(size_val.padding_x())
          .padding_top(size_val.padding_y())
          .padding_bottom(size_val.padding_y())
          .background(bg)
          .font_size(size_val.font_size())
          .font_weight(400) // Explicitly set to 400 (Normal)
          .border_radius(size_val.border_radius()) // Use size-adaptive radius
          .flex()
          .justify_center()
          .items_center()
          .text_center()
          .color(colors.text)
          // Set cursor based on disabled state (default to pointer)
          .cursor(if is_disabled {
            crate::Cursor::NotAllowed
          } else {
            crate::Cursor::Pointer
          });

        // Add border for Outline variant (use design token)
        if matches!(variant_val, ButtonVariant::Outline) {
          style = style.border_all(Border::THIN, colors.border);
        }

        // Hover effect
        if is_hovered && !is_disabled && !is_pressed {
          style = style.translate_y(-2.0);
        }

        // Press effect
        if is_pressed && !is_disabled {
          style = style.translate_y(1.0);
        }

        // Disabled state (use design token for opacity)
        if is_disabled {
          style = style.opacity(Opacity::DISABLED).pointer_events_none();
        }

        style
      })
      .on_mouse_enter(move |_| {
        *hovered.write() = true;
      })
      .on_mouse_leave(move |_| {
        *hovered.write() = false;
      })
      .on_mouse_down(move |_| {
        *pressed.write() = true;
      })
      .on_mouse_up(move |_| {
        *pressed.write() = false;
      })
      .child(
        // text
        text(label).style(move |s| {
          let colors = (*variant.read()).colors();
          s.color(colors.text)
        }),
      )
      .into_node()
  }
}

/// Convenience function for creating a button
///
/// # Examples
/// ```
/// use sig::{create_scope, button};
///
/// create_scope(|| {
///     let btn = button("Click Me");
/// });
/// ```
pub fn button<S: ToString>(label: S) -> Button {
  Button::new(label)
}

/// Create a primary button
pub fn primary_button<S: ToString>(label: S) -> Button {
  Button::new(label).variant(ButtonVariant::Primary)
}

/// Create a secondary button
pub fn secondary_button<S: ToString>(label: S) -> Button {
  Button::new(label).variant(ButtonVariant::Secondary)
}

/// Create an outline button
pub fn outline_button<S: ToString>(label: S) -> Button {
  Button::new(label).variant(ButtonVariant::Outline)
}

/// Create a danger button
pub fn danger_button<S: ToString>(label: S) -> Button {
  Button::new(label).variant(ButtonVariant::Danger)
}

/// Create a text button
pub fn text_button<S: ToString>(label: S) -> Button {
  Button::new(label).variant(ButtonVariant::Text)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::create_scope;

  #[test]
  fn test_button_creation() {
    create_scope(|| {
      let btn = button("Test");
      assert_eq!(btn.name(), "Button");
    });
  }

  #[test]
  fn test_button_variants() {
    create_scope(|| {
      let _primary = primary_button("Primary");
      let _secondary = secondary_button("Secondary");
      let _outline = outline_button("Outline");
      let _danger = danger_button("Danger");
      let _text = text_button("Text");
    });
  }

  #[test]
  #[test]
  fn test_button_disabled() {
    create_scope(|| {
      let btn = button("Disabled").disabled(true);
      assert_eq!(*btn.state.disabled.read(), true);
    });
  }

  #[test]
  fn test_button_with_style() {
    create_scope(|| {
      // Test new API: can directly call .style()
      let _btn = button("Styled")
        .variant(ButtonVariant::Primary)
        .style(|s| s.margin_top(20.0).width(200.0));
    });
  }
}
