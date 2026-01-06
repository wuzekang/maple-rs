use crate::{Signal, TextEditor, spawn};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

/// Shared state for TextInput and TextArea
pub struct InputState {
    // Text editor core
    pub editor: RefCell<Option<TextEditor>>,

    // Cursor blinking state
    pub cursor_visible: Signal<bool>,

    // Blink task handle
    pub blink_task: RefCell<Option<crate::Task>>,

    // Text change callback
    pub on_change: RefCell<Option<Rc<dyn Fn(String)>>>,

    // Initial text
    pub initial_text: RefCell<Option<String>>,

    // Scroll offset (x, y) in physical pixels
    // TextInput only uses x (y is always 0), TextArea uses both
    pub scroll_offset: Cell<(f64, f64)>,

    // Last layout position (logical pixels)
    pub last_layout_pos: Cell<(f64, f64)>,
    
    // Is selecting text with mouse
    pub is_selecting: Cell<bool>,

    // Double/Triple click support
    pub last_click_time: Cell<Option<Instant>>,
    pub last_click_pos: Cell<(f64, f64)>,
    pub click_count: Cell<u8>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            editor: RefCell::new(None),
            cursor_visible: Signal::new(true),
            blink_task: RefCell::new(None),
            on_change: RefCell::new(None),
            initial_text: RefCell::new(None),
            scroll_offset: Cell::new((0.0, 0.0)),
            last_layout_pos: Cell::new((0.0, 0.0)),
            is_selecting: Cell::new(false),
            last_click_time: Cell::new(None),
            last_click_pos: Cell::new((0.0, 0.0)),
            click_count: Cell::new(0),
        }
    }

    pub fn start_blink(&self) {
        self.stop_blink();
        *self.cursor_visible.write() = true;
        
        let cursor_visible = self.cursor_visible.clone();
        let task = spawn(async move {
            use tokio::time::{Duration, interval};
            let mut interval = interval(Duration::from_millis(530));
            interval.tick().await;

            loop {
                interval.tick().await;
                let current = *cursor_visible.read();
                *cursor_visible.write() = !current;
                
                crate::runtime::with_window(|window_state| {
                    if let Some(window) = &window_state.redraw_requester {
                        window.request_redraw();
                    }
                });
            }
        });

        *self.blink_task.borrow_mut() = Some(task);
    }

    pub fn stop_blink(&self) {
        if let Some(task) = self.blink_task.borrow_mut().take() {
            task.cancel();
        }
        *self.cursor_visible.write() = false;
    }

    pub fn reset_blink(&self) {
        self.start_blink();
    }

    pub fn notify_change(&self) {
        let editor_opt = self.editor.borrow();
        if let Some(editor) = editor_opt.as_ref() {
            let text = editor.text().to_string();
            drop(editor_opt);
            
            let on_change = self.on_change.borrow();
            if let Some(callback) = on_change.as_ref() {
                callback(text);
            }
        }
    }

    pub fn ensure_editor(&self, ctx: &crate::RenderContext, logical_font_size: f32) {
        let mut editor_opt = self.editor.borrow_mut();
        if editor_opt.is_none() {
            let physical_font_size = ctx.logical_to_physical(logical_font_size);
            let mut editor = TextEditor::with_physical_font_size(physical_font_size);
            
            if let Some(initial) = self.initial_text.borrow_mut().take() {
                editor.set_text(&initial);
                editor.move_to_text_end();
            }
            
            *editor_opt = Some(editor);
        }
    }

    /// Handle mouse down event, including click count detection logic
    /// Returns (is_double_click, is_triple_click)
    pub fn handle_click(&self, click_pos: (f64, f64)) -> (bool, bool) {
        let now = Instant::now();
        let last_time = self.last_click_time.get();
        let last_pos = self.last_click_pos.get();
        let count = self.click_count.get();
        
        // Thresholds
        let time_threshold = std::time::Duration::from_millis(500); // 500ms for double click
        let dist_threshold: f64 = 5.0; // 5 pixels movement allowed

        let is_within_time = if let Some(t) = last_time {
            now.duration_since(t) < time_threshold
        } else {
            false
        };

        let dist_sq = (click_pos.0 - last_pos.0).powi(2) + (click_pos.1 - last_pos.1).powi(2);
        let is_within_dist = dist_sq < dist_threshold.powi(2);

        let new_count = if is_within_time && is_within_dist {
            (count % 3) + 1
        } else {
            1
        };

        self.last_click_time.set(Some(now));
        self.last_click_pos.set(click_pos);
        self.click_count.set(new_count);

        (new_count == 2, new_count == 3)
    }
}
