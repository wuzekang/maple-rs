//! Resizable module - Resizable container component
//!
//! Provides a container that can be resized by dragging, supporting horizontal, vertical, or bidirectional resizing.
//!
//! # Examples
//!
//! ```rust
//! use sig::{Resizable, Signal, create_scope};
//!
//! create_scope(|| {
//!     let width = Signal::new(200.0);
//!     
//!     Resizable::horizontal(width.clone())
//!         .min_size(100.0)
//!         .max_size(400.0)
//!         .child(content);
//! });
//! ```

use crate::{Cursor, Element, Interactive, Signal, Styleable, View, ViewId, ViewTuple, view};
use vello::peniko::Color;

/// Resize direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeDirection {
    /// Horizontal direction (adjust width)
    Horizontal,
    /// Vertical direction (adjust height)
    Vertical,
    /// Both directions (adjust both width and height)
    Both,
}

/// Drag handle position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlePosition {
    /// Right side
    Right,
    /// Left side
    Left,
    /// Bottom
    Bottom,
    /// Top
    Top,
    /// Bottom-right corner
    BottomRight,
}

impl HandlePosition {
    /// Get default handle position based on direction
    fn default_for_direction(direction: ResizeDirection) -> Self {
        match direction {
            ResizeDirection::Horizontal => HandlePosition::Right,
            ResizeDirection::Vertical => HandlePosition::Bottom,
            ResizeDirection::Both => HandlePosition::BottomRight,
        }
    }
    
    /// Get cursor style for handle
    fn cursor(&self) -> Cursor {
        match self {
            HandlePosition::Right | HandlePosition::Left => Cursor::ResizeHorizontal,
            HandlePosition::Bottom | HandlePosition::Top => Cursor::ResizeVertical,
            HandlePosition::BottomRight => Cursor::ResizeNwSe,
        }
    }
}

/// Resizable container component
///
/// Resizable implements the Element trait, automatically providing:
/// - Styleable: All styling methods
/// - Interactive: All event handling methods
///
/// # Examples
///
/// ```rust
/// use sig::{create_scope, Resizable, Signal};
///
/// create_scope(|| {
///     let width = Signal::new(200.0);
///     
///     // Horizontal resize
///     Resizable::horizontal(width.clone())
///         .min_size(100.0)
///         .max_size(400.0)
///         .child(view());
/// });
/// ```
#[derive(Clone)]
pub struct Resizable {
    view: View,
    direction: ResizeDirection,
    size: Signal<f32>,
    min_size: f32,
    max_size: f32,
    handle_position: HandlePosition,
    handle_size: f32,
    content_node: Option<crate::Node>,
}

impl Resizable {
    /// Create a horizontally resizable container
    ///
    /// # Parameters
    /// - `width`: Width signal (can be modified by dragging)
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Resizable, Signal};
    ///
    /// create_scope(|| {
    ///     let width = Signal::new(200.0);
    ///     let r = Resizable::horizontal(width);
    /// });
    /// ```
    pub fn horizontal(width: Signal<f32>) -> Self {
        Self::new(ResizeDirection::Horizontal, width)
    }
    
    /// Create a vertically resizable container
    ///
    /// # Parameters
    /// - `height`: Height signal (can be modified by dragging)
    pub fn vertical(height: Signal<f32>) -> Self {
        Self::new(ResizeDirection::Vertical, height)
    }
    
    /// Create a generic resizable container
    ///
    /// # Parameters
    /// - `direction`: Resize direction
    /// - `size`: Size signal
    pub fn new(direction: ResizeDirection, size: Signal<f32>) -> Self {
        let view = view().name("Resizable");
        let handle_position = HandlePosition::default_for_direction(direction);
        
        Self {
            view,
            direction,
            size,
            min_size: 0.0,
            max_size: f32::MAX,
            handle_position,
            handle_size: 4.0,
            content_node: None,
        }
    }
    
    /// Set minimum size
    ///
    /// # Parameters
    /// - `min`: Minimum size in pixels
    pub fn min_size(mut self, min: f32) -> Self {
        self.min_size = min;
        self
    }
    
    /// Set maximum size
    ///
    /// # Parameters
    /// - `max`: Maximum size in pixels
    pub fn max_size(mut self, max: f32) -> Self {
        self.max_size = max;
        self
    }
    
    /// Set handle position
    pub fn handle_position(mut self, position: HandlePosition) -> Self {
        self.handle_position = position;
        self
    }
    
    /// Set handle size
    pub fn handle_size(mut self, size: f32) -> Self {
        self.handle_size = size;
        self
    }
    
    /// Set child content
    ///
    /// # Parameters
    /// - `content`: Child element
    pub fn child<VT: ViewTuple>(mut self, content: VT) -> Self {
        self.content_node = Some(content.into_node());
        self
    }
    
    /// Get current size
    pub fn current_size(&self) -> f32 {
        *self.size.read()
    }
}

