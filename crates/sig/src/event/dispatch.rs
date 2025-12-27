use glam::Vec2;
use crate::ViewId;
use super::hit_test::{hit_test, find_view_by_id};
use super::types::*;
use crate::event::handler::scroll;
use winit::window::Window;
use std::sync::Arc;

/// Dispatch generic events to the view tree
///
/// This is a new generic event dispatch function that supports arbitrary event types.
///
/// # Parameters
/// - `root`: Root view ID
/// - `event`: Event to dispatch (mutable reference)
///
/// # Workflow
/// 1. Perform hit testing (if needed)
/// 2. Build path from target to root
/// 3. Dispatch event along path with bubbling
/// 4. Call matching handlers at each node
pub fn dispatch_event<E: 'static>(root: &ViewId, event: &mut E) {
    // Special handling: check if hit testing is needed for Event<T> types
    // We need to handle different event types differently here

    // Since Rust doesn't have specialization, we use Any to check types
    use std::any::TypeId;

    let type_id = TypeId::of::<E>();

    // Check if it's a mouse event or wheel event (requires hit testing)
    if type_id == TypeId::of::<MouseEvent>() {
        dispatch_mouse_event(root, event);
    } else if type_id == TypeId::of::<WheelEvent>() {
        dispatch_wheel_event(root, event);
    } else if type_id == TypeId::of::<ScrollEvent>() {
        dispatch_scroll_event(event);
    } else if type_id == TypeId::of::<DragEvent>() {
        dispatch_drag_event(root, event);
    } else {
        // Other event types: dispatch directly to root node
        dispatch_to_root(root, event);
    }
}

/// Dispatch mouse events (requires hit testing)
fn dispatch_mouse_event<E: 'static>(root: &ViewId, event: &mut E) {
    // Use Any for type conversion
    if let Some(mouse_event) = (event as &mut dyn std::any::Any).downcast_mut::<MouseEvent>() {
        let position = mouse_event.position;
        
        if let Some(path) = hit_test(root, position, Vec2::ZERO) {
            // path is Target -> Root
            dispatch_along_path(&path, event);
        }
    }
}

/// Dispatch wheel events (requires hit testing + scroll handling)
fn dispatch_wheel_event<E: 'static>(root: &ViewId, event: &mut E) {
    if let Some(wheel_event) = (event as &mut dyn std::any::Any).downcast_mut::<WheelEvent>() {
        let position = wheel_event.position;

        if let Some(path) = hit_test(root, position, Vec2::ZERO) {
            // Handle scrolling first
            scroll::handle_wheel_scroll(&path, wheel_event);

            // If propagation stopped, don't continue
            if !wheel_event.propagation {
                return;
            }

            // Continue dispatching event
            dispatch_along_path(&path, event);
        }
    }
}

/// Dispatch scroll events (has target)
fn dispatch_scroll_event<E: 'static>(event: &mut E) {
    if let Some(scroll_event) = (event as &mut dyn std::any::Any).downcast_mut::<ScrollEvent>() {
        let target = scroll_event.target;

        // Set current target
        scroll_event.set_current(target);

        // Extract handler list first, then release runtime borrow
        let handlers = crate::runtime::with_layout(|runtime| {
            runtime.view_states
                .get(&target)
                .and_then(|state| {
                    let type_id = std::any::TypeId::of::<ScrollEvent>();
                    state.event_handlers.handlers.get(&type_id).cloned()
                })
        });

        // Now runtime borrow is released, can safely call handlers
        if let Some(handlers) = handlers {
            for handler in handlers {
                handler.borrow_mut()(event as &mut dyn std::any::Any);
            }
        }
    }
}

/// Dispatch drag events (has target)
fn dispatch_drag_event<E: 'static>(root: &ViewId, event: &mut E) {
    if let Some(drag_event) = (event as &mut dyn std::any::Any).downcast_mut::<DragEvent>() {
        let target = drag_event.target;

        // Find path from target to root
        if let Some(target_view) = find_view_by_id(root, target) {
            // Build path
            let path = build_path_to_root(root, target_view);
            dispatch_along_path(&path, event);
        }
    }
}

