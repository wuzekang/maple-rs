//! ReactiveContext - Similar to Dioxus's reactive context
//!
//! This allows tracking signal dependencies even when signals are read
//! inside async blocks during Future polling.

use sig_reactive::Effect;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// A future wrapper that polls the inner future within a reactive effect context
///
/// This ensures that any signal reads during Future::poll() are tracked as dependencies.
pub struct ReactiveContextFuture<F> {
    inner: Pin<Box<F>>,
    /// The effect that should be on the stack during polling
    effect_rc: Rc<RefCell<Effect>>,
}

impl<F> ReactiveContextFuture<F> {
    pub fn new(inner: F, effect_rc: Rc<RefCell<Effect>>) -> Self {
        Self {
            inner: Box::pin(inner),
            effect_rc,
        }
    }
}

impl<F: Future> Future for ReactiveContextFuture<F> {
    type Output = F::Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let effect_rc = this.effect_rc.clone();
        
        // Push effect onto stack before polling
        sig_reactive::push_effect(effect_rc);
        
        // Poll the inner future - any signal reads will now be tracked!
        let result = this.inner.as_mut().poll(cx);
        
        // Pop effect from stack after polling
        sig_reactive::pop_effect();
        
        result
    }
}
