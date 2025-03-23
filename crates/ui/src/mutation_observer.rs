use crate::runtime::RUNTIME;
use crate::ViewId;
use slotmap::DefaultKey;
use std::cell::Cell;
use std::rc::Rc;

pub struct MutationObserver {
    key: Cell<Option<DefaultKey>>,
    callback: Rc<dyn Fn()>,
}

#[derive(Clone)]
pub struct ObserveOptions {
    pub attributes: bool,
    pub child_list: bool,
    pub subtree: bool,
}

impl Default for ObserveOptions {
    fn default() -> Self {
        Self {
            attributes: true,
            child_list: false,
            subtree: false,
        }
    }
}

impl MutationObserver {
    pub fn new<T: Fn() + 'static>(callback: T) -> Self {
        Self {
            key: None.into(),
            callback: Rc::new(callback),
        }
    }

    pub fn observe(&self, target: ViewId, options: ObserveOptions) {
        if self.key.get().is_some() {
            return;
        }
        let callback = self.callback.clone();
        let key = RUNTIME
            .with_borrow_mut(|r| r.mutation_observers.borrow_mut().insert((target, callback, options)));
        self.key.set(Some(key));
    }

    pub fn disconnect(&self) {
        if let Some(key) = self.key.take() {
            RUNTIME.with_borrow_mut(|r| r.mutation_observers.borrow_mut().remove(key));
        }
    }
}
