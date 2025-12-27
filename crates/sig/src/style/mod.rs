//! Style system providing chainable style setting API

pub mod compute;
pub mod dimension;

use crate::create_effect;
pub use dimension::*;
use strum::EnumIter;
use taffy::{AlignContent, AlignItems, AlignSelf, Display, FlexDirection, FlexWrap};
use taffy::{JustifyContent, Overflow, Point, Position};
use vello::peniko::Color;

use crate::cursor::Cursor;

/// Default scrollbar width (logical pixels)
/// This matches the SCROLLBAR_WIDTH constant in render/view_render.rs
pub const DEFAULT_SCROLLBAR_WIDTH: f32 = 8.0;

/// Rendering style - stores style values for vello rendering
#[derive(Clone, Debug)]
pub struct Style {
  pub background: Color,
  pub color: Color,
  pub line_height: Option<f32>,
  pub font_size: Option<f32>,
  pub font_weight: Option<u16>,
  pub text_wrap: TextWrap,
  pub text_align: TextAlign,
  pub pointer_events: PointerEvents,
  pub translate: Point<taffy::LengthPercentage>,
  pub opacity: f32,
  pub clip: bool,
  pub border_color: Color,
  pub border_radius: f32,
  pub cursor: Cursor,
}

impl Style {
  /// Default style
  pub const DEFAULT: Self = Self {
    background: Color::TRANSPARENT,
    color: Color::BLACK,
    line_height: Some(16.0),
    font_size: Some(14.0),
    font_weight: Some(400),
    text_wrap: TextWrap::WordOrGlyph,
    text_align: TextAlign::Left,
    pointer_events: PointerEvents::Auto,
    translate: Point {
      x: taffy::LengthPercentage::Length(0.0),
      y: taffy::LengthPercentage::Length(0.0),
    },
    opacity: 1.0,
    clip: false,
    border_color: Color::TRANSPARENT,
    border_radius: 0.0,
    cursor: Cursor::Default,
  };
}

impl Default for Style {
  fn default() -> Self {
    Self::DEFAULT
  }
}

/// Text wrapping
#[derive(Debug, Clone, PartialEq)]
pub enum TextWrap {
  None,
  Glyph,
  Word,
  WordOrGlyph,
}

impl Default for TextWrap {
  fn default() -> Self {
    Self::WordOrGlyph
  }
}

/// Text alignment
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TextAlign {
  Left,
  Right,
  Center,
  Justified,
  End,
}

impl Default for TextAlign {
  fn default() -> Self {
    Self::Left
  }
}

/// Pointer events
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PointerEvents {
  #[default]
  Auto,
  None,
}

/// Style trigger - for fine-grained redraw control
///
/// Different style changes require different levels of redraw:
/// - None: No redraw needed (e.g. cursor)
/// - Composite: Only composition layer update (e.g. transform, opacity)
/// - Paint: Repaint needed (e.g. color, background)
/// - Layout: Re-layout needed (e.g. width, font-size)
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum StyleTrigger {
  None,
  Composite,
  Paint,
  Layout,
}

/// Rendering style property - used for vello rendering
#[derive(Clone, PartialEq, Debug)]
pub enum StyleProperty {
  Background(Color),
  Color(Color),
  LineHeight(Option<f32>),
  FontSize(Option<f32>),
  FontWeight(Option<u16>),
  TextWrap(TextWrap),
  TextAlign(TextAlign),
  PointerEvents(PointerEvents),
  Translate(Point<taffy::LengthPercentage>),
  TranslateX(taffy::LengthPercentage),
  TranslateY(taffy::LengthPercentage),
  Opacity(f32),
  Clip(bool),
  BorderColor(Color),
  BorderRadius(f32),
  Cursor(Cursor),
}

/// Rendering style property key
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum StylePropertyKey {
  Background,
  Color,
  LineHeight,
  FontSize,
  FontWeight,
  TextWrap,
  TextAlign,
  PointerEvents,
  Translate,
  TranslateX,
  TranslateY,
  Opacity,
  Clip,
  BorderColor,
  BorderRadius,
  Cursor,
}

