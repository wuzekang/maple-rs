//! Portal Simple API Demo
//!
//! Demonstrates the new simplified portal API with just 2 functions:
//! - portal::provide() - Create portal container
//! - portal::child(view) - Render to portal with auto cleanup

use sig::dynamic;
use sig::portal;
use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    // Create portal container (only once at root level)
    let modal_root = portal::provide();

    // Style the portal container
    let styled_modal_root = modal_root;

    // Main app
    view()
      .style(|s| {
        s.width(taffy::Dimension::Percent(1.0))
          .height(taffy::Dimension::Percent(1.0))
      })
      .child((main_content(), styled_modal_root))
  })
}

fn main_content() -> View {
  let open = show_modal();

  view()
    .style(|s| {
      s.width(taffy::Dimension::Percent(1.0))
        .height(taffy::Dimension::Percent(1.0))
        .padding(40.0)
        .flex()
        .flex_col()
        .gap(20.0)
        .items_center()
        .justify_center()
        .background(Color::from_rgb8(240, 244, 248))
    })
    .child((
      text("Portal Simple API Demo"),
      text("Only 2 functions needed: portal::provide() and portal::child()"),
      primary_button("Open Modal").on_click(move |_| {
        dbg!("Open Modal clicked");
        *open.write() = true;
      }),
    ))
}

fn show_modal() -> Signal<bool> {
  let open = Signal::new(false);

  // ✨ Magic happens here!
  // portal::child() renders to the portal container
  // Automatically cleaned up when this scope is destroyed

  portal::child(dynamic(move || {
    if *open.read() {
      dbg!("Rendering modal to portal");
      fragment(
        view()
          .style(|s| {
            s.position(taffy::Position::Absolute)
              .left(0)
              .top(0)
              .w_full()
              .h_full()
              .flex()
              .justify_content(taffy::JustifyContent::Center)
              .align_items(taffy::AlignItems::Center)
          })
          .child((
            // Background overlay layer - positioned absolutely to fill container
            view()
              .style(|s| {
                s.position(taffy::Position::Absolute)
                  .left(0)
                  .top(0)
                  .w_full()
                  .h_full()
                  .background(Color::from_rgba8(0, 0, 0, 128))
              })
              .on_click(move |_| {
                println!("Overlay clicked - closing modal");
                *open.write() = false;
              }),
            // Foreground dialog layer
            view()
              .style(|s| {
                s.width(500.0)
                  .height(300.0)
                  .padding(30.0)
                  .flex()
                  .flex_col()
                  .gap(20.0)
                  .background(Color::WHITE)
                  .border_radius(16.0)
              })
              .child((
                text("Modal Dialog"),
                text("Using portal::child()!"),
                text("Auto cleanup when scope ends!"),
                danger_button("Close").on_click(move |_| {
                  *open.write() = false;
                }),
              )),
          )),
      )
    } else {
      fragment(())
    }
  }));

  open
}
