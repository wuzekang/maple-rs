//! Focus test - debug version

use sig::prelude::*;
use vello::peniko::Color;

fn app_view() -> View {
    view()
        .style(|s| {
            s.w_full()
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(20.0)
                .background(Color::from_rgb8(250, 250, 250))
        })
        .child((
            view()
                .style(|s| {
                    s.padding(12.0)
                        .border_radius(8.0)
                        .background(Color::from_rgb8(59, 130, 246))
                })
                .child(text("Click me"))
                .focusable() // 🎯 标记为 focusable
                .on_click(|_e| {
                    println!("🖱️  on_click triggered!");
                })
                .on_focus(|_e| {
                    println!("✅ on_focus triggered!");
                })
                .on_blur(|_e| {
                    println!("❌ on_blur triggered!");
                }),
        ))
}

fn main() -> anyhow::Result<()> {
    sig::render::run(
        sig::AppConfig {
            title: "Focus Test".to_string(),
            width: 400.0,
            height: 300.0,
        },
        app_view,
    )
}
