use crate::event::Interactive;
use crate::root::EventDispatcher;
use crate::style::Styleable;
use crate::{element::Element, view_id::ViewId, view_tuple::ViewTuple};
use reactive::{create_effect, use_context};

#[derive(Debug, Clone, Copy)]
pub struct View {
    id: ViewId,
}

impl Default for View {
    fn default() -> Self {
        Self { id: ViewId::new() }
    }
}

impl Element for View {
    fn id(&self) -> ViewId {
        self.id
    }
}

impl Interactive for View {}
impl Styleable for View {}

pub fn view<VT: ViewTuple>(children: VT) -> View {
    View::new(ViewId::new(), children)
}

impl View {
    pub fn new<VT: ViewTuple>(id: ViewId, children: VT) -> Self {
        let children = children.into_vec();

        create_effect(move |_| {
            let manager: EventDispatcher = use_context().unwrap();
            let mounted = id.state().borrow().mounted;
            if mounted {
                let children = id.children();
                for child in children {
                    unmount(child, &manager);
                }
            }
            let children = children
                .iter()
                .map(|item| item.get())
                .flatten()
                .collect::<Vec<_>>();
            id.set_children(children);
            if mounted {
                let children = id.children();
                for child in children {
                    mount(child, &manager);
                }
            }
        });
        Self { id }
    }
}

fn mount(id: ViewId, ctx: &EventDispatcher) {
    if id.state().borrow().mounted {
        return;
    }
    ctx.mount(id);
    id.state().borrow_mut().mounted = true;
    for child in id.children() {
        mount(child, ctx);
    }
}

fn unmount(id: ViewId, ctx: &EventDispatcher) {
    if !id.state().borrow().mounted {
        return;
    }
    ctx.unmount(id);
    id.state().borrow_mut().mounted = false;
    for child in id.children() {
        unmount(child, ctx);
    }
}
