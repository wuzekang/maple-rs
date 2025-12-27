//! Custom application events for the event loop
//!
//! Following Dioxus's design, we use custom UserEvents to decouple
//! task polling from rendering.

/// Custom events that can be sent to the event loop
#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    /// A task has been woken and needs to be polled
    /// This allows tasks to wake up the event loop without waiting for a redraw
    PollTasks,
}
