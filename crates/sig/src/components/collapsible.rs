//! Collapsible component module
//!
//! Provides an expandable/collapsible container, commonly used in tree structures, accordions, etc.
//!
//! # Design Specification
//!
//! ## Tokens Used (Recommended)
//! - Trigger-Content gap: `theme::Spacing::SM` (8px)
//! - Content indentation (TreeView): `theme::Spacing::LG` or `theme::Spacing::XL` (16px/20px)
//! - Item spacing: `theme::Spacing::XS` to `theme::Spacing::SM` (4px-8px)
//!
//! ## Component Structure
//! - Trigger: Clickable element that toggles open/closed state
//! - Content: Container shown/hidden based on `open` signal
//!
//! ## Deviations from Standard
//! - Component doesn't enforce specific spacing - users should apply via `.style()`
//! - This flexibility allows different use cases (TreeView, Accordion, etc.)
//!
//! # Examples
//!
//! ```rust
//! use sig::{create_scope, Collapsible, Signal, Button, Text};
//! use sig::theme::Spacing;
//!
//! create_scope(|| {
//!     let open = Signal::new(false);
//!     
//!     // Recommended: Add spacing between trigger and content
//!     Collapsible::new(open)
//!         .trigger(Button::new("Click to toggle"))
//!         .content(Text::new("Hidden content"))
//!         .style(|s| s.gap(Spacing::SM));  // 8px gap
//! });
//! ```

use crate::{Element, Interactive, Signal, Styleable, View, ViewId, ViewTuple, create_effect, view};

/// Collapsible component
///
/// Collapsible implements the Element trait, automatically providing:
/// - Styleable: All styling methods
/// - Interactive: All event handling methods
///
/// # Design Philosophy
///
/// Uses the Builder Pattern:
/// - `trigger`: Trigger element that toggles the expanded/collapsed state when clicked
/// - `content`: Content area that is shown/hidden based on the `open` state
///
/// # Examples
///
/// ```rust
/// use sig::{create_scope, Collapsible, Signal, Button, VStack, Text};
///
/// create_scope(|| {
///     let open = Signal::new(false);
///     
///     // Basic usage
///     Collapsible::new(open.clone())
///         .trigger(Button::new("Toggle"))
///         .content(Text::new("Content"));
///     
///     // Tree structure
///     fn tree_node(name: String) -> impl Element {
///         let open = Signal::new(false);
///         Collapsible::new(open)
///             .trigger(Button::new(name))
///             .content(
///                 VStack::new((
///                     Text::new("Child 1"),
///                     Text::new("Child 2"),
///                 ))
///             )
///     }
/// });
/// ```
#[derive(Clone)]
pub struct Collapsible {
    view: View,
    open: Signal<bool>,
    trigger_node: Option<crate::Node>,
    content_node: Option<crate::Node>,
}

impl Collapsible {
    /// Create a collapsible component
    ///
    /// # Parameters
    /// - `open`: Signal that controls the expanded/collapsed state
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Collapsible, Signal};
    ///
    /// create_scope(|| {
    ///     let open = Signal::new(false);
    ///     let collapsible = Collapsible::new(open);
    /// });
    /// ```
    pub fn new(open: Signal<bool>) -> Self {
        let view = view().name("Collapsible");
        
        Self {
            view,
            open,
            trigger_node: None,
            content_node: None,
        }
    }
    
    /// Set the trigger element
    ///
    /// The trigger is typically a button or clickable element that toggles the `open` state when clicked.
    ///
    /// # Parameters
    /// - `trigger`: Trigger element that can be any type implementing ViewTuple
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Collapsible, Signal, Button};
    ///
    /// create_scope(|| {
    ///     let open = Signal::new(false);
    ///     Collapsible::new(open)
    ///         .trigger(Button::new("Toggle"));
    /// });
    /// ```
    pub fn trigger<VT: ViewTuple>(mut self, trigger: VT) -> Self {
        self.trigger_node = Some(trigger.into_node());
        self
    }
    
    /// Set the content area
    ///
    /// The content area is shown/hidden based on the `open` state.
    ///
    /// # Parameters
    /// - `content`: Content element that can be any type implementing ViewTuple
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Collapsible, Signal, Text};
    ///
    /// create_scope(|| {
    ///     let open = Signal::new(false);
    ///     Collapsible::new(open)
    ///         .trigger(Button::new("Toggle"))
    ///         .content(Text::new("Hidden content"));
    /// });
    /// ```
    pub fn content<VT: ViewTuple>(mut self, content: VT) -> Self {
        self.content_node = Some(content.into_node());
        self
    }
    
    /// Get the current expanded state
    pub fn is_open(&self) -> bool {
        *self.open.read()
    }
    
    /// Set the expanded state
    pub fn set_open(&self, open: bool) {
        *self.open.write() = open;
    }
    
    /// Toggle the expanded/collapsed state
    pub fn toggle(&self) {
        let current = *self.open.read();
        *self.open.write() = !current;
    }
}

