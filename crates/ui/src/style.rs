use peniko::Color;
use taffy::prelude::*;

#[derive(Default)]
pub struct StyleBuilder {
    pub taffy_style: taffy::Style,
    pub style: Style,
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
}

#[derive(Clone, Debug, Default)]
pub struct Style {
    pub background: Color,
    pub color: Color,
    pub line_height: Option<f32>,
    pub font_size: Option<f32>,
}

impl Style {
    pub fn background(self) -> Color {
        self.background
    }
    pub fn color(self) -> Color {
        self.color
    }
}
