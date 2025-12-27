//! Portal module - Simplified API using Context and dynamic::each
//!
//! Provides a minimalist portal system with just 2 functions:
//! - `portal::provide()` - Create container and provide context
//! - `portal::child(view)` - Render view to portal with automatic cleanup
//!
//! Based on dynamic::each for efficient rendering and lifecycle management.

use crate::{Dynamic, ViewId, ViewTuple, consume_context, create_effect, each, provide_context};
use std::cell::RefCell;
use std::rc::Rc;

/// Portal item identifier
type PortalItemId = u64;

/// Portal item
#[derive(Clone)]
struct PortalItem {
  id: PortalItemId,
  node: crate::Node,
}

// Manual PartialEq implementation - compare by ID only
impl PartialEq for PortalItem {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

/// Portal Context (internal use only)
#[derive(Clone)]
struct PortalContext {
  target: Dynamic,
  items: crate::Signal<Vec<PortalItem>>,
}

impl PortalContext {
  fn new() -> Self {
    let items: crate::Signal<Vec<PortalItem>> = crate::Signal::new(Vec::new());

    // Use dynamic::each for efficient rendering
    let target = each(move || items.read().clone(), |item| item.node);

    PortalContext { target, items }
  }

  fn add_child(&self, node: crate::Node) -> PortalItemId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static ITEM_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    let id = ITEM_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    self.items.write().push(PortalItem { id, node });

    id
  }

  fn remove_child(&self, item_id: PortalItemId) {
    self.items.write().retain(|item| item.id != item_id);
  }
}

/// Create a portal container and provide context
///
/// This function creates a portal container, initializes the context,
/// and automatically provides it to child scopes.
///
/// # Returns
/// A View that serves as the portal container
///
/// # Examples
/// ```
/// use sig::portal;
///
/// fn app() -> View {
///     // Create portal container (only once at root level)
///     let modal_root = portal::provide();
///     
///     view().child((
///         main_content(),
///         modal_root,
///     ))
/// }
///
/// fn main_content() -> View {
///     button("Open Modal").on_click(|_| {
///         // Render to portal from anywhere
///         portal::child(modal_dialog());
///     })
/// }
/// ```
pub fn provide() -> crate::Dynamic {
  // Create and provide context
  let ctx = PortalContext::new();
  // Create container
  let container = ctx.target;
  provide_context(ctx);
  container
}

/// Render a view into the portal with automatic cleanup
///
/// This function consumes the portal context and renders the view into it.
/// The view is automatically removed when the current scope is destroyed.
///
/// # Arguments
/// * `content` - The view to render into the portal
///
/// # Panics
/// Panics if called outside a scope with portal context.
/// Make sure `portal::provide()` was called in a parent scope.
///
/// # Examples
/// ```
/// use sig::portal;
///
/// fn show_modal() {
///     // Render modal to portal
///     portal::child(
///         view()
///             .style(|s| s.padding(20.0).background(Color::WHITE))
///             .child(text("Modal Content"))
///     );
///     
///     // Automatically cleaned up when this scope is destroyed!
/// }
/// ```
pub fn child<VT: ViewTuple>(content: VT) {
  // Consume context
  let ctx = consume_context::<PortalContext>()
    .expect("portal::child() called without portal::provide() in parent scope");

  // Add child and get ID for cleanup
  let node = content.into_node();
  let item_id = ctx.add_child(node);

  // Auto cleanup when scope is destroyed
  crate::on_cleanup(move || {
    ctx.remove_child(item_id);
  });
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::create_scope;

  #[test]
  fn test_portal_provide() {
    create_scope(|| {
      let container = provide();

      // Context should be available
      let ctx = consume_context::<PortalContext>();
      assert!(ctx.is_some());
    });
  }

  #[test]
  fn test_portal_child() {
    create_scope(|| {
      let _container = provide();

      create_scope(|| {
        let view = crate::view().name("test-view");
        child(view);

        // Child should be added to portal
        let ctx = consume_context::<PortalContext>().unwrap();
        let items = ctx.items.borrow();
        assert_eq!(items.read().len(), 1);
      });
    });
  }

  #[test]
  fn test_portal_cleanup() {
    create_scope(|| {
      let _container = provide();

      let ctx_outer = consume_context::<PortalContext>().unwrap();

      {
        create_scope(|| {
          let view = crate::view().name("temp-view");
          child(view);
        });
        // Scope destroyed, cleanup should have run
      }

      // Give cleanup a chance to run
      std::thread::sleep(std::time::Duration::from_millis(10));

      // Child should be removed
      let items = ctx_outer.items.borrow();
      assert_eq!(items.read().len(), 0);
    });
  }
}
