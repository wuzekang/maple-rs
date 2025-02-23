use crate::root::EventDispatcher;
use crate::{view_id::ViewId, view_tuple::ViewTuple};
use reactive::{
    as_child_of_current_scope, create_effect, create_signal,
    use_context, ReadSignal, Scope, SignalGet, SignalUpdate,
};

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

        let event_manager: EventDispatcher = use_context().unwrap();
        create_effect({
            let event_manager = event_manager.clone();
            move |prev: Option<(Vec<ViewId>, Scope)>| {
                if let Some((vec, scope)) = prev {
                    for item in vec {
                        event_manager.remove(item, scope)
                    }
                }

                let (vec, scope) = view_fn(());
                setter.set(vec.clone());
                (vec, scope)
            }
        });

        Self { signal: getter }
    }
}

pub fn dynamic<VT: ViewTuple, F: Fn() -> VT + 'static>(f: F) -> Dynamic {
    Dynamic::new(f)
}
