//! Window system runtime - window state and redraw management

use std::cell::RefCell;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

use crate::ViewId;

// ============================================================================
// Window state
// ============================================================================

/// Window system state
pub struct WindowState {
    /// Global redraw requester
    pub redraw_requester: Option<Arc<Window>>,
    /// Start time for logging
    pub start_time: Instant,
    /// Focused view
    pub focused_view: Option<ViewId>,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            redraw_requester: None,
            start_time: Instant::now(),
            focused_view: None,
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Thread-local window state
// ============================================================================

thread_local! {
    static WINDOW_STATE: RefCell<WindowState> = RefCell::new(WindowState::new());
}

// ============================================================================
// Public API
// ============================================================================

/// Access window state (immutable)
pub fn with_window<F, R>(f: F) -> R
where
    F: FnOnce(&WindowState) -> R,
{
    WINDOW_STATE.with(|state| f(&state.borrow()))
}

/// Access window state (mutable)
pub fn with_window_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut WindowState) -> R,
{
    WINDOW_STATE.with(|state| f(&mut state.borrow_mut()))
}

// ============================================================================
// Focus Management
// ============================================================================

/// Set focused view
pub fn set_focus(view_id: ViewId) {
    let old_focus = with_window(|state| state.focused_view);
    with_window_mut(|state| state.focused_view = Some(view_id));
    
    // Request style update for both old and new focused views
    if let Some(old_view) = old_focus {
        old_view.request_style();
    }
    view_id.request_style();
}

/// Clear focus
pub fn clear_focus() {
    let old_focus = with_window(|state| state.focused_view);
    with_window_mut(|state| state.focused_view = None);
    
    // Request style update for old focused view
    if let Some(old_view) = old_focus {
        old_view.request_style();
    }
}

/// Get focused view
pub fn get_focused() -> Option<ViewId> {
    with_window(|state| state.focused_view)
}

/// Check if a view is focused
pub fn is_focused(view_id: ViewId) -> bool {
    with_window(|state| state.focused_view == Some(view_id))
}
