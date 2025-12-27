//! Scroll Handler module - Uniformly handle scrolling logic at render layer
//!
//! Responsible for handling scrolling for any view with overflow: scroll/auto set

use crate::{ViewId, WheelEvent};
use glam::Vec2;

/// Handle wheel event scrolling
///
/// Traverse event path, find first scrollable view and update its scroll_offset
pub fn handle_wheel_scroll(path: &[ViewId], event: &mut WheelEvent) {
    for view_id in path.iter() {
        // Check if this view has overflow: scroll/auto
        let scroll_info = crate::runtime::with_layout(|runtime| {
            let node_id = view_id.node_id();

            // Get overflow property
            let (overflow_x, overflow_y) = if let Ok(style) = runtime.taffy.style(node_id) {
                (style.overflow.x, style.overflow.y)
            } else {
                return None;
            };

            // Check if scrollable
            let can_scroll_x = matches!(overflow_x, taffy::Overflow::Scroll);
            let can_scroll_y = matches!(overflow_y, taffy::Overflow::Scroll);

            // Skip if not scrollable
            if !can_scroll_x && !can_scroll_y {
                return None;
            }

            // Get container and content size to calculate max scroll offset
            let (max_x, max_y) = if let Ok(container_layout) = runtime.taffy.layout(node_id) {
                // Use content_size to get actual content dimensions
                let content_width = container_layout.content_size.width;
                let content_height = container_layout.content_size.height;
                let container_width = container_layout.size.width;
                let container_height = container_layout.size.height;

                let max_x = (content_width - container_width).max(0.0);
                let max_y = (content_height - container_height).max(0.0);
                (max_x, max_y)
            } else {
                (0.0, 0.0)
            };

            Some((can_scroll_x, can_scroll_y, max_x, max_y))
        });

        // If this view is scrollable, handle scrolling
        if let Some((can_scroll_x, can_scroll_y, max_offset_x, max_offset_y)) = scroll_info {
            // Check if scrolling actually occurred (not at boundary)
            let did_scroll = crate::runtime::with_layout_mut(|runtime| {
                if let Some(view_state) = runtime.view_states.get_mut(view_id) {
                    let (old_offset_x, old_offset_y) = view_state.scroll_offset;
                    let (mut new_offset_x, mut new_offset_y) = (old_offset_x, old_offset_y);

                    let mut scrolled_x = false;
                    let mut scrolled_y = false;

                    // Handle vertical scrolling
                    if can_scroll_y && event.delta_y.abs() > 0.001 {
                        new_offset_y -= event.delta_y;
                        new_offset_y = new_offset_y.max(0.0).min(max_offset_y);
                        scrolled_y = (new_offset_y - old_offset_y).abs() > 0.001;
                    }

                    // Handle horizontal scrolling
                    if can_scroll_x && event.delta_x.abs() > 0.001 {
                        new_offset_x -= event.delta_x;
                        new_offset_x = new_offset_x.max(0.0).min(max_offset_x);
                        scrolled_x = (new_offset_x - old_offset_x).abs() > 0.001;
                    }

                    // If scrolling occurred in either direction, update offset
                    if scrolled_x || scrolled_y {
                        view_state.scroll_offset = (new_offset_x, new_offset_y);
                        view_state.dirty = true;
                    }

                    scrolled_x || scrolled_y
                } else {
                    false
                }
            });

            // Only stop event propagation when scrolling actually occurred
            // If at boundary (no scrolling), continue propagating to outer layer
            if did_scroll {
                // Stop event propagation (handled)
                event.stop_propagation();

                // Trigger scroll event
                let scroll_event_data = crate::runtime::with_layout(|runtime| {
                    if let Some(view_state) = runtime.view_states.get(view_id) {
                        if let Ok(layout) = runtime.taffy.layout(view_id.node_id()) {
                            let (scroll_left, scroll_top) = view_state.scroll_offset;
                            let content_size = view_state.content_size.unwrap_or((0.0, 0.0));
                            let container_size = (layout.size.width, layout.size.height);

                            Some((
                                scroll_left,
                                scroll_top,
                                content_size.0,
                                content_size.1,
                                container_size.0,
                                container_size.1,
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                if let Some((scroll_left, scroll_top, scroll_width, scroll_height, client_width, client_height)) = scroll_event_data {
                    use crate::event::types::{ScrollEvent, ScrollData};

                    let mut scroll_event = ScrollEvent::new(*view_id, ScrollData {
                        scroll_left,
                        scroll_top,
                        scroll_width,
                        scroll_height,
                        client_width,
                        client_height,
                    });

                    // Dispatch directly to target view
                    crate::event::dispatch::dispatch_event(view_id, &mut scroll_event);
                }

                // Request redraw
                crate::runtime::with_window(|window_state| {
                    if let Some(window) = &window_state.redraw_requester {
                        window.request_redraw();
                    }
                });

                return;
            }
            // If no scrolling (at boundary), continue loop to check outer layer
        }
    }
}

/// Handle scrollbar dragging
///
/// Update scroll offset based on drag direction
///
/// # Parameters
/// - `view_id`: View ID of scroll container
/// - `delta`: Mouse movement delta
/// - `is_vertical`: true = vertical scrollbar, false = horizontal scrollbar
pub fn handle_scrollbar_drag(
    view_id: ViewId,
    delta: Vec2,
    is_vertical: bool,
) -> bool {
    let did_scroll = crate::runtime::with_layout_mut(|runtime| {
        if let Some(view_state) = runtime.view_states.get_mut(&view_id) {
            if is_vertical {
                // Handle vertical scrollbar drag
                if let Some((_content_w, content_h)) = view_state.content_size {
                    if let Some((_container_w, container_h)) = view_state.container_size {
                        let scrollable_height = content_h - container_h;
                        if scrollable_height > 0.0 {
                            // Calculate scrollbar movable range
                            let scrollbar_height = (container_h * container_h / content_h).max(20.0);
                            let track_height = container_h - 4.0;
                            let movable_range = track_height - scrollbar_height;

                            if movable_range > 0.0 {
                                let scroll_ratio = delta.y / movable_range;
                                let new_offset_y = (view_state.scroll_offset.1 + scroll_ratio * scrollable_height)
                                    .max(0.0)
                                    .min(scrollable_height);

                                view_state.scroll_offset.1 = new_offset_y;
                                view_state.dirty = true;
                                return true;
                            }
                        }
                    }
                }
            } else {
                // Handle horizontal scrollbar drag
                if let Some((content_w, _content_h)) = view_state.content_size {
                    if let Some((container_w, _container_h)) = view_state.container_size {
                        let scrollable_width = content_w - container_w;
                        if scrollable_width > 0.0 {
                            // Calculate scrollbar movable range
                            let scrollbar_width = (container_w * container_w / content_w).max(20.0);
                            let track_width = container_w - 4.0;
                            let movable_range = track_width - scrollbar_width;

                            if movable_range > 0.0 {
                                let scroll_ratio = delta.x / movable_range;
                                let new_offset_x = (view_state.scroll_offset.0 + scroll_ratio * scrollable_width)
                                 .max(0.0)
                                    .min(scrollable_width);

                                view_state.scroll_offset.0 = new_offset_x;
                                view_state.dirty = true;
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    });

    // Trigger scroll event (consistent with handle_wheel_scroll)
    if did_scroll {
        let scroll_event_data = crate::runtime::with_layout(|runtime| {
            if let Some(view_state) = runtime.view_states.get(&view_id) {
                if let Ok(layout) = runtime.taffy.layout(view_id.node_id()) {
                    let (scroll_left, scroll_top) = view_state.scroll_offset;
                    let content_size = view_state.content_size.unwrap_or((0.0, 0.0));
                    let container_size = (layout.size.width, layout.size.height);

                    Some((
                        scroll_left,
                        scroll_top,
                        content_size.0,
                        content_size.1,
                        container_size.0,
                        container_size.1,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some((scroll_left, scroll_top, scroll_width, scroll_height, client_width, client_height)) = scroll_event_data {
            use crate::event::types::{ScrollEvent, ScrollData};

            let mut scroll_event = ScrollEvent::new(view_id, ScrollData {
                scroll_left,
                scroll_top,
                scroll_width,
                scroll_height,
                client_width,
                client_height,
            });

            // Dispatch directly to target view
            crate::event::dispatch::dispatch_event(&view_id, &mut scroll_event);
        }
    }

    did_scroll
}

/// Check if mouse is on any scrollbar
///
/// # Returns
/// - `Some(true)`: On vertical scrollbar
/// - `Some(false)`: On horizontal scrollbar
/// - `None`: Not on scrollbar
pub fn check_scrollbar(view_id: ViewId, mouse_pos: Vec2) -> Option<bool> {
    crate::runtime::with_layout(|runtime| {
        if let Some(view_state) = runtime.view_states.get(&view_id) {
            // Check vertical scrollbar (priority)
            if let Some((x, y, w, h)) = view_state.vertical_scrollbar_rect {
                if mouse_pos.x >= x && mouse_pos.x <= x + w &&
                   mouse_pos.y >= y && mouse_pos.y <= y + h {
                    return Some(true); // Vertical scrollbar
                }
            }

            // Check horizontal scrollbar
            if let Some((x, y, w, h)) = view_state.horizontal_scrollbar_rect {
                if mouse_pos.x >= x && mouse_pos.x <= x + w &&
                   mouse_pos.y >= y && mouse_pos.y <= y + h {
                    return Some(false); // Horizontal scrollbar
                }
            }
        }

        None
    })
}

/// Check if mouse is on any scrollbar (simplified version for compatibility)
pub fn is_on_scrollbar(view_id: ViewId, mouse_pos: Vec2) -> bool {
    check_scrollbar(view_id, mouse_pos).is_some()
}
