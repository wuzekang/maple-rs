use slotmap::DefaultKey;
use crate::runtime::{Node, Runtime, RUNTIME};
use crate::signal::Signal;

/// An internal id which can reference a Signal/Effect/Scope.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Id(pub DefaultKey);

impl Into<DefaultKey> for Id {
    fn into(self) -> DefaultKey {
        self.0
    }
}

impl From<DefaultKey> for Id {
    fn from(id: DefaultKey) -> Id {
        Id(id)
    }
}

// impl Into<usize> for Id {
//     fn into(self) -> usize {
//         self.0
//     }
// }
//
// impl From<usize> for Id {
//     fn from(id: usize) -> Id {
//         Id(id)
//     }
// }


impl Id {
    pub(crate) fn next() -> Id {
        RUNTIME.with(|r| r.next())
    }

    /// Dispose the relevant resources that's linking to this Id, and the all the children
    /// and grandchildren.
    pub(crate) fn dispose(&self) {
        RUNTIME.with(|r| r.remove(*self));
    }

    pub(crate) fn signal(&self) -> Option<Signal> {
        let id = *self;
        Some(Signal {
            value: RUNTIME.with(|r| r.nodes.borrow()[id.into()].value.clone().unwrap()),
            id,
        })
    }
}