impl StylePropertyKey {
  /// Check if property is inheritable
  pub(crate) fn inherited(&self) -> bool {
    match self {
      Self::TextWrap => false,
      Self::Background => false,
      Self::Translate | Self::TranslateX | Self::TranslateY => false,
      Self::Opacity => false,
      Self::Clip => false,
      Self::BorderColor => false,
      Self::BorderRadius => false,
      _ => true,
    }
  }

  /// Get redraw trigger level for style change
  pub(crate) fn trigger(&self) -> StyleTrigger {
    match self {
      Self::Cursor => StyleTrigger::None,
      Self::PointerEvents => StyleTrigger::None,
      
      Self::Translate | Self::TranslateX | Self::TranslateY => StyleTrigger::Composite,
      Self::Opacity => StyleTrigger::Composite,
      
      Self::Background => StyleTrigger::Paint,
      Self::Color => StyleTrigger::Paint,
      Self::BorderColor => StyleTrigger::Paint,
      Self::BorderRadius => StyleTrigger::Paint,
      Self::Clip => StyleTrigger::Paint,
      
      Self::LineHeight => StyleTrigger::Layout,
      Self::FontSize => StyleTrigger::Layout,
      Self::FontWeight => StyleTrigger::Layout,
      Self::TextWrap => StyleTrigger::Layout,
      Self::TextAlign => StyleTrigger::Layout,
    }
  }

  /// Get property value from Style
  pub(crate) fn value(&self, style: &Style) -> StyleProperty {
    match self {
      Self::Background => StyleProperty::Background(style.background),
      Self::Color => StyleProperty::Color(style.color),
      Self::LineHeight => StyleProperty::LineHeight(style.line_height),
      Self::FontSize => StyleProperty::FontSize(style.font_size),
      Self::FontWeight => StyleProperty::FontWeight(style.font_weight),
      Self::TextWrap => StyleProperty::TextWrap(style.text_wrap.clone()),
      Self::TextAlign => StyleProperty::TextAlign(style.text_align),
      Self::PointerEvents => StyleProperty::PointerEvents(style.pointer_events.clone()),
      Self::Translate => StyleProperty::Translate(style.translate),
      Self::TranslateX => StyleProperty::TranslateX(style.translate.x),
      Self::TranslateY => StyleProperty::TranslateY(style.translate.y),
      Self::Opacity => StyleProperty::Opacity(style.opacity),
      Self::Clip => StyleProperty::Clip(style.clip),
      Self::BorderColor => StyleProperty::BorderColor(style.border_color),
      Self::BorderRadius => StyleProperty::BorderRadius(style.border_radius),
      Self::Cursor => StyleProperty::Cursor(style.cursor),
    }
  }
}

impl StyleProperty {
  /// Apply property to Style
  pub(crate) fn assign_to(&self, style: &mut Style) {
    match self {
      Self::Background(value) => {
        style.background = *value;
      }
      Self::Color(value) => {
        style.color = *value;
      }
      Self::LineHeight(value) => {
        style.line_height = *value;
      }
      Self::FontSize(value) => {
        style.font_size = *value;
      }
      Self::FontWeight(value) => {
        style.font_weight = *value;
      }
      Self::TextWrap(value) => {
        style.text_wrap = value.clone();
      }
      Self::TextAlign(value) => {
        style.text_align = *value;
      }
      Self::PointerEvents(value) => {
        style.pointer_events = value.clone();
      }
      Self::Translate(value) => {
        style.translate = *value;
      }
      Self::TranslateX(value) => {
        style.translate.x = *value;
      }
      Self::TranslateY(value) => {
        style.translate.y = *value;
      }
      Self::Opacity(value) => {
        style.opacity = *value;
      }
      Self::Clip(value) => {
        style.clip = *value;
      }
      Self::BorderColor(value) => {
        style.border_color = *value;
      }
      Self::BorderRadius(value) => {
        style.border_radius = *value;
      }
      Self::Cursor(value) => {
        style.cursor = *value;
      }
    }
  }

