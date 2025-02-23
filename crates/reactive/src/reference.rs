use crate::{
    effect::EffectTrait,
    id::Id,
    read::{SignalRead, SignalTrack, SignalWith},
    write::SignalWrite,
    SignalGet, SignalUpdate,
};
use std::{any::Any, cell, cell::RefCell, fmt, marker::PhantomData, rc::Rc};

pub struct Ref<T> {
    pub(crate) id: Id,
    pub(crate) ty: PhantomData<T>,
}

impl<T> Copy for Ref<T> {}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for Ref<T> {}

impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("RwReference");
        s.field("id", &self.id);
        s.field("ty", &self.ty);
        s.finish()
    }
}

impl<T> Ref<T> {
    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O
    where
        T: 'static,
    {
        self.id.reference().unwrap().with(f)
    }

    pub fn with_mut<O>(&self, f: impl FnOnce(&mut T) -> O) -> O
    where
        T: 'static,
    {
        self.id.reference().unwrap().with_mut(f)
    }
}

/// Creates a new RwSignal which can act both as a setter and a getter.
///
/// Accessing the signal value in an Effect will make the Effect subscribe
/// to the value change of the Signal. And whenever the signal value changes,
/// it will trigger an effect run.
pub fn create_ref<T>(value: T) -> Ref<T>
where
    T: Any + 'static,
{
    let id = Reference::create(value);
    id.set_scope();
    Ref {
        id,
        ty: PhantomData,
    }
}

#[derive(Clone)]
pub(crate) struct Reference {
    pub(crate) id: Id,
    pub(crate) value: Rc<dyn Any>,
}

impl Reference {
    pub fn create<T>(value: T) -> Id
    where
        T: Any + 'static,
    {
        let id = Id::next();
        let value = RefCell::new(value);
        let reference = Self {
            id,
            value: Rc::new(value),
        };
        id.add_reference(reference);
        id
    }

    pub fn borrow<T: 'static>(&self) -> cell::Ref<'_, T> {
        let value = self
            .value
            .downcast_ref::<RefCell<T>>()
            .expect("to downcast ref type");
        value.borrow()
    }

    pub(crate) fn get<T: Clone + 'static>(&self) -> T {
        let value = self.borrow::<T>();
        value.clone()
    }

    pub(crate) fn with<O, T: 'static>(&self, f: impl FnOnce(&T) -> O) -> O {
        let value = self.borrow::<T>();
        f(&value)
    }

    pub(crate) fn with_mut<U, T: 'static>(&self, f: impl FnOnce(&mut T) -> U) -> U {
        let result = self
            .value
            .downcast_ref::<RefCell<T>>()
            .expect("to downcast signal type");
        let result = f(&mut result.borrow_mut());
        result
    }
}
