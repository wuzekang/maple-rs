mod compute;
pub mod dimension;

use crate::{Drawable, Element, ViewId};
use bumpalo::Bump;
use cosmic_text::{Align, Wrap};
use peniko::Color;
use reactive::create_effect;
use sdl3_sys::everything::{SDL_CreateSystemCursor, SDL_Cursor, SDL_SystemCursor};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Pointer;
use std::mem;
use std::rc::Rc;
use taffy::prelude::*;
use taffy::{Overflow, Point};

#[derive(Clone, PartialEq)]
pub enum TextWrap {
    None,
    Glyph,
    Word,
    WordOrGlyph,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
}

impl From<TextWrap> for Wrap {
    fn from(t: TextWrap) -> Self {
        match t {
            TextWrap::None => Wrap::None,
            TextWrap::Glyph => Wrap::Glyph,
            TextWrap::Word => Wrap::Word,
            TextWrap::WordOrGlyph => Wrap::WordOrGlyph,
        }
    }
}

impl Default for TextWrap {
    fn default() -> Self {
        Self::WordOrGlyph
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justified,
    End,
}

impl From<TextAlign> for Align {
    fn from(t: TextAlign) -> Self {
        match t {
            TextAlign::Left => Self::Left,
            TextAlign::Right => Self::Right,
            TextAlign::Center => Self::Center,
            TextAlign::Justified => Self::Justified,
            TextAlign::End => Self::End,
        }
    }
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Clone)]
pub struct Style {
    pub background: Color,
    pub color: Color,
    pub line_height: Option<f32>,
    pub font_size: Option<f32>,
    pub text_wrap: TextWrap,
    pub text_align: TextAlign,
    pub cursor: Cursor,
    pub pointer_events: PointerEvents,
    pub translate: Point<LengthPercentage>,
    pub opacity: f32,
    pub clip: bool,
}

impl Style {
    pub const DEFAULT: Self = Self {
        background: Color::TRANSPARENT,
        color: Color::BLACK,
        line_height: None,
        font_size: None,
        text_wrap: TextWrap::WordOrGlyph,
        text_align: TextAlign::Left,
        cursor: Cursor::None,
        pointer_events: PointerEvents::Auto,
        translate: Point {
            x: LengthPercentage::ZERO,
            y: LengthPercentage::ZERO,
        },
        opacity: 1.0,
        clip: false,
    };

    pub fn background(self) -> Color {
        self.background
    }
    pub fn color(self) -> Color {
        self.color
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, PartialEq)]
pub enum StyleProperty {
    Background(Color),
    Color(Color),
    LineHeight(Option<f32>),
    FontSize(Option<f32>),
    TextWrap(TextWrap),
    TextAlign(TextAlign),
    Cursor(Cursor),
    PointerEvents(PointerEvents),
    Translate(Point<LengthPercentage>),
    TranslateX(LengthPercentage),
    TranslateY(LengthPercentage),
    Opacity(f32),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum StylePropertyKey {
    Background,
    Color,
    LineHeight,
    FontSize,
    TextWrap,
    TextAlign,
    Cursor,
    PointerEvents,
    Translate,
    TranslateX,
    TranslateY,
    Opacity,
}

impl StylePropertyKey {
    pub(crate) fn inherited(&self) -> bool {
        match self {
            Self::TextWrap => false,
            Self::Background => false,
            Self::Translate | Self::TranslateX | Self::TranslateY => false,
            Self::Opacity => false,
            _ => true,
        }
    }
}

impl StyleProperty {
    pub(crate) fn assign_to(&self, style: &mut Style) {
        match self {
            Self::Background(value) => {
                style.background = value.clone();
            }
            Self::Color(value) => {
                style.color = value.clone();
            }
            Self::LineHeight(value) => {
                style.line_height = value.clone();
            }
            Self::FontSize(value) => {
                style.font_size = value.clone();
            }
            Self::Cursor(value) => {
                style.cursor = value.clone();
            }
            Self::TextWrap(value) => {
                style.text_wrap = value.clone();
            }
            Self::TextAlign(value) => {
                style.text_align = value.clone();
            }
            Self::PointerEvents(value) => {
                style.pointer_events = value.clone();
            }
            Self::Translate(value) => {
                style.translate = value.clone();
            }
            Self::TranslateX(value) => {
                style.translate.x = value.clone();
            }
            Self::TranslateY(value) => {
                style.translate.y = value.clone();
            }
            Self::Opacity(value) => {
                style.opacity = value.clone();
            }
        }
    }

