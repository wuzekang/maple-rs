//! Text module - Text component
//!
//! Provides support for static text and reactive text
//!
//! # Design Philosophy
//!
//! Text component follows Component Design Plan A (Component as Wrapper Pattern):
//! - Holds internal View
//! - Implements Element trait
//! - Automatically gains Styleable and Interactive capabilities
//! - Provides two convenience functions: `text()` and `dynamic_text()`
//!
//! # Examples
//!
//! ```rust
//! use sig::{create_scope, text, dynamic_text, Signal};
//!
//! create_scope(|| {
//!     // Static text - use text()
//!     let t1 = text("Hello")
//!         .style(|s| s.font_size(20.0));
//!
//!     // Dynamic text - use dynamic_text()
//!     let count = Signal::new(0);
//!     let t2 = dynamic_text(move || format!("Count: {}", *count.read()))
//!         .style(|s| s.font_size(16.0));
//! });
//! ```

use crate::{Element, Signal, ViewId, Widget, create_effect};
use crate::style::TextWrap;
use std::rc::Rc;
use std::sync::OnceLock;
use std::cell::RefCell;
use taffy::{AvailableSpace, Size};
use vello::Scene;

/// Default font stack - Lazy initialization to avoid repeated creation during each render
static DEFAULT_FONT_STACK: OnceLock<parley::style::FontStack<'static>> = OnceLock::new();

/// Get default font stack
#[inline]
fn get_default_font_stack() -> &'static parley::style::FontStack<'static> {
  DEFAULT_FONT_STACK.get_or_init(|| {
    parley::style::FontStack::Source(
      "-apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', SimHei, sans-serif"
        .into(),
    )
  })
}

/// Text layout cache key - Used to determine if cache is valid
#[derive(Clone, Debug, PartialEq)]
struct LayoutCacheKey {
  text: String,
  physical_font_size: u32, // Use integers to avoid floating-point comparison issues
  font_weight: u16,
}

impl LayoutCacheKey {
  fn new(text: &str, physical_font_size: f32, font_weight: u16) -> Self {
    Self {
      text: text.to_string(),
      // Convert to fixed-precision integer (keep 2 decimal places)
      physical_font_size: (physical_font_size * 100.0) as u32,
      font_weight,
    }
  }
}

/// Text layout cache - Stores built Layout
struct LayoutCache {
  key: Option<LayoutCacheKey>,
  layout: Option<parley::Layout<()>>,
}

impl LayoutCache {
  fn new() -> Self {
    Self {
      key: None,
      layout: None,
    }
  }

  /// Check if cache is valid
  fn is_valid(&self, key: &LayoutCacheKey) -> bool {
    self.key.as_ref() == Some(key) && self.layout.is_some()
  }

  /// Update cache
  fn update(&mut self, key: LayoutCacheKey, layout: parley::Layout<()>) {
    self.key = Some(key);
    self.layout = Some(layout);
  }
}

/// Common function to build text layout
///
/// This function will:
/// 1. Check if cache is valid, if valid return cached Layout directly (without break lines)
/// 2. If cache is invalid, rebuild layout and update cache
/// 3. Apply unified text styles (font stack, font size, font weight)
/// 4. Caller needs to call break_all_lines() themselves to apply width constraints
///
/// # Parameters
/// - `cache`: Mutable reference to layout cache
/// - `text`: Text content to render
/// - `physical_font_size`: Physical pixel font size
/// - `font_weight`: Font weight
/// - `font_ctx`: Parley font context
/// - `layout_ctx`: Parley layout context
///
/// # Returns
/// Returns mutable reference to Layout (caller needs to call break_all_lines)
fn build_text_layout<'a>(
  cache: &'a mut LayoutCache,
  text: &str,
  physical_font_size: f32,
  font_weight: u16,
  font_ctx: &mut parley::FontContext,
  layout_ctx: &mut parley::LayoutContext<()>,
) -> &'a mut parley::Layout<()> {
  // 🎯 Generate cache key (does not include width_constraint)
  let cache_key = LayoutCacheKey::new(text, physical_font_size, font_weight);

  // 🎯 Check if cache is valid
  if !cache.is_valid(&cache_key) {
    // Cache invalid, rebuild layout
    let mut builder = layout_ctx.ranged_builder(font_ctx, text, 1.0, true);

    // Apply unified text styles
    use parley::style::{FontWeight, StyleProperty};
    builder.push_default(StyleProperty::FontStack(get_default_font_stack().clone()));
    builder.push_default(StyleProperty::FontSize(physical_font_size));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(font_weight as f32)));
    builder.push_default(StyleProperty::Brush(()));

    let layout = builder.build(text);
    // Note: Don't call break_all_lines here, caller will call as needed

    // 🎯 Update cache
    cache.update(cache_key, layout);
  }

  // 🎯 Return mutable reference, allowing caller to call break_all_lines
  cache.layout.as_mut().expect("Layout should be cached")
}