/// Dispatch event along path (bubbling)
fn dispatch_along_path<E: 'static>(path: &[ViewId], event: &mut E) {
    for view_id in path.iter() {
        // Set current target (if event supports it)
        set_current_if_supported(event, *view_id);

        // Check if should stop before dispatching
        let should_stop_before = check_propagation(event);
        if should_stop_before {
            break;
        }

        // Key: extract handler list first, then release runtime borrow
        let handlers = crate::runtime::with_layout(|runtime| {
            runtime.view_states
                .get(view_id)
                .and_then(|state| {
                    let type_id = std::any::TypeId::of::<E>();
                    state.event_handlers.handlers.get(&type_id).cloned()
                })
        });

        // Now runtime borrow is released, can safely call handlers
        if let Some(handlers) = handlers {
            for handler in handlers {
                handler.borrow_mut()(event as &mut dyn std::any::Any);
            }
        }

        // Check if should stop propagation after dispatch
        let should_stop_after = check_propagation(event);
        if should_stop_after {
            break;
        }
    }
}

/// Dispatch to root node (for events that don't need hit testing)
fn dispatch_to_root<E: 'static>(root: &ViewId, event: &mut E) {
    // Extract handler list first
    let handlers = crate::runtime::with_layout(|runtime| {
        runtime.view_states
            .get(root)
            .and_then(|state| {
                let type_id = std::any::TypeId::of::<E>();
                state.event_handlers.handlers.get(&type_id).cloned()
            })
    });

    // Call handlers after releasing borrow
    if let Some(handlers) = handlers {
        for handler in handlers {
            handler.borrow_mut()(event as &mut dyn std::any::Any);
        }
    }
}

/// Set current target (if event type supports it)
fn set_current_if_supported<E: 'static>(event: &mut E, view_id: ViewId) {
    use std::any::Any;

    let event_any = event as &mut dyn Any;

    // Try to convert to various event types and set current
    if let Some(e) = event_any.downcast_mut::<MouseEvent>() {
        e.set_current(view_id);
    } else if let Some(e) = event_any.downcast_mut::<WheelEvent>() {
        e.set_current(view_id);
    } else if let Some(e) = event_any.downcast_mut::<KeyboardEvent>() {
        e.set_current(view_id);
    } else if let Some(e) = event_any.downcast_mut::<DragEvent>() {
        e.set_current(view_id);
    } else if let Some(e) = event_any.downcast_mut::<ScrollEvent>() {
        e.set_current(view_id);
    }
    // Other event types can be added here
}

/// Check if event should stop propagation
fn check_propagation<E: 'static>(event: &E) -> bool {
    use std::any::Any;

    let event_any = event as &dyn Any;

    // Check propagation field of various event types
    if let Some(e) = event_any.downcast_ref::<MouseEvent>() {
        return !e.propagation;
    } else if let Some(e) = event_any.downcast_ref::<WheelEvent>() {
        return !e.propagation;
    } else if let Some(e) = event_any.downcast_ref::<KeyboardEvent>() {
        return !e.propagation;
    } else if let Some(e) = event_any.downcast_ref::<DragEvent>() {
        return !e.propagation;
    } else if let Some(e) = event_any.downcast_ref::<ScrollEvent>() {
        return !e.propagation;
    }

    false
}

/// Build path from specified view to root view
fn build_path_to_root(_root: &ViewId, target: ViewId) -> Vec<ViewId> {
    let path = vec![target];

    // Need to get parent-child relationship from runtime here
    // Simplified implementation: assume we have a way to get parent node
    // Actual implementation may need to be done in hit_test

    // TODO: Implement complete path building logic
    // Temporarily return path containing only target
    path
}

/// Directly dispatch event to specific view (for MouseEnter/MouseLeave)
///
/// # Note on Batching
/// This function does NOT batch effects. Batching should be done at the
/// system event level to ensure all related events are batched together.
pub fn dispatch_event_to_view<E: 'static>(target_id: ViewId, event: &mut E) {
    set_current_if_supported(event, target_id);

    // Extract handler list first, release runtime borrow
    let handlers = crate::runtime::with_layout(|runtime| {
        runtime.view_states
            .get(&target_id)
            .and_then(|state| {
                let type_id = std::any::TypeId::of::<E>();
                state.event_handlers.handlers.get(&type_id).cloned()
            })
    });

    // Now can safely call handlers
    if let Some(handlers) = handlers {
        for handler in handlers {
            handler.borrow_mut()(event as &mut dyn std::any::Any);
        }
    }
}

/// Update mouse cursor style
///
/// Update cursor style based on the view currently under the mouse
pub fn update_cursor(window: &Arc<Window>, view_id: ViewId) {
    use crate::event::cursor::Cursor;

    // Get cursor style from view state
    let cursor = crate::runtime::with_layout(|runtime| {
        runtime.view_states
            .get(&view_id)
            .map(|state| state.style.cursor)
            .unwrap_or(Cursor::Default)
    });

    // Set window cursor
    if let Some(winit_cursor) = cursor.to_winit_cursor() {
        window.set_cursor(winit_cursor);
    }
}
