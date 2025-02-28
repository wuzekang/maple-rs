use crate::{element::Node, view_tuple::ViewTuple};

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