/// Implement Element trait for Collapsible
///
/// This provides Collapsible with:
/// - Styleable: All styling methods (40+)
/// - Interactive: All event handling methods (10+)
/// - ViewTuple: Can be added as a child element to View (via blanket impl)
impl Element for Collapsible {
    fn id(&self) -> ViewId {
        self.view.id
    }

    fn name(&self) -> String {
        "Collapsible".to_string()
    }

    fn build(self) -> crate::Node {
        let Self {
            view,
            open,
            trigger_node,
            content_node,
        } = self;

        // Set vertical layout
        let view = view.style(|s| s.flex().flex_col());

        // If there's a trigger, create the trigger view
        let view = if let Some(trigger_node) = trigger_node {
            let trigger_view = crate::view()
                .name("CollapsibleTrigger")
                .child(trigger_node)
                .on_click(move |_| {
                    let current = *open.read();
                    *open.write() = !current;
                });
            
            view.child(trigger_view)
        } else {
            view
        };

        // If there's content, create the content view
        let view = if let Some(content_node) = content_node {
            let content_view = crate::view()
                .name("CollapsibleContent")
                .child(content_node);
            
            // Reactively show/hide
            create_effect(move || {
                let is_open = *open.read();
                if is_open {
                    content_view.style(|s| s.display(taffy::Display::Flex));
                } else {
                    content_view.style(|s| s.display(taffy::Display::None));
                }
            });
            
            view.child(content_view)
        } else {
            view
        };

        view.into_node()
    }
}

/// Convenience function for creating a collapsible component
///
/// # Examples
/// ```rust
/// use sig::{create_scope, collapsible, Signal};
///
/// create_scope(|| {
///     let open = Signal::new(false);
///     let c = collapsible(open);
/// });
/// ```
pub fn collapsible(open: Signal<bool>) -> Collapsible {
    Collapsible::new(open)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{button, text, create_scope};

    #[test]
    fn test_collapsible_creation() {
        create_scope(|| {
            let open = Signal::new(false);
            let c = Collapsible::new(open);
            assert_eq!(c.name(), "Collapsible");
            assert!(!c.is_open());
        });
    }

    #[test]
    fn test_collapsible_state() {
        create_scope(|| {
            let open = Signal::new(false);
            let c = Collapsible::new(open);
            
            assert!(!c.is_open());
            
            c.set_open(true);
            assert!(c.is_open());
            
            c.toggle();
            assert!(!c.is_open());
            
            c.toggle();
            assert!(c.is_open());
        });
    }

    #[test]
    fn test_collapsible_with_trigger_and_content() {
        create_scope(|| {
            let open = Signal::new(false);
            let _c = Collapsible::new(open)
                .trigger(button("Toggle"))
                .content(text("Content"));
        });
    }

    #[test]
    fn test_collapsible_with_style() {
        use crate::theme::Spacing;
        
        create_scope(|| {
            let open = Signal::new(false);
            
            // Recommended spacing pattern using design tokens
            let _c = collapsible(open)
                .trigger(button("Toggle"))
                .content(text("Content"))
                .style(|s| s
                    .gap(Spacing::SM)       // 8px gap between trigger and content
                    .padding(Spacing::MD)   // 12px padding
                    .margin_top(Spacing::LG)); // 16px top margin
        });
    }

    #[test]
    fn test_convenience_function() {
        create_scope(|| {
            let open = Signal::new(true);
            let c = collapsible(open);
            assert!(c.is_open());
        });
    }
}
