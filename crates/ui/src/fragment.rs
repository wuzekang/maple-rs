use crate::{view_id::ViewId, view_tuple::ViewTuple};
use reactive::SignalGet;

pub struct Fragment {
    pub children: Vec<Box<(dyn SignalGet<Vec<ViewId>> + 'static)>>,
}

impl Fragment {
    pub fn new<VT: ViewTuple>(children: VT) -> Self {
        Self {
            children: children.into_vec(),
        }
    }
}

pub fn fragment<VT: ViewTuple>(children: VT) -> Fragment {
    Fragment::new(children)
}
