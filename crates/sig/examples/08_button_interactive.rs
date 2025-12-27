use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let count = Signal::new(0);

    // Create content
    let content = view()
      .style(|s| s.width(900.0).padding(40.0).flex().flex_col().gap(30.0))
      .child((
        // Title
        view()
          .style(|s| {
            s.flex()
              .flex_col()
              .gap(8.0)
              .padding_bottom(10.0)
              .border_bottom(2.0)
              .border_color(Color::from_rgb8(226, 232, 240)) // slate-200
          })
          .child((
            text("Button Component - Interactive Demo")
              .style(|s| s.font_size(28.0).color(Color::from_rgb8(15, 23, 42))), // slate-900
            text("Hover over buttons to see effects, click to interact")
              .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139))), // slate-500
          })),
        // Counter display
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .border_all(2.0, Color::from_rgb8(226, 232, 240)) // slate-200
              .flex()
              .flex_col()
              .gap(12.0)
          })
          .child((
            text("Interactive Counter")
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(71, 85, 105))), // slate-600
            dynamic_text(move || format!("Current count: {}", *count.read()))
              .style(|s| s.font_size(24.0).color(Color::from_rgb8(59, 130, 246))), // blue-500
          })),
        // Button variants showcase
        view().style(|s| s.flex().flex_col().gap(16.0)).child((
          view().style(|s| s.flex().flex_col().gap(4.0)).child((
            text("Button Variants")
              .style(|s| s.font_size(18.0).color(Color::from_rgb8(30, 41, 59))), // slate-800
            text("Try hovering and clicking each button to see the effects")
              .style(|s| s.font_size(13.0).color(Color::from_rgb8(100, 116, 139))), // slate-500
          )),
          view().style(|s| s.flex().gap(12.0).flex_wrap()).child((
            primary_button("Primary (+1)").on_click(move |_| {
              let c = *count.read();
              *count.write() = c + 1;
              println!("Primary clicked! Count: {}", c + 1);
            }),
            secondary_button("Secondary").on_click(|_| println!("Secondary clicked!")),
            outline_button("Outline").on_click(|_| println!("Outline clicked!")),
            danger_button("Danger").on_click(|_| println!("Danger clicked!")),
            text_button("Text").on_click(|_| println!("Text clicked!")),
          })),
        )),
        // Button sizes showcase
        view().style(|s| s.flex().flex_col().gap(16.0)).child((
          view().style(|s| s.flex().flex_col().gap(4.0)).child((
            text("Button Sizes").style(|s| s.font_size(18.0).color(Color::from_rgb8(30, 41, 59))),
            text("Buttons are available in three sizes: Small, Medium, and Large")
              .style(|s| s.font_size(13.0).color(Color::from_rgb8(100, 116, 139))),
          )),
          view()
            .style(|s| s.flex().gap(12.0).items_center())
            .child((
              primary_button("Small")
                .size(ButtonSize::Small)
                .on_click(|_| println!("Small button clicked!")),
              primary_button("Medium (Default)")
                .size(ButtonSize::Medium)
                .on_click(|_| println!("Medium button clicked!")),
              primary_button("Large")
                .size(ButtonSize::Large)
                .on_click(|_| println!("Large button clicked!")),
            )),
        )),
        // Disabled state showcase
        view().style(|s| s.flex().flex_col().gap(16.0)).child((
          view().style(|s| s.flex().flex_col().gap(4.0)).child((
            text("Disabled State").style(|s| s.font_size(18.0).color(Color::from_rgb8(30, 41, 59))),
            text("Disabled buttons have reduced opacity and don't respond to interactions")
              .style(|s| s.font_size(13.0).color(Color::from_rgb8(100, 116, 139))),
          )),
          view().style(|s| s.flex().gap(12.0)).child((
            primary_button("Disabled Primary")
              .disabled(true)
              .on_click(|_| println!("This should not print!")),
            secondary_button("Disabled Secondary")
              .disabled(true)
              .on_click(|_| println!("This should not print!")),
            outline_button("Disabled Outline")
              .disabled(true)
              .on_click(|_| println!("This should not print!")),
          })),
        )),
        // Reset button
        view()
          .style(|s| {
            s.margin_top(10.0)
              .padding_top(20.0)
              .border_top(2.0)
              .border_color(Color::from_rgb8(226, 232, 240))
          })
          .child((view()
            .style(|s| s.flex().gap(12.0).items_center())
            .child((
              danger_button("Reset Counter")
                .size(ButtonSize::Large)
                .on_click(move |_| {
                  *count.write() = 0;
                  println!("Counter reset!");
                }),
              text("Click to reset the counter to 0")
                .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139))),
            )),)),
      }));

    // Use overflow style to enable scrolling
    view()
      .style(|s| {
        s.w_full()
          .h_full()
          .overflow_y_scroll()
          .background(Color::from_rgb8(248, 250, 252)) // slate-50
      })
      .child(content)
  })
}
