use crate::style::{Cursor, PointerEvents, Styleable, TextWrap};
use crate::text::text_measure;
use crate::{
    element::Element, runtime::RUNTIME, sdl::Renderer, view, view_id::ViewId, Interactive,
};
use cosmic_text::{Action, Buffer, Edit, Editor, Motion, Selection};
use glam::vec2;
use peniko::Color;
use reactive::{create_ref, use_context, Ref, SignalUpdate};
use sdl3_sys::everything::*;
use std::fmt::Display;
use taffy::prelude::{length, percent};
use taffy::{AlignItems, AvailableSpace, JustifyContent, Size};

pub struct OffsetEditor {
    pub id: ViewId,
    pub offset: taffy::Point<f32>,
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
    }

    fn action(&mut self, action: Action) {
        RUNTIME.with_borrow_mut(|s| {
            self.editor.action(&mut s.font_system, action);
        });
        self.update();
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

        if let Some((x, y)) = self.editor.cursor_position() {
            let x = x as f32;
            let y = y as f32;
            let rect = self.id.rect();
            let rect_width = (rect.width - 1.0).max(0.0);
            if self.offset.x + x > rect_width {
                self.offset.x = rect_width - x;
            }
            if self.offset.y + y > rect.height {
                self.offset.y = rect.height - y;
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
            if height + self.offset.x < rect.height {
                self.offset.y = (rect.height - height).min(0.0);
            }

            unsafe {
                SDL_SetTextInputArea(
                    use_context().unwrap(),
                    &SDL_Rect {
                        x: (rect.x + x + self.offset.x) as i32,
                        y: (rect.y + y + self.offset.y) as i32,
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
struct InputText {
    id: ViewId,
    editor: Ref<OffsetEditor>,
}

impl InputText {
    fn new() -> Self {
        let id = ViewId::new();
        Self {
            id,
            editor: create_ref(OffsetEditor::new(id)),
        }
    }

    fn action(&self, action: Action) {
        self.editor.with_mut(|editor| {
            editor.action(action);
        })
    }

    fn insert_string(&self, data: &str) {
        self.editor.with_mut(|editor| {
            editor.insert_string(data);
        })
    }
}

impl Element for InputText {
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

        self.editor.with_mut(|editor| {
            let offset = editor.offset;

            let line_height = editor.editor.with_buffer_mut(|buffer| {
                RUNTIME.with_borrow_mut(|s| {
                    ctx.fill_text(
                        style.color,
                        location,
                        size,
                        offset,
                        &mut s.swash_cache,
                        &mut s.font_system,
                        buffer,
                    )
                });

                buffer.metrics().line_height
            });

            ctx.fill_selection(Color::BLUE.multiply_alpha(0.5), location, size, editor);

            if let Some((x, y)) = editor.editor.cursor_position() {
                let p = location + offset;
                ctx.set_color(Color::BLACK);
                ctx.line(
                    p.x + x as f32,
                    p.y + y as f32,
                    p.x + x as f32,
                    p.y + y as f32 + line_height,
                )
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
impl Styleable for InputText {}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    id: ViewId,
}

impl Input {
    pub fn new() -> Self {
        let input_text = InputText::new();

        let element = view(
            (input_text.style(|s| {
                s.font_size(12.0)
                    .line_height(12.0)
                    .text_wrap(TextWrap::None)
                    .width(percent(1.0))
                    .pointer_events(PointerEvents::None)
            })),
        );

        element
            .on_click(|_| {})
            .on_focus(move |_| {
                dbg!("focus");
                let rect = element.id().rect();
                unsafe {
                    SDL_SetTextInputArea(
                        use_context().unwrap(),
                        &SDL_Rect {
                            x: rect.x as i32,
                            y: rect.y as i32,
                            w: rect.width as i32,
                            h: rect.height as i32,
                        },
                        0,
                    );
                    SDL_StartTextInput(use_context().unwrap());
                }
            })
            .on_blur(move |_| unsafe {
                dbg!("blur");
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

                    input_text.editor.with_mut(|editor| {
                        if shift && editor.editor.selection() == Selection::None {
                            editor
                                .editor
                                .set_selection(Selection::Normal(editor.editor.cursor()));
                        }
                    });

                    input_text.action(action);

                    input_text.editor.with_mut(|editor| {
                        if !shift {
                            editor.editor.set_selection(Selection::None);
                        }
                    });
                }
            })
            .on_text_input(move |event| {
                let text = event.text.as_str();
                input_text.insert_string(text);
            })
            .style(|s| {
                s.cursor(Cursor::system_text())
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Stretch)
                    .padding_left(length(4.0))
                    .padding_right(length(4.0))
            });

        Self { id: element.id() }
    }
}

impl Element for Input {
    fn id(&self) -> ViewId {
        self.id
    }
}

impl Interactive for Input {}
impl Styleable for Input {}