  /// Initial value list
  pub(crate) fn initial() -> Vec<(StylePropertyKey, Self)> {
    Vec::from([
      (
        StylePropertyKey::Background,
        Self::Background(Color::TRANSPARENT),
      ),
      (StylePropertyKey::Color, Self::Color(Color::BLACK)),
      (StylePropertyKey::FontSize, Self::FontSize(Some(14.0))),
      (StylePropertyKey::FontWeight, Self::FontWeight(Some(400))),
      (StylePropertyKey::LineHeight, Self::LineHeight(Some(16.0))),
      (
        StylePropertyKey::TextWrap,
        Self::TextWrap(TextWrap::WordOrGlyph),
      ),
      (
        StylePropertyKey::TextAlign,
        Self::TextAlign(TextAlign::Left),
      ),
      (
        StylePropertyKey::PointerEvents,
        Self::PointerEvents(PointerEvents::Auto),
      ),
      (StylePropertyKey::Opacity, Self::Opacity(1.0)),
      (StylePropertyKey::Clip, Self::Clip(false)),
      (
        StylePropertyKey::BorderColor,
        Self::BorderColor(Color::TRANSPARENT),
      ),
      (StylePropertyKey::BorderRadius, Self::BorderRadius(0.0)),
    ])
  }
}

/// Style property - Taffy style properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaffyStyleProperty {
  Display(Display),
  Overflow(Point<Overflow>),
  OverflowX(Overflow),
  OverflowY(Overflow),
  ScrollbarWidth(f32),  // ← 新增
  Position(Position),
  Inset(Rect<LengthPercentageAuto>),
  Top(LengthPercentageAuto),
  Right(LengthPercentageAuto),
  Bottom(LengthPercentageAuto),
  Left(LengthPercentageAuto),
  Size(taffy::Size<taffy::Dimension>),
  Width(taffy::Dimension),
  Height(taffy::Dimension),
  MinSize(taffy::Size<taffy::Dimension>),
  MinWidth(taffy::Dimension),
  MinHeight(taffy::Dimension),
  MaxSize(taffy::Size<taffy::Dimension>),
  MaxWidth(taffy::Dimension),
  MaxHeight(taffy::Dimension),
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
  Gap(taffy::Size<taffy::LengthPercentage>),
  GapRow(taffy::LengthPercentage),
  GapColumn(taffy::LengthPercentage),
  FlexDirection(FlexDirection),
  FlexWrap(FlexWrap),
  FlexBasis(taffy::Dimension),
  FlexGrow(f32),
  FlexShrink(f32),
}

/// Style property key - for deduplication and updates
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum TaffyStylePropertyKey {
  Display,
  Overflow,
  OverflowX,
  OverflowY,
  ScrollbarWidth,  // ← 新增
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
  /// Apply style property to taffy::Style
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
        style.inset = value.into();
      }
      Self::Top(value) => {
        style.inset.top = value.into();
      }
      Self::Right(value) => {
        style.inset.right = value.into();
      }
      Self::Bottom(value) => {
        style.inset.bottom = value.into();
      }
      Self::Left(value) => {
        style.inset.left = value.into();
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
        style.margin = value.into();
      }
      Self::MarginTop(value) => {
        style.margin.top = value.into();
      }
      Self::MarginRight(value) => {
        style.margin.right = value.into();
      }
      Self::MarginBottom(value) => {
        style.margin.bottom = value.into();
      }
      Self::MarginLeft(value) => {
        style.margin.left = value.into();
      }
      Self::Padding(value) => {
        style.padding = value.into();
      }
      Self::PaddingTop(value) => {
        style.padding.top = value.into();
      }
      Self::PaddingRight(value) => {
        style.padding.right = value.into();
      }
      Self::PaddingBottom(value) => {
        style.padding.bottom = value.into();
      }
      Self::PaddingLeft(value) => {
        style.padding.left = value.into();
      }
      Self::Border(value) => {
        style.border = value.into();
      }
      Self::BorderTop(value) => {
        style.border.top = value.into();
      }
      Self::BorderRight(value) => {
        style.border.right = value.into();
      }
      Self::BorderBottom(value) => {
        style.border.bottom = value.into();
      }
      Self::BorderLeft(value) => {
        style.border.left = value.into();
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

  /// Initial value list
  pub(crate) fn initial() -> Vec<(TaffyStylePropertyKey, Self)> {
    Vec::from([
      (
        TaffyStylePropertyKey::JustifyContent,
        Self::JustifyContent(Some(JustifyContent::FlexStart)),
      ),
      (
        TaffyStylePropertyKey::AlignItems,
        Self::AlignItems(Some(AlignItems::FlexStart)),
      ),
    ])
  }
}

