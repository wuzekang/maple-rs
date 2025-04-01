use crate::element::Node;
use crate::use_resource;
use crate::view_tuple::ViewTuple;
use reactive::{
    as_child_of_current_scope, create_effect, create_signal, ReadSignal, Scope, SignalUpdate,
};
use std::future::Future;

pub struct Dynamic {
    pub signal: ReadSignal<Vec<(Node, Scope)>>,
}

impl Dynamic {
    pub fn new<VT: ViewTuple, F: Fn() -> VT + 'static>(view_fn: F) -> Self {
        let view_fn = Box::new(as_child_of_current_scope(move |_: ()| view_fn().into_vec()));

        let (getter, setter) = create_signal(Vec::new());

        create_effect({
            move |_| {
                setter.set(vec![view_fn(())]);
            }
        });

        Dynamic { signal: getter }
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

    let (getter, setter) = create_signal(vec![view_fn(O::default())]);

    use_resource(resource_fn, move |value| setter.set(vec![view_fn(value)]));

    Dynamic { signal: getter }
}

pub fn each<Item, VT, DataFn, ViewFn>(data_fn: DataFn, view_fn: ViewFn) -> Dynamic
where
    Item: PartialEq + Clone + 'static,
    VT: ViewTuple,
    DataFn: Fn() -> Vec<Item> + 'static,
    ViewFn: Fn(Item) -> VT + 'static,
{
    let view_fn = Box::new(as_child_of_current_scope(move |value: Item| {
        view_fn(value).into_vec()
    }));

    let (getter, setter) = create_signal(Vec::new());

    create_effect(move |prev: Option<(Vec<Item>, Vec<(Node, Scope)>)>| {
        let data = data_fn();

        let views = if let Some((_data, mut _vec)) = prev {
            let mut vec = Vec::with_capacity(data.len());
            let mut left = 0;
            let mut right = 0;

            let len = _data.len();

            for (i, item) in data.iter().enumerate() {
                if len > i && item.eq(&_data[i]) {
                    left = i + 1;
                } else {
                    break;
                }
            }

            for (i, item) in data.iter().rev().enumerate() {
                if len > left + i + 1 && item.eq(&_data[len - 1 - i]) {
                    right = i + 1;
                } else {
                    break;
                }
            }

            for i in 0..left {
                vec.push(_vec[i].clone());
            }
            for i in left..data.len() - right {
                vec.push(view_fn(data[i].clone()));
            }
            for i in 0..right {
                vec.push(_vec[len - i - 1].clone());
            }

            vec
        } else {
            data.clone()
                .into_iter()
                .map(&view_fn)
                .collect::<Vec<_>>()
        };

        setter.set(views.clone());

        (data, views)
    });

    Dynamic { signal: getter }
}
