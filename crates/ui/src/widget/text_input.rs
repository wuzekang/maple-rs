use crate::style::dimension::percent;
use crate::style::{Cursor, PointerEvents, Styleable, TextWrap};
use crate::widget::text::{fill_text, text_measure};
use crate::{
    element::Element, runtime::RUNTIME, sdl::Renderer, view, view_id::ViewId, Interactive, Texture,
};
use cosmic_text::{Action, Buffer, Edit, Editor, Motion, Selection};
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{create_ref, use_context, Ref, SignalUpdate};
use sdl3_sys::everything::*;
use std::fmt::Display;
use log::trace;
use taffy::{AlignItems, AvailableSpace, JustifyContent, Point, Size};

pub struct OffsetEditor {
    pub id: ViewId,
    pub offset: Vec2,
    pub editor: Editor<'static>,
}

impl OffsetEditor {
    fn new(id: ViewId) -> Self {
        Self {
            id,
            offset: Default::default(),
            editor: RUNTIME.with_borrow_mut(move |s| {
                let metrics = cosmic_text::Metrics::new(11.0, 11.0);
                let mut buffer = Buffer::new(&mut s.font_system, metrics);
                buffer.set_size(&mut s.font_system, Some(100.0), Some(20.0));
                Editor::new(buffer)
            }),
        }
    }
    fn insert_string(&mut self, data: &str) {
        self.editor.insert_string(data, None);
        self.update();
        self.editor.with_buffer(|buf| {
            let text = buf
                .lines
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>()
                .join("\n");
            dbg!(text);
        })
    }

    fn action(&mut self, action: Action) {
        RUNTIME.with_borrow_mut(|s| {
            self.editor.action(&mut s.font_system, action);
        });
        self.update();
        self.editor.with_buffer(|buf| {
            let text = buf
                .lines
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>()
                .join("\n");
            dbg!(text);
        })
    }

    fn update(&mut self) {
        let (width, height) = self.editor.with_buffer_mut(|buffer| {
            RUNTIME.with_borrow_mut(|s| {
                buffer.shape_until_scroll(&mut s.font_system, true);
            });
            let (width, total_lines) = buffer
                .layout_runs()
                .fold((0.0, 0usize), |(width, total_lines), run| {
                    (run.line_w.max(width), total_lines + 1)
                });

            let height = total_lines as f32 * buffer.metrics().line_height;
            (width, height)
        });

        let dpr = 2.0f32;
        if let Some((x, y)) = self.editor.cursor_position() {
            let x = x as f32;
            let y = y as f32;
            let rect = self.id.bounding_rect();
            let rect_width = ((rect.width - 1.0) * dpr).max(0.0);
            let rect_height = rect.height * dpr;
            if self.offset.x + x > rect_width {
                self.offset.x = rect_width - x;
            }
            if self.offset.y + y > rect_height {
                self.offset.y = rect_height - y;
            }
            if self.offset.x + x < 0.0 {
                self.offset.x = -x;
            }
            if self.offset.y + y < 0.0 {
                self.offset.y = -y;
            }
            if width + self.offset.x < rect_width {
                self.offset.x = (rect_width - width).min(0.0);
            }
            if height + self.offset.x < rect_height {
                self.offset.y = (rect_height - height).min(0.0);
            }

            unsafe {
                SDL_SetTextInputArea(
                    use_context().unwrap(),
                    &SDL_Rect {
                        x: (rect.x + (x + self.offset.x) / dpr) as i32,
                        y: (rect.y + (y + self.offset.y) / dpr) as i32,
                        w: 1,
                        h: rect.height as i32,
                    },
                    0,
                );
            }
        }
    }
}

#[derive(Clone, Copy)]
struct TextView {
    id: ViewId,
    focused: Ref<bool>,
    editor: Ref<OffsetEditor>,
    cache: Ref<(Option<Texture>, Option<Texture>)>,
}

impl TextView {
    fn new(focused: Ref<bool>) -> Self {
        let id = ViewId::new();
        Self {
            id,
            focused,
            editor: create_ref(OffsetEditor::new(id)),
            cache: create_ref((None, None)),
        }
    }

    fn action(&self, action: Action) {
        self.editor.with_mut(|editor| {
            editor.action(action);
        });
        self.cache.with_mut(|(a, _)| *a = None);
    }

