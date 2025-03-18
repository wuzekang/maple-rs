use crate::{id::Id, Scope};
use slab::Slab;
use slotmap::{DefaultKey, SlotMap};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::Cell;
use std::{
    any::{Any, TypeId},
    cell,
    cell::RefCell,
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
};

thread_local! {
    pub(crate) static RUNTIME: Runtime = Runtime::new();
}

#[derive(Default)]
pub struct Node {
    pub contexts: Cow<'static, HashMap<TypeId, Rc<dyn Any>>>,
    pub children: SmallVec<[Id; 16]>,
    pub cleanups: SmallVec<[Box<dyn FnOnce()>; 2]>,
    pub value: Option<Rc<dyn Any>>,
    pub observers: HashSet<Id>,
    pub subscribers: HashMap<Id, Rc<dyn EffectTrait>>,
}

/// The internal reactive Runtime which stores all the reactive system states in a
/// thread local
pub(crate) struct Runtime {
    pub(crate) current_effect: RefCell<Option<Rc<dyn EffectTrait>>>,
    pub(crate) current_scope: Rc<RefCell<Id>>,
    pub(crate) nodes: RefCell<SlotMap<DefaultKey, Node>>,
    pub(crate) batching: Cell<bool>,
    pub(crate) pending_effects: RefCell<SmallVec<[Rc<dyn EffectTrait>; 10]>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let mut nodes = SlotMap::new();
        let current = nodes.insert(Default::default());
        Self {
            current_effect: Default::default(),
            current_scope: Rc::new(RefCell::new(current.into())),
            nodes: nodes.into(),
            batching: Default::default(),
            pending_effects: Default::default(),
        }
    }

    pub(crate) fn add_pending_effect(&self, effect: Rc<dyn EffectTrait>) {
        let has_effect = self
            .pending_effects
            .borrow()
            .iter()
            .any(|e| e.id() == effect.id());
        if !has_effect {
            self.pending_effects.borrow_mut().push(effect);
        }
    }

    pub(crate) fn run_pending_effects(&self) {
        let pending_effects = { mem::take(&mut *self.pending_effects.borrow_mut()) };
        for effect in pending_effects {
            self.run_effect(effect);
        }
    }

    pub fn dispose(&self, id: Id) -> Vec<Id> {
        if !self.nodes.borrow().contains_key(id.into()) {
            return vec![];
        }
        let mut ret = vec![];
        let (children, subscribers, cleanups) = {
            let binding = &mut *self.nodes.borrow_mut();
            let node = &mut binding[id.into()];
            (
                mem::take(&mut node.children),
                mem::take(&mut node.subscribers),
                mem::take(&mut node.cleanups),
            )
        };

        for child in children {
            ret.append(&mut self.dispose(child));
        }

        // signal
        for (_, effect) in subscribers {
            self.observer_clean_up(effect.id());
        }

        for cleanup in cleanups {
            cleanup();
        }
        ret.push(id);
        ret
    }

    pub fn remove(&self, id: Id) {
        for id in self.dispose(id) {
            self.nodes.borrow_mut().remove(id.into());
        }
    }

    pub fn observer_clean_up(&self, id: Id) {
        let observers = mem::take(&mut self.nodes.borrow_mut()[id.into()].observers);
        for observer in observers {
            self.nodes.borrow_mut()[observer.into()]
                .subscribers
                .remove(&id);
        }
    }

    pub fn on_cleanup(&self, cleanup: impl FnOnce() + 'static) {
        self.nodes.borrow_mut()[self.current_scope.borrow().0]
            .cleanups
            .push(Box::new(cleanup));
    }

    pub fn add_signal(&self, value: Rc<dyn Any>) -> Id {
        let id = self
            .nodes
            .borrow_mut()
            .insert(Node {
                value: Some(value),
                ..Default::default()
            })
            .into();
        self.nodes.borrow_mut()[self.current_scope.borrow().0]
            .children
            .push(id);
        id
    }

    pub fn add_reference(&self, value: Rc<dyn Any>) -> Id {
        let id = self
            .nodes
            .borrow_mut()
            .insert(Node {
                value: Some(value),
                ..Default::default()
            })
            .into();
        self.set_scope(id);
        id
    }

    pub fn set_scope(&self, id: Id) {
        self.nodes.borrow_mut()[self.current_scope.borrow().0]
            .children
            .push(id);
    }

    pub fn with<O, T: 'static>(&self, id: Id, f: impl FnOnce(&T) -> O) -> O {
        let value = { self.nodes.borrow()[id.into()].value.clone().unwrap() };
        let value = value
            .downcast_ref::<RefCell<T>>()
            .expect("to downcast ref type");

        let x = f(&value.borrow());
        x
    }

    pub fn with_mut<O, T: 'static>(&self, id: Id, f: impl FnOnce(&mut T) -> O) -> O {
        let value = { self.nodes.borrow()[id.into()].value.clone().unwrap() };
        let value = value
            .downcast_ref::<RefCell<T>>()
            .expect("to downcast ref type");

        let x = f(&mut value.borrow_mut());
        x
    }

    pub fn next(&self) -> Id {
        let contexts = self.nodes.borrow()[self.current_scope.borrow().0]
            .contexts
            .clone();
        self.nodes
            .borrow_mut()
            .insert(Node {
                contexts,
                ..Default::default()
            })
            .into()
    }

    pub fn create_effect<T>(&self, f: impl Fn(Option<T>) -> T + 'static)
    where
        T: Any + 'static,
    {
        let id = self.next();
        let effect = Rc::new(Effect {
            id,
            f,
            value: RefCell::new(None),
            observers: RefCell::new(HashSet::default()),
        });
        self.set_scope(id);
        self.run_initial_effect(effect);
    }

    pub fn run_initial_effect(&self, effect: Rc<dyn EffectTrait>) {
        let effect_id = effect.id();

        *self.current_effect.borrow_mut() = Some(effect.clone());

        let effect_scope = Scope(effect_id);

        self.with_scope(effect_scope, || {
            // effect_scope.track();
            effect.run();
        });

        *self.current_effect.borrow_mut() = None;
    }

    pub fn with_scope<T>(&self, scope: Scope, f: impl FnOnce() -> T) -> T {
        let prev_scope = { self.current_scope.borrow().clone() };
        {
            *self.current_scope.borrow_mut() = scope.0
        };
        let result = f();
        {
            *self.current_scope.borrow_mut() = prev_scope;
        }
        result
    }

    pub fn untrack<T>(&self, f: impl FnOnce() -> T) -> T {
        let prev_effect = self.current_effect.take();
        let result = f();
        *self.current_effect.borrow_mut() = prev_effect;
        result
    }

    pub fn batch<T>(&self, f: impl FnOnce() -> T) -> T {
        let already_batching = {
            let batching = self.batching.get();
            if !batching {
                self.batching.set(true);
            }
            batching
        };

        let result = f();
        if !already_batching {
            self.batching.set(false);
            self.run_pending_effects();
        }

        result
    }

    pub fn run_effects(&self, id: Id) {
        // If we are batching then add it as a pending effect
        if self.batching.get() {
            let subscribers = { self.nodes.borrow()[id.into()].subscribers.clone() };
            for (_, subscriber) in subscribers {
                self.add_pending_effect(subscriber);
            }
            return;
        }

        let subscribers = { self.nodes.borrow_mut()[id.into()].subscribers.clone() };
        for (_, subscriber) in subscribers {
            self.run_effect(subscriber);
        }
    }
    pub fn run_effect(&self, effect: Rc<dyn EffectTrait>) {
        let effect_id = effect.id();

        self.dispose(effect_id);

        self.observer_clean_up(effect_id);

        {
            *self.current_effect.borrow_mut() = Some(effect.clone())
        };

        let effect_scope = Scope(effect_id);
        self.with_scope(effect_scope, move || {
            // effect_scope.track();
            effect.run();
        });

        {
            *self.current_effect.borrow_mut() = None
        };
    }

    pub fn subscribe(&self, id: Id) {
        if let Some(effect) = self.current_effect.borrow().as_ref() {
            self.nodes.borrow_mut()[id.into()]
                .subscribers
                .insert(effect.id(), effect.clone());
            self.nodes.borrow_mut()[effect.id().into()]
                .observers
                .insert(id);
        }
    }
}

pub(crate) trait EffectTrait {
    fn id(&self) -> Id;
    fn run(&self) -> bool;
}

struct Effect<T, F>
where
    T: 'static,
    F: Fn(Option<T>) -> T,
{
    id: Id,
    f: F,
    value: RefCell<Option<T>>,
    observers: RefCell<HashSet<Id>>,
}

// impl<T, F> Drop for Effect<T, F>
// where
//     T: 'static,
//     F: Fn(Option<T>) -> T,
// {
//     fn drop(&mut self) {
//         self.id.dispose();
//     }
// }

impl<T, F> EffectTrait for Effect<T, F>
where
    T: 'static,
    F: Fn(Option<T>) -> T,
{
    fn id(&self) -> Id {
        self.id
    }

    fn run(&self) -> bool {
        let curr_value = self.value.borrow_mut().take();

        // run the effect
        let new_value = (self.f)(curr_value);

        *self.value.borrow_mut() = Some(new_value);

        true
    }
}
