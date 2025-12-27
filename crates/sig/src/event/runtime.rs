//! Input system runtime - keyboard and IME state management

use std::cell::RefCell;

// ============================================================================
// Input state
// ============================================================================

/// Input system state
pub struct InputState {
    // Empty for now, reserved for future keyboard/IME state
}

impl InputState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Thread-local input state
// ============================================================================

thread_local! {
    static INPUT_STATE: RefCell<InputState> = RefCell::new(InputState::new());
}

// ============================================================================
// Public API
// ============================================================================

/// Access input state (immutable)
pub fn with_input<F, R>(f: F) -> R
where
    F: FnOnce(&InputState) -> R,
{
    INPUT_STATE.with(|state| f(&state.borrow()))
}

/// Access input state (mutable)
pub fn with_input_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut InputState) -> R,
{
    INPUT_STATE.with(|state| f(&mut state.borrow_mut()))
}
