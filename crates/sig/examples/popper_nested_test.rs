//! Popper Nested Container Test
//!
//! Tests popper positioning in nested containers to verify absolute positioning fix

use sig::popper::{Placement, popper};
use sig::prelude::*;
use sig::{Element, portal};
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let portal_root = portal::provide();

    view()
      .style(|s| {
        s.width(taffy::Dimension::Percent(1.0))
          .height(taffy::Dimension::Percent(1.0))
      })
      .child((test_content(), portal_root))
  })
}

fn test_content() -> View {
  view()
    .style(|s| {
      s.width(taffy::Dimension::Percent(1.0))
        .height(taffy::Dimension::Percent(1.0))
        .padding(20.0)
        .flex()
        .flex_col()
        .gap(20.0)
        .background(Color::from_rgb8(240, 244, 248))
    })
    .child((
      text("Popper Nested Container Test").style(|s| s.font_size(24.0).font_weight(700)),
      text("This demo tests popper positioning in nested containers.")
        .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 100, 100))),
      // Level 1: Top-level container
      nested_container("Level 1 Container", 0, Color::from_rgb8(255, 240, 240)),
      // Level 2: Nested in padding
      view()
        .style(|s| {
          s.padding(40.0)
            .background(Color::from_rgb8(240, 255, 240))
            .border_radius(8.0)
        })
        .child(nested_container(
          "Level 2 Container (Nested)",
          40,
          Color::from_rgb8(240, 240, 255),
        )),
      // Level 3: Deeply nested with scroll
      view()
        .style(|s| {
          s.padding(20.0)
            .background(Color::from_rgb8(255, 255, 240))
            .border_radius(8.0)
        })
        .child(
          view()
            .style(|s| {
              s.padding(30.0)
                .background(Color::from_rgb8(255, 240, 255))
                .border_radius(8.0)
            })
            .child(nested_container(
              "Level 3 Container (Deeply Nested)",
              90,
              Color::from_rgb8(240, 255, 255),
            )),
        ),
    ))
}

fn nested_container(label: &'static str, offset_hint: u8, bg_color: Color) -> View {
  view()
    .style(move |s| {
      s.padding(20.0)
        .background(bg_color)
        .border_radius(8.0)
        .flex()
        .flex_col()
        .gap(10.0)
    })
    .child((
      text(label).style(|s| s.font_size(16.0).font_weight(600)),
      view().style(|s| s.flex().gap(10.0)).child((
        // Button at different positions to test offset calculation
        popper_button("Top", Placement::Top, offset_hint),
        popper_button("Bottom", Placement::Bottom, offset_hint + 10),
        popper_button("Left", Placement::Left, offset_hint + 20),
        popper_button("Right", Placement::Right, offset_hint + 30),
      )),
      text(format!(
        "Expected offset hint: {} (for verification)",
        offset_hint
      ))
      .style(|s| s.font_size(12.0).color(Color::from_rgb8(150, 150, 150))),
    ))
}

fn popper_button(label: &'static str, placement: Placement, visual_offset: u8) -> View {
  let show = Signal::new(false);

  view().child({
    let button = secondary_button(label)
      .size(ButtonSize::Small)
      .on_mouse_enter(move |_| {
        *show.write() = true;
      })
      .on_mouse_leave(move |_| {
        *show.write() = false;
      });

    // Different colored poppers to easily identify which one is showing
    let bg_color = match placement {
      Placement::Top | Placement::TopStart | Placement::TopEnd => Color::from_rgb8(255, 100, 100),
      Placement::Bottom | Placement::BottomStart | Placement::BottomEnd => {
        Color::from_rgb8(100, 255, 100)
      }
      Placement::Left | Placement::LeftStart | Placement::LeftEnd => {
        Color::from_rgb8(100, 100, 255)
      }
      _ => Color::from_rgb8(255, 200, 100),
    };

    let content = view()
      .style(move |s| {
        s.padding(12.0)
          .background(bg_color)
          .border_radius(8.0)
          .min_width(120.0)
          .flex()
          .flex_col()
          .gap(4.0)
      })
      .child((
        text(format!("{} Popper", label))
          .style(|s| s.color(Color::WHITE).font_size(14.0).font_weight(600)),
        text(format!("Offset: {}", visual_offset)).style(|s| s.color(Color::WHITE).font_size(12.0)),
      ));

    popper(button)
      .content(content)
      .placement(placement)
      .offset(8.0)
      .open(show)
  })
}
