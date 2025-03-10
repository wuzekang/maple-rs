use crate::element::Node;
use crate::event::Interactive;
use crate::root::EventDispatcher;
use crate::style::Styleable;
use crate::{element::Element, view_id::ViewId, view_tuple::ViewTuple};
use reactive::{create_effect, use_context, Scope};

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

    fn name(&self) -> String {
        "View".to_string()
    }
}

impl Interactive for View {}
impl Styleable for View {}

pub fn view() -> View {
    View::new()
}

impl View {
    pub fn new() -> Self {
        Self { id: ViewId::new() }
    }

    pub fn children<VT: ViewTuple>(self, children: VT) -> Self {
        let id = self.id;
        let children = children.into_vec();
        let ctx: EventDispatcher = use_context().unwrap();
        subscribe(id, &children, ctx.clone());
        create_effect(move |_| {
            id.set_children(flatten(&children));
            id.request_repaint();
        });
        self
    }

    pub fn composite(self) -> Self {
        self.id.state().borrow_mut().composite = true;
        self
    }
}

fn flatten(item: &Node) -> Vec<ViewId> {
    match item {
        Node::Static(id) => vec![*id],
        Node::Fragment(vec) => vec.iter().flat_map(flatten).collect(),
        Node::Dynamic(signal) => flatten(&signal.get().0),
    }
}

fn collect(item: &Node) -> (Vec<ViewId>, Vec<Scope>) {
    match item {
        Node::Static(id) => (vec![*id], vec![]),
        Node::Fragment(vec) => {
            let mut views = vec![];
            let mut scopes = vec![];
            for item in vec {
                match item {
                    Node::Static(_) => {}
                    node => {
                        let (mut _views, mut _scopes) = collect(node);
                        views.append(&mut _views);
                        scopes.append(&mut _scopes);
                    }
                }
            }
            for item in vec {
                match item {
                    Node::Static(id) => {
                        views.push(*id);
                    }
                    _ => {}
                }
            }
            (views, scopes)
        }
        Node::Dynamic(signal) => {
            let (node, scope) = signal.get_untracked();
            let (views, mut scopes) = collect(&node);
            scopes.push(scope);
            (views, scopes)
        }
    }
}

fn subscribe(parent: ViewId, node: &Node, ctx: EventDispatcher) {
    let mounted = parent.state().borrow().mounted;
    match node.clone() {
        Node::Static(id) => {
            if mounted {
                mount(id, parent, &ctx);
            }
        }
        Node::Fragment(vec) => {
            for item in vec {
                subscribe(parent, &item, ctx.clone());
            }
        }
        Node::Dynamic(signal) => {
            let ctx = ctx.clone();
            let signal = signal.clone();
            create_effect(move |prev| {
                if let Some((node, scope)) = prev {
                    let (views, scopes) = collect(&node);
                    for child in views {
                        unmount(child, parent, &ctx);
                    }
                    for scope in scopes {
                        ctx.dispose(scope);
                    }
                    ctx.dispose(scope);
                }

                let (node, scope) = signal.get();
                subscribe(parent, &node, ctx.clone());
                (node, scope)
            });
        }
    }
}

fn mount(id: ViewId, parent: ViewId, ctx: &EventDispatcher) {
    if id.state().borrow().mounted {
        return;
    }
    ctx.mount(id, parent);
    id.state().borrow_mut().mounted = true;
    for child in id.children() {
        mount(child, parent, ctx);
    }
}

fn unmount(id: ViewId, parent: ViewId, ctx: &EventDispatcher) {
    if !id.state().borrow().mounted {
        return;
    }
    ctx.unmount(id, parent);
    id.state().borrow_mut().mounted = false;
    for child in id.children() {
        unmount(child, parent, ctx);
    }
}
