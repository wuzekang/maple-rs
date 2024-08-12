use crate::{
    element::Element,
    sdl::Painter,
    style::{Style, StyleBuilder},
    view_id::ViewId,
    view_tuple::ViewTuple,
};
use peniko::Color;
use reactive::create_effect;
use sdl3_sys::events::{SDL_Event, SDL_EventType};

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

pub fn view<VT: ViewTuple>(children: VT) -> View {
    View::new(children)
}

impl View {
    pub fn new<VT: ViewTuple>(children: VT) -> Self {
        let id = ViewId::new();
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
            state.borrow_mut().style = Style {
                background: style.background,
                color: style.color,
            };
        });
        self
    }

    pub fn on_event<F: (Fn(&SDL_Event) -> ()) + 'static>(self, f: F) -> Self {
        self.id.add_event_listener(Box::new(f));
        self
    }

    pub fn on_click<F: (Fn(&SDL_Event) -> ()) + 'static>(self, f: F) -> Self {
        let id = self.id();
        self.id.add_event_listener(Box::new(move |event| {
            let layout = id.get_layout();
            if SDL_EventType(unsafe { event.r#type }) != SDL_EventType::MOUSE_BUTTON_DOWN {
                return;
            }
            if !layout.is_some() {
                return;
            }
            let layout = layout.unwrap();
            let x = unsafe { event.button.x };
            let y = unsafe { event.button.y };
            // println!(
            //     "{:?} {:?} {:?}",
            //     layout,
            //     unsafe { event.button.x },
            //     unsafe { event.button.y }
            // );

            let left = layout.location.x;
            let top = layout.location.y;
            let right = left + layout.size.width;
            let bottom = top + layout.size.height;

            if x >= left && x < right && y >= top && y < bottom {
                f(event);
            }
        }));
        self
    }
}
