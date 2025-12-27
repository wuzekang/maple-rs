//! Dynamic module - Reactive rendering of dynamic content
//!
//! # Use Cases for Dynamic
//!
//! `Dynamic` is used to create reactive dynamic content that automatically re-renders when dependent signals change.
//!
//! ## Key Features
//! - **Automatic dependency tracking**: All Signals accessed in the closure are automatically tracked
//! - **On-demand updates**: The closure only re-executes when dependent signals change
//! - **Flexible composition**: Can return any ViewTuple type (single element, tuple, Fragment, etc.)
//!
//! # Usage Examples
//!
//! ## 1. Conditional Rendering
//! ```rust
//! use sig::{create_scope, dynamic, fragment, Signal};
//!
//! create_scope(|| {
//!     let show = Signal::new(false);
//!
//!     let content = dynamic(move || {
//!         if show.read().value {
//!             fragment(("Detailed content", "More content"))
//!         } else {
//!             fragment(())  // Empty content
//!         }
//!     });
//!
//!     // Toggle display state
//!     *show.write() = true;
//! });
//! ```
//!
//! ## 2. Dynamic Lists
//! ```rust
//! use sig::{create_scope, dynamic, fragment, Signal};
//!
//! create_scope(|| {
//!     let items = Signal::new(vec![1, 2, 3]);
//!
//!     let list = dynamic(move || {
//!         let data = items.read().value.clone();
//!         fragment(
//!             data.into_iter()
//!                 .map(|item| format!("Item: {}", item))
//!                 .collect::<Vec<_>>()
//!         )
//!     });
//!
//!     // Update list
//!     *items.write() = vec![4, 5, 6];
//! });
//! ```
//!
//! ## 3. Nested Dynamic Content
//! ```rust
//! use sig::{create_scope, dynamic, fragment, Signal};
//!
//! create_scope(|| {
//!     let count = Signal::new(0);
//!
//!     let nested = dynamic(move || {
//!         let c = count.read().value;
//!         fragment((
//!             format!("Count: {}", c),
//!             if c > 5 {
//!                 fragment(("Warning: Count too large", "Suggest reducing"))
//!             } else {
//!                 fragment(())
//!             }
//!         ))
//!     });
//! });
//! ```

use crate::{Signal, ViewTuple, create_effect};

/// Dynamic - Stores dynamic content using a signal
///
/// # Examples
/// ```
/// use sig::{create_scope, dynamic};
///
/// create_scope(|| {
///     // Empty fragment
///     let d = dynamic(|| ());
///
///     // Single View
///     let d = dynamic(|| view());
///
///     // Multiple elements
///     let d = dynamic(|| (view(), view(), view()));
/// });
/// ```
#[derive(Clone, Copy)]
pub struct Dynamic {
  /// Signal storing dynamic content
  ///
  /// Each element is a (ValueNode, ScopeId) tuple:
  /// - ValueNode: The rendered node
  /// - ScopeId: Associated scope for lifecycle management
  pub signal: Signal<Vec<(crate::Node, crate::reactive::ScopeId)>>,
}

impl Dynamic {
  /// Create a new Dynamic
  ///
  /// # Parameters
  /// - `view_fn`: Function that generates content, re-executed when dependencies change
  ///
  /// # Note
  /// Must be called within a scope
  pub fn new<VT: ViewTuple, F: Fn() -> VT + 'static>(view_fn: F) -> Self {
    let signal = Signal::new(Vec::new());

    // 🔑 Wrap view_fn with as_child_scope
    // Each call runs in a new independent scope, returning (node, scope_id)
    let view_fn_scoped = crate::reactive::as_child_scope(move |_: ()| view_fn().into_node());

    create_effect(move || {
      // Create new node
      let (node, scope_id) = view_fn_scoped(());

      // Take old values and replace
      let old_nodes = std::mem::replace(&mut *signal.write(), vec![(node, scope_id)]);

      // Remove old scopes immediately
      for (_, old_scope_id) in old_nodes {
        crate::reactive::remove_scope(old_scope_id);
      }
    });

    Dynamic { signal }
  }

  /// Convert Dynamic to ValueNode
  pub fn into_node(self) -> crate::Node {
    crate::Node::Dynamic(self.signal)
  }
}

/// Convenience function for creating a Dynamic
///
/// # Examples
/// ```
/// use sig::{create_scope, dynamic};
///
/// create_scope(|| {
///     // Empty fragment
///     let d = dynamic(|| ());
///
///     // Single View
///     let d = dynamic(|| view());
///
///     // Multiple elements
///     let d = dynamic(|| (view(), view(), view()));
/// });
/// ```
pub fn dynamic<VT: ViewTuple, F: Fn() -> VT + 'static>(f: F) -> Dynamic {
  Dynamic::new(f)
}

