//! Fragment module - Combine multiple values together without creating additional hierarchy

use crate::{Element, ViewTuple};

/// Fragment - Wraps multiple values together
///
/// # Examples
/// ```
/// use sig::fragment;
///
/// // Empty fragment
/// let f = fragment(());
///
/// // Single View
/// let f = fragment(view());
///
/// // Multiple elements
/// let f = fragment((view(), view(), view()));
/// ```
#[derive(Clone)]
pub struct Fragment {
  pub children: crate::Node,
}

impl Fragment {
  /// Create a new Fragment
  pub fn new<VT: ViewTuple>(children: VT) -> Self {
    Self {
      children: children.into_node(),
    }
  }

  /// Convert Fragment to ValueNode
  pub fn into_node(self) -> crate::Node {
    self.children
  }
}

/// Convenience function for creating a Fragment
///
/// # Examples
/// ```
/// use sig::fragment;
///
/// // Empty fragment
/// let f = fragment(());
///
/// // Single View
/// let f = fragment(view());
///
/// // Multiple elements
/// let f = fragment((view(), view(), view()));
/// ```
pub fn fragment<VT: ViewTuple>(children: VT) -> Fragment {
  Fragment::new(children)
}

// Implement ViewTuple for Fragment so it can be used directly in dynamic
impl ViewTuple for Fragment {
  fn into_node(self) -> crate::Node {
    self.children
  }
}

impl<T: Element> ViewTuple for Vec<T> {
  fn into_node(self) -> crate::Node {
    crate::Node::Fragment(
      self
        .into_iter()
        .map(|i| crate::Node::View(i.id()))
        .collect(),
    )
  }
}

impl From<Fragment> for crate::Node {
  fn from(frag: Fragment) -> Self {
    frag.children
  }
}
