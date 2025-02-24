use crate::{
    dynamic::Dynamic, fragment::Fragment, runtime::RUNTIME, sdl::Renderer, text::Text,
    view_id::ViewId,
};
use glam::vec2;
use peniko::Color;
use reactive::{Scope, SignalGet, SignalWith};
use std::rc::Rc;
use taffy::{AvailableSpace, Size};

pub trait Element {
    fn id(&self) -> ViewId;

    fn paint(&self, ctx: &Renderer) {
        let id = self.id();
        let layout = id.layout().unwrap();
        let state = id.state();
        let viewport = state.borrow().viewport;
        let style = state.borrow().style.clone();

        let location = layout.location + viewport;
        let size = layout.size;

        if style.background != Color::TRANSPARENT {
            ctx.fill_rect(
                style.background,
                vec2(location.x, location.y),
                vec2(size.width, size.height),
            );
        }

        for child in self.id().children() {
            child.element().paint(ctx);
        }
    }
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        Size::ZERO
    }
}

#[derive(Clone)]
pub enum Node {
    Static(ViewId),
    Fragment(Vec<Node>),
    Dynamic(Rc<dyn SignalGet<(Node, Scope)>>),
}

impl Default for Node {
    fn default() -> Self {
        Self::Fragment(Vec::new())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(id1), Self::Static(id2)) => id1 == id2,
            _ => false,
        }
    }
}

pub trait IntoElement: Sized {
    fn into_element(self) -> Node;
}

impl<T: Element + 'static> IntoElement for T {
    fn into_element(self) -> Node {
        let id = self.id();
        RUNTIME.with_borrow_mut(|r| {
            r.elements.insert(id.node().into(), Rc::new(self));
        });
        Node::Static(id)
    }
}

impl<T: Element + 'static> IntoElement for Rc<T> {
    fn into_element(self) -> Node {
        let id = self.id();
        RUNTIME.with_borrow_mut({
            let element = self.clone() as Rc<dyn Element>;
            move |r| {
                r.elements.insert(id.node().into(), element);
            }
        });
        Node::Static(id)
    }
}
impl IntoElement for Fragment {
    fn into_element(self) -> Node {
        self.children
    }
}

impl IntoElement for Dynamic {
    fn into_element(self) -> Node {
        Node::Dynamic(Rc::new(self.signal))
    }
}

impl<T: IntoElement + 'static> IntoElement for Vec<T> {
    fn into_element(self) -> Node {
        Node::Fragment(
            self.into_iter()
                .map(|item| item.into_element())
                .collect::<Vec<_>>(),
        )
    }
}

impl IntoElement for i32 {
    fn into_element(self) -> Node {
        Text::new(move || self).into_element()
    }
}