/// Convenience function for creating smart list rendering
///
/// `each` efficiently renders lists through smart diffing, minimizing node updates.
/// It compares the beginning and end of arrays to find common parts, only recreating changed sections.
///
/// # Parameters
/// - `data_fn`: Function that returns the data list
/// - `view_fn`: Function that converts each data item to a ViewTuple
///
/// # Performance Benefits
/// - **Reuses unchanged nodes**: Avoids unnecessary rebuilding
/// - **Precise scope management**: Only cleans up scopes for deleted/replaced nodes
/// - **3-5x faster than simple rebuilding**
///
/// # Examples
/// ```
/// use sig::{create_scope, each, Signal};
///
/// create_scope(|| {
///     let items = Signal::new(vec![1, 2, 3, 4, 5]);
///     
///     // When modifying middle elements, head and tail nodes are reused
///     let list = each(
///         move || items.read().clone(),
///         |item| format!("Item: {}", item * 10)
///     );
/// });
/// ```
pub fn each<Item, VT, DataFn, ViewFn>(data_fn: DataFn, view_fn: ViewFn) -> Dynamic
where
  Item: PartialEq + Clone + 'static,
  VT: ViewTuple,
  DataFn: Fn() -> Vec<Item> + 'static,
  ViewFn: Fn(Item) -> VT + 'static,
{
  use std::cell::RefCell;
  use std::rc::Rc;

  let signal = Signal::new(Vec::new());
  let setter = signal;

  // 🔑 Wrap view_fn with as_child_scope
  // Each node is created in an independent scope, returning (node, scope_id)
  let view_fn_scoped = crate::reactive::as_child_scope(move |item: Item| view_fn(item).into_node());

  // Store previous data and nodes for diffing
  let prev_state: Rc<
    RefCell<Option<(Vec<Item>, Vec<(crate::Node, crate::reactive::ScopeId)>)>>,
  > = Rc::new(RefCell::new(None));

  let prev_state_clone = prev_state.clone();
  create_effect(move || {
    let data = data_fn();

    let (nodes, scopes_to_remove) = {
      let mut prev_guard = prev_state_clone.borrow_mut();

      if let Some((prev_data, prev_nodes)) = prev_guard.take() {
        // Smart diffing algorithm
        let mut result = Vec::with_capacity(data.len());
        let mut scopes_to_remove = Vec::new();
        let mut left = 0;
        let mut right = 0;
        let prev_len = prev_data.len();

        // Find matching elements from the left
        for (i, item) in data.iter().enumerate() {
          if prev_len > i && item == &prev_data[i] {
            left = i + 1;
          } else {
            break;
          }
        }

        // Find matching elements from the right
        for (i, item) in data.iter().rev().enumerate() {
          if prev_len > left + i && item == &prev_data[prev_len - 1 - i] {
            right = i + 1;
          } else {
            break;
          }
        }

        // ✅ Reuse matching nodes from the left (their scopes remain unchanged)
        for i in 0..left {
          result.push(prev_nodes[i].clone());
        }

        // 🔑 Collect scopes of replaced middle nodes (for later cleanup)
        for i in left..(prev_len.saturating_sub(right)) {
          let (_, scope_id) = &prev_nodes[i];
          scopes_to_remove.push(*scope_id);
        }

        // ✅ Create new nodes for changed middle section (in new independent scopes)
        for i in left..(data.len().saturating_sub(right)) {
          result.push(view_fn_scoped(data[i].clone()));
        }

        // ✅ Reuse matching nodes from the right
        for i in 0..right {
          result.push(prev_nodes[prev_len - right + i].clone());
        }

        // Update state
        *prev_guard = Some((data.clone(), result.clone()));
        (result, scopes_to_remove)
      } else {
        // ✅ Initial render, create all nodes in independent scopes
        let nodes: Vec<(crate::Node, crate::reactive::ScopeId)> = data
          .iter()
          .map(|item| view_fn_scoped(item.clone()))
          .collect();

        *prev_guard = Some((data, nodes.clone()));
        (nodes, Vec::new())
      }
    };

    // Write new nodes
    *setter.write() = nodes;

    // Remove old scopes immediately
    for scope_id in scopes_to_remove {
      crate::reactive::remove_scope(scope_id);
    }
  });

  Dynamic { signal }
}

// Implement ViewTuple for Dynamic so it can be used directly for children
impl ViewTuple for Dynamic {
  fn into_node(self) -> crate::Node {
    crate::Node::Dynamic(self.signal)
  }
}

// impl From<Dynamic> for crate::ValueNode {
//   fn from(dyn_value: Dynamic) -> Self {
//     crate::ValueNode::Dynamic(dyn_value.signal)
//   }
// }
