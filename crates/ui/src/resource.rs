use crate::runtime::RUNTIME;
use futures::Future;
use reactive::{create_effect, on_cleanup};
use slotmap::DefaultKey;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use tokio::task;

pub struct Resource {
    pub id: u64,
    pub callback_key: DefaultKey,
    pub value: Box<dyn Any>,
}

impl Resource {
    pub fn try_recv() {
        let vec = RUNTIME.with_borrow(|s| {
            let mut vec = vec![];
            while let Ok(r) = s.receiver.try_recv() {
                vec.push(r);
            }
            vec
        });

        for r in vec {
            r.recv();
        }
    }

    pub fn recv(self) {
        let callback_key = self.callback_key;
        let callback =
            RUNTIME.with_borrow_mut(|s| s.resources.borrow_mut().get(callback_key).cloned());
        if let Some(callback) = callback {
            callback(self);
        }
    }
}

unsafe impl Send for Resource {}
unsafe impl Sync for Resource {}

pub struct Unbox<T> {
    pub value: T,
}
impl<T> Unbox<T> {
    pub fn unbox(self) -> T {
        self.value
    }
}

pub struct ResourceResult {
    current: Rc<RefCell<u64>>,
    cancelled: Rc<RefCell<HashSet<u64>>>
}

impl ResourceResult {
    pub fn cancel(&self) -> bool {
        self.cancelled.borrow_mut().insert(*self.current.borrow())
    }

    pub fn run(&self) {

    }
}


pub fn use_resource<O, T, F, C>(f: F, c: C) -> ResourceResult
where
    O: Default + Send + 'static,
    T: Future<Output = O> + Send + 'static,
    F: Fn() -> T + 'static,
    C: Fn(O) + 'static,
{
    let current = Rc::new(RefCell::new(0u64));
    let cancelled = Rc::new(RefCell::new(HashSet::new()));
    let callback = Rc::new({
        let current = current.clone();
        let cancelled = cancelled.clone();
        move |resource: Resource| {
            if resource.id != *current.borrow() {
                return;
            }
            if cancelled.borrow_mut().remove(&resource.id) {
                return;
            }
            let value = resource.value.downcast::<Unbox<O>>().unwrap();
            let value = value.unbox();
            c(value);
        }
    });

    let (sender, callback_key) =
        RUNTIME.with_borrow(|r| (r.sender.clone(), r.resources.borrow_mut().insert(callback)));

    on_cleanup(move || {
        RUNTIME.with_borrow(|r| {
            r.resources.borrow_mut().remove(callback_key);
        });
    });

    create_effect({
        let current = current.clone();
        move |_| {
            let id = current.borrow().checked_add(u64::MAX).unwrap_or(0);
            *current.borrow_mut() = id;
            let future = f();
            task::spawn({
                let sender = sender.clone();
                async move {
                    sender
                        .send(Resource {
                            id,
                            callback_key,
                            value: Box::new(Unbox {
                                value: future.await,
                            }),
                        })
                        .unwrap();
                }
            });
        }
    });

    ResourceResult {
        current,
        cancelled,
    }
}
