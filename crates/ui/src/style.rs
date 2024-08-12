use peniko::Color;
use taffy::{Dimension, Position};

#[derive(Default)]
pub struct StyleBuilder {
    pub taffy_style: taffy::Style,
    pub background: Color,
    pub color: Color,
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

    pub fn flex_direction(mut self, value: taffy::style::FlexDirection) -> Self {
        self.taffy_style.flex_direction = value;
        self
    }

    pub fn flex_grow(mut self, value: f32) -> Self {
        self.taffy_style.flex_grow = value;
        self
    }

    pub fn justify_content(mut self, value: taffy::style::JustifyContent) -> Self {
        self.taffy_style.justify_content = Some(value);
        self
    }

    pub fn align_items(mut self, value: taffy::style::AlignItems) -> Self {
        self.taffy_style.align_items = Some(value);
        self
    }

    pub fn background(mut self, value: Color) -> Self {
        self.background = value;
        self
    }

    pub fn color(mut self, value: Color) -> Self {
        self.color = value;
        self
    }

    pub fn position(mut self, value: Position) -> Self {
        self.taffy_style.position = value;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct Style {
    pub background: Color,
    pub color: Color,
}

impl Style {
    pub fn background(self) -> Color {
        self.background
    }
    pub fn color(self) -> Color {
        self.color
    }
}
