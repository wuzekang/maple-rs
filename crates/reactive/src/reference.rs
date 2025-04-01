use crate::id::Id;
use crate::runtime::RUNTIME;
use std::cell::RefCell;
use std::{any::Any, fmt, marker::PhantomData, rc::Rc};

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
        RUNTIME.with(|r| r.with(self.id, f))
    }

    pub fn with_mut<O>(&self, f: impl FnOnce(&mut T) -> O) -> O
    where
        T: 'static,
    {
        RUNTIME.with(|r| r.with_mut(self.id, f))
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
    Ref {
        id: RUNTIME.with(|r| r.add_reference(Rc::new(RefCell::new(value)))),
        ty: PhantomData,
    }
}
