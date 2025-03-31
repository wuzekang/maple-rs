use crate::runtime::RUNTIME;
use std::any::Any;

pub fn on_cleanup(f: impl FnOnce() + 'static) {
    RUNTIME.with(|runtime| runtime.on_cleanup(f))
}

/// Signals that's wrapped this untrack will not subscribe to any effect
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|runtime| runtime.untrack(f))
}

pub fn batch<T>(f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|runtime| runtime.batch(f))
}

pub fn create_effect<T>(f: impl Fn(Option<T>) -> T + 'static)
where
    T: Any + 'static,
{
    RUNTIME.with(|runtime| runtime.create_effect(f))
}
