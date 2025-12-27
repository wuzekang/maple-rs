use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let count = Signal::new(0);

    view()
      .style(|s| {
        s.width(800.0)
          .height(600.0)
          .padding(40.0)
          .flex()
          .flex_col()
          .gap(20.0)
          .background(Color::from_rgb8(248, 250, 252)) // slate-50
      })
      .child((
        // Title
        view()
          .style(|s| s.font_size(24.0).color(Color::from_rgb8(30, 41, 59))) // slate-800
          .child(text("Button Component Example")),
        // Counter display
        view()
          .style(|s| {
            s.padding(16.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .border_all(1.0, Color::from_rgb8(226, 232, 240)) // slate-200
          })
          .child(
            dynamic_text(move || format!("Counter: {}", *count.read())).style(|s| s.text_nowrap()),
          ),
        // Button variants display
        view().style(|s| s.flex().flex_col().gap(16.0)).child((
          view()
            .child(text("Button Variants:"))
            .style(|s| s.font_size(16.0).color(Color::from_rgb8(71, 85, 105))), // slate-600
          view().style(|s| s.flex().gap(12.0)).child((
            primary_button("Primary").on_click(move |_| {
              let c = *count.read();
              *count.write() = c + 1;
              println!("Primary clicked! Count: {}", c + 1);
            }),
            secondary_button("Secondary").on_click(|_| println!("Secondary clicked!")),
            outline_button("Outline").on_click(|_| println!("Outline clicked!")),
            danger_button("Danger").on_click(|_| println!("Danger clicked!")),
            text_button("Text").on_click(|_| println!("Text clicked!")),
          )),
        )),
        // Button sizes display
        view().style(|s| s.flex().flex_col().gap(16.0)).child((
          view()
            .child(text("Button Sizes:"))
            .style(|s| s.font_size(16.0).color(Color::from_rgb8(71, 85, 105))),
          view().style(|s| s.flex().gap(12.0).items_center()).child((
            primary_button("Small")
              .size(ButtonSize::Small)
              .on_click(|_| println!("Small button clicked!")),
            primary_button("Medium")
              .size(ButtonSize::Medium)
              .on_click(|_| println!("Medium button clicked!")),
            primary_button("Large")
              .size(ButtonSize::Large)
              .on_click(|_| println!("Large button clicked!")),
          )),
        )),
        // Disabled state display
        view().style(|s| s.flex().flex_col().gap(16.0)).child((
          view()
            .child(text("Disabled State:"))
            .style(|s| s.font_size(16.0).color(Color::from_rgb8(71, 85, 105))),
          view().style(|s| s.flex().gap(12.0)).child((
            primary_button("Disabled")
              .disabled(true)
              .on_click(|_| println!("This should not print!")),
            secondary_button("Disabled")
              .disabled(true)
              .on_click(|_| println!("This should not print!")),
          )),
        )),
        // Reset button
        view().style(|s| s.margin_top(20.0)).child(
          danger_button("Reset Counter")
            .size(ButtonSize::Large)
            .on_click(move |_| {
              *count.write() = 0;
              println!("Counter reset!");
            }),
        ),
      ))
  })
}