/// Create resize handle
fn create_resize_handle(
    direction: ResizeDirection,
    size: Signal<f32>,
    min_size: f32,
    max_size: f32,
    handle_position: HandlePosition,
    handle_size: f32,
) -> View {
    let handle = view().name("ResizeHandle");
    
    // Initial size when drag starts
    let drag_start_size = Signal::new(0.0);
    
    // Handle style
    let cursor = handle_position.cursor();
    
    match direction {
        ResizeDirection::Horizontal => {
            handle
                .style(move |s| s
                    .width(handle_size)
                    .h_full()
                    .background(Color::from_rgba8(200, 200, 200, 128))
                    .cursor(cursor)
                    .position(taffy::Position::Absolute)
                    .right(0.0)
                    .top(0.0)
                )
                .on_drag_start(move |_event| {
                    *drag_start_size.write() = *size.read();
                })
                .on_drag(move |event| {
                    let delta = event.total_delta.x;
                    let new_size = (*drag_start_size.read() + delta).clamp(min_size, max_size);
                    *size.write() = new_size;
                });
        }
        ResizeDirection::Vertical => {
            handle
                .style(move |s| s
                    .height(handle_size)
                    .w_full()
                    .background(Color::from_rgba8(200, 200, 200, 128))
                    .cursor(cursor)
                    .position(taffy::Position::Absolute)
                    .bottom(0.0)
                    .left(0.0)
                )
                .on_drag_start(move |_event| {
                    *drag_start_size.write() = *size.read();
                })
                .on_drag(move |event| {
                    let delta = event.total_delta.y;
                    let new_size = (*drag_start_size.read() + delta).clamp(min_size, max_size);
                    *size.write() = new_size;
                });
        }
        ResizeDirection::Both => {
            handle
                .style(move |s| s
                    .width(handle_size * 2.0)
                    .height(handle_size * 2.0)
                    .background(Color::from_rgba8(200, 200, 200, 128))
                    .cursor(cursor)
                    .position(taffy::Position::Absolute)
                    .right(0.0)
                    .bottom(0.0)
                )
                .on_drag_start(move |_event| {
                    *drag_start_size.write() = *size.read();
                })
                .on_drag(move |event| {
                    let delta = event.total_delta.x;
                    let new_size = (*drag_start_size.read() + delta).clamp(min_size, max_size);
                    *size.write() = new_size;
                });
        }
    }
    
    handle
}

/// Implement Element trait for Resizable
impl Element for Resizable {
    fn id(&self) -> ViewId {
        self.view.id
    }

    fn name(&self) -> String {
        "Resizable".to_string()
    }

