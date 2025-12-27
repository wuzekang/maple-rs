//! Render context runtime - rendering state management

use std::cell::RefCell;

// ============================================================================
// Render context state
// ============================================================================

/// Render context state
pub struct RenderCtxState {
    pub current: crate::RenderContext,
}

impl RenderCtxState {
    pub fn new() -> Self {
        Self {
            current: crate::RenderContext::default(),
        }
    }
}

impl Default for RenderCtxState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Thread-local render context state
// ============================================================================

thread_local! {
    static RENDER_CTX_STATE: RefCell<RenderCtxState> = RefCell::new(RenderCtxState::new());
}

// ============================================================================
// Public API
// ============================================================================

/// Access render context state (immutable)
pub fn with_render_ctx<F, R>(f: F) -> R
where
    F: FnOnce(&RenderCtxState) -> R,
{
    RENDER_CTX_STATE.with(|state| f(&state.borrow()))
}

/// Access render context state (mutable)
pub fn with_render_ctx_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut RenderCtxState) -> R,
{
    RENDER_CTX_STATE.with(|state| f(&mut state.borrow_mut()))
}
