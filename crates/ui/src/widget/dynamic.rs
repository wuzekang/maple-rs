use crate::element::Node;
use crate::use_resource;
use crate::view_tuple::ViewTuple;
use reactive::{
    as_child_of_current_scope, create_effect, create_signal, ReadSignal, Scope, SignalUpdate,
};
use std::future::Future;

pub struct Dynamic {
    pub signal: ReadSignal<(Node, Scope)>,
}

impl Dynamic {
     pub fn new<VT: ViewTuple, F: Fn() -> VT + 'static>(view_fn: F) -> Self {
        let view_fn = Box::new(as_child_of_current_scope(move |_: ()| view_fn().into_vec()));

        let (getter, setter) =
            create_signal((Node::Fragment(vec![]), Scope::current().create_child()));

        create_effect({
            move |_| {
                setter.set(view_fn(()));
            }
        });

        Self { signal: getter }
    }
}

pub fn dynamic<VT: ViewTuple, F: Fn() -> VT + 'static>(f: F) -> Dynamic {
    Dynamic::new(f)
}

pub fn lazy<O, T, R, VT, F>(resource_fn: R, view_fn: F) -> Dynamic
where
    O: Default + Send + 'static,
    T: Future<Output = O> + Send + 'static,
    R: Fn() -> T + 'static,
    VT: ViewTuple,
    F: Fn(O) -> VT + 'static,
{
    let view_fn = Box::new(as_child_of_current_scope(move |value: O| {
        view_fn(value).into_vec()
    }));

    let (getter, setter) = create_signal(view_fn(O::default()));

    use_resource(resource_fn, move |value| setter.set(view_fn(value)));

    Dynamic { signal: getter }
}
