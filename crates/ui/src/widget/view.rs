use crate::element::Node;
use crate::event::Interactive;
use crate::root::AppContext;
use crate::style::{StyleTrigger, Styleable};
use crate::{element::Element, view_id::ViewId, view_tuple::ViewTuple};
use hashbrown::HashSet;
use reactive::{create_effect, use_context, Scope, SignalGet};

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
        format!(
            "View layer={} repaint={}",
            self.id.state().borrow().layer.is_some(),
            self.id.state().borrow().repaint
        )
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
        let ctx: AppContext = use_context().unwrap();
        subscribe(id, &children, ctx.clone());
        create_effect(move |_| {
            id.set_children(flatten(&children));
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
        Node::Dynamic(signal) => signal
            .get()
            .iter()
            .flat_map(|(node, _)| flatten(node))
            .collect(),
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
                if let Node::Static(id) = item {
                    views.push(*id);
                }
            }
            (views, scopes)
        }
        Node::Dynamic(signal) => {
            let vec = signal.get_untracked();
            let mut nodes = vec![];
            let mut scopes = vec![];

            for (node, scope) in vec {
                nodes.push(node);
                scopes.push(scope);
            }

            let mut collected = collect(&Node::Fragment(nodes));

            collected.1.append(&mut scopes);

            collected
        }
    }
}

fn subscribe(parent: ViewId, node: &Node, ctx: AppContext) {
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
        Node::Dynamic(signal) => create_effect(move |prev| {
            let vec = signal.get();

            let mut next_scopes = HashSet::with_capacity(vec.len());
            for (_, scope) in &vec {
                next_scopes.insert(*scope);
            }

            if let Some(vec) = prev {
                for (node, scope) in vec {
                    let (views, scopes) = collect(&node);
                    for child in views {
                        unmount(child, parent, &ctx);
                    }

                    if next_scopes.contains(&scope) {
                        continue;
                    }

                    for scope in scopes {
                        ctx.dispose(scope);
                    }

                    ctx.dispose(scope);
                }
            }

            for (node, _) in &vec {
                subscribe(parent, node, ctx.clone());
            }

            vec
        }),
    }
}

fn mount(id: ViewId, parent: ViewId, ctx: &AppContext) {
    if id.state().borrow().mounted {
        return;
    }
    ctx.mount(id, parent);
    id.state().borrow_mut().mounted = true;
    parent.request_repaint(StyleTrigger::Layout);
    ctx.request_style(id);
    for child in id.children().iter() {
        mount(*child, parent, ctx);
    }
}

fn unmount(id: ViewId, parent: ViewId, ctx: &AppContext) {
    if !id.state().borrow().mounted {
        return;
    }
    ctx.unmount(id, parent);
    parent.request_repaint(StyleTrigger::Layout);
    id.state().borrow_mut().mounted = false;
    for child in id.children().iter() {
        unmount(*child, parent, ctx);
    }
}
