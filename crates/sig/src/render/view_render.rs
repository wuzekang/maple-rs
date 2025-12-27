use crate::ViewId;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect, Stroke};
use vello::peniko::Color;

/// Scrollbar style configuration
const SCROLLBAR_WIDTH: f64 = 8.0; // Scrollbar width (logical pixels)
const SCROLLBAR_MIN_SIZE: f64 = 20.0; // Scrollbar minimum length
const SCROLLBAR_PADDING: f64 = 2.0; // Spacing between scrollbar and edge
const SCROLLBAR_RADIUS: f64 = 4.0; // Scrollbar corner radius

/// Get scrollbar color
fn scrollbar_color() -> Color {
  Color::from_rgba8(0, 0, 0, 100) // Semi-transparent black
}

/// Render scrollbars
fn render_scrollbars(
  scene: &mut Scene,
  view: ViewId,
  container_x: f64,
  container_y: f64,
  container_width: f64,
  container_height: f64,
  scroll_offset: (f32, f32),
  ctx: &crate::RenderContext,
) {
  // Use taffy's content_size to get the actual content dimensions
  let (content_width, content_height) = crate::runtime::with_layout(|runtime| {
    let node_id = view.node_id();

    // Directly use layout.content_size, which is the content size calculated by taffy
    if let Ok(layout) = runtime.taffy.layout(node_id) {
      (
        layout.content_size.width as f64,
        layout.content_size.height as f64,
      )
    } else {
      (container_width, container_height)
    }
  });

  // Save content and container dimensions to ViewState (for drag calculation)
  crate::runtime::with_layout_mut(|runtime| {
    if let Some(view_state) = runtime.view_states.get_mut(&view) {
      view_state.content_size = Some((content_width as f32, content_height as f32));
      view_state.container_size = Some((container_width as f32, container_height as f32));
    }
  });

  // Check if vertical scrollbar is needed
  let needs_vertical_scrollbar = content_height > container_height;

  // Check if horizontal scrollbar is needed
  let needs_horizontal_scrollbar = content_width > container_width;

  // Get overflow settings
  let (overflow_x, overflow_y) = crate::runtime::with_layout(|runtime| {
    let node_id = view.node_id();
    if let Ok(taffy_style) = runtime.taffy.style(node_id) {
      (taffy_style.overflow.x, taffy_style.overflow.y)
    } else {
      (taffy::Overflow::Visible, taffy::Overflow::Visible)
    }
  });

  // Draw vertical scrollbar
  if needs_vertical_scrollbar && overflow_y != taffy::Overflow::Visible {
    render_vertical_scrollbar(
      scene,
      view,
      container_x,
      container_y,
      container_width,
      container_height,
      content_height,
      scroll_offset.1 as f64,
      ctx,
    );
  } else {
    // Clear scrollbar area information
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(view_state) = runtime.view_states.get_mut(&view) {
        view_state.vertical_scrollbar_rect = None;
      }
    });
  }

  // Draw horizontal scrollbar
  if needs_horizontal_scrollbar && overflow_x != taffy::Overflow::Visible {
    render_horizontal_scrollbar(
      scene,
      view,
      container_x,
      container_y,
      container_width,
      container_height,
      content_width,
      scroll_offset.0 as f64,
      ctx,
    );
  } else {
    // Clear scrollbar area information
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(view_state) = runtime.view_states.get_mut(&view) {
        view_state.horizontal_scrollbar_rect = None;
      }
    });
  }
}

