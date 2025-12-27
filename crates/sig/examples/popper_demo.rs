//! Popper Demo
//!
//! Demonstrates the popper component for positioning content relative to reference elements.
//! Shows various placements, tooltips, and dropdowns.

use sig::popper::{Placement, popper};
use sig::prelude::*;
use sig::{Element, portal};
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    // Create portal container for poppers
    let portal_root = portal::provide();

    view()
      .style(|s| s.flex().w_full().h_full())
      .child((demo_content(), portal_root))
  })
}

fn demo_content() -> View {
  view()
    .style(|s| {
      s.width(taffy::Dimension::Percent(1.0))
        .height(taffy::Dimension::Percent(1.0))
        .padding(40.0)
        .flex()
        .flex_col()
        .gap(40.0)
        .background(Color::from_rgb8(240, 244, 248))
    })
    .child((
      text("Popper Component Demo").style(|s| s.font_size(24.0).font_weight(700)),
      // Tooltip example
      tooltip_example(),
      // Dropdown example
      dropdown_example(),
      // All placements example
      placements_grid(),
    ))
}

fn tooltip_example() -> View {
  let show_tooltip = Signal::new(false);

  view().style(|s| s.flex().flex_col().gap(10.0)).child((
    text("Tooltip Example:").style(|s| s.font_weight(600)),
    view().style(|s| s.flex().gap(10.0)).child({
      let tooltip_content = view()
        .style(|s| {
          s.padding(8.0)
            .background(Color::from_rgb8(30, 30, 30))
            .border_radius(4.0)
            .min_width(100.0) // Add explicit min-width
        })
        .child(text("This is a tooltip!").style(|s| s.color(Color::WHITE).font_size(12.0)));

      popper(
        primary_button("Hover for tooltip")
          .on_mouse_enter(move |_| {
            *show_tooltip.write() = true;
          })
          .on_mouse_leave(move |_| {
            *show_tooltip.write() = false;
          }),
      )
      .content(tooltip_content)
      .placement(Placement::Top)
      .offset(8.0)
      .open(show_tooltip)
      .build_to_portal();

      button
    }),
  ))
}

fn dropdown_example() -> View {
  let show_dropdown = Signal::new(false);

  view().style(|s| s.flex().flex_col().gap(10.0)).child((
    text("Dropdown Example:").style(|s| s.font_weight(600)),
    view().style(|s| s.flex().gap(10.0)).child({
      let dropdown_content = view()
        .style(|s| {
          s.padding(8.0)
            .background(Color::WHITE)
            .border_radius(8.0)
            .flex()
            .flex_col()
            .gap(4.0)
            .min_width(150.0)
        })
        .child((
          dropdown_item("Option 1", show_dropdown),
          dropdown_item("Option 2", show_dropdown),
          dropdown_item("Option 3", show_dropdown),
        ));

      popper(primary_button("Toggle Dropdown").on_click(move |_| {
        *show_dropdown.write() = { !*show_dropdown.read() };
      }))
      .content(dropdown_content)
      .placement(Placement::BottomStart)
      .offset(4.0)
      .open(show_dropdown)
    }),
  ))
}

fn dropdown_item(label: &'static str, dropdown_open: Signal<bool>) -> View {
  let hovered = Signal::new(false);

  view()
    .style(move |s| {
      let bg = if *hovered.read() {
        Color::from_rgb8(240, 244, 248)
      } else {
        Color::TRANSPARENT
      };

      s.padding(8.0)
        .padding_left(12.0)
        .padding_right(12.0)
        .background(bg)
        .border_radius(4.0)
        .cursor(sig::Cursor::Pointer)
    })
    .on_mouse_enter(move |_| {
      *hovered.write() = true;
    })
    .on_mouse_leave(move |_| {
      *hovered.write() = false;
    })
    .on_click(move |_| {
      println!("Clicked: {}", label);
      *dropdown_open.write() = false;
    })
    .child(text(label))
}

fn placements_grid() -> View {
  view().style(|s| s.flex().flex_col().gap(10.0)).child((
    text("All Placements:").style(|s| s.font_weight(600)),
    view()
      .style(|s| s.flex().flex_wrap().gap(10.0))
      .child((
        placement_button("Top", Placement::Top),
        placement_button("TopStart", Placement::TopStart),
        placement_button("TopEnd", Placement::TopEnd),
        placement_button("Bottom", Placement::Bottom),
        placement_button("BottomStart", Placement::BottomStart),
        placement_button("BottomEnd", Placement::BottomEnd),
      ))
      .child((
        placement_button("Left", Placement::Left),
        placement_button("LeftStart", Placement::LeftStart),
        placement_button("LeftEnd", Placement::LeftEnd),
        placement_button("Right", Placement::Right),
        placement_button("RightStart", Placement::RightStart),
        placement_button("RightEnd", Placement::RightEnd),
      )),
  ))
}

fn placement_button(label: &'static str, placement: Placement) -> View {
  let show = Signal::new(false);

  view().child({
    let content = view()
      .style(|s| {
        s.padding(6.0)
          .padding_left(10.0)
          .padding_right(10.0)
          .background(Color::from_rgb8(59, 130, 246))
          .border_radius(4.0)
          .min_width(60.0) // Add explicit min-width
      })
      .child(text(label).style(|s| s.color(Color::WHITE).font_size(12.0)));

    popper(
      secondary_button(label)
        .size(ButtonSize::Small)
        .on_mouse_enter(move |_| {
          *show.write() = true;
        })
        .on_mouse_leave(move |_| {
          *show.write() = false;
        }),
    )
    .content(content)
    .placement(placement)
    .offset(8.0)
    .open(show)
    .build_to_portal()
  })
}
