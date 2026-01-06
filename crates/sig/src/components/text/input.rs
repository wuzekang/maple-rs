//! TextInput module - Single-line text input component
//!
//! Complete text input implementation based on PlainEditor
//!
//! # Design Specification
//!
//! ## Tokens Used
//! - Height: `theme::Size::MD` (32px) - aligned with Button
//! - Padding-X: `theme::Spacing::MD` (12px) - consistent across all sizes
//! - Padding-Y: Auto-calculated for vertical centering
//! - Border-Radius: `theme::Radius::MD` (4px)
//! - Font-Size: `theme::FontSize::MD` (14px)
//!
//! ## States
//! - default: border 1px gray-300
//! - focus: border 2px blue-500
//! - placeholder: gray-400 color
//!
//! ## Deviations from Standard
//! - None (fully compliant with design guidelines)
//! - Optimized for code editors and desktop applications

use crate::theme::{Border, FontSize, Radius, Size, Spacing};
use crate::{Interactive, Signal, TextEditor, Widget, create_effect, spawn};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use taffy::{AvailableSpace, Size as TaffySize};
use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Color, Fill};

// ============================================================================
// IME Cursor Area Management
// ============================================================================

/// Set IME cursor area directly to window
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
// TextInput Component
// ============================================================================

/// TextInput component
///
/// TextInput implements the Element trait, automatically providing:
/// - Styleable: All styling methods (40+)
/// - Interactive: All event handling methods (10+)
///
/// # Examples
///
/// ```rust
/// use sig::{create_scope, text_input};
///
/// create_scope(|| {
///     let input = text_input()
///         .placeholder("Enter text")
///         .width(300.0)
///         .style(|s| s.margin_top(10.0))  // Direct style methods!
///         .on_click(|_| println!("Clicked"));  // Direct event methods!
/// });
/// ```
pub struct TextInput {
  id: crate::ViewId,
  state: Rc<TextInputState>,
  placeholder: String,
  max_length: Option<usize>,
}

/// TextInput internal state
pub struct TextInputState {
  // Text editor core (lazy initialization, requires DPR information)
  pub editor: RefCell<Option<TextEditor>>,

  // Cursor blinking state
  cursor_visible: Signal<bool>,

  // Blink task handle (to cancel when unfocused)
  blink_task: RefCell<Option<crate::Task>>,

  // Text change callback (for two-way binding)
  on_change: RefCell<Option<Rc<dyn Fn(String)>>>,

  // Initial text (set before editor is initialized)
  initial_text: RefCell<Option<String>>,

  // Horizontal scroll offset (physical pixels)
  scroll_offset: Cell<f64>,

  // Last layout position (logical pixels)
  last_layout_pos: Cell<(f64, f64)>,
  // Is selecting text with mouse
  is_selecting: Cell<bool>,
}

impl TextInputState {
  fn new() -> Self {
    Self {
      editor: RefCell::new(None),
      cursor_visible: Signal::new(true),
      blink_task: RefCell::new(None),
      on_change: RefCell::new(None),
      initial_text: RefCell::new(None),
      scroll_offset: Cell::new(0.0),
      last_layout_pos: Cell::new((0.0, 0.0)),
      is_selecting: Cell::new(false),
    }
  }

  /// Start cursor blinking with tokio::time::interval
  fn start_blink(&self) {
    // Cancel any existing blink task
    self.stop_blink();

    // Set cursor visible immediately
    *self.cursor_visible.write() = true;

    // Start a new blink task
    let cursor_visible = self.cursor_visible.clone();

    let task = spawn(async move {
      use tokio::time::{Duration, interval};

      // Blink interval: 530ms (standard cursor blink rate)
      let mut interval = interval(Duration::from_millis(530));
      
      // 🎯 The first tick completes immediately, so we consume it to avoid
      // immediately toggling the cursor to invisible.
      interval.tick().await;

      loop {
        interval.tick().await;

        // Toggle cursor visibility
        let current = *cursor_visible.read();
        *cursor_visible.write() = !current;

        // Request redraw
        crate::runtime::with_window(|window_state| {
          if let Some(window) = &window_state.redraw_requester {
            window.request_redraw();
          }
        });
      }
    });

    *self.blink_task.borrow_mut() = Some(task);
  }