/// Style builder - provides chainable API
#[derive(Clone, PartialEq)]
pub struct StyleBuilder {
  pub(crate) taffy_style_props: Vec<(TaffyStylePropertyKey, TaffyStyleProperty)>,
  pub(crate) style_props: Vec<(StylePropertyKey, StyleProperty)>,
  pub(crate) taffy_node_id: Option<taffy::NodeId>,
}

impl Default for StyleBuilder {
  fn default() -> Self {
    Self {
      taffy_style_props: Vec::new(),
      style_props: Vec::new(),
      taffy_node_id: None,
    }
  }
}

impl Drop for StyleBuilder {
  fn drop(&mut self) {
    // Styles are applied immediately in chained methods, not in Drop
  }
}

impl StyleBuilder {
  /// Create a new style builder
  pub fn new() -> Self {
    Self::default()
  }

  /// Create style builder for a specific taffy node
  pub fn new_for_node(node_id: taffy::NodeId) -> Self {
    Self {
      taffy_style_props: Vec::new(),
      style_props: Vec::new(),
      taffy_node_id: Some(node_id),
    }
  }

  // ==================== Display & Overflow ====================

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

  // ==================== Position ====================

  pub fn position(mut self, value: Position) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Position,
      TaffyStyleProperty::Position(value),
    ));
    self
  }

  pub fn inset(mut self, value: impl Into<Rect<LengthPercentageAuto>>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Inset,
      TaffyStyleProperty::Inset(value.into()),
    ));
    self
  }

  pub fn top(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Top,
      TaffyStyleProperty::Top(value.into()),
    ));
    self
  }

  pub fn right(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Right,
      TaffyStyleProperty::Right(value.into()),
    ));
    self
  }

  pub fn bottom(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Bottom,
      TaffyStyleProperty::Bottom(value.into()),
    ));
    self
  }

  pub fn left(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Left,
      TaffyStyleProperty::Left(value.into()),
    ));
    self
  }

  // ==================== Size ====================

  pub fn size(mut self, value: taffy::Size<taffy::Dimension>) -> Self {
    self
      .taffy_style_props
      .push((TaffyStylePropertyKey::Size, TaffyStyleProperty::Size(value)));
    self
  }

  pub fn width(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Width,
      TaffyStyleProperty::Width(value.into().into()),
    ));
    self
  }

  pub fn height(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Height,
      TaffyStyleProperty::Height(value.into().into()),
    ));
    self
  }

  pub fn min_size(mut self, value: taffy::Size<taffy::Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MinSize,
      TaffyStyleProperty::MinSize(value),
    ));
    self
  }

  pub fn min_width(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MinWidth,
      TaffyStyleProperty::MinWidth(value.into().into()),
    ));
    self
  }

  pub fn min_height(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MinHeight,
      TaffyStyleProperty::MinHeight(value.into().into()),
    ));
    self
  }

  pub fn max_size(mut self, value: taffy::Size<taffy::Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MaxSize,
      TaffyStyleProperty::MaxSize(value),
    ));
    self
  }

  pub fn max_width(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MaxWidth,
      TaffyStyleProperty::MaxWidth(value.into().into()),
    ));
    self
  }

  pub fn max_height(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MaxHeight,
      TaffyStyleProperty::MaxHeight(value.into().into()),
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

  // ==================== Margin ====================

  pub fn margin(mut self, value: impl Into<Rect<LengthPercentageAuto>>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Margin,
      TaffyStyleProperty::Margin(value.into()),
    ));
    self
  }

  pub fn margin_top(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MarginTop,
      TaffyStyleProperty::MarginTop(value.into()),
    ));
    self
  }

  pub fn margin_right(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MarginRight,
      TaffyStyleProperty::MarginRight(value.into()),
    ));
    self
  }

  pub fn margin_bottom(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MarginBottom,
      TaffyStyleProperty::MarginBottom(value.into()),
    ));
    self
  }

  pub fn margin_left(mut self, value: impl Into<LengthPercentageAuto>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::MarginLeft,
      TaffyStyleProperty::MarginLeft(value.into()),
    ));
    self
  }

  // ==================== Padding ====================

  pub fn padding(mut self, value: impl Into<Rect<LengthPercentage>>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Padding,
      TaffyStyleProperty::Padding(value.into()),
    ));
    self
  }

  pub fn padding_top(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::PaddingTop,
      TaffyStyleProperty::PaddingTop(value.into()),
    ));
    self
  }

  pub fn padding_right(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::PaddingRight,
      TaffyStyleProperty::PaddingRight(value.into()),
    ));
    self
  }

  pub fn padding_bottom(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::PaddingBottom,
      TaffyStyleProperty::PaddingBottom(value.into()),
    ));
    self
  }

  pub fn padding_left(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::PaddingLeft,
      TaffyStyleProperty::PaddingLeft(value.into()),
    ));
    self
  }

  // ==================== Border ====================

  pub fn border(mut self, value: impl Into<Rect<LengthPercentage>>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Border,
      TaffyStyleProperty::Border(value.into()),
    ));
    self
  }

  pub fn border_top(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::BorderTop,
      TaffyStyleProperty::BorderTop(value.into()),
    ));
    self
  }

  pub fn border_right(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::BorderRight,
      TaffyStyleProperty::BorderRight(value.into()),
    ));
    self
  }

  pub fn border_bottom(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::BorderBottom,
      TaffyStyleProperty::BorderBottom(value.into()),
    ));
    self
  }

  pub fn border_left(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::BorderLeft,
      TaffyStyleProperty::BorderLeft(value.into()),
    ));
    self
  }

  // ==================== Alignment ====================

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

  // ==================== Gap ====================

  pub fn gap(mut self, value: impl Into<crate::style::dimension::Size<LengthPercentage>>) -> Self {
    let size: crate::style::dimension::Size<LengthPercentage> = value.into();
    let taffy_size: taffy::Size<taffy::LengthPercentage> = size.into();
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Gap,
      TaffyStyleProperty::Gap(taffy_size),
    ));
    self
  }

  pub fn gap_row(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::GapRow,
      TaffyStyleProperty::GapRow(value.into().into()),
    ));
    self
  }

  pub fn gap_column(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::GapColumn,
      TaffyStyleProperty::GapColumn(value.into().into()),
    ));
    self
  }

  // ==================== Flex ====================

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

  pub fn flex_basis(mut self, value: impl Into<Dimension>) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::FlexBasis,
      TaffyStyleProperty::FlexBasis(value.into().into()),
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

  // ==================== Rendering style properties (for Vello) ====================

  pub fn background(mut self, color: Color) -> Self {
    self.style_props.push((
      StylePropertyKey::Background,
      StyleProperty::Background(color),
    ));
    self
  }

  pub fn color(mut self, color: Color) -> Self {
    self
      .style_props
      .push((StylePropertyKey::Color, StyleProperty::Color(color)));
    self
  }

  pub fn font_size(mut self, value: f32) -> Self {
    self.style_props.push((
      StylePropertyKey::FontSize,
      StyleProperty::FontSize(Some(value)),
    ));
    self
  }

  /// Font weight
  /// 
  /// Common values:
  /// - 100: Thin, 200: Extra Light, 300: Light
  /// - 400: Normal (default), 500: Medium
  /// - 600: Semi Bold, 700: Bold
  /// - 800: Extra Bold, 900: Black
  pub fn font_weight(mut self, value: u16) -> Self {
    self.style_props.push((
      StylePropertyKey::FontWeight,
      StyleProperty::FontWeight(Some(value)),
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
    self
      .style_props
      .push((StylePropertyKey::TextWrap, StyleProperty::TextWrap(value)));
    self
  }

  pub fn text_align(mut self, value: TextAlign) -> Self {
    self
      .style_props
      .push((StylePropertyKey::TextAlign, StyleProperty::TextAlign(value)));
    self
  }

  pub fn pointer_events(mut self, value: PointerEvents) -> Self {
    self.style_props.push((
      StylePropertyKey::PointerEvents,
      StyleProperty::PointerEvents(value),
    ));
    self
  }

  pub fn translate(mut self, value: Point<taffy::LengthPercentage>) -> Self {
    self
      .style_props
      .push((StylePropertyKey::Translate, StyleProperty::Translate(value)));
    self
  }

  pub fn translate_x(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.style_props.push((
      StylePropertyKey::TranslateX,
      StyleProperty::TranslateX(value.into().into()),
    ));
    self
  }

  pub fn translate_y(mut self, value: impl Into<LengthPercentage>) -> Self {
    self.style_props.push((
      StylePropertyKey::TranslateY,
      StyleProperty::TranslateY(value.into().into()),
    ));
    self
  }

  pub fn opacity(mut self, value: f32) -> Self {
    self
      .style_props
      .push((StylePropertyKey::Opacity, StyleProperty::Opacity(value)));
    self
  }

  pub fn clip(mut self, value: bool) -> Self {
    self
      .style_props
      .push((StylePropertyKey::Clip, StyleProperty::Clip(value)));
    self
  }

  /// Set border (sets both Taffy layout border and rendering color)
  ///
  /// This method sets:
  /// 1. Taffy layout border - affects layout calculation, occupies box model space
  /// 2. Rendering border color - draws visible border
  pub fn border_all(mut self, width: f32, color: Color) -> Self {
    self.taffy_style_props.push((
      TaffyStylePropertyKey::Border,
      TaffyStyleProperty::Border(Rect::from(LengthPercentage::from(width))),
    ));

    self.style_props.push((
      StylePropertyKey::BorderColor,
      StyleProperty::BorderColor(color),
    ));

    self
  }

  /// Set border color (only affects rendering, not layout)
  pub fn border_color(mut self, color: Color) -> Self {
    self.style_props.push((
      StylePropertyKey::BorderColor,
      StyleProperty::BorderColor(color),
    ));
    self
  }

  pub fn border_radius(mut self, radius: f32) -> Self {
    self.style_props.push((
      StylePropertyKey::BorderRadius,
      StyleProperty::BorderRadius(radius),
    ));
    self
  }

  pub fn cursor(mut self, cursor: Cursor) -> Self {
    self
      .style_props
      .push((StylePropertyKey::Cursor, StyleProperty::Cursor(cursor)));
    self
  }

  // ==================== Utility Methods ====================

  #[inline]
  pub fn w_full(self) -> Self {
    self.width(taffy::Dimension::Percent(1.0))
  }

  #[inline]
  pub fn h_full(self) -> Self {
    self.height(taffy::Dimension::Percent(1.0))
  }

  #[inline]
  pub fn absolute(self) -> Self {
    self.position(Position::Absolute)
  }

  #[inline]
  pub fn block(self) -> Self {
    self.display(Display::Block)
  }

  #[inline]
  pub fn flex(self) -> Self {
    self.display(Display::Flex)
  }

  #[inline]
  pub fn hidden(self) -> Self {
    self.display(Display::None)
  }

  #[inline]
  pub fn overflow_hidden(self) -> Self {
    self.overflow(Point {
      x: Overflow::Hidden,
      y: Overflow::Hidden,
    })
  }

  #[inline]
  pub fn overflow_clip(self) -> Self {
    self.overflow(Point {
      x: Overflow::Clip,
      y: Overflow::Clip,
    })
  }

  #[inline]
  pub fn overflow_scroll(self) -> Self {
    self.overflow(Point {
      x: Overflow::Scroll,
      y: Overflow::Scroll,
    })
    // Don't set scrollbar_width - use overlay mode (modern style)
  }

  #[inline]
  pub fn overflow_x_hidden(self) -> Self {
    self.overflow_x(Overflow::Hidden)
  }

  #[inline]
  pub fn overflow_x_clip(self) -> Self {
    self.overflow_x(Overflow::Clip)
  }

  #[inline]
  pub fn overflow_x_scroll(self) -> Self {
    self.overflow_x(Overflow::Scroll)
    // Don't set scrollbar_width - use overlay mode (modern style)
  }

  #[inline]
  pub fn overflow_y_hidden(self) -> Self {
    self.overflow_y(Overflow::Hidden)
  }

  #[inline]
  pub fn overflow_y_clip(self) -> Self {
    self.overflow_y(Overflow::Clip)
  }

  #[inline]
  pub fn overflow_y_scroll(self) -> Self {
    self.overflow_y(Overflow::Scroll)
    // Don't set scrollbar_width - use overlay mode (modern style)
  }

  /// Enable vertical scrolling with reserved space for scrollbar (Windows style)
  /// 
  /// This reserves layout space for the scrollbar, reducing content width.
  /// Content will not be obscured by scrollbar.
  /// 
  /// For overlay scrollbar (macOS/iOS style), use `overflow_y_scroll()` instead.
  #[inline]
  pub fn overflow_y_scroll_reserved(self) -> Self {
    self.overflow_y(Overflow::Scroll)
        .scrollbar_width(DEFAULT_SCROLLBAR_WIDTH)
  }

  /// Enable horizontal scrolling with reserved space for scrollbar (Windows style)
  #[inline]
  pub fn overflow_x_scroll_reserved(self) -> Self {
    self.overflow_x(Overflow::Scroll)
        .scrollbar_width(DEFAULT_SCROLLBAR_WIDTH)
  }

  /// Enable both scrolling with reserved space for scrollbar (Windows style)
  #[inline]
  pub fn overflow_scroll_reserved(self) -> Self {
    self.overflow(Point {
      x: Overflow::Scroll,
      y: Overflow::Scroll,
    })
    .scrollbar_width(DEFAULT_SCROLLBAR_WIDTH)
  }

  #[inline]
  pub fn overflow_y_visible(self) -> Self {
    self.overflow_y(Overflow::Visible)
  }

  #[inline]
  pub fn size_full(self) -> Self {
    self.w_full().h_full()
  }

  #[inline]
  pub fn flex_row(self) -> Self {
    self.flex_direction(FlexDirection::Row)
  }

  #[inline]
  pub fn flex_col(self) -> Self {
    self.flex_direction(FlexDirection::Column)
  }

  #[inline]
  pub fn flex_row_reverse(self) -> Self {
    self.flex_direction(FlexDirection::RowReverse)
  }

  #[inline]
  pub fn flex_col_reverse(self) -> Self {
    self.flex_direction(FlexDirection::ColumnReverse)
  }

  #[inline]
  pub fn justify_start(self) -> Self {
    self.justify_content(JustifyContent::Start)
  }

  #[inline]
  pub fn justify_end(self) -> Self {
    self.justify_content(JustifyContent::End)
  }

  #[inline]
  pub fn justify_center(self) -> Self {
    self.justify_content(JustifyContent::Center)
  }

  #[inline]
  pub fn justify_stretch(self) -> Self {
    self.justify_content(JustifyContent::Stretch)
  }

  #[inline]
  pub fn justify_between(self) -> Self {
    self.justify_content(JustifyContent::SpaceBetween)
  }

  #[inline]
  pub fn items_start(self) -> Self {
    self.align_items(AlignItems::Start)
  }

  #[inline]
  pub fn items_end(self) -> Self {
    self.align_items(AlignItems::End)
  }

  #[inline]
  pub fn items_center(self) -> Self {
    self.align_items(AlignItems::Center)
  }

  #[inline]
  pub fn items_stretch(self) -> Self {
    self.align_items(AlignItems::Stretch)
  }

  // ==================== Color shortcuts ====================

  #[inline]
  pub fn bg_white(self) -> Self {
    self.background(Color::WHITE)
  }

  #[inline]
  pub fn bg_black(self) -> Self {
    self.background(Color::BLACK)
  }

  #[inline]
  pub fn bg_transparent(self) -> Self {
    self.background(Color::TRANSPARENT)
  }

  #[inline]
  pub fn text_white(self) -> Self {
    self.color(Color::WHITE)
  }

  #[inline]
  pub fn text_black(self) -> Self {
    self.color(Color::BLACK)
  }

  // ==================== Text shortcuts ====================

  #[inline]
  pub fn text_center(self) -> Self {
    self.text_align(TextAlign::Center)
  }

  #[inline]
  pub fn text_left(self) -> Self {
    self.text_align(TextAlign::Left)
  }

  #[inline]
  pub fn text_right(self) -> Self {
    self.text_align(TextAlign::Right)
  }

  #[inline]
  pub fn text_nowrap(self) -> Self {
    self.text_wrap(TextWrap::None)
  }

  #[inline]
  pub fn text_wrap_glyph(self) -> Self {
    self.text_wrap(TextWrap::Glyph)
  }

  #[inline]
  pub fn text_wrap_word(self) -> Self {
    self.text_wrap(TextWrap::Word)
  }

  #[inline]
  pub fn text_wrap_both(self) -> Self {
    self.text_wrap(TextWrap::WordOrGlyph)
  }

  // ==================== Pointer events shortcuts ====================

  #[inline]
  pub fn pointer_events_none(self) -> Self {
    self.pointer_events(PointerEvents::None)
  }

  #[inline]
  pub fn pointer_events_auto(self) -> Self {
    self.pointer_events(PointerEvents::Auto)
  }
}

/// Styleable trait - provides style setting capability for all Elements
///
/// Through blanket implementation, all components implementing Element trait
/// automatically gain style setting capability.
///
/// Uses blanket impl pattern: `impl<T: Element> Styleable for T`
pub trait Styleable: crate::Element {
  /// Set styles with reactive updates support
  ///
  /// Implementation strategy: Dirty Bubbling
  /// 
  /// Uses two-phase processing:
  /// 1. **Mark phase** (in effect): Only mark dirty state, bubble up
  /// 2. **Compute phase** (in render loop): Traverse from root, smart pruning
  /// 
  /// Advantages:
  /// - Batch process multiple updates
  /// - Avoid redundant calculations
  /// - Auto prune clean subtrees
  fn style<F: Fn(StyleBuilder) -> StyleBuilder + 'static>(self, f: F) -> Self {
    let view_id = self.id();
    let node_id = view_id.node_id();

    let index = crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&view_id) {
        let index = state.styles.len();
        state.styles.push(None);
        index
      } else {
        0
      }
    });

    let view_for_effect = view_id;

    create_effect(move || {
      let builder = f(StyleBuilder::default());

      let changed = crate::runtime::with_layout_mut(|runtime| {
        let mut style = runtime
          .taffy
          .style(node_id)
          .cloned()
          .unwrap_or(taffy::Style::DEFAULT);

        for (_key, prop) in &builder.taffy_style_props {
          (*prop).assign_to(&mut style);
        }

        let _ = runtime.taffy.set_style(node_id, style);

        // Check if StyleBuilder changed
        if let Some(state) = runtime.view_states.get_mut(&view_id) {
          if index < state.styles.len() {
            let prev = &state.styles[index];
            let next = Some(builder.clone());
            let changed = prev != &next;

            state.styles[index] = next;
            return changed;
          }
        }
        false
      });

      // Dirty Bubbling: only mark, don't compute
      // Actual style computation happens in render loop batch processing
      if changed {
        view_for_effect.request_style();
      }
    });

    self
  }
}

/// Blanket implementation - all Elements automatically gain Styleable
impl<T: crate::Element> Styleable for T {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_style_builder() {
    let builder = StyleBuilder::new()
      .width(800.0)
      .height(600.0)
      .flex()
      .flex_col()
      .items_center();

    assert_eq!(builder.taffy_style_props.len(), 5);
  }

  #[test]
  fn test_style_builder_utilities() {
    let builder = StyleBuilder::new().w_full().h_full().flex_row();

    assert_eq!(builder.taffy_style_props.len(), 3);
  }
}
