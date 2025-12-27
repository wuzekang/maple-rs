//! IntoElement module - Convert various types to ValueNode
//!
//! IntoElement trait provides a unified interface for converting different types (strings, numbers, etc)
//! to ValueNode. The difference from ViewTuple:
//!
//! - `ViewTuple`: Mainly for tuple decomposition and composition, flexible combination of multiple children
//! - `IntoElement`: For primitive type conversion (strings, numbers)
//!
//! # Use Cases
//!
//! ## 1. Text Content
//! ```rust
//! use sig::{view, IntoElement};
//!
//! // String literal
//! view().child("Hello World");
//!
//! // String type
//! let text = String::from("Dynamic text");
//! view().child(text);
//!
//! // Numbers
//! view().child(42);
//! ```
//!
//! ## 2. Dynamic Lists
//! ```rust
//! use sig::{view, IntoElement};
//!
//! let items = vec!["Item 1", "Item 2", "Item 3"];
//! view().child(items); // Vec auto conversion
//! ```

use crate::Node;

/// IntoElement trait - Convert values to ValueNode
///
/// This trait provides a unified interface for converting various primitive types to renderable ValueNode.
///
/// # Difference from ViewTuple
///
/// - `IntoElement`: Primitive types -> ValueNode (Into semantics)
/// - `ViewTuple`: Tuples/elements -> ValueNode (tuple deconstruction and element wrapping)
///
/// # Implementation Notes
///
/// Note: Do not implement blanket implementation for `Element` trait,
/// as it conflicts with ViewTuple blanket impl.
/// Element types should use ViewTuple for conversion.
///
/// # Examples
/// ```
/// use sig::{IntoElement, ValueNode};
///
/// // String conversion
/// let node: ValueNode = "Hello".into_element();
///
/// // Number conversion
/// let node: ValueNode = 42.into_element();
/// ```
pub trait IntoElement: Sized {
    /// Convert self to ValueNode
    fn into_element(self) -> Node;
}

// ============================================================================
// Core type implementations
// ============================================================================

/// ValueNode itself implements IntoElement (identity conversion)
impl IntoElement for Node {
    fn into_element(self) -> Node {
        self
    }
}

// ============================================================================
// Text type implementations
// ============================================================================

/// &str implements IntoElement - converts to Text component
impl IntoElement for &str {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// String implements IntoElement - converts to Text component
impl IntoElement for String {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

// ============================================================================
// Number type implementations
// ============================================================================

/// i32 implements IntoElement - converts to text display
impl IntoElement for i32 {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// i64 implements IntoElement - converts to text display
impl IntoElement for i64 {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// u32 implements IntoElement - converts to text display
impl IntoElement for u32 {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// u64 implements IntoElement - converts to text display
impl IntoElement for u64 {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// f32 implements IntoElement - converts to text display
impl IntoElement for f32 {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// f64 implements IntoElement - converts to text display
impl IntoElement for f64 {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

/// bool implements IntoElement - converts to text display ("true" / "false")
impl IntoElement for bool {
    fn into_element(self) -> Node {
        use crate::components::text::text;
        use crate::ViewTuple;
        text(self).into_node()
    }
}

// ============================================================================
// Collection type implementations
// ============================================================================

/// Vec<T> implements IntoElement - converts to Fragment
///
/// Note: T must implement IntoElement to avoid conflicts with ViewTuple
impl<T: IntoElement> IntoElement for Vec<T> {
    fn into_element(self) -> Node {
        Node::Fragment(
            self.into_iter()
                .map(|item| item.into_element())
                .collect()
        )
    }
}

/// Option<T> implements IntoElement - Some converts, None becomes empty Fragment
impl<T: IntoElement> IntoElement for Option<T> {
    fn into_element(self) -> Node {
        match self {
            Some(value) => value.into_element(),
            None => Node::Fragment(Vec::new()),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_into_element() {
        crate::create_scope(|| {
            let node = "Hello".into_element();
            // Should create a View node (Text component)
            match node {
                Node::View(_) => {},
                _ => panic!("Expected View node, got {:?}", node),
            }
        });
    }

    #[test]
    fn test_number_into_element() {
        crate::create_scope(|| {
            let node = 42.into_element();
            match node {
                Node::View(_) => {},
                _ => panic!("Expected View node"),
            }
        });
    }

    #[test]
    fn test_vec_into_element() {
        crate::create_scope(|| {
            let items = vec!["A", "B", "C"];
            let node = items.into_element();
            match node {
                Node::Fragment(children) => {
                    assert_eq!(children.len(), 3);
                },
                _ => panic!("Expected Fragment node"),
            }
        });
    }

    #[test]
    fn test_option_into_element() {
        crate::create_scope(|| {
            let some_node = Some("text").into_element();
            match some_node {
                Node::View(_) => {},
                _ => panic!("Expected View node"),
            }

            let none_node: Node = None::<&str>.into_element();
            match none_node {
                Node::Fragment(children) => {
                    assert_eq!(children.len(), 0);
                },
                _ => panic!("Expected empty Fragment"),
            }
        });
    }

    #[test]
    fn test_bool_into_element() {
        crate::create_scope(|| {
            let node = true.into_element();
            match node {
                Node::View(_) => {},
                _ => panic!("Expected View node"),
            }
        });
    }
}
