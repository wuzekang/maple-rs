use crate::element::Node;
use crate::view_tuple::ViewTuple;
use reactive::{
    as_child_of_current_scope, create_effect, create_signal, ReadSignal, Scope, SignalUpdate,
};

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
