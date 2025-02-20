use crate::event::Interactive;
use crate::{
    element::Element, sdl::Renderer, style::StyleBuilder, view_id::ViewId, view_tuple::ViewTuple,
};
use glam::vec2;
use peniko::Color;
use reactive::create_effect;

#[derive(Debug)]
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
    fn paint(&self, ctx: &Renderer) {
        let layout = self.id.layout().unwrap();
        let state = self.id.state();
        let viewport = state.borrow().viewport;
        let style = &state.borrow().style;
        let location = layout.location + viewport;

        if style.background != Color::TRANSPARENT {
            ctx.fill_rect(
                style.background,
                vec2(location.x, location.y),
                vec2(layout.size.width, layout.size.height),
            );
        }

        for child in self.id().children() {
            child.element().paint(ctx);
        }
    }
}

impl Interactive for View {}

pub fn view<VT: ViewTuple>(children: VT) -> View {
    View::new(ViewId::new(), children)
}

impl View {
    pub fn new<VT: ViewTuple>(id: ViewId, children: VT) -> Self {
        let children = children.into_vec();
        create_effect(move |_| {
            let children = children
                .iter()
                .map(|item| item.get())
                .flatten()
                .collect::<Vec<_>>();
            id.set_children(children);
        });
        Self { id }
    }

    pub fn style<F: Fn(StyleBuilder) -> StyleBuilder + 'static>(self, f: F) -> Self {
        create_effect(move |_| {
            let id = self.id;
            let state = id.state();
            let node = id.node();
            let style = f(StyleBuilder::default());
            id.taffy()
                .borrow_mut()
                .set_style(node, style.taffy_style.clone())
                .unwrap();
            state.borrow_mut().style = style.style.clone();
        });
        self
    }
}
