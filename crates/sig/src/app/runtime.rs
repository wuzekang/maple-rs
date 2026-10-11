//! Window system runtime - window state and redraw management

use std::cell::Cell;

thread_local! {
  /// True while the render pass is producing a frame. Damage discovered
  /// mid-render is already being drawn; requesting another frame from inside
  /// the pass made the loop self-sustaining (continuous vsync rendering).
  static RENDER_IN_FLIGHT: Cell<bool> = const { Cell::new(false) };
}

pub fn set_rendering(v: bool) {
  RENDER_IN_FLIGHT.with(|f| f.set(v));
}

pub fn is_rendering() -> bool {
  RENDER_IN_FLIGHT.with(|f| f.get())
}

/// RAII scope for the render-in-flight flag: redraw requests issued inside
/// are suppressed — the damage they report is part of the frame being
/// produced right now, so scheduling another frame would render the same
/// content twice. Drop restores the flag even on panic.
pub struct RenderingGuard;

impl RenderingGuard {
  pub fn new() -> Self {
    set_rendering(true);
    Self
  }
}

impl Default for RenderingGuard {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for RenderingGuard {
  fn drop(&mut self) {
    set_rendering(false);
  }
}

/// Request a window redraw. No-op while the render pass is in flight: the
/// damage being reported is part of the frame currently being produced.
thread_local! {
  /// A redraw request has already been forwarded to the window since the
  /// last delivered RedrawRequested. winit coalesces the requests anyway,
  /// but skipping the repeated window calls cuts ~1000 calls/frame down to
  /// one in restyle-heavy frames. Cleared when a RedrawRequested arrives
  /// (and on focus/resume so an occluded window can never stay gated).
  static REDRAW_GATE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Clear the redraw gate — called when a RedrawRequested is delivered.
pub fn clear_redraw_gate() {
  REDRAW_GATE.with(|g| g.set(false));
}

pub fn request_redraw() {
  if is_rendering() {
    return;
  }
  if REDRAW_GATE.with(|g| g.get()) {
    return; // already requested; winit will deliver one RedrawRequested
  }
  REDRAW_GATE.with(|g| g.set(true));
  // Cached: getenv takes a global lock and request_redraw fires per dirty
  // view — a per-call env read showed up as ~6% of wall time in profiles.
  static COUNTING: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
  if *COUNTING.get_or_init(|| std::env::var("SIG_FRAME_STATS").is_ok()) {
    REDRAW_REQUESTS.with(|c| c.set(c.get() + 1));
  }
  with_window(|window_state| {
    if let Some(window) = &window_state.redraw_requester {
      window.request_redraw();
    }
  });
}

thread_local! {
  static REDRAW_REQUESTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub fn redraw_request_count() -> usize {
  REDRAW_REQUESTS.with(|c| c.get())
}

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
