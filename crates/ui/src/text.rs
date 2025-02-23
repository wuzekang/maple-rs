use crate::style::Styleable;
use crate::{element::Element, runtime::RUNTIME, sdl::Renderer, view_id::ViewId};
use cosmic_text::{Attrs, Family, Metrics};
use glam::vec2;
use peniko::Color;
use reactive::{create_effect, create_ref, Ref, SignalWith};
use std::fmt::Display;
use taffy::{AvailableSpace, Size};
#[derive(Debug)]
pub struct Text {
    id: ViewId,
    buffer: Ref<cosmic_text::Buffer>,
}
impl Text {
    pub fn new<S>(f: impl (Fn() -> S) + 'static) -> Self
    where
        S: Display + 'static,
    {
        let id = ViewId::new();
        let buffer = create_ref({
            RUNTIME.with_borrow_mut(|s| {
                let metrics = cosmic_text::Metrics::new(24.0, 20.0);
                cosmic_text::Buffer::new(&mut s.font_system, metrics)
            })
        });

        create_effect(move |_| {
            let content = f();
            RUNTIME.with_borrow_mut(move |s| {
                buffer.with_mut(|buffer| {
                    let len = content.to_string().len();
                    buffer.set_text(
                        &mut s.font_system,
                        &content.to_string(),
                        Attrs::new().family(Family::Name("SimSun")),
                        cosmic_text::Shaping::Basic,
                    );
                    // for line in &mut buffer.lines {
                    //     line.set_align(Some(cosmic_text::Align::Center));
                    // }
                    // // TODO: layout
                    // buffer.set_wrap(&mut s.font_system, cosmic_text::Wrap::Word);
                    // buffer.set_wrap(&mut s.font_system, cosmic_text::Wrap::None);
                });
            })
        });
        Self { id, buffer }
    }
}

pub fn text_measure(
    id: ViewId,
    buffer: &mut cosmic_text::Buffer,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    prune: bool,
) -> Size<f32> {
    let width_constraint = known_dimensions.width.or(match available_space.width {
        AvailableSpace::MinContent => Some(0.0),
        AvailableSpace::MaxContent => None,
        AvailableSpace::Definite(width) => Some(width),
    });

    let height_constraint = known_dimensions.height.or(match available_space.height {
        AvailableSpace::MinContent => Some(0.0),
        AvailableSpace::MaxContent => None,
        AvailableSpace::Definite(width) => Some(width),
    });

    let style = id.state().borrow().style.clone();
    let metrics = Metrics::new(
        style.font_size.unwrap_or(20.0),
        style.line_height.unwrap_or(24.0),
    );

    RUNTIME.with_borrow_mut(|s| {
        buffer.set_metrics_and_size(
            &mut s.font_system,
            metrics,
            width_constraint,
            height_constraint,
        );
        buffer.set_wrap(&mut s.font_system, style.text_wrap.into());
        buffer.shape_until_scroll(&mut s.font_system, prune);
    });

    let (width, total_lines) = buffer
        .layout_runs()
        .fold((0.0, 0usize), |(width, total_lines), run| {
            (run.line_w.max(width), total_lines + 1)
        });

    let height = total_lines as f32 * buffer.metrics().line_height;

    Size {
        width: if let Some(max_width) = width_constraint {
            max_width.min(width)
        } else {
            width
        },
        height: if let Some(max_height) = height_constraint {
            max_height.min(height)
        } else {
            height
        },
    }
}

impl Element for Text {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&self, ctx: &Renderer) {
        let layout = self.id.layout().unwrap();
        let state = self.id.state();
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

        RUNTIME.with_borrow_mut(|s| {
            self.buffer.with(|buffer| {
                ctx.fill_text(
                    style.color(),
                    location,
                    size,
                    taffy::Point::ZERO,
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
        self.buffer.with_mut(|buffer| {
            text_measure(self.id, buffer, known_dimensions, available_space, false)
        })
    }
}

impl Styleable for Text {}

pub fn text<S>(f: impl (Fn() -> S) + 'static) -> Text
where
    S: Display + 'static,
{
    Text::new(f)
}