/// Text component
///
/// Text implements Element trait, automatically gains:
/// - Styleable: All style setting methods (40+)
/// - Interactive: All event handling methods (10+)
///
/// # Examples
///
/// ```rust
/// use sig::{text, dynamic_text, create_scope, Signal};
///
/// create_scope(|| {
///     // Static text
///     let t1 = text("Hello, World!")
///         .style(|s| s.font_size(24.0).margin_bottom(10.0));
///
///     // Dynamic text
///     let count = Signal::new(0);
///     let t2 = dynamic_text(move || format!("Count: {}", *count.read()))
///         .style(|s| s.font_size(16.0))
///         .on_click(|_| println!("Text clicked!"));
/// });
/// ```
#[derive(Clone)]
pub struct Text {
  id: ViewId,
  content: Signal<String>,
}

impl Text {
  /// Create static text
  ///
  /// # Parameters
  /// - `content`: Text content (any type that can be converted to String)
  ///
  /// # Examples
  /// ```
  /// use sig::{text::Text, create_scope};
  ///
  /// create_scope(|| {
  ///     // String literal
  ///     let t1 = Text::new(\"Hello, World!\");
  ///
  ///     // String type
  ///     let t2 = Text::new(String::from(\"Hello\"));
  ///
  ///     // format! result
  ///     let t3 = Text::new(format!(\"Count: {}\", 100));
  ///
  ///     // Number types
  ///     let t4 = Text::new(42);
  ///     let t5 = Text::new(3.14);
  /// });
  /// ```
  pub fn new<S: ToString>(content: S) -> Self {
    let id = ViewId::new();

    let content_signal = Signal::new(content.to_string());

    id.widget(Rc::new(TextWidget {
      content: content_signal.clone(),
      layout_cache: RefCell::new(LayoutCache::new()),
    }));

    Self {
      id,
      content: content_signal,
    }
  }

  /// Create dynamic text (reactive)
  ///
  /// # Parameters
  /// - `content_fn`: Function that returns text content
  ///
  /// # Examples
  /// ```
  /// use sig::{create_scope, text::Text, Signal};
  ///
  /// create_scope(|| {
  ///     let count = Signal::new(0);
  ///     let t = Text::dynamic(move || format!("Count: {}", *count.read()));
  /// });
  /// ```
  pub fn dynamic<F: Fn() -> String + 'static>(content_fn: F) -> Self {
    let id = ViewId::new();

    // Set name in ViewState
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(state) = runtime.view_states.get_mut(&id) {
        state.name = "Text".to_string();
      }
    });

    let content_signal = Signal::new(String::new());

    // 创建并注册 Widget
    let widget = Rc::new(TextWidget {
      content: content_signal.clone(),
      layout_cache: RefCell::new(LayoutCache::new()),
    });

    id.widget(widget);

    // 设置响应式更新
    let view_for_effect = id;
    create_effect(move || {
      let new_text = content_fn();
      *content_signal.write() = new_text;
      view_for_effect.mark_dirty();
    });

    Self {
      id,
      content: content_signal,
    }
  }

  /// Get current text content
  pub fn get(&self) -> String {
    self.content.read().clone()
  }
}

/// TextWidget - Implements text measurement logic
struct TextWidget {
  content: Signal<String>,
  // 🎯 Layout cache - shared by measure and paint
  layout_cache: RefCell<LayoutCache>,
}