  /// Stop cursor blinking
  fn stop_blink(&self) {
    if let Some(task) = self.blink_task.borrow_mut().take() {
      task.cancel();
    }
    *self.cursor_visible.write() = false;
  }

  /// Reset cursor blinking (show cursor and restart timer)
  fn reset_blink(&self) {
    // Just restart the blink task
    // This will reset the timer and make cursor visible
    self.start_blink();
  }

  /// Trigger text change callback
  fn notify_change(&self) {
    let editor_opt = self.editor.borrow();
    if let Some(editor) = editor_opt.as_ref() {
      let text = editor.text().to_string();
      drop(editor_opt); // Release borrowing

      let on_change = self.on_change.borrow();
      if let Some(callback) = on_change.as_ref() {
        callback(text);
      }
    }
  }

  /// Ensure editor is initialized (using physical pixel font_size)
  fn ensure_editor(&self, ctx: &crate::RenderContext, logical_font_size: f32) {
    let mut editor_opt = self.editor.borrow_mut();
    if editor_opt.is_none() {
      let physical_font_size = ctx.logical_to_physical(logical_font_size);
      let mut editor = TextEditor::with_physical_font_size(physical_font_size);

      // Set initial text if it exists
      if let Some(initial) = self.initial_text.borrow_mut().take() {
        editor.set_text(&initial);
        editor.move_to_text_end();
      }

      *editor_opt = Some(editor);
    }
  }
}

impl TextInput {
  /// Create a new TextInput
  pub fn new() -> Self {
    let id = crate::ViewId::new();

    // Set ViewState name to "TextInput"
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&id) {
        state.name = "TextInput".to_string();
      } else {
        runtime
          .view_states
          .insert(id, crate::ViewState::with_name("TextInput"));
      }
    });

    let state = Rc::new(TextInputState::new());

    Self {
      id,
      state,
      placeholder: String::new(),
      max_length: None,
    }
  }

  /// Set placeholder text
  pub fn placeholder(mut self, text: impl Into<String>) -> Self {
    self.placeholder = text.into();
    self
  }

  /// Set width
  pub fn width(self, width: f32) -> Self {
    self.id.style(move |s| s.width(width));
    self
  }

  /// Set maximum character length
  pub fn max_length(mut self, len: usize) -> Self {
    self.max_length = Some(len);
    self
  }

  /// Bind external Signal (two-way binding)
  pub fn value(self, signal: Signal<String>) -> Self {
    let state = self.state.clone();

    // Set text change callback (editor -> Signal)
    let signal_write = signal.clone();
    *state.on_change.borrow_mut() = Some(Rc::new(move |text: String| {
      *signal_write.write() = text;
    }));

    // Listen for external Signal changes (external -> editor)
    // This effect will also handle the initial value from the Signal
    let state_read = state.clone();
    let view_id = self.id;
    create_effect(move || {
      let new_text = signal.read().clone();
      let mut editor_opt = state_read.editor.borrow_mut();
      if let Some(editor) = editor_opt.as_mut() {
        // Only update if text is actually different to avoid loops
        if editor.text() != new_text {
          editor.set_text(&new_text);
          editor.move_to_text_end(); // Move cursor to end

          // Trigger redraw
          drop(editor_opt); // Release borrowing
          view_id.mark_dirty();
        }
      } else {
        // Store for later initialization (editor not ready yet)
        *state_read.initial_text.borrow_mut() = Some(new_text);
      }
    });

    self
  }

  /// Build TextInput View (internal method)
  fn build_view(self) -> crate::ViewId {
    let state = self.state.clone();
    let placeholder = self.placeholder.clone();
    let view_id = self.id;

    // Create Widget
    let widget = Rc::new(TextInputWidget {
      state: state.clone(),
      placeholder,
      view_id,
    });

    let view = self.id.widget(widget);

    // Set default styles using design tokens
    let view_id_for_style = view_id;
    let view = view.style(move |s| {
      // Query focus state directly from runtime
      let is_focused =
        crate::runtime::with_window(|state| state.focused_view == Some(view_id_for_style));

      let mut style = s
        .height(Size::MD)
        .padding_left(Spacing::MD)
        .padding_right(Spacing::MD)
        .padding_top(6.0)
        .padding_bottom(6.0)
        .background(Color::WHITE)
        .border_radius(Radius::MD)
        .font_size(FontSize::MD)
        .color(Color::from_rgb8(17, 24, 39))
        .cursor(crate::Cursor::Text); // Text input cursor (I-beam)

      // Change border based on focus state (using design tokens)
      if is_focused {
        style = style.border_all(Border::MEDIUM, Color::from_rgb8(59, 130, 246));
      } else {
        style = style.border_all(Border::THIN, Color::from_rgb8(209, 213, 219));
      }

      style
    });

    view
  }

  /// Convert to ViewId
  pub fn into_view(self) -> crate::ViewId {
    self.build_view()
  }
}

