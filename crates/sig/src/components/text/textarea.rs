//! TextArea module - Multi-line text input component
//!
//! Complete multi-line text input implementation based on PlainEditor
//!
//! # Design Specification
//!
//! ## Tokens Used
//! - Padding: `theme::Spacing::MD` (12px)
//! - Border-Radius: `theme::Radius::MD` (4px)
//! - Font-Size: `theme::FontSize::MD` (14px)
//!
//! ## States
//! - default: border 1px gray-300
//! - focus: border 2px blue-500
//! - placeholder: gray-400 color
//!
//! ## Features
//! - Multi-line editing
//! - Vertical and Horizontal scrolling
//! - Enter to insert new line

use crate::theme::{Border, FontSize, Radius, Spacing};
use crate::{Interactive, Signal, Widget, create_effect};
use std::rc::Rc;
use taffy::{AvailableSpace, Size as TaffySize};
use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Color, Fill};
use super::input_state::InputState;

// ============================================================================
// IME Cursor Area Management
// ============================================================================

fn set_ime_cursor_area(x: f64, y: f64, width: f64, height: f64) {
  crate::runtime::with_window(|window_state| {
    if let Some(window) = &window_state.redraw_requester {
      use winit::dpi::PhysicalPosition;
      let position = PhysicalPosition::new(x, y);
      let size = winit::dpi::PhysicalSize::new(width.max(1.0), height.max(1.0));
      window.set_ime_cursor_area(position, size);
    }
  });
}

// ============================================================================
// TextArea Component
// ============================================================================

/// TextArea component
///
/// TextArea implements the Element trait, providing multi-line text editing.
///
/// # Examples
///
/// ```rust
/// use sig::{create_scope, text_area};
///
/// create_scope(|| {
///     let input = text_area()
///         .placeholder("Enter multi-line text")
///         .width(300.0)
///         .height(150.0);
/// });
/// ```
pub struct TextArea {
  id: crate::ViewId,
  state: Rc<InputState>,
  placeholder: String,
  max_length: Option<usize>,
}

impl TextArea {
  pub fn new() -> Self {
    let id = crate::ViewId::new();
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&id) {
        state.name = "TextArea".to_string();
      } else {
        runtime
          .view_states
          .insert(id, crate::ViewState::with_name("TextArea"));
      }
    });

    let state = Rc::new(InputState::new());

    Self {
      id,
      state,
      placeholder: String::new(),
      max_length: None,
    }
  }

  pub fn placeholder(mut self, text: impl Into<String>) -> Self {
    self.placeholder = text.into();
    self
  }

  pub fn width(self, width: f32) -> Self {
    self.id.style(move |s| s.width(width));
    self
  }

  pub fn height(self, height: f32) -> Self {
    self.id.style(move |s| s.height(height));
    self
  }

  pub fn max_length(mut self, len: usize) -> Self {
    self.max_length = Some(len);
    self
  }

  pub fn value(self, signal: Signal<String>) -> Self {
    let state = self.state.clone();
    let signal_write = signal.clone();
    *state.on_change.borrow_mut() = Some(Rc::new(move |text: String| {
      *signal_write.write() = text;
    }));

    let state_read = state.clone();
    let view_id = self.id;
    create_effect(move || {
      let new_text = signal.read().clone();
      let mut editor_opt = state_read.editor.borrow_mut();
      if let Some(editor) = editor_opt.as_mut() {
        if editor.text() != new_text {
          editor.set_text(&new_text);
          editor.move_to_text_end();
          drop(editor_opt);
          view_id.mark_dirty();
        }
      } else {
        *state_read.initial_text.borrow_mut() = Some(new_text);
      }
    });

    self
  }

  fn build_view(self) -> crate::ViewId {
    let state = self.state.clone();
    let placeholder = self.placeholder.clone();
    let view_id = self.id;

    let widget = Rc::new(TextAreaWidget {
      state: state.clone(),
      placeholder,
      view_id,
    });

    let view = self.id.widget(widget);
    let view_id_for_style = view_id;

    view.style(move |s| {
      let is_focused =
        crate::runtime::with_window(|state| state.focused_view == Some(view_id_for_style));

      let mut style = s
        // Default height for TextArea (approx 4 lines)
        .height(80.0)
        .padding_left(Spacing::MD)
        .padding_right(Spacing::MD)
        .padding_top(6.0)
        .padding_bottom(6.0)
        .background(Color::WHITE)
        .border_radius(Radius::MD)
        .font_size(FontSize::MD)
        .color(Color::from_rgb8(17, 24, 39))
        .cursor(crate::Cursor::Text);

      if is_focused {
        style = style.border_all(Border::MEDIUM, Color::from_rgb8(59, 130, 246));
      } else {
        style = style.border_all(Border::THIN, Color::from_rgb8(209, 213, 219));
      }
      style
    })
  }

  pub fn into_view(self) -> crate::ViewId {
    self.build_view()
  }
}

