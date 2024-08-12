use crate::{
    dynamic::create_children_effect, element::Element, runtime::RUNTIME, view_id::{self, ViewId},
    view_tuple::ViewTuple,
};
use reactive::SignalGet;
use taffy::NodeId;

pub struct Fragment {
    pub children: Vec<Box<(dyn SignalGet<Vec<view_id::ViewId>> + 'static)>>
}


impl Fragment {
    pub fn new<VT: ViewTuple>(children: VT) -> Self {
        Self { children: children.into_vec() }
    }
}