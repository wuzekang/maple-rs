use crate::event::Interactive;
use crate::{
    element::Element, sdl::Painter, style::StyleBuilder, view_id::ViewId, view_tuple::ViewTuple,
};
use peniko::Color;
use reactive::create_effect;

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
    fn paint(&self, ctx: &Painter) {
        let layout = self.id.get_layout().unwrap();
        let state = self.id.state();
        let viewport = state.borrow().viewport;
        let style = &state.borrow().style;

        if style.background != Color::TRANSPARENT {
            ctx.fill_rect(style.background, layout.location + viewport, layout.size);
        }

        for child in self.id().children() {
            child.element().borrow().paint(ctx);
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