impl Default for TextArea {
  fn default() -> Self {
    Self::new()
  }
}

struct TextAreaWidget {
  state: Rc<InputState>,
  placeholder: String,
  view_id: crate::ViewId,
}

impl Widget for TextAreaWidget {
  fn measure(
    &self,
    known_dimensions: TaffySize<Option<f32>>,
    _available_space: TaffySize<AvailableSpace>,
    _style: &crate::style::Style,
    _ctx: &crate::RenderContext,
    _font_ctx: &mut parley::FontContext,
    _layout_ctx: &mut parley::LayoutContext<()>,
  ) -> TaffySize<f32> {
    let width = known_dimensions.width.unwrap_or(300.0);
    let height = known_dimensions.height.unwrap_or(80.0);
    TaffySize { width, height }
  }

  fn paint(
    &self,
    scene: &mut Scene,
    width: f32,
    height: f32,
    abs_x: f64,
    abs_y: f64,
    style: &crate::style::Style,
    ctx: &crate::RenderContext,
  ) {
    let is_focused = crate::runtime::with_window(|state| state.focused_view == Some(self.view_id));
    let padding_left = Spacing::MD;
    let padding_right = Spacing::MD;
    let padding_top = 6.0;

    let font_size = style.font_size.unwrap_or(FontSize::MD);
    self.state.ensure_editor(ctx, font_size);

    let mut editor_opt = self.state.editor.borrow_mut();
    let editor = match editor_opt.as_mut() {
      Some(ed) => ed,
      None => return,
    };

    let physical_width = ctx.logical_to_physical(width) as f64;
    let physical_height = ctx.logical_to_physical(height) as f64;
    self.state.last_layout_pos.set((abs_x, abs_y));

    let physical_padding_left = ctx.logical_to_physical(padding_left) as f64;
    let physical_padding_right = ctx.logical_to_physical(padding_right) as f64;
    let physical_padding_top = ctx.logical_to_physical(padding_top) as f64;
    
    let visible_width = (physical_width - physical_padding_left - physical_padding_right).max(0.0);
    let visible_height = (physical_height - physical_padding_top * 2.0).max(0.0);

    let (mut scroll_x, mut scroll_y) = self.state.scroll_offset.get();
    let physical_font_size = ctx.logical_to_physical(font_size);

    if is_focused {
      if let Some((cursor_x, cursor_y, _, cursor_h)) = editor.cursor_geometry(physical_font_size) {
        let cursor_x = cursor_x as f64;
        let cursor_y = cursor_y as f64;
        let cursor_h = cursor_h as f64;
        let margin = 2.0;

        // Horizontal scroll
        if cursor_x < scroll_x + margin {
          scroll_x = (cursor_x - margin).max(0.0);
        } else if cursor_x > scroll_x + visible_width - margin {
          scroll_x = cursor_x - visible_width + margin;
        }

        // Vertical scroll
        if cursor_y < scroll_y + margin {
          scroll_y = (cursor_y - margin).max(0.0);
        } else if cursor_y + cursor_h > scroll_y + visible_height - margin {
          scroll_y = cursor_y + cursor_h - visible_height + margin;
        }
      }
    }

    // Clamp scroll
    let text_width = editor.layout().width() as f64;
    let text_height = editor.layout().height() as f64;

    if text_width <= visible_width {
      scroll_x = 0.0;
    }
    if text_height <= visible_height {
      scroll_y = 0.0;
    }

    let current_scroll = (scroll_x, scroll_y);
    if (current_scroll.0 - self.state.scroll_offset.get().0).abs() > 0.1 
       || (current_scroll.1 - self.state.scroll_offset.get().1).abs() > 0.1 {
      self.state.scroll_offset.set(current_scroll);
    }

    let physical_x = ctx.logical_to_physical(abs_x as f32) as f64;
    let physical_y = ctx.logical_to_physical(abs_y as f32) as f64;

    let clip_rect = RoundedRect::new(
      physical_x,
      physical_y,
      physical_x + physical_width,
      physical_y + physical_height,
      ctx.logical_to_physical(style.border_radius) as f64,
    );
    scene.push_layer(
      vello::peniko::BlendMode::default(),
      1.0,
      Affine::IDENTITY,
      &clip_rect,
    );

    let text = editor.text();
    let is_empty = text.is_empty();
    let display_text = if is_empty && !is_focused {
      &self.placeholder
    } else {
      &text
    };
    
    let placeholder_mode = is_empty && !is_focused && !self.placeholder.is_empty();
    if placeholder_mode {
      editor.set_text(&self.placeholder);
    }

    if !display_text.is_empty() {
      use parley::layout::PositionedLayoutItem;
      let text_color = if placeholder_mode {
        Color::from_rgba8(156, 163, 175, 255)
      } else {
        style.color
      };

      let text_x = physical_x + physical_padding_left - scroll_x;
      let text_y = physical_y + physical_padding_top - scroll_y;
      let transform = Affine::translate((text_x, text_y));

      for line in editor.layout().lines() {
        for item in line.items() {
          if let PositionedLayoutItem::GlyphRun(run) = item {
            let glyphs = run.positioned_glyphs().map(|g| vello::Glyph {
              id: g.id as u32,
              x: g.x,
              y: g.y,
            });

            scene
              .draw_glyphs(run.run().font())
              .font_size(run.run().font_size())
              .transform(transform)
              .brush(text_color)
              .draw(Fill::NonZero, glyphs);
          }
        }
      }
    }

    if placeholder_mode {
      editor.set_text("");
    }

    if is_focused && *self.state.cursor_visible.read() {
      if let Some((cursor_x, cursor_y, cursor_w, cursor_h)) =
        editor.cursor_geometry(physical_font_size)
      {
        let abs_cursor_x = physical_x + physical_padding_left + cursor_x as f64 - scroll_x;
        let abs_cursor_y = physical_y + physical_padding_top + cursor_y as f64 - scroll_y;

        set_ime_cursor_area(
          abs_cursor_x,
          abs_cursor_y,
          cursor_w as f64,
          cursor_h as f64,
        );

        let cursor_rect = Rect::new(
          abs_cursor_x,
          abs_cursor_y,
          abs_cursor_x + 1.5,
          abs_cursor_y + cursor_h as f64,
        );

        scene.fill(
          Fill::NonZero,
          Affine::IDENTITY,
          style.color,
          None,
          &cursor_rect,
        );
      }
    }

    if is_focused {
      let selection_rects = editor.selection_geometry();
      if !selection_rects.is_empty() {
        let selection_color = Color::from_rgba8(59, 130, 246, 64);
        for rect in selection_rects {
          let abs_rect = Rect::new(
            physical_x + physical_padding_left - scroll_x + rect.min_x(),
            physical_y + physical_padding_top - scroll_y + rect.min_y(),
            physical_x + physical_padding_left - scroll_x + rect.max_x(),
            physical_y + physical_padding_top - scroll_y + rect.max_y(),
          );
          scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            selection_color,
            None,
            &abs_rect,
          );
        }
      }
    }