/// Render vertical scrollbar
fn render_vertical_scrollbar(
  scene: &mut Scene,
  view: ViewId,
  container_x: f64,
  container_y: f64,
  container_width: f64,
  container_height: f64,
  content_height: f64,
  scroll_offset_y: f64,
  ctx: &crate::RenderContext,
) {
  // Get border dimensions
  let (border_right, border_top, border_bottom) = crate::runtime::with_layout(|runtime| {
    let node_id = view.node_id();
    if let Ok(layout) = runtime.taffy.layout(node_id) {
      (
        layout.border.right as f64,
        layout.border.top as f64,
        layout.border.bottom as f64,
      )
    } else {
      (0.0, 0.0, 0.0)
    }
  });

  // Calculate scrollbar dimensions and position
  let scrollable_height = content_height - container_height;
  let scroll_ratio = if scrollable_height > 0.0 {
    scroll_offset_y / scrollable_height
  } else {
    0.0
  };

  // Scrollbar height = container height * (container height / content height)
  let scrollbar_height =
    (container_height * container_height / content_height).max(SCROLLBAR_MIN_SIZE);

  // Movable range (account for border and padding)
  let track_height = container_height - border_top - border_bottom - 2.0 * SCROLLBAR_PADDING;
  let movable_range = track_height - scrollbar_height;

  // Scrollbar Y position (account for border)
  let scrollbar_y = container_y + border_top + SCROLLBAR_PADDING + scroll_ratio * movable_range;
  // Scrollbar X position (account for border) - FIX: subtract border_right
  let scrollbar_x =
    container_x + container_width - border_right - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

  // Save scrollbar area to ViewState (for drag detection)
  crate::runtime::with_layout_mut(|runtime| {
    if let Some(view_state) = runtime.view_states.get_mut(&view) {
      view_state.vertical_scrollbar_rect = Some((
        scrollbar_x as f32,
        scrollbar_y as f32,
        SCROLLBAR_WIDTH as f32,
        scrollbar_height as f32,
      ));
    }
  });

  // Convert to physical pixels
  let physical_x = ctx.logical_to_physical(scrollbar_x as f32) as f64;
  let physical_y = ctx.logical_to_physical(scrollbar_y as f32) as f64;
  let physical_width = ctx.logical_to_physical(SCROLLBAR_WIDTH as f32) as f64;
  let physical_height = ctx.logical_to_physical(scrollbar_height as f32) as f64;
  let physical_radius = ctx.logical_to_physical(SCROLLBAR_RADIUS as f32) as f64;

  // Draw scrollbar
  let scrollbar_rect = RoundedRect::new(
    physical_x,
    physical_y,
    physical_x + physical_width,
    physical_y + physical_height,
    physical_radius,
  );

  scene.fill(
    vello::peniko::Fill::NonZero,
    Affine::IDENTITY,
    scrollbar_color(),
    None,
    &scrollbar_rect,
  );
}

