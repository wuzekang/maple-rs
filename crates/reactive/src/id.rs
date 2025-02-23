use std::sync::atomic::AtomicU64;

use crate::reference::Reference;
use crate::{effect::observer_clean_up, runtime::RUNTIME, signal::Signal};

/// An internal id which can reference a Signal/Effect/Scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Id(u64);

impl Id {
    /// Create a new Id that's next in order
    pub(crate) fn next() -> Id {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Id(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }

    /// Try to get the Signal that links with this Id
    pub(crate) fn signal(&self) -> Option<Signal> {
        RUNTIME.with(|runtime| runtime.signals.borrow().get(self).cloned())
    }

    /// Try to set the Signal to be linking with this Id
    pub(crate) fn add_signal(&self, signal: Signal) {
        RUNTIME.with(|runtime| runtime.signals.borrow_mut().insert(*self, signal));
    }

    pub(crate) fn reference(&self) -> Option<Reference> {
        RUNTIME.with(|runtime| runtime.references.borrow().get(self).cloned())
    }

    pub(crate) fn add_reference(&self, reference: Reference) {
        RUNTIME.with(|runtime| runtime.references.borrow_mut().insert(*self, reference));
    }

    pub(crate) fn add_cleanup(&self, f: impl Fn() + 'static) {
        RUNTIME.with(|runtime| {
            runtime
                .cleanups
                .borrow_mut()
                .entry(*self)
                .or_default()
                .push(Box::new(f))
        });
    }

    /// Make this Id a child of the current Scope
    pub(crate) fn set_scope(&self) {
        RUNTIME.with(|runtime| {
            let scope = runtime.current_scope.borrow();
            let mut children = runtime.children.borrow_mut();
            let children = children.entry(*scope).or_default();
            children.insert(*self);
        });
    }

    /// Dispose the relevant resources that's linking to this Id, and the all the children
    /// and grandchildren.
    pub(crate) fn dispose(&self) {
        if let Ok((children, signal, cleanup, _)) = RUNTIME.try_with(|runtime| {
            (
                runtime.children.borrow_mut().remove(self),
                runtime.signals.borrow_mut().remove(self),
                runtime.cleanups.borrow_mut().remove(self),
                runtime.references.borrow_mut().remove(self),
            )
        }) {
            if let Some(children) = children {
                for child in children {
                    child.dispose();
                }
            }

            if let Some(signal) = signal {
                for (_, effect) in signal.subscribers() {
                    observer_clean_up(&effect);
                }
            }

            if let Some(cleanup) = cleanup {
                for callback in cleanup {
                    callback();
                }
            }
        }
    }
}
