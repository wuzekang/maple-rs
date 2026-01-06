//! TextInput Component Example
//!
//! Demonstrates the basic usage and features of TextInput

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    // Create two-way bound Signal
    let username = Signal::new(String::new());
    let email = Signal::new(String::new());
    let bio = Signal::new(String::new());

    let root = view()
      .style(|s| {
        s.w_full()
          .h_full()
          .background(Color::from_rgb8(243, 244, 246)) // gray-100
          .flex()
          .flex_col()
          .items_center()
          .gap(20.0)
          .padding(40.0)
          .overflow_y_scroll()
      })
      .child((
        // Title
        text("TextInput Component Example").style(|s| {
          s.font_size(24.0)
            .color(Color::from_rgb8(17, 24, 39)) // gray-900
            .margin_bottom(20.0)
        }),
        // Form container
        view()
          .style(|s| {
            s.background(Color::WHITE)
              .padding(30.0)
              .border_radius(12.0)
              .border_all(1.0, Color::from_rgb8(229, 231, 235)) // gray-200
              .flex()
              .flex_col()
              .gap(20.0)
              .width(400.0)
          })
          .child((
            // Username input
            view().style(|s| s.flex().flex_col().gap(8.0)).child((
              text("Username").style(|s| {
                s.font_size(14.0).color(Color::from_rgb8(55, 65, 81)) // gray-700
              }),
              text_input()
                .placeholder("Enter username")
                .width(340.0)
                .value(username.clone()),
            )),
            // Email input
            view().style(|s| s.flex().flex_col().gap(8.0)).child((
              text("Email").style(|s| s.font_size(14.0).color(Color::from_rgb8(55, 65, 81))),
              text_input()
                .placeholder("example@email.com")
                .width(340.0)
                .value(email.clone()),
            )),
            // Input with length limit
            view().style(|s| s.flex().flex_col().gap(8.0)).child((
              text("Nickname (max 10 characters)")
                .style(|s| s.font_size(14.0).color(Color::from_rgb8(55, 65, 81))),
              text_input()
                .placeholder("Enter nickname")
                .width(340.0)
                .max_length(10),
            )),
            // Bio input (TextArea)
            view().style(|s| s.flex().flex_col().gap(8.0)).child((
              text("Bio (Multi-line)")
                .style(|s| s.font_size(14.0).color(Color::from_rgb8(55, 65, 81))),
              text_area()
                .placeholder("Tell us about yourself...\nSupports multiple lines.")
                .width(340.0)
                .height(100.0)
                .value(bio.clone()),
            )),
          )),
        // Real-time display of input content
        view()
          .style(|s| {
            s.background(Color::from_rgb8(239, 246, 255)) // blue-50
              .padding(20.0)
              .border_radius(8.0)
              .flex()
              .flex_col()
              .gap(8.0)
              .width(400.0)
          })
          .child((
            text("Real-time Preview:").style(|s| {
              s.font_size(14.0).color(Color::from_rgb8(30, 64, 175)) // blue-800
            }),
            dynamic_text(move || format!("Username: {}", username.read().clone())).style(|s| {
              s.font_size(14.0).color(Color::from_rgb8(59, 130, 246)) // blue-500
            }),
            dynamic_text(move || format!("Email: {}", email.read().clone()))
              .style(|s| s.font_size(14.0).color(Color::from_rgb8(59, 130, 246))),
            dynamic_text(move || format!("Bio: {}", bio.read().clone()))
              .style(|s| s.font_size(14.0).color(Color::from_rgb8(59, 130, 246))),
          )),
        // Instructions
        text("Tip: Click input box to focus, press ESC to lose focus").style(|s| {
          s.font_size(12.0)
            .color(Color::from_rgb8(107, 114, 128)) // gray-500
            .margin_top(10.0)
        }),
        text("Supported: Home/End, Left/Right arrow keys, Backspace/Delete")
          .style(|s| s.font_size(12.0).color(Color::from_rgb8(107, 114, 128))),
      ));

    root
  })
}
