//! Resizable Component Demo Example
//!
//! Demonstrates various usages of resizable containers

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    view()
      .style(
        |s| {
          s.w_full()
            .h_full()
            .padding(40.0)
            .flex()
            .flex_col()
            .background(Color::from_rgb8(248, 250, 252))
        }, // slate-50
      )
      .child((
        // Title
        view()
          .style(|s| {
            s.font_size(28.0)
              .color(Color::from_rgb8(15, 23, 42)) // slate-900
              .margin_bottom(24.0)
          })
          .child(text("Resizable Component Demo")),
        // Main demo area
        view().style(|s| s.flex().flex_row().h_full()).child((
          create_left_demo(),
          view().style(|s| s.width(20.0)), // Spacing
          create_right_demo(),
        )),
      ))
  })
}

/// Left demo: Horizontally resizable panel
fn create_left_demo() -> View {
  let width = Signal::new(300.0);

  view().style(|s| s.flex().h_full()).child(
    view().child(
      Resizable::horizontal(width.clone())
        .min_size(200.0)
        .max_size(500.0)
        .child(
          view()
            .style(|s| {
              s.h_full()
                .background(Color::WHITE)
                .padding(20.0)
                .border_radius(12.0)
                .border_all(1.0, Color::from_rgb8(229, 231, 235))
                .flex()
                .flex_col()
            })
            .child((
              view()
                .style(|s| {
                  s.font_size(18.0)
                    .color(Color::from_rgb8(51, 65, 85))
                    .margin_bottom(16.0)
                })
                .child(text("Horizontal Resize")),
              view()
                .style(|s| s.color(Color::from_rgb8(100, 116, 139)).margin_bottom(8.0))
                .child(text("Drag the right edge to adjust width")),
              view()
                .style(|s| {
                  s.padding(12.0)
                    .background(Color::from_rgb8(241, 245, 249))
                    .border_radius(8.0)
                    .margin_bottom(16.0)
                })
                .child(
                  view()
                    .child(text("Drag to see width changes in real-time"))
                    .style(|s| s.color(Color::from_rgb8(59, 130, 246))),
                ),
              view().style(|s| s.flex().flex_col()).child((
                view()
                  .child(text("• Can be resized by dragging"))
                  .style(|s| s.color(Color::from_rgb8(100, 116, 139)).margin_bottom(4.0)),
                view()
                  .child(text("• Supports min/max limits"))
                  .style(|s| s.color(Color::from_rgb8(100, 116, 139)).margin_bottom(4.0)),
                view()
                  .child(text("• Cursor changes automatically"))
                  .style(|s| s.color(Color::from_rgb8(100, 116, 139))),
              )),
            )),
        ),
    ),
  )
}

/// Right demo: Vertical resize and combined demo
fn create_right_demo() -> View {
  view()
    .style(|s| s.flex().flex_col().flex_grow(1.0).h_full())
    .child((
      create_vertical_demo(),
      view().style(|s| s.height(20.0)),
      create_combined_demo(),
    ))
}

/// Vertical resize demo
fn create_vertical_demo() -> View {
  let height = Signal::new(200.0);

  view().child(
    Resizable::vertical(height.clone())
      .min_size(150.0)
      .max_size(350.0)
      .child(
        view()
          .style(|s| {
            s.w_full()
              .background(Color::WHITE)
              .padding(20.0)
              .border_radius(12.0)
              .border_all(1.0, Color::from_rgb8(229, 231, 235))
              .flex()
              .flex_col()
          })
          .child((
            view()
              .style(|s| {
                s.font_size(18.0)
                  .color(Color::from_rgb8(51, 65, 85))
                  .margin_bottom(16.0)
              })
              .child(text("Vertical Resize")),
            view()
              .style(|s| s.color(Color::from_rgb8(100, 116, 139)).margin_bottom(8.0))
              .child(text("Drag the bottom edge to adjust height")),
            view()
              .style(|s| {
                s.padding(12.0)
                  .background(Color::from_rgb8(241, 245, 249))
                  .border_radius(8.0)
              })
              .child(
                view()
                  .child(text("Drag to see height changes in real-time"))
                  .style(|s| s.color(Color::from_rgb8(16, 185, 129))),
              ),
          )),
      ),
  )
}

/// Combined demo: Simulating column layout
fn create_combined_demo() -> View {
  let left_width = Signal::new(200.0);
  let right_width = Signal::new(200.0);

  view()
    .style(|s| {
      s.flex()
        .flex_grow(1.0)
        .background(Color::WHITE)
        .border_radius(12.0)
        .border_all(1.0, Color::from_rgb8(229, 231, 235))
        .overflow_hidden()
    })
    .child((
      // Title bar
      view()
        .style(|s| {
          s.w_full()
            .padding(16.0)
            .background(Color::from_rgb8(249, 250, 251))
            .border_bottom(1.0)
        })
        .child(
          view()
            .style(|s| s.font_size(16.0).color(Color::from_rgb8(51, 65, 85)))
            .child(text("Three-Column Layout Demo")),
        ),
      // Three-column content
      view()
        .style(|s| s.flex().flex_row().flex_grow(1.0))
        .child((
          // Left column (resizable)
          view().child(
            Resizable::horizontal(left_width.clone())
              .min_size(150.0)
              .max_size(300.0)
              .child(create_column(
                "Left Column",
                Color::from_rgb8(239, 246, 255),
                left_width.clone(),
              )),
          ),
          // Middle column (fixed)
          view()
            .style(|s| {
              s.flex_grow(1.0)
                .padding(16.0)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
            })
            .child((
              view()
                .style(|s| {
                  s.font_size(16.0)
                    .color(Color::from_rgb8(100, 116, 139))
                    .margin_bottom(8.0)
                })
                .child(text("Middle Column (Fixed)")),
              view()
                .style(|s| s.color(Color::from_rgb8(156, 163, 175)))
                .child(text("Automatically fills remaining space")),
            )),
          // Right column (resizable)
          view().child(
            Resizable::horizontal(right_width.clone())
              .min_size(150.0)
              .max_size(300.0)
              .handle_position(HandlePosition::Left)
              .child(create_column(
                "Right Column",
                Color::from_rgb8(254, 243, 199),
                right_width.clone(),
              )),
          ),
        )),
    ))
}

/// Create a column
fn create_column(title: &str, bg_color: Color, _width: Signal<f32>) -> View {
  let title = title.to_string();
  view()
    .style(move |s| {
      s.h_full()
        .padding(16.0)
        .background(bg_color)
        .flex()
        .flex_col()
    })
    .child(
      view()
        .style(|s| s.font_size(14.0).color(Color::from_rgb8(71, 85, 105)))
        .child(text(title)),
    )
}
