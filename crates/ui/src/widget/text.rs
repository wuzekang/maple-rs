use crate::style::{Style, Styleable};
use crate::{element::Element, runtime::RUNTIME, sdl::Renderer, view_id::ViewId, Texture};
use cosmic_text::{Attrs, Family, Metrics};
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{create_effect, create_ref, Ref, SignalWith};
use sdl3_sys::everything::SDL_FlipMode;
use std::fmt::Display;
use std::mem;
use taffy::{AvailableSpace, Point, Size};
#[derive(Debug)]
pub struct Text {
    id: ViewId,
    buffer: Ref<cosmic_text::Buffer>,
    cache: Ref<(Option<Texture>, Option<Texture>)>,
}
impl Text {
    pub fn new<S>(f: impl (Fn() -> S) + 'static) -> Self
    where
        S: Display + 'static,
    {
        let dpr = 2.0f32;
        let id = ViewId::new();
        let buffer = create_ref({
            RUNTIME.with_borrow_mut(|s| {
                let metrics = cosmic_text::Metrics::new(24.0 * dpr, 20.0 * dpr);
                cosmic_text::Buffer::new(&mut s.font_system, metrics)
            })
        });

        let cache = create_ref((None, None));

        create_effect(move |_| {
            let content = f();
            RUNTIME.with_borrow_mut(move |s| {
                buffer.with_mut(|buffer| {
                    buffer.set_text(
                        &mut s.font_system,
                        &content.to_string(),
                        Attrs::new().family(Family::Name("Arial")),
                        cosmic_text::Shaping::Advanced,
                    );
                    cache.with_mut(|(a, _)| {
                        *a = None;
                    });
                    // for line in &mut buffer.lines {
                    //     line.set_align(Some(cosmic_text::Align::Center));
                    // }
                });
            });
            id.taffy().borrow_mut().mark_dirty(id.0).unwrap();
        });
        Self { id, buffer, cache }
    }
}

pub fn text_measure(
    id: ViewId,
    buffer: &mut cosmic_text::Buffer,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    prune: bool,
) -> Size<f32> {
    let dpr = 2.0f32;
    let width_constraint = known_dimensions.width.or(match available_space.width {
        AvailableSpace::MinContent => Some(0.0),
        AvailableSpace::MaxContent => None,
        AvailableSpace::Definite(width) => Some(width * dpr),
    });

    let height_constraint = known_dimensions.height.or(match available_space.height {
        AvailableSpace::MinContent => Some(0.0),
        AvailableSpace::MaxContent => None,
        AvailableSpace::Definite(width) => Some(width * dpr),
    });

    let style = id.state().borrow().style.clone();
    let metrics = Metrics::new(
        style.font_size.unwrap_or(20.0) * dpr,
        style.line_height.unwrap_or(24.0) * dpr,
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

    let size = Size {
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
    };

    Size {
        width: size.width / dpr,
        height: size.height / dpr,
    }
}

pub fn fill_text(
    ctx: &mut Renderer,
    buffer: &cosmic_text::Buffer,
    cache: Ref<(Option<Texture>, Option<Texture>)>,
    style: Style,
    location: Vec2,
    size: Vec2,
    offset: Vec2,
) {
    cache.with_mut(|(a, b)| {
        if let Some(texture) = b {
            ctx.render_texture_flip(texture, SDL_FlipMode::NONE, location, size, Vec2::ZERO);
        } else if let Some(texture) = a {
            ctx.render_texture_flip(texture, SDL_FlipMode::NONE, location, size, Vec2::ZERO);
        }

        if a.is_none() {
            *a = mem::take(b);
        }
        if b.is_none() {
            let texture = ctx.create_texture(size * 2.0);
            RUNTIME.with_borrow_mut(|s| {
                ctx.with_target(&texture, |ctx| {
                    ctx.save();
                    ctx.reset();
                    ctx.clear();
                    ctx.fill_text(
                        style.color(),
                        Vec2::ZERO,
                        size * 2.0,
                        offset,
                        &mut s.swash_cache,
                        &mut s.font_system,
                        buffer,
                    );
                    ctx.restore();
                });
            });

            *b = Some(texture);
        }
    });
}

impl Element for Text {
    fn id(&self) -> ViewId {
        self.id
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
                Vec2::ZERO,
                vec2(size.width, size.height),
            );
        }

        self.buffer.with(|buffer| {
            fill_text(
                ctx,
                buffer,
                self.cache,
                style,
                Vec2::ZERO,
                vec2(size.width, size.height),
                Vec2::ZERO,
            );
        })

        // self.cache.with_mut(|cache| {
        //     if let Some(texture) = cache.as_ref() {
        //         ctx.render_texture_rotated(
        //             texture,
        //             SDL_FlipMode::NONE,
        //             vec2(location.x, location.y),
        //             vec2(size.width, size.height),
        //             Vec2::ZERO,
        //         );
        //     } else {
        //         let texture = ctx.create_texture(vec2(size.width * 2.0, size.height * 2.0));
        //         RUNTIME.with_borrow_mut(|s| {
        //             self.buffer.with(|buffer| {
        //                 ctx.with_target(&texture, |ctx| {
        //                     ctx.fill_text(
        //                         style.color(),
        //                         Point::ZERO,
        //                         vec2(size.width, size.height) * 2.0,
        //                         taffy::Point::ZERO,
        //                         &mut s.swash_cache,
        //                         &mut s.font_system,
        //                         buffer,
        //                     )
        //                 });
        //             })
        //         });
        //
        //         *cache = Some(texture);
        //     }
        // });
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