    fn build(self) -> crate::Node {
        let Self {
            view,
            direction,
            size,
            min_size,
            max_size,
            handle_position,
            handle_size,
            content_node,
        } = self;

        // Set default style with dynamic sizing
        let view = view.style(move |s| {
            let current_size = *size.read();  // 在样式闭包中读取 Signal
            let mut style = s.flex()
                .position(taffy::Position::Relative)
                .flex_shrink(0.0);  // 防止收缩
            
            // 根据方向设置尺寸
            match direction {
                ResizeDirection::Horizontal => {
                    style = style.width(current_size)
                        .flex_shrink(0.0)  // ✅ 改为 flex_shrink(0.0)，防止收缩但不伸展
                        .h_full();  // ✅ 使用 h_full() 填充高度
                }
                ResizeDirection::Vertical => {
                    style = style.height(current_size).w_full();
                }
                ResizeDirection::Both => {
                    style = style.width(current_size).height(current_size);
                }
            }
            style
        });

        // If there's content, create content container
        let view = if let Some(content_node) = content_node {
            let content_view = crate::view()
                .name("ResizableContent")
                .style(move |s| {
                    // 设置基础样式，根据方向设置高度/宽度填充
                    let mut style = s.flex().flex_col();
                    match direction {
                        ResizeDirection::Horizontal => {
                            style = style.w_full()
                                .h_full();  // ✅ 使用 h_full() 填充高度
                        }
                        ResizeDirection::Vertical => {
                            style = style.w_full().h_full();
                        }
                        ResizeDirection::Both => {
                            style = style.w_full().h_full();
                        }
                    }
                    style
                })
                .child(content_node);
            
            // Create drag handle
            let handle = create_resize_handle(
                direction,
                size,
                min_size,
                max_size,
                handle_position,
                handle_size,
            );
            
            // Combine content and handle based on handle position
            match handle_position {
                HandlePosition::Right | HandlePosition::Bottom | HandlePosition::BottomRight => {
                    view.child((content_view, handle))
                }
                HandlePosition::Left | HandlePosition::Top => {
                    view.child((handle, content_view))
                }
            }
        } else {
            view
        };

        view.into_node()
    }
}

/// Convenience function for creating a horizontally resizable container
///
/// # Examples
/// ```rust
/// use sig::{create_scope, resizable_horizontal, Signal};
///
/// create_scope(|| {
///     let width = Signal::new(200.0);
///     let r = resizable_horizontal(width);
/// });
/// ```
pub fn resizable_horizontal(width: Signal<f32>) -> Resizable {
    Resizable::horizontal(width)
}

/// Convenience function for creating a vertically resizable container
pub fn resizable_vertical(height: Signal<f32>) -> Resizable {
    Resizable::vertical(height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_scope, view};

    #[test]
    fn test_resizable_horizontal() {
        create_scope(|| {
            let width = Signal::new(200.0);
            let r = Resizable::horizontal(width.clone());
            assert_eq!(r.name(), "Resizable");
            assert_eq!(r.direction, ResizeDirection::Horizontal);
            assert_eq!(*width.read(), 200.0);
        });
    }

    #[test]
    fn test_resizable_vertical() {
        create_scope(|| {
            let height = Signal::new(300.0);
            let r = Resizable::vertical(height.clone());
            assert_eq!(r.direction, ResizeDirection::Vertical);
            assert_eq!(*height.read(), 300.0);
        });
    }

    #[test]
    fn test_resizable_with_constraints() {
        create_scope(|| {
            let width = Signal::new(200.0);
            let r = Resizable::horizontal(width)
                .min_size(100.0)
                .max_size(400.0);
            
            assert_eq!(r.min_size, 100.0);
            assert_eq!(r.max_size, 400.0);
        });
    }

    #[test]
    fn test_resizable_with_child() {
        create_scope(|| {
            let width = Signal::new(200.0);
            let _r = Resizable::horizontal(width)
                .child(view());
        });
    }

    #[test]
    fn test_convenience_functions() {
        create_scope(|| {
            let width = Signal::new(100.0);
            let height = Signal::new(200.0);
            
            let _h = resizable_horizontal(width);
            let _v = resizable_vertical(height);
        });
    }
}
