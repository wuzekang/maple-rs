//! ViewTuple module - Convert tuples to Node lists
//!
//! Similar to crates/ui's ViewTuple trait, used for flexible composition of multiple child elements

use crate::Node;

/// ViewTuple trait - Convert tuples or other types to Nodes
///
/// # Examples
/// ```
/// use sig::{ViewTuple, Node};
///
/// // Tuple nodes
/// let tuple_nodes = ViewTuple::into_node((view(), view(), view()));
/// ```
pub trait ViewTuple {
  /// Convert self to Node
  fn into_node(self) -> Node;
}

// Implement for Node itself
impl ViewTuple for Node {
  fn into_node(self) -> Node {
    self
  }
}

// Implement for Vec<Node>
impl ViewTuple for Vec<Node> {
  fn into_node(self) -> Node {
    Node::Fragment(self)
  }
}

// Implement for empty tuple
impl ViewTuple for () {
  fn into_node(self) -> Node {
    Node::Fragment(Vec::new())
  }
}

// Macro for implementing tuples
macro_rules! impl_view_tuple {
    ( $( $t:ident ),* ; $( $s:tt ),* ) => {
        impl< $( $t: ViewTuple, )* > ViewTuple for ( $( $t, )* ) {
            fn into_node(self) -> Node {
                Node::Fragment(vec![ $( self.$s.into_node(), )* ])
            }
        }
    }
}

// Support tuples with 1-8 elements
impl_view_tuple!(V0; 0);
impl_view_tuple!(V0, V1; 0, 1);
impl_view_tuple!(V0, V1, V2; 0, 1, 2);
impl_view_tuple!(V0, V1, V2, V3; 0, 1, 2, 3);
impl_view_tuple!(V0, V1, V2, V3, V4; 0, 1, 2, 3, 4);
impl_view_tuple!(V0, V1, V2, V3, V4, V5; 0, 1, 2, 3, 4, 5);
impl_view_tuple!(V0, V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5, 6);
impl_view_tuple!(V0, V1, V2, V3, V4, V5, V6, V7; 0, 1, 2, 3, 4, 5, 6, 7);

pub const VIEW_TUPLE_MAX_ELEMENTS: usize = 8;

// Blanket implementation: All Elements automatically implement ViewTuple
//
// This way any component implementing the Element trait (including View and Button) can:
// 1. Be added as a child element to View
// 2. Be used in tuples
// 3. Automatically convert to Node
impl<T: crate::Element> ViewTuple for T {
  fn into_node(self) -> Node {
    self.build()
  }
}
