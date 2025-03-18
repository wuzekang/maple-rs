use crate::geometry::Rect;
use crate::render::renderer::Renderer;
use crate::style::{StyleTrigger, Styleable};
use crate::{element::Element, runtime::RUNTIME, view_id::ViewId};
use cosmic_text::{Attrs, Family, Metrics};
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{create_effect, create_ref, use_context, Ref, SignalWith};
use sdl3_sys::everything::SDL_GetWindowPixelDensity;
use std::fmt::Display;
use taffy::{AvailableSpace, Size};

#[derive(Debug, Copy, Clone)]
pub struct Text {
    id: ViewId,
    dpr: f32,
    buffer: Ref<cosmic_text::Buffer>,
}
impl Text {
    pub fn new<S>(f: impl (Fn() -> S) + 'static) -> Self
    where
        S: Display + 'static,
    {
        let dpr = unsafe { SDL_GetWindowPixelDensity(use_context().unwrap()) };
        let id = ViewId::new();
        let buffer = create_ref({
            RUNTIME.with_borrow_mut(|s| {
                let metrics = Metrics::new(24.0 * dpr, 20.0 * dpr);
                cosmic_text::Buffer::new(&mut s.font_system, metrics)
            })
        });

        create_effect(move |prev| {
            let content = f().to_string();

            if let Some(prev) = prev {
                if prev == content {
                    return prev;
                }
            }

            let content = RUNTIME.with_borrow_mut({
                move |s| {
                    buffer.with_mut(|buffer| {
                        buffer.set_text(
                            &mut s.font_system,
                            &content,
                            Attrs::new().family(Family::Name("Arial")),
                            cosmic_text::Shaping::Advanced,
                        );
                        content
                    })
                }
            });

            id.taffy().borrow_mut().mark_dirty(id.0).unwrap();
            id.request_repaint(StyleTrigger::Layout);

            content
        });
        Self { id, buffer, dpr }
    }
}

impl Element for Text {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "Text".to_string()
    }

    fn paint(&self, ctx: &mut Renderer) {
        let id = self.id();
        let state = id.state();
        let style = state.borrow().style.clone();

        let layout = id.layout();
        let size = layout.size;

        if style.background != Color::TRANSPARENT {
            ctx.fill_rect(
                style.background,
                Rect::from((Vec2::ZERO, vec2(size.width, size.height))),
            );
        }

        self.buffer.with(|buffer| {
            RUNTIME.with_borrow_mut(|s| {
                ctx.fill_text(
                    style.color,
                    Vec2::ZERO,
                    vec2(size.width, size.height) * 2.0,
                    Vec2::ZERO,
                    &mut s.swash_cache,
                    &mut s.font_system,
                    buffer,
                )
            });
        })
    }

    fn measure(
        &self,
        ctx: &mut Renderer,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        self.buffer.with_mut(|buffer| {
            ctx.measure_text(self.id, buffer, known_dimensions, available_space, false)
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