fn render_horizontal_scrollbar(
  scene: &mut Scene,
  view: ViewId,
  container_x: f64,
  container_y: f64,
  container_width: f64,
  container_height: f64,
  content_width: f64,
  scroll_offset_x: f64,
  ctx: &crate::RenderContext,
) {
  // Get border dimensions
  let (border_left, border_right, border_bottom) = crate::runtime::with_layout(|runtime| {
    let node_id = view.node_id();
    if let Ok(layout) = runtime.taffy.layout(node_id) {
      (
        layout.border.left as f64,
        layout.border.right as f64,
        layout.border.bottom as f64,
      )
    } else {
      (0.0, 0.0, 0.0)
    }
  });

  // Calculate scrollbar dimensions and position
  let scrollable_width = content_width - container_width;
  let scroll_ratio = if scrollable_width > 0.0 {
    scroll_offset_x / scrollable_width
  } else {
    0.0
  };

  // Scrollbar width = container width * (container width / content width)
  let scrollbar_width = (container_width * container_width / content_width).max(SCROLLBAR_MIN_SIZE);

  // Movable range (account for borders)
  let track_width = container_width - border_left - border_right - 2.0 * SCROLLBAR_PADDING;
  let movable_range = track_width - scrollbar_width;

  // Scrollbar X position (account for border_left)
  let scrollbar_x = container_x + border_left + SCROLLBAR_PADDING + scroll_ratio * movable_range;
  // Scrollbar Y position (account for border_bottom) - FIX: subtract border_bottom
  let scrollbar_y =
    container_y + container_height - border_bottom - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

  // Save scrollbar area to ViewState (for drag detection)
  crate::runtime::with_layout_mut(|runtime| {
    if let Some(view_state) = runtime.view_states.get_mut(&view) {
      view_state.horizontal_scrollbar_rect = Some((
        scrollbar_x as f32,
        scrollbar_y as f32,
        scrollbar_width as f32,
        SCROLLBAR_WIDTH as f32,
      ));
    }
  });

  // Convert to physical pixels
  let physical_x = ctx.logical_to_physical(scrollbar_x as f32) as f64;
  let physical_y = ctx.logical_to_physical(scrollbar_y as f32) as f64;
  let physical_width = ctx.logical_to_physical(scrollbar_width as f32) as f64;
  let physical_height = ctx.logical_to_physical(SCROLLBAR_WIDTH as f32) as f64;
  let physical_radius = ctx.logical_to_physical(SCROLLBAR_RADIUS as f32) as f64;

  // Draw scrollbar
  let scrollbar_rect = RoundedRect::new(
    physical_x,
    physical_y,
    physical_x + physical_width,
    physical_y + physical_height,
    physical_radius,
  );

  scene.fill(
    vello::peniko::Fill::NonZero,
    Affine::IDENTITY,
    scrollbar_color(),
    None,
    &scrollbar_rect,
  );
}

/// Check if rectangle intersects with viewport (simple version, doesn't consider transform)
fn is_in_viewport(x: f64, y: f64, width: f64, height: f64, ctx: &crate::RenderContext) -> bool {
  let viewport_width = ctx.window_size.0 as f64;
  let viewport_height = ctx.window_size.1 as f64;

  // Check if completely outside viewport
  if x + width < 0.0 || x > viewport_width || y + height < 0.0 || y > viewport_height {
    return false;
  }

  true
}

