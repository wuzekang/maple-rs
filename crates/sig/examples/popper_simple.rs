//! Simple Popper Example
//!
//! A minimal example demonstrating the popper component

use sig::popper::{Placement, popper};
use sig::prelude::*;
use sig::{Element, portal};
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    // Create portal container
    let portal_root = portal::provide();

    view()
      .style(|s| {
        s.left(0)
          .top(0)
          .width(taffy::Dimension::Percent(1.0))
          .height(taffy::Dimension::Percent(1.0))
      })
      .child((app_content(), portal_root))
  })
}

fn app_content() -> View {
  let show_tooltip = Signal::new(false);

  view()
    .style(|s| {
      s.width(taffy::Dimension::Percent(1.0))
        .height(taffy::Dimension::Percent(1.0))
        .flex()
        .items_center()
        .justify_center()
        .background(Color::from_rgb8(240, 244, 248))
    })
    .child({
      let tooltip = view()
        .style(|s| {
          s.padding(8.0)
            .min_width(100.0)
            .background(Color::from_rgb8(30, 30, 30))
            .border_radius(4.0)
            .pointer_events_none() // ✅ FIX: Prevent tooltip from blocking mouse events
        })
        .child(text("This is a tooltip!").style(|s| s.color(Color::WHITE).font_size(12.0)));

      popper(
        primary_button("Hover me")
          .on_mouse_enter(move |_| {
            *show_tooltip.write() = true;
          })
          .on_mouse_leave(move |_| {
            *show_tooltip.write() = false;
          }),
      )
      .content(tooltip)
      .placement(Placement::Top)
      .offset(8.0)
      .open(show_tooltip)
    })
}
