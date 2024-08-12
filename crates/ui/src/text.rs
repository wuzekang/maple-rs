use crate::{
    element::Element,
    runtime::RUNTIME,
    sdl::Painter,
    style::{Style, StyleBuilder},
    view_id::ViewId,
};
use cosmic_text::Attrs;
use reactive::{create_effect, RwSignal, SignalUpdate, SignalWith};
use taffy::{AvailableSpace, Size};

pub struct Text {
    id: ViewId,
    buffer: RwSignal<cosmic_text::Buffer>,
}
impl Text {
    pub fn new<F: (Fn() -> String) + 'static>(f: F) -> Self {
        let id = ViewId::new();
        let buffer = RwSignal::new({
            RUNTIME.with_borrow_mut(|s| {
                let metrics = cosmic_text::Metrics::new(24.0, 20.0);
                cosmic_text::Buffer::new(&mut s.font_system, metrics)
            })
        });

        create_effect(move |_| {
            let content = f();
            RUNTIME.with_borrow_mut(move |s| {
                buffer.update(|buffer| {
                    buffer.set_text(
                        &mut s.font_system,
                        &content,
                        Attrs::new(),
                        cosmic_text::Shaping::Advanced,
                    );
                    for line in &mut buffer.lines {
                        line.set_align(Some(cosmic_text::Align::Center));
                    }
                    // TODO: layout
                    buffer.set_wrap(&mut s.font_system, cosmic_text::Wrap::None);
                    buffer.set_wrap(&mut s.font_system, cosmic_text::Wrap::Word);
                });
            })
        });
        Self { id, buffer }
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
}

impl Element for Text {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&self, cx: &Painter) {
        let layout = self.id.get_layout().unwrap();
        let state = self.id.state();
        let viewport = state.borrow().viewport;
        let style = state.borrow().style.clone();
        RUNTIME.with_borrow_mut(|s| {
            self.buffer.with_untracked(|buffer| {
                cx.fill_text(
                    style.color(),
                    layout.location + viewport,
                    &mut s.swash_cache,
                    &mut s.font_system,
                    buffer,
                );
            })
        })
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });

        RUNTIME.with_borrow_mut(|s| {
            self.buffer.update(|buffer| {
                buffer.set_size(&mut s.font_system, width_constraint, None);
                buffer.shape_until_scroll(&mut s.font_system, false);
            });
        });

        let (width, total_lines) = self.buffer.with_untracked(|buffer| {
            buffer
                .layout_runs()
                .fold((0.0, 0usize), |(width, total_lines), run| {
                    (run.line_w.max(width), total_lines + 1)
                })
        });
        let height = self
            .buffer
            .with_untracked(|buffer| total_lines as f32 * buffer.metrics().line_height);

        taffy::Size { width, height }
    }
}
