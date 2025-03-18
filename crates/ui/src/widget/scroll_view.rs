use crate::style::Styleable;
use crate::view_tuple::ViewTuple;
use crate::{view, Element, Interactive, ViewId};
use glam::{vec2, Vec2};
use reactive::{create_rw_signal, SignalGet, SignalUpdate};

pub struct ScrollView {
    id: ViewId,
}

impl ScrollView {
    pub fn new<VT: ViewTuple>(children: VT) -> Self {
        let translate = create_rw_signal(Vec2::ZERO);
        let content = view()
            .composite()
            .style(move |s| {
                let translate = translate.get();
                s.translate_x(translate.x)
                    .translate_y(translate.y)
                    .block()
            })
            .children(children);

        let content_id = content.id();
        let scroll = view();
        let scroll_id = scroll.id();

        let scroll = scroll
            .on_mouse_wheel(move |event| {
                let size = content_id.layout().size - scroll_id.layout().size;
                let size = vec2(size.width, size.height).max(Vec2::ZERO);
                translate.update(move |value| {
                    *value = (*value + event.wheel * 20.0).clamp(-size, Vec2::ZERO)
                });
            })
            .style(|s| s.block().overflow_y_scroll())
            .children(content);
        ScrollView { id: scroll.id() }
    }
}

impl Interactive for ScrollView {}
impl Styleable for ScrollView {}

impl Element for ScrollView {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "ScrollView".to_string()
    }
}

pub fn scroll_view<VT: ViewTuple>(children: VT) -> ScrollView {
    ScrollView::new(children)
}
