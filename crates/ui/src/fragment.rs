use crate::{element::Node, view_id::ViewId, view_tuple::ViewTuple};
use reactive::SignalGet;

pub struct Fragment {
    pub children: Node,
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
