use crate::{Drawable, Element};
use cosmic_text::Wrap;
use peniko::Color;
use reactive::create_effect;
use sdl3_sys::everything::{SDL_CreateSystemCursor, SDL_Cursor, SDL_SystemCursor};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Pointer;
use std::rc::Rc;
use taffy::prelude::*;
use taffy::{Overflow, Point};

#[derive(Clone)]
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

#[derive(Clone, Default)]
pub struct Style {
    pub background: Color,
    pub color: Color,
    pub line_height: Option<f32>,
    pub font_size: Option<f32>,
    pub text_wrap: TextWrap,
    pub cursor: Cursor,
    pub pointer_events: PointerEvents,
}

impl Style {
    pub fn background(self) -> Color {
        self.background
    }
    pub fn color(self) -> Color {
        self.color
    }
}

#[derive(Clone)]
pub enum StyleProperty {
    Background(Color),
    Color(Color),
    LineHeight(Option<f32>),
    FontSize(Option<f32>),
    TextWrap(TextWrap),
    Cursor(Cursor),
    PointerEvents(PointerEvents),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum StylePropertyKey {
    Background,
    Color,
    LineHeight,
    FontSize,
    TextWrap,
    Cursor,
    PointerEvents,
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
            Self::PointerEvents(value) => {
                style.pointer_events = value.clone();
            }
        }
    }

    pub(crate) fn inherited(&self) -> bool {
        match self {
            Self::TextWrap(_) => false,
            Self::Background(_) => false,
            _ => true,
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
                StylePropertyKey::Cursor,
                Self::Cursor(Cursor::system_default()),
            ),
            (
                StylePropertyKey::PointerEvents,
                Self::PointerEvents(PointerEvents::Auto),
            ),
        ])
    }
}

#[derive(Clone)]
pub enum TaffyStyleProperty {
    Display(Display),
    Overflow(Point<Overflow>),
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

#[derive(Clone)]
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
    pub fn top(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props
            .push((TaffyStylePropertyKey::Top, TaffyStyleProperty::Top(value)));
        self
    }
    pub fn right(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Right,
            TaffyStyleProperty::Right(value),
        ));
        self
    }
    pub fn bottom(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Bottom,
            TaffyStyleProperty::Bottom(value),
        ));
        self
    }
    pub fn left(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props
            .push((TaffyStylePropertyKey::Left, TaffyStyleProperty::Left(value)));
        self
    }
    pub fn size(mut self, value: Size<Dimension>) -> Self {
        self.taffy_style_props
            .push((TaffyStylePropertyKey::Size, TaffyStyleProperty::Size(value)));
        self
    }
    pub fn width(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Width,
            TaffyStyleProperty::Width(value),
        ));
        self
    }
    pub fn height(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::Height,
            TaffyStyleProperty::Height(value),
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
    pub fn min_width(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MinWidth,
            TaffyStyleProperty::MinWidth(value),
        ));
        self
    }
    pub fn min_height(mut self, value: Dimension) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MinHeight,
            TaffyStyleProperty::MinHeight(value),
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
    pub fn margin_top(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginTop,
            TaffyStyleProperty::MarginTop(value),
        ));
        self
    }
    pub fn margin_right(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginRight,
            TaffyStyleProperty::MarginRight(value),
        ));
        self
    }
    pub fn margin_bottom(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginBottom,
            TaffyStyleProperty::MarginBottom(value),
        ));
        self
    }
    pub fn margin_left(mut self, value: LengthPercentageAuto) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::MarginLeft,
            TaffyStyleProperty::MarginLeft(value),
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
    pub fn padding_top(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingTop,
            TaffyStyleProperty::PaddingTop(value),
        ));
        self
    }
    pub fn padding_right(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingRight,
            TaffyStyleProperty::PaddingRight(value),
        ));
        self
    }
    pub fn padding_bottom(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingBottom,
            TaffyStyleProperty::PaddingBottom(value),
        ));
        self
    }
    pub fn padding_left(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::PaddingLeft,
            TaffyStyleProperty::PaddingLeft(value),
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
    pub fn border_top(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderTop,
            TaffyStyleProperty::BorderTop(value),
        ));
        self
    }
    pub fn border_right(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderRight,
            TaffyStyleProperty::BorderRight(value),
        ));
        self
    }
    pub fn border_bottom(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderBottom,
            TaffyStyleProperty::BorderBottom(value),
        ));
        self
    }
    pub fn border_left(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::BorderLeft,
            TaffyStyleProperty::BorderLeft(value),
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
    pub fn gap_row(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::GapRow,
            TaffyStyleProperty::GapRow(value),
        ));
        self
    }
    pub fn gap_column(mut self, value: LengthPercentage) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::GapColumn,
            TaffyStyleProperty::GapColumn(value),
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
    pub fn flex_wrap(mut self, value: FlexWrap) -> Self {
        self.taffy_style_props.push((
            TaffyStylePropertyKey::FlexWrap,
            TaffyStyleProperty::FlexWrap(value),
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

    pub fn cursor(mut self, value: impl Into<Cursor>) -> Self {
        self.style_props
            .push((StylePropertyKey::Cursor, StyleProperty::Cursor(value.into())));
        self
    }

    pub fn pointer_events(mut self, value: PointerEvents) -> Self {
        self.style_props.push((
            StylePropertyKey::PointerEvents,
            StyleProperty::PointerEvents(value),
        ));
        self
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

pub struct StyleComputeContext {
    pub style: HashMap<StylePropertyKey, StyleProperty>,
    pub stack: Vec<HashMap<StylePropertyKey, StyleProperty>>,
}

impl StyleComputeContext {
    pub fn new() -> Self {
        Self {
            style: StyleProperty::initial(),
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

pub trait Styleable: Sized + Element {
    fn style<F: Fn(StyleBuilder) -> StyleBuilder + 'static>(self, f: F) -> Self {
        let id = self.id();
        let state = id.state();
        let index = state.borrow().styles.len();
        state.borrow_mut().styles.push(None);

        create_effect(move |_| {
            id.state().borrow_mut().styles[index] = Some(f(StyleBuilder::default()));
        });
        self
    }
}
