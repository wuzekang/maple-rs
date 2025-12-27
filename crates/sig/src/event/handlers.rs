//! Event Handlers - Event handler management
//!
//! Provides TypeId-based event handler storage and dispatch

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::interactive::EventHandler;

/// Type-erased event handler
///
/// Uses Rc<RefCell<>> to support cloning and mutable calls
pub type ErasedEventHandler = Rc<RefCell<dyn FnMut(&mut dyn Any)>>;

/// Event handler collection
///
/// Stores handlers for different event types grouped by TypeId
///
/// # Design
///
/// - Uses `HashMap<TypeId, Vec<Handler>>` for storage
/// - TypeId as key for O(1) lookup
/// - Supports arbitrary event type extension
/// - Uses Rc cloning to avoid borrowing conflicts
pub struct EventHandlers {
    pub(crate) handlers: HashMap<TypeId, Vec<ErasedEventHandler>>,
}

impl EventHandlers {
    /// Create new event handler collection
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Add type-safe event handler
    ///
    /// # Parameters
    /// - `handler`: Event handling function
    ///
    /// # Type Parameters
    /// - `E`: Event type (automatically inferred from handler)
    ///
    /// # Example
    /// ```
    /// use sig::event::handlers::EventHandlers;
    /// use sig::event::types::MouseEvent;
    ///
    /// let mut handlers = EventHandlers::new();
    /// handlers.add_handler(|event: &mut MouseEvent| {
    ///     println!("Mouse at {:?}", event.position);
    /// });
    /// ```
    pub fn add_handler<E>(&mut self, mut handler: impl EventHandler<E> + 'static)
    where
        E: 'static,
    {
        let type_id = TypeId::of::<E>();

        // Convert type-safe handler to type-erased version
        let erased: ErasedEventHandler = Rc::new(RefCell::new(move |event: &mut dyn Any| {
            if let Some(typed_event) = event.downcast_mut::<E>() {
                handler.call(typed_event);
            }
        }));

        self.handlers.entry(type_id).or_default().push(erased);
    }

    /// Dispatch event to all matching handlers
    ///
    /// # Parameters
    /// - `event`: Event to dispatch
    ///
    /// # Returns
    /// - `true`: Event was handled
    /// - `false`: No matching handler found
    ///
    /// # Notes
    ///
    /// This method clones the handler list to avoid borrowing conflicts,
    /// allowing handlers to access runtime (reactive system) internally
    pub fn dispatch<E>(&mut self, event: &mut E) -> bool
    where
        E: 'static,
    {
        let type_id = TypeId::of::<E>();

        if let Some(handlers) = self.handlers.get(&type_id) {
            // Clone handler list to avoid borrowing conflicts
            let handlers_clone: Vec<_> = handlers.iter().cloned().collect();

            for handler in handlers_clone {
                handler.borrow_mut()(event as &mut dyn Any);
            }
            true
        } else {
            false
        }
    }

    /// Clear all handlers
    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    /// Remove all handlers of specific type
    pub fn clear_type<E: 'static>(&mut self) {
        let type_id = TypeId::of::<E>();
        self.handlers.remove(&type_id);
    }
}

impl Default for EventHandlers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{MouseEvent, MouseData, MouseEventType};
    use crate::ViewId;

    #[test]
    fn test_add_and_dispatch() {
        let mut handlers = EventHandlers::new();
        let mut called = false;

        handlers.add_handler(|_event: &mut MouseEvent| {
            called = true;
        });

        let mut event = MouseEvent::new(ViewId::default(), MouseData {
            position: glam::Vec2::ZERO,
            button: None,
            state: winit::event::ElementState::Pressed,
            r#type: MouseEventType::Click,
        });

        let dispatched = handlers.dispatch(&mut event);

        assert!(dispatched);
        assert!(called);
    }

    #[test]
    fn test_multiple_handlers() {
        let mut handlers = EventHandlers::new();
        let mut count = 0;

        handlers.add_handler(|_event: &mut MouseEvent| {
            count += 1;
        });

        handlers.add_handler(|_event: &mut MouseEvent| {
            count += 1;
        });

        let mut event = MouseEvent::new(ViewId::default(), MouseData {
            position: glam::Vec2::ZERO,
            button: None,
            state: winit::event::ElementState::Pressed,
            r#type: MouseEventType::Click,
        });

        handlers.dispatch(&mut event);

        assert_eq!(count, 2);
    }

    #[test]
    fn test_type_isolation() {
        use crate::event::types::{WheelEvent, WheelData};

        let mut handlers = EventHandlers::new();
        let mut mouse_called = false;
        let mut wheel_called = false;

        handlers.add_handler(|_event: &mut MouseEvent| {
            mouse_called = true;
        });

        handlers.add_handler(|_event: &mut WheelEvent| {
            wheel_called = true;
        });

        // Dispatch mouse event
        let mut mouse_event = MouseEvent::new(ViewId::default(), MouseData {
            position: glam::Vec2::ZERO,
            button: None,
            state: winit::event::ElementState::Pressed,
            r#type: MouseEventType::Click,
        });
        handlers.dispatch(&mut mouse_event);

        assert!(mouse_called);
        assert!(!wheel_called);

        // Dispatch wheel event
        let mut wheel_event = WheelEvent::new(ViewId::default(), WheelData {
            position: glam::Vec2::ZERO,
            delta_x: 0.0,
            delta_y: 10.0,
        });
        handlers.dispatch(&mut wheel_event);

        assert!(wheel_called);
    }
}
