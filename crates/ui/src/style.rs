use crate::Drawable;
use peniko::Color;
use sdl3_sys::everything::{
    SDL_CreateSystemCursor, SDL_Cursor, SDL_DestroyCursor, SDL_SystemCursor,
};
use std::rc::Rc;
use taffy::prelude::*;

#[derive(Default)]
pub struct StyleBuilder {
    pub taffy_style: taffy::Style,
    pub style: Style,
}

#[derive(Clone, PartialEq)]
pub struct SystemCursor {
    cursor: *mut SDL_Cursor,
}

impl SystemCursor {
    pub fn new(cursor: SDL_SystemCursor) -> Self {
        Self {
            cursor: unsafe { SDL_CreateSystemCursor(cursor) },
        }
    }
}

#[derive(Clone)]
pub struct DrawableCursor(pub Rc<dyn Fn() -> Box<dyn Drawable>>);

impl PartialEq for DrawableCursor {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Drop for SystemCursor {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyCursor(self.cursor);
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Cursor {
    None,
    Inherit,
    System(SystemCursor),
    Drawable(DrawableCursor),
}

impl Cursor {
    pub fn is_some(&self) -> bool {
        match *self {
            Cursor::None => false,
            _ => true,
        }
    }

    pub fn is_inherit(&self) -> bool {
        match *self {
            Cursor::Inherit => true,
            _ => false,
        }
    }

    pub fn system_pointer() -> Self {
        Cursor::System(SystemCursor::new(SDL_SystemCursor::POINTER))
    }

    pub fn system_default() -> Self {
        Cursor::System(SystemCursor::new(SDL_SystemCursor::DEFAULT))
    }

    pub fn from_drawable<T>(f: T) -> Self
    where
        T: Fn() -> Box<dyn Drawable> + 'static,
    {
        Self::Drawable(DrawableCursor(Rc::new(f)))
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::Inherit
    }
}

impl StyleBuilder {
    pub fn width(mut self, value: Dimension) -> Self {
        self.taffy_style.size.width = value;
        self
    }

    pub fn height(mut self, value: Dimension) -> Self {
        self.taffy_style.size.height = value;
        self
    }

    pub fn padding(mut self, value: Rect<LengthPercentage>) -> Self {
        self.taffy_style.padding = value;
        self
    }

    pub fn padding_top(mut self, value: LengthPercentage) -> Self {
        self.taffy_style.padding.top = value;
        self
    }

    pub fn padding_right(mut self, value: LengthPercentage) -> Self {
        self.taffy_style.padding.right = value;
        self
    }

    pub fn padding_left(mut self, value: LengthPercentage) -> Self {
        self.taffy_style.padding.left = value;
        self
    }

    pub fn margin_top(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.margin.top = value;
        self
    }

    pub fn margin_left(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.margin.left = value;
        self
    }

    pub fn margin_right(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.margin.right = value;
        self
    }

    pub fn margin_bottom(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.margin.bottom = value;
        self
    }

    pub fn flex_direction(mut self, value: FlexDirection) -> Self {
        self.taffy_style.flex_direction = value;
        self
    }

    pub fn flex_grow(mut self, value: f32) -> Self {
        self.taffy_style.flex_grow = value;
        self
    }

    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.taffy_style.flex_shrink = value;
        self
    }

    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.taffy_style.justify_content = Some(value);
        self
    }

    pub fn align_items(mut self, value: AlignItems) -> Self {
        self.taffy_style.align_items = Some(value);
        self
    }

    pub fn gap(mut self, gap: Size<LengthPercentage>) -> Self {
        self.taffy_style.gap = gap;
        self
    }

    pub fn column_gap(mut self, width: LengthPercentage) -> Self {
        self.taffy_style.gap.width = width;
        self
    }

    pub fn row_gap(mut self, height: LengthPercentage) -> Self {
        self.taffy_style.gap.height = height;
        self
    }

    pub fn position(mut self, value: Position) -> Self {
        self.taffy_style.position = value;
        self
    }

    pub fn left(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.inset.left = value;
        self
    }

    pub fn right(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.inset.right = value;
        self
    }

    pub fn top(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.inset.top = value;
        self
    }

    pub fn bottom(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style.inset.bottom = value;
        self
    }

    pub fn display(mut self, value: Display) -> Self {
        self.taffy_style.display = value;
        self
    }

    pub fn background(mut self, value: Color) -> Self {
        self.style.background = value;
        self
    }

    pub fn color(mut self, value: Color) -> Self {
        self.style.color = value;
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        self.style.font_size = Some(value);
        self
    }

    pub fn line_height(mut self, value: f32) -> Self {
        self.style.line_height = Some(value);
        self
    }

    pub fn cursor(mut self, value: Cursor) -> Self {
        self.style.cursor = value;
        self
    }
}

#[derive(Clone, Default)]
pub struct Style {
    pub background: Color,
    pub color: Color,
    pub line_height: Option<f32>,
    pub font_size: Option<f32>,
    pub cursor: Cursor,
}

impl Style {
    pub fn background(self) -> Color {
        self.background
    }
    pub fn color(self) -> Color {
        self.color
    }
}

pub struct StyleComputeContext {
    pub style: Style,
    pub stack: Vec<Style>,
}

impl StyleComputeContext {
    pub fn new() -> Self {
        Self {
            style: Style {
                cursor: Cursor::default(),
                ..Default::default()
            },
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self) {
        self.stack.push(self.style.clone());
    }

    pub fn pop(&mut self) {
        if let Some(style) = self.stack.pop() {
            self.style = style;
        }
    }
}
