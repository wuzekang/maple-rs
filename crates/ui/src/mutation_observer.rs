use std::cell::Cell;
use crate::runtime::RUNTIME;
use crate::ViewId;
use slotmap::DefaultKey;
use std::rc::Rc;

pub struct MutationObserver {
    key: Cell<Option<DefaultKey>>,
    callback: Rc<dyn Fn()>,
}

impl MutationObserver {
    pub fn new<T: Fn() + 'static>(callback: T) -> Self {
        Self {
            key: None.into(),
            callback: Rc::new(callback),
        }
    }

    pub fn observe(&self, target: ViewId) {
        let callback = self.callback.clone();
        let key = RUNTIME
            .with_borrow_mut(|r| r.mutation_observers.borrow_mut().insert((target, callback)));
        self.key.set(Some(key));
    }

    pub fn disconnect(&self) {
        if let Some(key) = self.key.take() {
            RUNTIME.with_borrow_mut(|r| r.mutation_observers.borrow_mut().remove(key));
        }
    }
}
