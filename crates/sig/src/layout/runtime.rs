//! Layout system runtime - taffy layout state management

use std::cell::RefCell;

pub use super::LayoutState;

// ============================================================================
// Thread-local layout state
// ============================================================================

thread_local! {
    static LAYOUT_STATE: RefCell<LayoutState> = RefCell::new(LayoutState::new());
}

// ============================================================================
// Public API
// ============================================================================

/// Access layout state (immutable)
pub fn with_layout<F, R>(f: F) -> R
where
    F: FnOnce(&LayoutState) -> R,
{
    LAYOUT_STATE.with(|state| f(&state.borrow()))
}

/// Access layout state (mutable)
pub fn with_layout_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut LayoutState) -> R,
{
    LAYOUT_STATE.with(|state| f(&mut state.borrow_mut()))
}
