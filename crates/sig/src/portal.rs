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
  /// Store a factory function instead of the node directly
  /// This allows the node to be created in the portal context's scope
  factory: Rc<dyn Fn() -> crate::Node>,
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
    // The factory is called here, in the portal context's scope
    let target = each(move || items.read().clone(), |item| (item.factory)());

    PortalContext { target, items }
  }

  fn add_child(&self, factory: Rc<dyn Fn() -> crate::Node>) -> PortalItemId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static ITEM_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    let id = ITEM_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    self.items.write().push(PortalItem { id, factory });

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
/// The content factory will be called in the portal context's scope, ensuring
/// that any Signals created (e.g., from Dynamic) have the correct lifecycle.
///
/// # Arguments
/// * `content_factory` - A function that produces the view to render into the portal
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
///     // Render modal to portal using a closure
///     portal::child(move || {
///         dynamic(move || {
///             view()
///                 .style(|s| s.padding(20.0).background(Color::WHITE))
///                 .child(text("Modal Content"))
///         })
///     });
///     
///     // Automatically cleaned up when this scope is destroyed!
/// }
/// ```
pub fn child<F, VT>(content_factory: F)
where
  F: Fn() -> VT + 'static,
  VT: ViewTuple,
{
  // Consume context
  let ctx = consume_context::<PortalContext>()
    .expect("portal::child() called without portal::provide() in parent scope");

  // Wrap the factory to convert ViewTuple to Node
  let factory = Rc::new(move || content_factory().into_node());

  // Add child and get ID for cleanup
  let item_id = ctx.add_child(factory);

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
        // Use closure to create view
        child(move || crate::view().name("test-view"));

        // Child should be added to portal
        let ctx = consume_context::<PortalContext>().unwrap();
        assert_eq!(ctx.items.read().len(), 1);
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
          // Use closure to create view
          child(move || crate::view().name("temp-view"));
        });
        // Scope destroyed, cleanup should have run
      }

      // Give cleanup a chance to run
      std::thread::sleep(std::time::Duration::from_millis(10));

      // Child should be removed
      assert_eq!(ctx_outer.items.read().len(), 0);
    });
  }
}
