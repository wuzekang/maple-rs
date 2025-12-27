//! TextEditor module - Text editing core based on PlainEditor
//!
//! Wraps parley::PlainEditor, providing complete text editing, cursor positioning, and selection functionality

use parley::{FontContext, Layout, LayoutContext, PlainEditor};
use vello::peniko::Brush;

/// Text editor wrapper
pub struct TextEditor {
    editor: PlainEditor<Brush>,
    font_cx: FontContext,
    layout_cx: LayoutContext<Brush>,
}

impl TextEditor {
    /// Create a new text editor
    ///
    /// **Note**: This method assumes DPR=2.0 and is for testing only.
    /// In production, use `with_physical_font_size`
    pub fn new() -> Self {
        Self::with_physical_font_size(28.0) // Assume 14.0 * 2.0
    }

    /// Create text editor with specified physical font size
    ///
    /// # Parameters
    /// - `physical_font_size`: Physical pixel font size (logical size × DPR)
    pub fn with_physical_font_size(physical_font_size: f32) -> Self {
        Self {
            editor: PlainEditor::new(physical_font_size),
            font_cx: FontContext::default(),
            layout_cx: LayoutContext::new(),
        }
    }

    /// Set text content
    pub fn set_text(&mut self, text: &str) {
        self.editor.set_text(text);
        self.editor
            .refresh_layout(&mut self.font_cx, &mut self.layout_cx);
    }

    /// Get text content
    pub fn text(&self) -> String {
        self.editor.text().to_string()
    }

    /// Get layout reference
    pub fn layout(&mut self) -> &Layout<Brush> {
        self.editor
            .layout(&mut self.font_cx, &mut self.layout_cx)
    }

    /// Insert text (at cursor position or replace selection)
    pub fn insert_or_replace(&mut self, text: &str) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .insert_or_replace_selection(text);
    }

    /// Delete character (forward delete, Delete key)
    pub fn delete(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .delete();
    }

    /// Backspace character (backward delete, Backspace key)
    pub fn backdelete(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .backdelete();
    }

    /// Delete selected text
    pub fn delete_selection(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .delete_selection();
    }

    // ========== Cursor movement ==========

    /// Move cursor left
    pub fn move_left(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_left();
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_right();
    }

    /// Move cursor up
    pub fn move_up(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_up();
    }

    /// Move cursor down
    pub fn move_down(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_down();
    }

    /// Move cursor left by one word
    pub fn move_word_left(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_word_left();
    }

    /// Move cursor right by one word
    pub fn move_word_right(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_word_right();
    }

    /// Move cursor to line start
    pub fn move_to_line_start(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_to_line_start();
    }

    /// Move cursor to line end
    pub fn move_to_line_end(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_to_line_end();
    }

    /// Move cursor to text start
    pub fn move_to_text_start(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_to_text_start();
    }

    /// Move cursor to text end
    pub fn move_to_text_end(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_to_text_end();
    }

    /// Move cursor to specified coordinates
    pub fn move_to_point(&mut self, x: f32, y: f32) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .move_to_point(x, y);
    }

    // ========== Text selection ==========

    /// Extend selection left
    pub fn select_left(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_left();
    }

    /// Extend selection right
    pub fn select_right(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_right();
    }

    /// Extend selection up
    pub fn select_up(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_up();
    }

    /// Extend selection down
    pub fn select_down(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_down();
    }

    /// Extend selection left by one word
    pub fn select_word_left(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_word_left();
    }

    /// Extend selection right by one word
    pub fn select_word_right(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_word_right();
    }

    /// Select all
    pub fn select_all(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_all();
    }

    /// Cancel selection (collapse to cursor)
    pub fn collapse_selection(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .collapse_selection();
    }

    /// Select word at specified coordinates (double-click)
    pub fn select_word_at_point(&mut self, x: f32, y: f32) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_word_at_point(x, y);
    }

    /// Select entire line at specified coordinates (triple-click)
    pub fn select_line_at_point(&mut self, x: f32, y: f32) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .select_hard_line_at_point(x, y);
    }

    /// Extend selection to specified coordinates (drag)
    pub fn extend_selection_to_point(&mut self, x: f32, y: f32) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .extend_selection_to_point(x, y);
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.editor.selected_text().map(|s| s.to_string())
    }

    /// Check if selection range is empty
    pub fn has_selection(&self) -> bool {
        !self.editor.raw_selection().is_collapsed()
    }

    // ========== IME support ==========

    /// Check if currently composing with IME
    pub fn is_composing(&self) -> bool {
        self.editor.is_composing()
    }

    /// Set IME composition text
    pub fn set_compose(&mut self, text: &str) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .set_compose(text, None);
    }

    /// Clear IME composition
    pub fn clear_compose(&mut self) {
        self.editor
            .driver(&mut self.font_cx, &mut self.layout_cx)
            .clear_compose();
    }

    // ========== Rendering helpers ==========

    /// Get cursor geometry information (x, y, width, height)
    pub fn cursor_geometry(&self, font_size: f32) -> Option<(f32, f32, f32, f32)> {
        self.editor
            .cursor_geometry(font_size)
            .map(|bbox| {
                (
                    bbox.x0 as f32,
                    bbox.y0 as f32,
                    (bbox.x1 - bbox.x0) as f32,
                    (bbox.y1 - bbox.y0) as f32,
                )
            })
    }

    /// Get selection area geometry information
    pub fn selection_geometry(&self) -> Vec<vello::kurbo::Rect> {
        let mut rects = Vec::new();
        self.editor
            .selection_geometry_with(|bbox, _line_index| {
                rects.push(vello::kurbo::Rect::new(bbox.x0, bbox.y0, bbox.x1, bbox.y1))
            });
        rects
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Create text editor
pub fn create_editor() -> TextEditor {
    TextEditor::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_editing() {
        let mut editor = TextEditor::new();
        editor.set_text("Hello");
        assert_eq!(editor.text(), "Hello");

        editor.insert_or_replace(" World");
        assert_eq!(editor.text(), "Hello World");
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = TextEditor::new();
        editor.set_text("Hello World");

        editor.move_to_text_start();
        editor.move_word_right();
        // 光标应该在 "Hello" 之后
    }

    #[test]
    fn test_selection() {
        let mut editor = TextEditor::new();
        editor.set_text("Hello World");

        editor.select_all();
        assert_eq!(editor.selected_text(), Some("Hello World".to_string()));

        editor.collapse_selection();
        assert_eq!(editor.selected_text(), None);
    }
}
