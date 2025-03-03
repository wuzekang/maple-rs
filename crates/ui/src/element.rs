use crate::event::Event;
use crate::{
    runtime::RUNTIME, sdl::Renderer, view_id::ViewId, widget::dynamic::Dynamic,
    widget::fragment::Fragment, widget::text::Text,
};
use glam::vec2;
use peniko::Color;
use reactive::{Scope, SignalGet};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::{AvailableSpace, Size};

pub trait Element: Any {
    fn id(&self) -> ViewId;

    fn update(&mut self, delta: f32) {}

    fn event(&mut self, event: &mut Event) {}

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
            child.element().borrow().paint(ctx);
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

pub trait IntoElement: Sized {
    fn into_element(self) -> Node;
}

impl<T: Element + 'static> IntoElement for T {
    fn into_element(self) -> Node {
        let id = self.id();
        id.set_element(Rc::new(RefCell::new(self)));
        Node::Static(id)
    }
}

impl<T: Element + 'static> IntoElement for Rc<RefCell<T>> {
    fn into_element(self) -> Node {
        let id = self.borrow().id();
        id.set_element(self.clone() as Rc<RefCell<dyn Element>>);
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

impl IntoElement for &str {
    fn into_element(self) -> Node {
        let content = Rc::new(self.to_string());
        Text::new({
            let content = content.clone();
            move || content.clone()
        })
        .into_element()
    }
}
