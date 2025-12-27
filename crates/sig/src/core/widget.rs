use std::any::Any;
use taffy::{AvailableSpace, Size};
use vello::Scene;
use crate::RenderContext;

/// Widget trait - provides measurement and potentially other component-specific logic
pub trait Widget: Any {
  /// Measure the widget given known dimensions and available space
  /// The style parameter provides access to the computed style (font_size, etc.)
  /// The ctx parameter provides rendering context (DPR, window size, etc.)
  /// The font_ctx and layout_ctx parameters provide text layout capabilities
  fn measure(
    &self,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    style: &crate::style::Style,
    ctx: &RenderContext,
    font_ctx: &mut parley::FontContext,
    layout_ctx: &mut parley::LayoutContext<()>,
  ) -> Size<f32> {
    let _ = (known_dimensions, available_space, style, ctx, font_ctx, layout_ctx);
    Size::ZERO
  }

  /// Paint the widget
  /// The style parameter provides access to the computed style (color, font_size, etc.)
  /// The ctx parameter provides rendering context (DPR, window size, etc.)
  fn paint(&self, scene: &mut Scene, width: f32, height: f32, abs_x: f64, abs_y: f64, style: &crate::style::Style, ctx: &RenderContext) {
    let _ = (scene, width, height, abs_x, abs_y, style, ctx);
  }
}