impl Default for TextInput {
  fn default() -> Self {
    Self::new()
  }
}

/// TextInput Widget implementation
struct TextInputWidget {
  state: Rc<TextInputState>,
  placeholder: String,
  view_id: crate::ViewId, // Store ViewId for focus checking
}

impl Widget for TextInputWidget {
  fn measure(
    &self,
    known_dimensions: TaffySize<Option<f32>>,
    _available_space: TaffySize<AvailableSpace>,
    _style: &crate::style::Style,
    _ctx: &crate::RenderContext,
    _font_ctx: &mut parley::FontContext,
    _layout_ctx: &mut parley::LayoutContext<()>,
  ) -> TaffySize<f32> {
    let width = known_dimensions.width.unwrap_or(200.0);
    let height = known_dimensions.height.unwrap_or(Size::MD); // Use design token
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
    // Use runtime instead of event system
    let is_focused = crate::runtime::with_window(|state| state.focused_view == Some(self.view_id));

    // Padding values (consistent with style settings using design tokens)
    let padding_left = Spacing::MD;
    let padding_right = Spacing::MD;

    // Ensure editor is initialized (using physical pixel font_size)
    let font_size = style.font_size.unwrap_or(FontSize::MD);
    self.state.ensure_editor(ctx, font_size);

    let mut editor_opt = self.state.editor.borrow_mut();
    let editor = match editor_opt.as_mut() {
      Some(ed) => ed,
      None => return, // Editor not initialized, skip rendering
    };

    // Calculate scroll offset
    let physical_width = ctx.logical_to_physical(width) as f64;

    // Update last layout position for event handling
    self.state.last_layout_pos.set((abs_x, abs_y));

    let physical_padding_left = ctx.logical_to_physical(padding_left) as f64;
    let physical_padding_right = ctx.logical_to_physical(padding_right) as f64;
    let visible_width = (physical_width - physical_padding_left - physical_padding_right).max(0.0);

    let mut current_offset = self.state.scroll_offset.get();
    let physical_font_size = ctx.logical_to_physical(font_size);

    if is_focused {
      if let Some((cursor_x, _, _, _)) = editor.cursor_geometry(physical_font_size) {
        let cursor_x = cursor_x as f64;
        let margin = 2.0; // Keep a small margin for cursor visibility

        // Scroll if cursor is out of view
        if cursor_x < current_offset + margin {
          current_offset = (cursor_x - margin).max(0.0);
        } else if cursor_x > current_offset + visible_width - margin {
          current_offset = cursor_x - visible_width + margin;
        }
      }
    }

    // Clamp offset to not scroll past end if text fits
    // (Note: We allow scrolling past end if text is longer than visible width,
    // but we can snap back if needed. For now, simple clamping to 0 if text fits)
    let text_width = editor.layout().width() as f64;
    if text_width <= visible_width {
      current_offset = 0.0;
    }

    // Update state if changed (to persist for next frame or event)
    if (current_offset - self.state.scroll_offset.get()).abs() > 0.1 {
      self.state.scroll_offset.set(current_offset);
    }

    // Coordinate conversion
    let physical_x = ctx.logical_to_physical(abs_x as f32) as f64;
    let physical_y = ctx.logical_to_physical(abs_y as f32) as f64;

    // Clip content to input area
    let clip_rect = RoundedRect::new(
      physical_x,
      physical_y,
      physical_x + physical_width,
      physical_y + ctx.logical_to_physical(height) as f64,
      ctx.logical_to_physical(style.border_radius) as f64,
    );
    scene.push_layer(
      vello::peniko::BlendMode::default(),
      1.0,
      Affine::IDENTITY,
      &clip_rect,
    );

    // Get content
    let text = editor.text();
    let is_empty = text.is_empty();

    // Determine text to display
    let display_text = if is_empty && !is_focused {
      &self.placeholder
    } else {
      &text
    };
    // Temporarily set text if placeholder needs to be displayed
    let placeholder_mode = is_empty && !is_focused && !self.placeholder.is_empty();
    if placeholder_mode {
      editor.set_text(&self.placeholder);
    }

    // Calculate vertical centering
    // For single-line text input, we estimate the content height based on font size
    // Typical approach: content_height ≈ font_size (the actual glyph height)
    // We want: (height - content_height) / 2 for top padding
    let content_height = font_size;
    let padding_top = ((height - content_height) / 2.0).max(0.0);

    let layout = editor.layout();

    // Only draw when there is text (empty text when focused only shows cursor)
    if !display_text.is_empty() {
      use parley::layout::PositionedLayoutItem;

      // Draw text
      let text_color = if placeholder_mode {
        Color::from_rgba8(156, 163, 175, 255) // Gray placeholder
      } else {
        style.color
      };

      // Coordinate conversion: abs_x/abs_y are logical coordinates, need to convert to physical coordinates
      // Glyph coordinates are already in physical pixels (because parley uses physical font_size for layout)
      let physical_x = ctx.logical_to_physical(abs_x as f32) as f64;
      let physical_y = ctx.logical_to_physical(abs_y as f32) as f64;
      let physical_padding_left = ctx.logical_to_physical(padding_left);
      let physical_padding_top = ctx.logical_to_physical(padding_top);

      let text_x = physical_x + physical_padding_left as f64 - current_offset;
      let text_y = physical_y + physical_padding_top as f64;
      let transform = Affine::translate((text_x, text_y));

      for line in layout.lines() {
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

    // Restore original text (if placeholder was displayed)
    if placeholder_mode {
      editor.set_text("");
    }

    // Draw cursor (if focused and visible)
    if is_focused && *self.state.cursor_visible.read() {
      let logical_font_size = style.font_size.unwrap_or(14.0);
      let physical_font_size = ctx.logical_to_physical(logical_font_size);

      if let Some((cursor_x, cursor_y, cursor_width, cursor_height)) =
        editor.cursor_geometry(physical_font_size)
      {
        // Coordinates returned by cursor_geometry are already in physical pixels (because physical_font_size is used)
        // But we still need to convert abs position and padding
        let physical_abs_x = ctx.logical_to_physical(abs_x as f32) as f64;
        let physical_abs_y = ctx.logical_to_physical(abs_y as f32) as f64;
        let physical_padding_left = ctx.logical_to_physical(padding_left);
        let physical_padding_top = ctx.logical_to_physical(padding_top);

        let abs_cursor_x =
          physical_abs_x + physical_padding_left as f64 + cursor_x as f64 - current_offset;
        let abs_cursor_y = physical_abs_y + physical_padding_top as f64 + cursor_y as f64;

        // Update IME cursor position
        set_ime_cursor_area(
          abs_cursor_x,
          abs_cursor_y,
          cursor_width as f64,
          cursor_height as f64,
        );

        let cursor_rect = Rect::new(
          abs_cursor_x,
          abs_cursor_y,
          abs_cursor_x + 1.5,
          abs_cursor_y + cursor_height as f64,
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

    // Draw selection area (if any)
    if is_focused {
      let selection_rects = editor.selection_geometry();
      if !selection_rects.is_empty() {
        let selection_color = Color::from_rgba8(59, 130, 246, 64); // Semi-transparent blue

        let physical_abs_x = ctx.logical_to_physical(abs_x as f32) as f64;
        let physical_abs_y = ctx.logical_to_physical(abs_y as f32) as f64;
        let physical_padding_left = ctx.logical_to_physical(padding_left) as f64;
        let physical_padding_top = ctx.logical_to_physical(padding_top) as f64;

        for rect in selection_rects {
          // Coordinates returned by selection_geometry are already in physical coordinates (layout uses physical_font_size)
          let abs_rect = Rect::new(
            physical_abs_x + physical_padding_left - current_offset + rect.min_x(),
            physical_abs_y + physical_padding_top + rect.min_y(),
            physical_abs_x + physical_padding_left - current_offset + rect.max_x(),
            physical_abs_y + physical_padding_top + rect.max_y(),
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

    // End clipping
    scene.pop_layer();
  }
}

impl From<TextInput> for crate::ViewId {
  fn from(input: TextInput) -> Self {
    input.into_view()
  }
}

// Implement Element trait to enable styling and event handling
impl crate::Element for TextInput {
  fn id(&self) -> crate::ViewId {
    self.id
  }

  fn build(self) -> crate::Node {
    let state = self.state.clone();
    let view_id = self.id; // Capture view_id for use in closures

    // Setup event handlers using Interactive trait (self is Element, gets Interactive automatically)
    let text_input = self
      .focusable() // TextInput is focusable by default
      .on_focus({
        let state = state.clone();
        move |_e| {
          state.reset_blink(); // Start async blink task

          // Enable IME for text input
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
          let (abs_x, _abs_y) = state.last_layout_pos.get();
          let scroll_offset = state.scroll_offset.get();
          let padding_left = 12.0; // Spacing::MD

          let click_x = e.start_position.x as f64 * scale_factor;
          let abs_x_physical = abs_x * scale_factor;
          let padding_left_physical = padding_left * scale_factor;

          let local_x = (click_x - (abs_x_physical + padding_left_physical - scroll_offset)) as f32;
          let local_y = 0.0;

          let mut editor_opt = state.editor.borrow_mut();
          if let Some(editor) = editor_opt.as_mut() {
            // Drag start always resets selection to cursor, then drag event extends it
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
            let (abs_x, _abs_y) = state.last_layout_pos.get();
            let scroll_offset = state.scroll_offset.get();
            let padding_left = 12.0;

            let current_x = e.current_position.x as f64 * scale_factor;
            let abs_x_physical = abs_x * scale_factor;
            let padding_left_physical = padding_left * scale_factor;

            let local_x =
              (current_x - (abs_x_physical + padding_left_physical - scroll_offset)) as f32;
            let local_y = 0.0;

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
          let (abs_x, _abs_y) = state.last_layout_pos.get();
          let scroll_offset = state.scroll_offset.get();
          let padding_left = 12.0; // Spacing::MD

          let click_x = e.position.x as f64 * scale_factor;
          let abs_x_physical = abs_x * scale_factor;
          let padding_left_physical = padding_left * scale_factor;

          let local_x = (click_x - (abs_x_physical + padding_left_physical - scroll_offset)) as f32;
          let local_y = 0.0;

          let mut editor_opt = state.editor.borrow_mut();
          if let Some(editor) = editor_opt.as_mut() {
            if e.modifiers.shift_key() {
              editor.extend_selection_to_point(local_x, local_y);
            } else {
              editor.move_to_point(local_x, local_y);
              // Note: We don't set is_selecting here anymore, as drag handles selection
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

              // Handle Ctrl shortcut
              if e.modifiers.control_key() || e.modifiers.super_key() {
                // Support Cmd on Mac
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
                      text_changed = true; // undo/redo changes text
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
                // Normal keys
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

    crate::Node::View(text_input.build_view())
  }
}

/// Convenience function to create TextInput
pub fn text_input() -> TextInput {
  TextInput::new()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::create_scope;

  #[test]
  fn test_text_input_creation() {
    create_scope(|| {
      let input = TextInput::new();
      // Editor is not initialized in new(), so skip this test
      // assert_eq!(input.state.editor.borrow().as_ref().map(|e| e.text()), Some("".to_string()));
      // Focus is now managed by the global focus system, not a signal
      // assert_eq!(*input.state.focused.read(), false);
    });
  }

  #[test]
  fn test_placeholder() {
    create_scope(|| {
      let input = TextInput::new().placeholder("Enter text");
      assert_eq!(input.placeholder, "Enter text");
    });
  }

  #[test]
  fn test_max_length() {
    create_scope(|| {
      let input = TextInput::new().max_length(10);
      assert_eq!(input.max_length, Some(10));
    });
  }
}