    scene.pop_layer();
  }
}

impl From<TextArea> for crate::ViewId {
  fn from(input: TextArea) -> Self {
    input.into_view()
  }
}

impl crate::Element for TextArea {
  fn id(&self) -> crate::ViewId {
    self.id
  }

  fn build(self) -> crate::Node {
    let state = self.state.clone();
    let view_id = self.id;

    let area = self
      .focusable()
      .on_focus({
        let state = state.clone();
        move |_e| {
          state.reset_blink();
          crate::runtime::with_window(|window_state| {
            if let Some(window) = &window_state.redraw_requester {
              window.set_ime_allowed(true);
              window.request_redraw();
            }
          });
        }
      })
      .on_blur({
        let state = state.clone();
        move |_e| {
          state.stop_blink();
          crate::runtime::with_window(|window_state| {
            if let Some(window) = &window_state.redraw_requester {
              window.set_ime_allowed(false);
              window.request_redraw();
            }
          });
        }
      })
      .on_drag_start({
        let state = state.clone();
        let view_id = view_id;
        move |e| {
          let scale_factor = crate::runtime::with_window(|s| {
            s.redraw_requester
              .as_ref()
              .map(|w| w.scale_factor())
              .unwrap_or(1.0)
          });
          let (abs_x, abs_y) = state.last_layout_pos.get();
          let (scroll_x, scroll_y) = state.scroll_offset.get();
          let padding_left = 12.0;
          let padding_top = 6.0;

          let click_x = e.start_position.x as f64 * scale_factor;
          let click_y = e.start_position.y as f64 * scale_factor;
          let abs_x_phys = abs_x * scale_factor;
          let abs_y_phys = abs_y * scale_factor;
          let pad_left_phys = padding_left * scale_factor;
          let pad_top_phys = padding_top * scale_factor;

          let local_x = (click_x - (abs_x_phys + pad_left_phys - scroll_x)) as f32;
          let local_y = (click_y - (abs_y_phys + pad_top_phys - scroll_y)) as f32;

          let mut editor_opt = state.editor.borrow_mut();
          if let Some(editor) = editor_opt.as_mut() {
            editor.move_to_point(local_x, local_y);
            state.is_selecting.set(true);
            state.reset_blink();
            view_id.mark_dirty();
          }
        }
      })
      .on_drag({
        let state = state.clone();
        let view_id = view_id;
        move |e| {
          if state.is_selecting.get() {
            let scale_factor = crate::runtime::with_window(|s| {
              s.redraw_requester
                .as_ref()
                .map(|w| w.scale_factor())
                .unwrap_or(1.0)
            });
            let (abs_x, abs_y) = state.last_layout_pos.get();
            let (scroll_x, scroll_y) = state.scroll_offset.get();
            let padding_left = 12.0;
            let padding_top = 6.0;

            let curr_x = e.current_position.x as f64 * scale_factor;
            let curr_y = e.current_position.y as f64 * scale_factor;
            let abs_x_phys = abs_x * scale_factor;
            let abs_y_phys = abs_y * scale_factor;
            let pad_left_phys = padding_left * scale_factor;
            let pad_top_phys = padding_top * scale_factor;

            let local_x = (curr_x - (abs_x_phys + pad_left_phys - scroll_x)) as f32;
            let local_y = (curr_y - (abs_y_phys + pad_top_phys - scroll_y)) as f32;

            let mut editor_opt = state.editor.borrow_mut();
            if let Some(editor) = editor_opt.as_mut() {
              editor.extend_selection_to_point(local_x, local_y);
              state.reset_blink();
              view_id.mark_dirty();
            }
          }
        }
      })
      .on_drag_end({
        let state = state.clone();
        move |_e| {
          state.is_selecting.set(false);
        }
      })
      .on_mouse_down({
        let state = state.clone();
        let view_id = view_id;
        move |e| {
          if e.button != Some(winit::event::MouseButton::Left) {
            return;
          }

          let scale_factor = crate::runtime::with_window(|s| {
            s.redraw_requester
              .as_ref()
              .map(|w| w.scale_factor())
              .unwrap_or(1.0)
          });
          let (abs_x, abs_y) = state.last_layout_pos.get();
          let (scroll_x, scroll_y) = state.scroll_offset.get();
          let padding_left = 12.0;
          let padding_top = 6.0;

          let click_x = e.position.x as f64 * scale_factor;
          let click_y = e.position.y as f64 * scale_factor;
          let abs_x_phys = abs_x * scale_factor;
          let abs_y_phys = abs_y * scale_factor;
          let pad_left_phys = padding_left * scale_factor;
          let pad_top_phys = padding_top * scale_factor;

          let local_x = (click_x - (abs_x_phys + pad_left_phys - scroll_x)) as f32;
          let local_y = (click_y - (abs_y_phys + pad_top_phys - scroll_y)) as f32;

          let (double_click, triple_click) = state.handle_click((click_x, click_y));

          let mut editor_opt = state.editor.borrow_mut();
          if let Some(editor) = editor_opt.as_mut() {
            if triple_click {
              editor.select_line_at_point(local_x, local_y);
            } else if double_click {
              editor.select_word_at_point(local_x, local_y);
            } else if e.modifiers.shift_key() {
              editor.extend_selection_to_point(local_x, local_y);
            } else {
              editor.move_to_point(local_x, local_y);
            }
            state.reset_blink();
            view_id.mark_dirty();
          }
        }
      })
      .on_keyboard({
        let state = state.clone();
        move |e| {
          if e.state != winit::event::ElementState::Pressed {
            return;
          }

          let mut text_changed = false;
          let handled;

          {
            let mut editor_opt = state.editor.borrow_mut();
            if let Some(editor) = editor_opt.as_mut() {
              use winit::keyboard::{Key, NamedKey};

              if e.modifiers.control_key() || e.modifiers.super_key() {
                handled = match &e.logical_key {
                  Key::Character(ch) => match ch.as_str() {
                    "a" => {
                      editor.select_all();
                      state.reset_blink();
                      true
                    }
                    "c" => {
                      if let Some(text) = editor.selected_text() {
                        if let Ok(ctx) = clipboard_rs::ClipboardContext::new() {
                          use clipboard_rs::Clipboard;
                          let _ = ctx.set_text(text);
                        }
                      }
                      true
                    }
                    "x" => {
                      if let Some(text) = editor.selected_text() {
                        if let Ok(ctx) = clipboard_rs::ClipboardContext::new() {
                          use clipboard_rs::Clipboard;
                          let _ = ctx.set_text(text);
                        }
                        editor.delete_selection();
                        text_changed = true;
                        state.reset_blink();
                      }
                      true
                    }
                    "v" => {
                      if let Ok(ctx) = clipboard_rs::ClipboardContext::new() {
                        use clipboard_rs::Clipboard;
                        if let Ok(text) = ctx.get_text() {
                          editor.insert_or_replace(&text);
                          text_changed = true;
                          state.reset_blink();
                        }
                      }
                      true
                    }
                    "z" => {
                        if e.modifiers.shift_key() {
                            editor.redo();
                        } else {
                            editor.undo();
                        }
                        text_changed = true;
                        state.reset_blink();
                        true
                    }
                    "y" => {
                        editor.redo();
                        text_changed = true;
                        state.reset_blink();
                        true
                    }
                    _ => false,
                  },
                  _ => false,
                };
              } else {
                handled = match &e.logical_key {
                  Key::Character(ch) => {
                    if ch.chars().all(|c| !c.is_control()) {
                      editor.insert_or_replace(ch.as_ref());
                      text_changed = true;
                      state.reset_blink();
                      true
                    } else {
                      false
                    }
                  }
                  Key::Named(NamedKey::Enter) => {
                    editor.insert_or_replace("\n");
                    text_changed = true;
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::Backspace) => {
                    editor.backdelete();
                    text_changed = true;
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::Delete) => {
                    editor.delete();
                    text_changed = true;
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::ArrowLeft) => {
                    if e.modifiers.shift_key() {
                      editor.select_left();
                    } else {
                      editor.move_left();
                    }
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::ArrowRight) => {
                    if e.modifiers.shift_key() {
                      editor.select_right();
                    } else {
                      editor.move_right();
                    }
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::ArrowUp) => {
                    if e.modifiers.shift_key() {
                      editor.select_up();
                    } else {
                      editor.move_up();
                    }
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::ArrowDown) => {
                    if e.modifiers.shift_key() {
                      editor.select_down();
                    } else {
                      editor.move_down();
                    }
                    state.reset_blink();
                    true
                  }
                  Key::Named(NamedKey::Escape) => {
                    crate::runtime::with_window_mut(|state| state.focused_view = None);
                    state.stop_blink();
                    true
                  }
                  _ => false,
                };
              }
            } else {
              handled = false;
            }
          }

          if text_changed {
            state.notify_change();
          }

          if handled {
            e.stop_propagation();
          }
        }
      })
      .on_ime({
        let state = state.clone();
        move |e| {
          use winit::event::Ime;
          match &e.ime {
            Ime::Commit(text) => {
              let text_changed = {
                let mut editor_opt = state.editor.borrow_mut();
                if let Some(editor) = editor_opt.as_mut() {
                  editor.insert_or_replace(text);
                  state.reset_blink();
                  true
                } else {
                  false
                }
              };

              if text_changed {
                state.notify_change();
              }

              e.stop_propagation();
            }
            Ime::Preedit(text, _cursor_pos) => {
              let mut editor_opt = state.editor.borrow_mut();
              if let Some(editor) = editor_opt.as_mut() {
                if text.is_empty() {
                  editor.clear_compose();
                } else {
                  editor.set_compose(text);
                }
                state.reset_blink();
              }
              e.stop_propagation();
            }
            _ => {}
          }
        }
      });

    crate::Node::View(area.build_view())
  }
}

pub fn text_area() -> TextArea {
  TextArea::new()
}