impl Widget for TextWidget {
  fn measure(
    &self,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    style: &crate::style::Style,
    ctx: &crate::RenderContext,
    font_ctx: &mut parley::FontContext,
    layout_ctx: &mut parley::LayoutContext<()>,
  ) -> Size<f32> {
    let text: crate::ReadGuard<String> = self.content.read_untracked();

    // 🎯 Get logical pixel font size from style
    let logical_font_size = style.font_size.unwrap_or(16.0);
    // 🎯 Convert to physical pixels to ensure clarity
    let physical_font_size = ctx.logical_to_physical(logical_font_size);
    // 🎯 Get font weight
    let font_weight = style.font_weight.unwrap_or(400);

    // 🎯 Decide whether to wrap based on text_wrap style
    let width_constraint = if style.text_wrap == TextWrap::None {
      // text_nowrap: no wrapping
      None
    } else if let Some(w) = known_dimensions.width {
      // Has explicit width: convert to physical pixels
      Some(ctx.logical_to_physical(w))
    } else {
      // Decide based on available space
      match available_space.width {
        taffy::AvailableSpace::Definite(w) => Some(ctx.logical_to_physical(w)),
        taffy::AvailableSpace::MinContent => None,
        taffy::AvailableSpace::MaxContent => None,
      }
    };

    // 🎯 Use cached layout building function (does not include width_constraint)
    let mut cache = self.layout_cache.borrow_mut();
    let layout = build_text_layout(
      &mut cache,
      &text,
      physical_font_size,
      font_weight,
      font_ctx,
      layout_ctx,
    );

    // 🎯 Apply width constraint (needs to be recalculated each time as width may change)
    layout.break_all_lines(width_constraint);

    // 🎯 Return logical pixel dimensions
    let width = if let Some(w) = known_dimensions.width {
      w
    } else {
      ctx.physical_to_logical(layout.width() + 1.0)
    };

    let height = if let Some(h) = known_dimensions.height {
      h
    } else {
      ctx.physical_to_logical(layout.height() + 1.0)
    };

    taffy::Size { width, height }
  }

  fn paint(
    &self,
    scene: &mut Scene,
    width: f32,
    _height: f32,
    abs_x: f64,
    abs_y: f64,
    style: &crate::style::Style,
    ctx: &crate::RenderContext,
  ) {
    let text: crate::ReadGuard<String> = self.content.read_untracked();
    use vello::kurbo::Affine;

    // 🎯 Get logical pixel font size from style
    let logical_font_size = style.font_size.unwrap_or(16.0);
    // 🎯 Convert to physical pixels
    let physical_font_size = ctx.logical_to_physical(logical_font_size);
    // 🎯 Get font weight
    let font_weight = style.font_weight.unwrap_or(400);
    let color = style.color;

    // 🎯 Decide whether to wrap based on text_wrap style
    let width_constraint = if style.text_wrap == TextWrap::None {
      // text_nowrap: no wrapping
      None
    } else {
      // Has wrapping: use physical pixel width
      Some(ctx.logical_to_physical(width) + 1.0)
    };

    crate::runtime::with_layout_mut(|layout_state| {
      // 🎯 Use cached layout building function (does not include width_constraint)
      let mut cache = self.layout_cache.borrow_mut();
      let layout = build_text_layout(
        &mut cache,
        &text,
        physical_font_size,
        font_weight,
        &mut layout_state.font_context,
        &mut layout_state.layout_context,
      );

      // 🎯 Apply width constraint (needs to be recalculated each time)
      layout.break_all_lines(width_constraint);


      for line in layout.lines() {
        for item in line.items() {
          if let parley::layout::PositionedLayoutItem::GlyphRun(run) = item {
            let font = run.run().font();
            let font_size = run.run().font_size();

            // Create glyph iterator mapped to Vello's Glyph type
            let glyphs = run.positioned_glyphs().map(|g| vello::Glyph {
              id: g.id as u32,
              x: g.x,
              y: g.y,
            });

            // 🎯 坐标转换：将逻辑像素位置转换为物理像素
            // 注意：字形坐标已经是物理像素（使用 physical_font_size 生成）
            // 所以只需要将位置从逻辑像素转换为物理像素，不需要额外缩放
            let physical_x = ctx.logical_to_physical(abs_x as f32) as f64;
            let physical_y = ctx.logical_to_physical(abs_y as f32) as f64;
            let transform = Affine::translate(vello::kurbo::Vec2::new(physical_x, physical_y));

            // 使用样式中的颜色渲染文本
            scene
              .draw_glyphs(font)
              .font_size(font_size)
              .transform(transform)
              .brush(color)
              .draw(vello::peniko::Fill::NonZero, glyphs);
          }
        }
      }
    });
  }
}

/// Implement Element trait for Text
///
/// This gives Text automatic access to:
/// - Styleable: All style setting methods (40+)
/// - Interactive: All event handling methods (10+)
/// - ViewTuple: Can be added as child element to View (through blanket impl)
impl Element for Text {
  fn id(&self) -> ViewId {
    self.id
  }

