use crate::{view_id::ViewId, view_tuple::ViewTuple};
use reactive::{
    as_child_of_current_scope, create_effect, create_memo, create_signal, Memo, SignalGet,
    SignalRead, SignalUpdate,
};

pub fn create_children_effect<VT: ViewTuple, F: Fn() -> VT + 'static>(id: ViewId, f: F) {
    create_effect(move |_| {
        let children = f()
            .into_vec()
            .into_iter()
            .map(|item| item.get())
            .flatten()
            .collect::<Vec<_>>();
        id.set_children(children);
    });
}

pub struct Dynamic {
    pub signal: Memo<Vec<ViewId>>,
}

impl Dynamic {
    pub fn new<VT: ViewTuple, F: Fn() -> VT + 'static>(view_fn: F) -> Self {
        let id = ViewId::new();
        let view_fn = Box::new(as_child_of_current_scope(move |_: ()| {
            view_fn()
                .into_vec()
                .into_iter()
                .map(|item| item.get())
                .flatten()
                .collect::<Vec<_>>()
        }));
        let (getter, setter) = create_signal(view_fn(()));
        let reader = getter.read_untracked();
        create_effect(move |_| {
            reader.borrow().1.dispose();
            for child in id.children() {
                child.remove();
            }
            setter.set(view_fn(()));
        });

        let signal = create_memo(move |_| getter.get().0);

        Self { signal }
    }
}

pub fn dynamic<VT: ViewTuple, F: Fn() -> VT + 'static>(f: F) -> Dynamic {
    Dynamic::new(f)
}