    fn insert_string(&self, data: &str) {
        self.editor.with_mut(|editor| {
            editor.insert_string(data);
        });
        self.cache.with_mut(|(a, _)| *a = None);
    }
}

impl Element for TextView {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "TextView".to_string()
    }

    fn paint(&self, ctx: &mut Renderer) {
        let id = self.id();
        let state = id.state();
        let style = state.borrow().style.clone();

        let layout = id.layout();
        let size = layout.size;

        if style.background != Color::TRANSPARENT {
            ctx.fill_rect(style.background, Vec2::ZERO, vec2(size.width, size.height));
        }

        let dpr = 2.0f32;

        self.editor.with_mut(|editor| {
            let offset = editor.offset;

            let line_height = editor.editor.with_buffer_mut(|buffer| {
                fill_text(
                    ctx,
                    buffer,
                    // self.cache,
                    style.clone(),
                    Vec2::ZERO,
                    vec2(size.width, size.height),
                    offset,
                );
                buffer.metrics().line_height
            }) / dpr;

            if self.focused.with(|v| *v) {
                ctx.fill_selection(
                    Color::BLUE.multiply_alpha(0.5),
                    Vec2::ZERO,
                    vec2(size.width, size.height),
                    editor,
                    dpr,
                );
                if let Some((x, y)) = editor.editor.cursor_position() {
                    let x = x as f32 / dpr;
                    let y = y as f32 / dpr;
                    let p = Point {
                        x: offset.x / dpr,
                        y: offset.y / dpr,
                    };
                    ctx.set_color(style.color);
                    ctx.line(vec2(p.x + x, p.y + y), vec2(p.x + x, p.y + y + line_height))
                }
            }
        });
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        self.editor.with_mut(|editor| {
            editor.editor.with_buffer_mut(|buffer| {
                text_measure(self.id, buffer, known_dimensions, available_space, true)
            })
        })
    }
}
impl Styleable for TextView {}

#[derive(Debug)]
pub struct TextInput {
    id: ViewId,
}

impl TextInput {
    pub fn new() -> Self {
        let focused = create_ref(false);
        let text_view = TextView::new(focused);

        let element = view().tab_index(0).children(
            (text_view.style(|s| {
                s.font_size(12.0)
                    .line_height(12.0)
                    .text_wrap(TextWrap::None)
                    .width(percent(1.0))
                    .pointer_events(PointerEvents::None)
            })),
        );

        let element = element
            .on_click(|_| {})
            .on_focus(move |_| unsafe {
                focused.with_mut(|f| *f = true);
                text_view.editor.with_mut(|editor| editor.update());
                SDL_StartTextInput(use_context().unwrap());
            })
            .on_blur(move |_| unsafe {
                focused.with_mut(|f| *f = false);
                SDL_StopTextInput(use_context().unwrap());
            })
            .on_key_down(move |event| {
                if let Some(action) = match event.key {
                    SDLK_BACKSPACE => Some(Action::Backspace),
                    SDLK_UP => Some(Action::Motion(Motion::Up)),
                    SDLK_RIGHT => Some(Action::Motion(Motion::Right)),
                    SDLK_DOWN => Some(Action::Motion(Motion::Down)),
                    SDLK_LEFT => Some(Action::Motion(Motion::Left)),
                    _ => None,
                } {
                    let shift = event.r#mod & SDL_KMOD_SHIFT != 0;

                    text_view.editor.with_mut(|editor| {
                        if shift && editor.editor.selection() == Selection::None {
                            editor
                                .editor
                                .set_selection(Selection::Normal(editor.editor.cursor()));
                        }
                    });

                    text_view.action(action);

                    text_view.editor.with_mut(|editor| {
                        if !shift {
                            editor.editor.set_selection(Selection::None);
                        }
                    });
                }
            })
            .on_text_input(move |event| {
                let text = event.text.as_str();
                text_view.insert_string(text);
            })
            .style(|s| {
                s.cursor(Cursor::system_text())
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Stretch)
                    .padding_left(4)
                    .padding_right(4)
            });


        Self { id: element.id() }
    }
}

impl Element for TextInput {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "TextInput".to_string()
    }
}

impl Interactive for TextInput {}
impl Styleable for TextInput {}