    pub(crate) fn initial() -> HashMap<StylePropertyKey, Self> {
        HashMap::from([
            (
                StylePropertyKey::Background,
                Self::Background(Color::WHITE.multiply_alpha(0.0)),
            ),
            (StylePropertyKey::Color, Self::Color(Color::BLACK)),
            (StylePropertyKey::FontSize, Self::FontSize(Some(12.0))),
            (StylePropertyKey::LineHeight, Self::LineHeight(Some(12.0))),
            (
                StylePropertyKey::TextWrap,
                Self::TextWrap(TextWrap::WordOrGlyph),
            ),
            (
                StylePropertyKey::TextAlign,
                Self::TextAlign(TextAlign::Left),
            ),
            (
                StylePropertyKey::Cursor,
                Self::Cursor(Cursor::system_default()),
            ),
            (
                StylePropertyKey::PointerEvents,
                Self::PointerEvents(PointerEvents::Auto),
            ),
            (StylePropertyKey::Opacity, Self::Opacity(1.0)),
        ])
    }
}

#[derive(Clone, PartialEq)]
pub enum TaffyStyleProperty {
    Display(Display),
    Overflow(Point<Overflow>),
    OverflowX(Overflow),
    OverflowY(Overflow),
    ScrollbarWidth(f32),
    Position(Position),
    Inset(Rect<LengthPercentageAuto>),
    Top(LengthPercentageAuto),
    Right(LengthPercentageAuto),
    Bottom(LengthPercentageAuto),
    Left(LengthPercentageAuto),
    Size(Size<Dimension>),
    Width(Dimension),
    Height(Dimension),
    MinSize(Size<Dimension>),
    MinWidth(Dimension),
    MinHeight(Dimension),
    MaxSize(Size<Dimension>),
    MaxWidth(Dimension),
    MaxHeight(Dimension),
    AspectRatio(Option<f32>),
    Margin(Rect<LengthPercentageAuto>),
    MarginTop(LengthPercentageAuto),
    MarginRight(LengthPercentageAuto),
    MarginBottom(LengthPercentageAuto),
    MarginLeft(LengthPercentageAuto),
    Padding(Rect<LengthPercentage>),
    PaddingTop(LengthPercentage),
    PaddingRight(LengthPercentage),
    PaddingBottom(LengthPercentage),
    PaddingLeft(LengthPercentage),
    Border(Rect<LengthPercentage>),
    BorderTop(LengthPercentage),
    BorderRight(LengthPercentage),
    BorderBottom(LengthPercentage),
    BorderLeft(LengthPercentage),
    AlignItems(Option<AlignItems>),
    AlignSelf(Option<AlignSelf>),
    JustifyItems(Option<AlignItems>),
    JustifySelf(Option<AlignSelf>),
    AlignContent(Option<AlignContent>),
    JustifyContent(Option<JustifyContent>),
    Gap(Size<LengthPercentage>),
    GapRow(LengthPercentage),
    GapColumn(LengthPercentage),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    FlexBasis(Dimension),
    FlexGrow(f32),
    FlexShrink(f32),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TaffyStylePropertyKey {
    Display,
    Overflow,
    OverflowX,
    OverflowY,
    ScrollbarWidth,
    Position,
    Inset,
    Top,
    Right,
    Bottom,
    Left,
    Size,
    Width,
    Height,
    MinSize,
    MinWidth,
    MinHeight,
    MaxSize,
    MaxWidth,
    MaxHeight,
    AspectRatio,
    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    Border,
    BorderTop,
    BorderRight,
    BorderBottom,
    BorderLeft,
    AlignItems,
    AlignSelf,
    JustifyItems,
    JustifySelf,
    AlignContent,
    JustifyContent,
    Gap,
    GapRow,
    GapColumn,
    FlexDirection,
    FlexWrap,
    FlexBasis,
    FlexGrow,
    FlexShrink,
}

impl TaffyStyleProperty {
    pub fn assign_to(self, style: &mut taffy::Style) {
        match self {
            Self::Display(value) => {
                style.display = value;
            }
            Self::Overflow(value) => {
                style.overflow = value;
            }
            Self::OverflowX(value) => {
                style.overflow.x = value;
            }
            Self::OverflowY(value) => {
                style.overflow.y = value;
            }
            Self::ScrollbarWidth(value) => {
                style.scrollbar_width = value;
            }
            Self::Position(value) => {
                style.position = value;
            }
            Self::Inset(value) => {
                style.inset = value;
            }
            Self::Top(value) => {
                style.inset.top = value;
            }
            Self::Right(value) => {
                style.inset.right = value;
            }
            Self::Bottom(value) => {
                style.inset.bottom = value;
            }
            Self::Left(value) => {
                style.inset.left = value;
            }
            Self::Size(value) => {
                style.size = value;
            }
            Self::Width(value) => {
                style.size.width = value;
            }
            Self::Height(value) => {
                style.size.height = value;
            }
            Self::MinSize(value) => {
                style.min_size = value;
            }
            Self::MinWidth(value) => {
                style.min_size.width = value;
            }
            Self::MinHeight(value) => {
                style.min_size.height = value;
            }
            Self::MaxSize(value) => {
                style.max_size = value;
            }
            Self::MaxWidth(value) => {
                style.max_size.width = value;
            }
            Self::MaxHeight(value) => {
                style.max_size.height = value;
            }
            Self::AspectRatio(value) => {
                style.aspect_ratio = value;
            }
            Self::Margin(value) => {
                style.margin = value;
            }
            Self::MarginTop(value) => {
                style.margin.top = value;
            }
            Self::MarginRight(value) => {
                style.margin.right = value;
            }
            Self::MarginBottom(value) => {
                style.margin.bottom = value;
            }
            Self::MarginLeft(value) => {
                style.margin.left = value;
            }
            Self::Padding(value) => {
                style.padding = value;
            }
            Self::PaddingTop(value) => {
                style.padding.top = value;
            }
            Self::PaddingRight(value) => {
                style.padding.right = value;
            }
            Self::PaddingBottom(value) => {
                style.padding.bottom = value;
            }
            Self::PaddingLeft(value) => {
                style.padding.left = value;
            }
            Self::Border(value) => {
                style.border = value;
            }
            Self::BorderTop(value) => {
                style.border.top = value;
            }
            Self::BorderRight(value) => {
                style.border.right = value;
            }
            Self::BorderBottom(value) => {
                style.border.bottom = value;
            }
            Self::BorderLeft(value) => {
                style.border.left = value;
            }
            Self::AlignItems(value) => {
                style.align_items = value;
            }
            Self::AlignSelf(value) => {
                style.align_self = value;
            }
            Self::JustifyItems(value) => {
                style.justify_items = value;
            }
            Self::JustifySelf(value) => {
                style.justify_self = value;
            }
            Self::AlignContent(value) => {
                style.align_content = value;
            }
            Self::JustifyContent(value) => {
                style.justify_content = value;
            }
            Self::Gap(value) => {
                style.gap = value;
            }
            Self::GapRow(value) => {
                style.gap.width = value;
            }
            Self::GapColumn(value) => {
                style.gap.height = value;
            }
            Self::FlexDirection(value) => {
                style.flex_direction = value;
            }
            Self::FlexWrap(value) => {
                style.flex_wrap = value;
            }
            Self::FlexBasis(value) => {
                style.flex_basis = value;
            }
            Self::FlexGrow(value) => {
                style.flex_grow = value;
            }
            Self::FlexShrink(value) => {
                style.flex_shrink = value;
            }
        }
    }
    pub fn initial() -> Vec<(TaffyStylePropertyKey, Self)> {
        Vec::from([
            (
                TaffyStylePropertyKey::JustifyContent,
                Self::JustifyContent(Some(taffy::JustifyContent::FlexStart)),
            ),
            (
                TaffyStylePropertyKey::AlignItems,
                Self::AlignItems(Some(taffy::AlignItems::FlexStart)),
            ),
        ])
    }
}

#[derive(Clone, PartialEq)]
pub struct StyleBuilder {
    pub taffy_style_props: Vec<(TaffyStylePropertyKey, TaffyStyleProperty)>,
    pub style_props: Vec<(StylePropertyKey, StyleProperty)>,
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self {
            taffy_style_props: Default::default(),
            style_props: Default::default(),
        }
    }
}

impl StyleBuilder {
    pub fn display(mut self, value: Display) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Display,
            TaffyStyleProperty::Display(value),
        ));
        self
    }
    pub fn overflow(mut self, value: Point<Overflow>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Overflow,
            TaffyStyleProperty::Overflow(value),
        ));
        self
    }

    pub fn overflow_x(mut self, value: Overflow) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::OverflowX,
            TaffyStyleProperty::OverflowX(value),
        ));
        self
    }

    pub fn overflow_y(mut self, value: Overflow) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::OverflowY,
            TaffyStyleProperty::OverflowY(value),
        ));
        self
    }
    pub fn scrollbar_width(mut self, value: f32) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::ScrollbarWidth,
            TaffyStyleProperty::ScrollbarWidth(value),
        ));
        self
    }
    pub fn position(mut self, value: Position) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Position,
            TaffyStyleProperty::Position(value),
        ));
        self
    }
    pub fn inset(mut self, value: Rect<LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Inset,
            TaffyStyleProperty::Inset(value),
        ));
        self
    }
    pub fn top(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Top,
            TaffyStyleProperty::Top(value.into().into()),
        ));
        self
    }
    pub fn right(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Right,
            TaffyStyleProperty::Right(value.into().into()),
        ));
        self
    }
    pub fn bottom(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Bottom,
            TaffyStyleProperty::Bottom(value.into().into()),
        ));
        self
    }
    pub fn left(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Left,
            TaffyStyleProperty::Left(value.into().into()),
        ));
        self
    }
    pub fn size(mut self, value: Size<Dimension>) -> Self {
        self.taffy_style_props
            .push((TaffyStylePropertyKey::Size, TaffyStyleProperty::Size(value)));
        self
    }
    pub fn width(mut self, value: impl Into<dimension::Dimension>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Width,
            TaffyStyleProperty::Width(value.into().into()),
        ));
        self
    }
    pub fn height(mut self, value: impl Into<dimension::Dimension>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Height,
            TaffyStyleProperty::Height(value.into().into()),
        ));
        self
    }
    pub fn min_size(mut self, value: Size<Dimension>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MinSize,
            TaffyStyleProperty::MinSize(value),
        ));
        self
    }
    pub fn min_width(mut self, value: impl Into<dimension::Dimension>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MinWidth,
            TaffyStyleProperty::MinWidth(value.into().into()),
        ));
        self
    }
    pub fn min_height(mut self, value: impl Into<dimension::Dimension>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MinHeight,
            TaffyStyleProperty::MinHeight(value.into().into()),
        ));
        self
    }
    pub fn max_size(mut self, value: Size<Dimension>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MaxSize,
            TaffyStyleProperty::MaxSize(value),
        ));
        self
    }
    pub fn max_width(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MaxWidth,
            TaffyStyleProperty::MaxWidth(value),
        ));
        self
    }
    pub fn max_height(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MaxHeight,
            TaffyStyleProperty::MaxHeight(value),
        ));
        self
    }
    pub fn aspect_ratio(mut self, value: Option<f32>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::AspectRatio,
            TaffyStyleProperty::AspectRatio(value),
        ));
        self
    }
    pub fn margin(mut self, value: Rect<LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Margin,
            TaffyStyleProperty::Margin(value),
        ));
        self
    }
    pub fn margin_top(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginTop,
            TaffyStyleProperty::MarginTop(value.into().into()),
        ));
        self
    }
    pub fn margin_right(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginRight,
            TaffyStyleProperty::MarginRight(value.into().into()),
        ));
        self
    }
    pub fn margin_bottom(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginBottom,
            TaffyStyleProperty::MarginBottom(value.into().into()),
        ));
        self
    }
    pub fn margin_left(mut self, value: impl Into<dimension::LengthPercentageAuto>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginLeft,
            TaffyStyleProperty::MarginLeft(value.into().into()),
        ));
        self
    }
    pub fn padding(mut self, value: Rect<LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Padding,
            TaffyStyleProperty::Padding(value),
        ));
        self
    }
    pub fn padding_top(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingTop,
            TaffyStyleProperty::PaddingTop(value.into().into()),
        ));
        self
    }
    pub fn padding_right(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingRight,
            TaffyStyleProperty::PaddingRight(value.into().into()),
        ));
        self
    }
    pub fn padding_bottom(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingBottom,
            TaffyStyleProperty::PaddingBottom(value.into().into()),
        ));
        self
    }
    pub fn padding_left(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingLeft,
            TaffyStyleProperty::PaddingLeft(value.into().into()),
        ));
        self
    }
    pub fn border(mut self, value: Rect<LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Border,
            TaffyStyleProperty::Border(value),
        ));
        self
    }
    pub fn border_top(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderTop,
            TaffyStyleProperty::BorderTop(value.into().into()),
        ));
        self
    }
    pub fn border_right(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderRight,
            TaffyStyleProperty::BorderRight(value.into().into()),
        ));
        self
    }
    pub fn border_bottom(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderBottom,
            TaffyStyleProperty::BorderBottom(value.into().into()),
        ));
        self
    }
    pub fn border_left(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderLeft,
            TaffyStyleProperty::BorderLeft(value.into().into()),
        ));
        self
    }
    pub fn align_items(mut self, value: AlignItems) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::AlignItems,
            TaffyStyleProperty::AlignItems(Some(value)),
        ));
        self
    }
    pub fn align_self(mut self, value: Option<AlignSelf>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::AlignSelf,
            TaffyStyleProperty::AlignSelf(value),
        ));
        self
    }
    pub fn justify_items(mut self, value: Option<AlignItems>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::JustifyItems,
            TaffyStyleProperty::JustifyItems(value),
        ));
        self
    }
    pub fn justify_self(mut self, value: Option<AlignSelf>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::JustifySelf,
            TaffyStyleProperty::JustifySelf(value),
        ));
        self
    }
    pub fn align_content(mut self, value: Option<AlignContent>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::AlignContent,
            TaffyStyleProperty::AlignContent(value),
        ));
        self
    }
    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::JustifyContent,
            TaffyStyleProperty::JustifyContent(Some(value)),
        ));
        self
    }
    pub fn gap(mut self, value: Size<LengthPercentage>) -> Self {
        self.taffy_style_props
            .push((TaffyStylePropertyKey::Gap, TaffyStyleProperty::Gap(value)));
        self
    }
    pub fn gap_row(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::GapRow,
            TaffyStyleProperty::GapRow(value.into().into()),
        ));
        self
    }
    pub fn gap_column(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::GapColumn,
            TaffyStyleProperty::GapColumn(value.into().into()),
        ));
        self
    }
    pub fn flex_direction(mut self, value: FlexDirection) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexDirection,
            TaffyStyleProperty::FlexDirection(value),
        ));
        self
    }
    pub fn flex_wrap(mut self) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexWrap,
            TaffyStyleProperty::FlexWrap(FlexWrap::Wrap),
        ));
        self
    }

    pub fn flex_nowrap(mut self) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexWrap,
            TaffyStyleProperty::FlexWrap(FlexWrap::NoWrap),
        ));
        self
    }

    pub fn flex_wrap_reverse(mut self) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexWrap,
            TaffyStyleProperty::FlexWrap(FlexWrap::WrapReverse),
        ));
        self
    }

    pub fn flex_basis(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexBasis,
            TaffyStyleProperty::FlexBasis(value),
        ));
        self
    }
    pub fn flex_grow(mut self, value: f32) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexGrow,
            TaffyStyleProperty::FlexGrow(value),
        ));
        self
    }
    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexShrink,
            TaffyStyleProperty::FlexShrink(value),
        ));
        self
    }

    pub fn background(mut self, value: Color) -> Self {
        self.style_props.push((
            StylePropertyKey::Background,
            StyleProperty::Background(value),
        ));
        self
    }

    pub fn color(mut self, value: Color) -> Self {
        self.style_props
            .push((StylePropertyKey::Color, StyleProperty::Color(value)));
        self
    }

    pub fn font_size(mut self, value: f32) -> Self {
        self.style_props.push((
            StylePropertyKey::FontSize,
            StyleProperty::FontSize(Some(value)),
        ));
        self
    }

    pub fn line_height(mut self, value: f32) -> Self {
        self.style_props.push((
            StylePropertyKey::LineHeight,
            StyleProperty::LineHeight(Some(value)),
        ));
        self
    }

    pub fn text_wrap(mut self, value: TextWrap) -> Self {
        self.style_props
            .push((StylePropertyKey::TextWrap, StyleProperty::TextWrap(value)));
        self
    }

    pub fn text_align(mut self, value: TextAlign) -> Self {
        self.style_props
            .push((StylePropertyKey::TextAlign, StyleProperty::TextAlign(value)));
        self
    }

    pub fn cursor(mut self, value: impl Into<Cursor>) -> Self {
        self.style_props.push((
            StylePropertyKey::Cursor,
            StyleProperty::Cursor(value.into()),
        ));
        self
    }

    pub fn pointer_events(mut self, value: PointerEvents) -> Self {
        self.style_props.push((
            StylePropertyKey::PointerEvents,
            StyleProperty::PointerEvents(value),
        ));
        self
    }

    pub fn translate(mut self, value: Point<LengthPercentage>) -> Self {
        self.style_props
            .push((StylePropertyKey::Translate, StyleProperty::Translate(value)));
        self
    }

    pub fn translate_x(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.style_props.push((
            StylePropertyKey::TranslateX,
            StyleProperty::TranslateX(value.into().into()),
        ));
        self
    }

    pub fn translate_y(mut self, value: impl Into<dimension::LengthPercentage>) -> Self {
        self.style_props.push((
            StylePropertyKey::TranslateY,
            StyleProperty::TranslateY(value.into().into()),
        ));
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.style_props
            .push((StylePropertyKey::Opacity, StyleProperty::Opacity(value)));
        self
    }

    #[inline]
    pub fn w_full(mut self) -> Self {
        self.width(dimension::percent(1.0))
    }

    #[inline]
    pub fn h_full(mut self) -> Self {
        self.height(dimension::percent(1.0))
    }

    #[inline]
    pub fn absolute(mut self) -> Self {
        self.position(Position::Absolute)
    }

    #[inline]
    pub fn block(mut self) -> Self {
        self.display(Display::Block)
    }

    #[inline]
    pub fn overflow_hidden(mut self) -> Self {
        self.overflow(Point {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
    }

    #[inline]
    pub fn overflow_clip(mut self) -> Self {
        self.overflow(Point {
            x: Overflow::Clip,
            y: Overflow::Clip,
        })
    }

    #[inline]
    pub fn overflow_scroll(mut self) -> Self {
        self.overflow(Point {
            x: Overflow::Scroll,
            y: Overflow::Scroll,
        })
    }

    #[inline]
    pub fn overflow_x_hidden(mut self) -> Self {
        self.overflow_x(Overflow::Hidden)
    }

    #[inline]
    pub fn overflow_x_clip(mut self) -> Self {
        self.overflow_x(Overflow::Clip)
    }

    #[inline]
    pub fn overflow_x_scroll(mut self) -> Self {
        self.overflow_x(Overflow::Scroll)
    }

    #[inline]
    pub fn overflow_y_hidden(mut self) -> Self {
        self.overflow_y(Overflow::Hidden)
    }

    #[inline]
    pub fn overflow_y_clip(mut self) -> Self {
        self.overflow_y(Overflow::Clip)
    }

    #[inline]
    pub fn overflow_y_scroll(mut self) -> Self {
        self.overflow_y(Overflow::Scroll)
    }

    #[inline]
    pub fn size_full(mut self) -> Self {
        self.w_full().h_full()
    }

    #[inline]
    pub fn flex_row(mut self) -> Self {
        self.flex_direction(FlexDirection::Row)
    }

    #[inline]
    pub fn flex_col(mut self) -> Self {
        self.flex_direction(FlexDirection::Column)
    }

    #[inline]
    pub fn flex_row_reverse(mut self) -> Self {
        self.flex_direction(FlexDirection::RowReverse)
    }

    #[inline]
    pub fn flex_col_reverse(mut self) -> Self {
        self.flex_direction(FlexDirection::ColumnReverse)
    }

    #[inline]
    pub fn justify_start(mut self) -> Self {
        self.justify_content(JustifyContent::Start)
    }

    #[inline]
    pub fn justify_end(mut self) -> Self {
        self.justify_content(JustifyContent::End)
    }

    #[inline]
    pub fn justify_center(mut self) -> Self {
        self.justify_content(JustifyContent::Center)
    }

    #[inline]
    pub fn justify_stretch(mut self) -> Self {
        self.justify_content(JustifyContent::Stretch)
    }

    #[inline]
    pub fn items_start(mut self) -> Self {
        self.align_items(AlignItems::Start)
    }

    #[inline]
    pub fn items_end(mut self) -> Self {
        self.align_items(AlignItems::End)
    }

    #[inline]
    pub fn items_center(mut self) -> Self {
        self.align_items(AlignItems::Center)
    }

    #[inline]
    pub fn items_stretch(mut self) -> Self {
        self.align_items(AlignItems::Stretch)
    }

    #[inline]
    pub fn bg_white(mut self) -> Self {
        self.background(Color::WHITE)
    }

    #[inline]
    pub fn bg_black(mut self) -> Self {
        self.background(Color::BLACK)
    }

    #[inline]
    pub fn pointer_events_none(mut self) -> Self {
        self.pointer_events(PointerEvents::None)
    }

    #[inline]
    pub fn pointer_events_auto(mut self) -> Self {
        self.pointer_events(PointerEvents::Auto)
    }

    #[inline]
    pub fn text_center(mut self) -> Self {
        self.text_align(TextAlign::Center)
    }
}

#[derive(Clone, PartialEq)]
pub struct SystemCursor {
    pub cursor: *mut SDL_Cursor,
}

thread_local! {
    static SYSTEM_CURSORS: RefCell<HashMap<SDL_SystemCursor, *mut SDL_Cursor>> = Default::default();
}
impl SystemCursor {
    pub fn new(cursor: SDL_SystemCursor) -> Self {
        Self {
            cursor: SYSTEM_CURSORS.with(move |s| {
                *s.borrow_mut()
                    .entry(cursor)
                    .or_insert_with(|| unsafe { SDL_CreateSystemCursor(cursor) })
            }),
        }
    }
}

#[derive(Clone)]
pub struct DrawableCursor(pub Rc<dyn Fn() -> Box<dyn Drawable>>);

impl PartialEq for DrawableCursor {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(Rc::as_ptr(&self.0), Rc::as_ptr(&other.0))
    }
}

