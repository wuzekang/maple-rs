use crate::{view_id::ViewId, view_tuple::ViewTuple};
use reactive::{as_child_of_current_scope, create_effect, create_memo, create_rw_signal, create_signal, Memo, ReadSignal, Scope, SignalGet, SignalRead, SignalUpdate};

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
    pub signal: ReadSignal<Vec<ViewId>>,
}

impl Dynamic {
    pub fn new<VT: ViewTuple, F: Fn() -> VT + 'static>(view_fn: F) -> Self {
        let view_fn = Box::new(as_child_of_current_scope(move |_: ()| {
            view_fn()
                .into_vec()
                .into_iter()
                .map(|item| item.get())
                .flatten()
                .collect::<Vec<_>>()
        }));

        let (getter, setter) = create_signal(vec![]);

        create_effect(move |prev: Option<Scope>| {
            if let Some(scope) = prev {
                scope.dispose();
            }

            let (vec, scope) = view_fn(());
            setter.set(vec);
            scope
        });

        Self { signal: getter }
    }
}

pub fn dynamic<VT: ViewTuple, F: Fn() -> VT + 'static>(f: F) -> Dynamic {
    Dynamic::new(f)
}