  fn name(&self) -> String {
    "Text".to_string()
  }
}

/// Convenience function to create static text
///
/// # Parameters
/// - `content`: Text content (types supporting ToString trait)
///
/// # Examples
/// ```
/// use sig::{text, create_scope};
///
/// create_scope(|| {
///     // String
///     let t1 = text(\"Hello, World!\");
///
///     // Numbers
///     let t2 = text(42);
///     let t3 = text(3.14);
/// });
/// ```
pub fn text<S: ToString>(content: S) -> Text {
  Text::new(content)
}

/// Convenience function to create dynamic text
///
/// # Parameters
/// - `content_fn`: Closure that returns text content
///
/// # Examples
/// ```
/// use sig::{dynamic_text, create_scope, Signal};
///
/// create_scope(|| {
///     let count = Signal::new(0);
///     let t = dynamic_text(move || format!("Count: {}", *count.read()));
/// });
/// ```
pub fn dynamic_text<F: Fn() -> String + 'static>(content_fn: F) -> Text {
  Text::dynamic(content_fn)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Interactive, Styleable, create_scope};

  #[test]
  fn test_static_text_str() {
    create_scope(|| {
      let t = text("Hello");
      assert_eq!(t.get(), "Hello");
      assert_eq!(t.name(), "Text");
    });
  }

  #[test]
  fn test_static_text_string() {
    create_scope(|| {
      let s = String::from("World");
      let t = text(s);
      assert_eq!(t.get(), "World");
    });
  }

  #[test]
  fn test_static_text_format() {
    create_scope(|| {
      let t = text(format!("Count: {}", 100));
      assert_eq!(t.get(), "Count: 100");
    });
  }

  #[test]
  fn test_dynamic_text() {
    create_scope(|| {
      let signal = Signal::new("World".to_string());
      let t = dynamic_text(move || signal.read().clone());

      assert_eq!(t.get(), "World");

      *signal.write() = "Rust".to_string();
      // Note: effect runs asynchronously, so we can't assert the new value immediately
    });
  }

  #[test]
  fn test_dynamic_text_with_format() {
    create_scope(|| {
      let count = Signal::new(0);
      let t = dynamic_text(move || format!("Count: {}", *count.read()));

      assert_eq!(t.get(), "Count: 0");
    });
  }

  #[test]
  fn test_text_with_style() {
    create_scope(|| {
      // ✅ 静态文本可以直接调用 .style()
      let _t1 = text("Styled").style(|s| s.font_size(20.0));

      // ✅ 动态文本也可以直接调用 .style()
      let count = Signal::new(0);
      let _t2 =
        dynamic_text(move || format!("Count: {}", *count.read())).style(|s| s.font_size(16.0));
    });
  }

  #[test]
  fn test_text_with_events() {
    create_scope(|| {
      // ✅ 可以直接调用 .on_click()
      let _t = text("Clickable").on_click(|_| println!("Clicked!"));
    });
  }

  #[test]
  fn test_text_method_chaining() {
    create_scope(|| {
      // ✅ 测试方法链式调用
      let _t = text("Chained")
        .style(|s| s.font_size(18.0).margin_top(10.0))
        .on_click(|_| println!("Clicked!"))
        .on_mouse_enter(|_| println!("Mouse entered!"));
    });
  }

  #[test]
  fn test_layout_cache_key_equality() {
    // Test cache key equality (does not include width_constraint)
    let key1 = LayoutCacheKey::new("Hello", 16.0, 400);
    let key2 = LayoutCacheKey::new("Hello", 16.0, 400);
    let key3 = LayoutCacheKey::new("Hello", 16.01, 400); // Slightly different font size
    let key4 = LayoutCacheKey::new("World", 16.0, 400); // Different text

    assert_eq!(key1, key2, "Same parameters should generate same cache key");
    assert_ne!(key1, key3, "Different font size should generate different cache key");
    assert_ne!(key1, key4, "Different text should generate different cache key");
  }

  #[test]
  fn test_layout_cache_invalidation() {
    // Test cache invalidation logic
    let mut cache = LayoutCache::new();

    let key1 = LayoutCacheKey::new("Hello", 16.0, 400);
    assert!(!cache.is_valid(&key1), "Initial cache should be invalid");

    // Note: We cannot directly test the update method here as it requires an actual Layout object
    // But we can verify the cache key comparison logic
  }
}
