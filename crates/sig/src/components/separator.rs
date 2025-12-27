//! Separator component module
//!
//! Provides horizontal and vertical separators for visually dividing content areas.
//!
//! # Examples
//!
//! ```rust
//! use sig::{create_scope, Separator, VStack};
//!
//! create_scope(|| {
//!     VStack::new((
//!         text("Section 1"),
//!         Separator::horizontal(),
//!         text("Section 2"),
//!     ))
//! });
//! ```

use crate::{Element, Styleable, View, ViewId, ViewTuple, view};
use vello::peniko::Color;

/// Separator orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Horizontal separator
    Horizontal,
    /// Vertical separator
    Vertical,
}

/// Separator component
///
/// Separator implements the Element trait, automatically providing:
/// - Styleable: All styling methods
/// - Interactive: All event handling methods (though separators typically don't need interaction)
///
/// # Examples
///
/// ```rust
/// use sig::{create_scope, Separator};
///
/// create_scope(|| {
///     // Default horizontal separator
///     let h_sep = Separator::horizontal();
///     
///     // Vertical separator
///     let v_sep = Separator::vertical();
///     
///     // Custom styling
///     let custom = Separator::horizontal()
///         .style(|s| s
///             .background(Color::RED)
///             .height(2.0)
///             .margin_top(10.0)
///             .margin_bottom(10.0));
/// });
/// ```
#[derive(Clone)]
pub struct Separator {
    view: View,
    orientation: Orientation,
}

impl Separator {
    /// Create a horizontal separator
    ///
    /// Default style:
    /// - Height: 1px
    /// - Width: 100%
    /// - Color: Light gray (#E5E7EB)
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Separator};
    ///
    /// create_scope(|| {
    ///     let sep = Separator::horizontal();
    /// });
    /// ```
    pub fn horizontal() -> Self {
        let view = view().name("Separator");
        
        Self {
            view,
            orientation: Orientation::Horizontal,
        }
    }
    
    /// Create a vertical separator
    ///
    /// Default style:
    /// - Width: 1px
    /// - Height: 100%
    /// - Color: Light gray (#E5E7EB)
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Separator};
    ///
    /// create_scope(|| {
    ///     let sep = Separator::vertical();
    /// });
    /// ```
    pub fn vertical() -> Self {
        let view = view().name("Separator");
        
        Self {
            view,
            orientation: Orientation::Vertical,
        }
    }
    
    /// Get separator orientation
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}

/// Implement Element trait for Separator
///
/// This provides Separator with:
/// - Styleable: All styling methods (40+)
/// - Interactive: All event handling methods (10+)
/// - ViewTuple: Can be added as a child element to View (via blanket impl)
impl Element for Separator {
    fn id(&self) -> ViewId {
        self.view.id
    }

    fn name(&self) -> String {
        "Separator".to_string()
    }

    fn build(self) -> crate::Node {
        let Self { view, orientation } = self;
        
        // Apply default styles based on orientation
        let view = match orientation {
            Orientation::Horizontal => {
                view.style(|s| s
                    .height(1.0)
                    .w_full()
                    .background(Color::from_rgb8(229, 231, 235)) // gray-200
                )
            }
            Orientation::Vertical => {
                view.style(|s| s
                    .width(1.0)
                    .h_full()
                    .background(Color::from_rgb8(229, 231, 235)) // gray-200
                )
            }
        };
        
        view.into_node()
    }
}

/// Convenience function for creating a horizontal separator
///
/// # Examples
/// ```rust
/// use sig::{create_scope, separator};
///
/// create_scope(|| {
///     let sep = separator();
/// });
/// ```
pub fn separator() -> Separator {
    Separator::horizontal()
}

/// Convenience function for creating a horizontal separator (explicit name)
///
/// # Examples
/// ```rust
/// use sig::{create_scope, horizontal_separator};
///
/// create_scope(|| {
///     let sep = horizontal_separator();
/// });
/// ```
pub fn horizontal_separator() -> Separator {
    Separator::horizontal()
}

/// Convenience function for creating a vertical separator
///
/// # Examples
/// ```rust
/// use sig::{create_scope, vertical_separator};
///
/// create_scope(|| {
///     let sep = vertical_separator();
/// });
/// ```
pub fn vertical_separator() -> Separator {
    Separator::vertical()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_scope;

    #[test]
    fn test_horizontal_separator() {
        create_scope(|| {
            let sep = Separator::horizontal();
            assert_eq!(sep.orientation(), Orientation::Horizontal);
            assert_eq!(sep.name(), "Separator");
        });
    }

    #[test]
    fn test_vertical_separator() {
        create_scope(|| {
            let sep = Separator::vertical();
            assert_eq!(sep.orientation(), Orientation::Vertical);
            assert_eq!(sep.name(), "Separator");
        });
    }

    #[test]
    fn test_separator_with_style() {
        create_scope(|| {
            let _sep = separator()
                .style(|s| s
                    .background(Color::from_rgb8(255, 0, 0)) // Red
                    .height(2.0)
                    .margin_top(10.0));
        });
    }

    #[test]
    fn test_convenience_functions() {
        create_scope(|| {
            let _h1 = separator();
            let _h2 = horizontal_separator();
            let _v = vertical_separator();
        });
    }
}
