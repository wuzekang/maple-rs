use sig::*;
use vello::peniko::Color;

fn main() {
    app("Button Hover Test", 400, 300, || {
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
                    .gap(20.0)
                    .background(Color::from_rgb8(245, 245, 245))
            })
            .children((
                // Counter display
                view().style(|s| s.flex().gap(20.0)).children((
                    text(move || format!("Enter: {}", enter_count.get()))
                        .style(|s| s.font_size(16.0).color(Color::from_rgb8(34, 197, 94))),
                    text(move || format!("Leave: {}", leave_count.get()))
                        .style(|s| s.font_size(16.0).color(Color::from_rgb8(239, 68, 68))),
                )),
                // Test button
                button("Hover Me")
                    .size(ButtonSize::Large)
                    .on_mouse_enter(move |_| {
                        *enter_count.write() += 1;
                        println!("🟢 Button MouseEnter (count: {})", enter_count.get());
                    })
                    .on_mouse_leave(move |_| {
                        *leave_count.write() += 1;
                        println!("🔴 Button MouseLeave (count: {})", leave_count.get());
                    }),
                // Instructions
                text("Move your mouse over the button text and background")
                    .style(|s| {
                        s.font_size(14.0)
                            .color(Color::from_rgb8(107, 114, 128))
                            .text_center()
                    }),
                text("Expected: Enter=1, Leave=0 while hovering")
                    .style(|s| {
                        s.font_size(12.0)
                            .color(Color::from_rgb8(107, 114, 128))
                            .text_center()
                    }),
            ))
    })
    .run();
}
