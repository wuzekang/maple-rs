//! ReactiveContext - Similar to Dioxus's reactive context
//!
//! This allows tracking signal dependencies even when signals are read
//! inside async blocks during Future polling.

use sig_reactive::Effect;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A future wrapper that polls the inner future within a reactive effect context
///
/// This ensures that any signal reads during Future::poll() are tracked as dependencies.
pub struct ReactiveContextFuture<F> {
    inner: Pin<Box<F>>,
    /// The effect whose tracking context is entered while polling
    effect: Effect,
}

impl<F> ReactiveContextFuture<F> {
    pub fn new(inner: F, effect: Effect) -> Self {
        Self {
            inner: Box::pin(inner),
            effect,
        }
    }
}

impl<F: Future> Future for ReactiveContextFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        // Enter the effect's tracking context before polling; the guard
        // restores the previous context on drop.
        let _guard = this.effect.enter_context();

        // Poll the inner future - any signal reads will now be tracked!
        this.inner.as_mut().poll(cx)
    }
}
