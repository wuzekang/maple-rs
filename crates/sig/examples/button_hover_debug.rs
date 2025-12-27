use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let enter_count = Signal::new(0);
    let leave_count = Signal::new(0);

    view()
      .style(|s| {
        s.w_full()
          .h_full()
          .flex()
          .flex_col()
          .items_center()
          .justify_center()
          .gap(30.0)
          .background(Color::from_rgb8(248, 250, 252))
      })
      .child((
        // Title
        view().style(|s| s.flex().flex_col().gap(10.0).items_center()).child((
          text("🐛 Button Hover Event Debugging")
            .style(|s| s.font_size(24.0).color(Color::from_rgb8(30, 41, 59))),
          text("Move your mouse over the button text and background repeatedly")
            .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139))),
        )),
        // Event counters
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .flex()
              .gap(30.0)
          })
          .child((
            dynamic_text(move || format!("🟢 Enter: {}", *enter_count.read()))
              .style(|s| s.font_size(18.0).color(Color::from_rgb8(34, 197, 94))),
            dynamic_text(move || format!("🔴 Leave: {}", *leave_count.read()))
              .style(|s| s.font_size(18.0).color(Color::from_rgb8(239, 68, 68))),
          )),
        // Test button
        primary_button("Hover Me!")
          .size(ButtonSize::Large)
          .on_mouse_enter(move |_| {
            *enter_count.write() += 1;
            println!("🟢 MouseEnter #{}", *enter_count.read());
          })
          .on_mouse_leave(move |_| {
            *leave_count.write() += 1;
            println!("🔴 MouseLeave #{}", *leave_count.read());
          }),
        // Expected behavior
        view()
          .style(|s| {
            s.padding(15.0)
              .background(Color::from_rgb8(254, 249, 195))
              .border_radius(6.0)
              .max_width(400.0)
          })
          .child((
            text("✅ Expected: Enter=1, Leave=0 while hovering")
              .style(|s| s.font_size(14.0).color(Color::from_rgb8(113, 63, 18)).text_center()),
          )),
      ))
  })
}