/// Render sig's View to vello Scene
///
/// # Viewport Culling Strategy
///
/// 1. **Nodes with overflow constraints**: Can safely prune
///    - overflow: hidden/scroll/auto clips child content
///    - If parent node is off-screen, child nodes are not visible
///
/// 2. **overflow: visible nodes**: Conservative handling
///    - Even if parent node is off-screen, child nodes may return to screen via transform
///    - Don't prune, continue traversing child nodes
///
/// 3. **Performance tradeoff**:
///    - Most scroll containers have overflow: scroll/hidden
///    - These scenarios can safely prune for maximum performance gain
///    - overflow: visible scenarios continue traversal, overhead is acceptable
pub fn render_view(
  scene: &mut Scene,
  view: ViewId,
  x: f64,
  y: f64,
  ctx: &crate::RenderContext,
  hover_top_view: Option<ViewId>,
) {
  // 🎯 Get all needed data at once to avoid nested borrows
  let render_data = crate::runtime::with_layout(|runtime| {
    let node_id = view.node_id;

    // Get layout
    let layout = runtime.taffy.layout(node_id).ok().cloned();

    // Get widget and style
    let widget = runtime.widgets.get(view.node_id.into()).cloned();
    let style = runtime.styles.get(&node_id).cloned().unwrap_or_default();

    // Get overflow information from taffy
    let has_overflow = if let Ok(taffy_style) = runtime.taffy.style(node_id) {
      taffy_style.overflow.x != taffy::Overflow::Visible
        || taffy_style.overflow.y != taffy::Overflow::Visible
    } else {
      false
    };

    // Get scroll offset from view_states
    let scroll_offset = runtime
      .view_states
      .get(&view)
      .map(|state| state.scroll_offset)
      .unwrap_or((0.0, 0.0));

    (layout, widget, style, has_overflow, scroll_offset)
  });

  let (layout_opt, widget, style, has_overflow, mut scroll_offset) = render_data;

  // 🔑 Validate and correct scroll offset (immediately before rendering)
  if let Some(ref layout) = layout_opt {
    let content_width = layout.content_size.width;
    let content_height = layout.content_size.height;
    let container_width = layout.size.width;
    let container_height = layout.size.height;

    // Calculate maximum allowed scroll offset
    let max_offset_x = (content_width - container_width).max(0.0);
    let max_offset_y = (content_height - container_height).max(0.0);

    // Clamp scroll offset to valid range
    let new_offset_x = scroll_offset.0.min(max_offset_x).max(0.0);
    let new_offset_y = scroll_offset.1.min(max_offset_y).max(0.0);

    // If changed, update ViewState
    if (new_offset_x - scroll_offset.0).abs() > 0.001
      || (new_offset_y - scroll_offset.1).abs() > 0.001
    {
      scroll_offset = (new_offset_x, new_offset_y);

      crate::runtime::with_layout_mut(|runtime| {
        if let Some(view_state) = runtime.view_states.get_mut(&view) {
          view_state.scroll_offset = scroll_offset;
          view_state.dirty = true;
        }
      });

      // Request redraw to apply new offset
      crate::runtime::with_window(|window_state| {
        if let Some(window) = &window_state.redraw_requester {
          window.request_redraw();
        }
      });
    }
  }

  // Get view's layout information
  if let Some(layout) = layout_opt {
    let layout_x = layout.location.x as f64;
    let layout_y = layout.location.y as f64;
    let width = layout.size.width as f64;
    let height = layout.size.height as f64;

    // 🎨 Calculate transform from style.translate
    let (tx, ty) = {
      let translate_x = style.translate.x;
      let translate_y = style.translate.y;

      // Resolve LengthPercentage to actual pixels
      // Percent values are in 0-1 range (0.5 = 50%, 1.0 = 100%)
      let tx = match translate_x {
        taffy::LengthPercentage::Length(v) => v as f64,
        taffy::LengthPercentage::Percent(v) => width * v as f64,
      };

      let ty = match translate_y {
        taffy::LengthPercentage::Length(v) => v as f64,
        taffy::LengthPercentage::Percent(v) => height * v as f64,
      };

      (tx, ty)
    };

    // Calculate absolute position (parent position + layout position)
    let abs_x = x + layout_x + tx;
    let abs_y = y + layout_y + ty;

    // 🎯 Convert logical pixel coordinates to physical pixels (calculate early for later use)
    let physical_x = ctx.logical_to_physical(abs_x as f32) as f64;
    let physical_y = ctx.logical_to_physical(abs_y as f32) as f64;
    let physical_width = ctx.logical_to_physical(width as f32) as f64;
    let physical_height = ctx.logical_to_physical(height as f32) as f64;
    let physical_radius = ctx.logical_to_physical(style.border_radius) as f64;

    // 🔑 Cache absolute position to ViewState for popper positioning
    crate::runtime::with_layout_mut(|runtime| {
      if let Some(view_state) = runtime.view_states.get_mut(&view) {
        view_state.absolute_position = Some((abs_x as f32, abs_y as f32));
      }
    });

    // 🎯 Viewport culling: Determine pruning strategy based on overflow property
    let in_viewport = is_in_viewport(abs_x, abs_y, width, height, ctx);

    if !in_viewport && has_overflow {
      // ✅ Safe pruning: Parent off-screen + has overflow constraint
      // In this case child nodes are clipped and cannot be visible
      // Example: scroll container scrolled off-screen, entire subtree not visible
      return;
    }

    // Only render views with dimensions that are in viewport
    if width > 0.0 && height > 0.0 && in_viewport {
      let rect = RoundedRect::new(
        physical_x,
        physical_y,
        physical_x + physical_width,
        physical_y + physical_height,
        physical_radius,
      );

      // 🎨 Draw background (transform applied via layer if needed)
      if style.background != Color::TRANSPARENT {
        scene.fill(
          vello::peniko::Fill::NonZero,
          Affine::IDENTITY,
          style.background,
          None,
          &rect,
        );
      }

      // 🎨 Draw border (transform applied via layer if needed)
      let border_width =
        (layout.border.left + layout.border.right + layout.border.top + layout.border.bottom) / 4.0;
      let physical_border_width = ctx.logical_to_physical(border_width) as f64;

      if physical_border_width > 0.0 && style.border_color != Color::TRANSPARENT {
        let stroke = Stroke::new(physical_border_width);
        scene.stroke(&stroke, Affine::IDENTITY, style.border_color, None, &rect);
      }

      // 🎨 Draw Widget content (if present)
      if let Some(widget) = widget {
        // 🎯 Pass render context to widget.paint (call outside with_layout to avoid nested borrow)
        widget.paint(
          scene,
          width as f32,
          height as f32,
          abs_x,
          abs_y,
          &style,
          ctx,
        );
      }

      // If overflow, add clipping layer
      if has_overflow {
        // 🎯 Calculate border width (logical pixels)
        let border_left = layout.border.left as f64;
        let border_right = layout.border.right as f64;
        let border_top = layout.border.top as f64;
        let border_bottom = layout.border.bottom as f64;

        // 🎯 Clip region should be inside border
        let clip_x = physical_x + ctx.logical_to_physical(border_left as f32) as f64;
        let clip_y = physical_y + ctx.logical_to_physical(border_top as f32) as f64;
        let clip_width = physical_width
          - ctx.logical_to_physical(border_left as f32) as f64
          - ctx.logical_to_physical(border_right as f32) as f64;
        let clip_height = physical_height
          - ctx.logical_to_physical(border_top as f32) as f64
          - ctx.logical_to_physical(border_bottom as f32) as f64;

        // Use rounded rectangle clipping to support border_radius
        let physical_radius = ctx.logical_to_physical(style.border_radius) as f64;
        let clip_shape = RoundedRect::new(
          clip_x,
          clip_y,
          clip_x + clip_width,
          clip_y + clip_height,
          physical_radius,
        );

        scene.push_layer(
          vello::peniko::BlendMode::default(),
          1.0,
          Affine::IDENTITY,
          &clip_shape,
        );
      }
    }

    // ⚠️ Note: Even if current node is not in viewport, may still need to traverse child nodes
    // Because child nodes may return to viewport via transform or positioning
    // Only early return when has_overflow and not in viewport (see optimization above)

    // Recursively render child views (apply scroll offset only)
    let child_x = abs_x - scroll_offset.0 as f64;
    let child_y = abs_y - scroll_offset.1 as f64;

    // 🔑 Get child view ID list directly from ViewState and recursively render
    let child_ids = view.get_children();
    for child_id in child_ids.iter() {
      render_view(scene, *child_id, child_x, child_y, ctx, hover_top_view);
    }

    // If clip layer was added, need to pop it
    if has_overflow && in_viewport {
      scene.pop_layer();
    }

    // 🎨 Draw scrollbars (after clip layer, so scrollbars won't be clipped)
    if has_overflow && in_viewport && width > 0.0 && height > 0.0 {
      render_scrollbars(scene, view, abs_x, abs_y, width, height, scroll_offset, ctx);
    }

    // 🎯 Draw AABB for hover_top_view (debug feature)
    // Draw after all children are rendered, so it's on top
    if let Some(hover_view) = hover_top_view {
      if view == hover_view {
        use vello::kurbo::Rect;
        let rect = Rect::new(
          physical_x,
          physical_y,
          physical_x + physical_width,
          physical_y + physical_height,
        );

        // Draw red border for AABB
        let stroke = Stroke::new(2.0);
        scene.stroke(
          &stroke,
          Affine::IDENTITY,
          Color::from_rgba8(255, 0, 0, 180),
          None,
          &rect,
        );
      }
    }
  }
}