impl Drop for SystemCursor {
    fn drop(&mut self) {
        unsafe {
            // SDL_DestroyCursor(self.cursor);
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
    pub fn system_text() -> Self {
        Cursor::System(SystemCursor::new(SDL_SystemCursor::TEXT))
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

pub trait Styleable: Sized + Element {
    fn style<F: Fn(StyleBuilder) -> StyleBuilder + 'static>(self, f: F) -> Self {
        let id = self.id();
        let state = id.state();
        let index = state.borrow().styles.len();
        state.borrow_mut().styles.push(None);

        create_effect(move |_| {
            let style_dirty = {
                let state = id.state();
                let prev = mem::take(&mut state.borrow_mut().styles[index]);
                let next = Some(f(StyleBuilder::default()));
                let style_dirty = prev != next;
                // let inherited_style_dirty = style_dirty && {
                //     let prev = prev
                //         .map(|v| v.style_props)
                //         .unwrap_or_default()
                //         .into_iter()
                //         .collect::<HashMap<_, _>>();
                //     let next = next
                //         .clone()
                //         .map(|v| v.style_props)
                //         .unwrap_or_default()
                //         .into_iter()
                //         .collect::<HashMap<_, _>>();
                //     let keys = prev.keys().chain(next.keys()).collect::<HashSet<_>>();
                //     keys.into_iter()
                //         .any(|k| k.inherited() && prev.get(k) != next.get(k))
                // };
                let mut state = state.borrow_mut();
                state.style_dirty = style_dirty;
                state.styles[index] = next;
                style_dirty
            };
            // {
            //     if style_dirty {
            //         id.request_repaint();
            //     }
            // }
        });
        self
    }
}

pub use compute::*;
