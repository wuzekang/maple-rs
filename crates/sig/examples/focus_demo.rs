//! Focus management demo - simplified

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
            text("Focus Management Demo")
                .style(|s| s.font_size(24.0).color(Color::from_rgb8(30, 30, 30))),
            
            // Button 1
            view()
                .style(|s| {
                    s.padding(12.0)
                        .border_radius(8.0)
                        .background(Color::from_rgb8(59, 130, 246))
                        .cursor(sig::event::Cursor::Pointer)
                })
                .child(text("Button 1").style(|s| s.color(Color::WHITE)))
                .on_focus(|_e| {
                    println!("✅ Button 1 got focus!");
                })
                .on_blur(|_e| {
                    println!("❌ Button 1 lost focus");
                }),
            
            // Button 2
            view()
                .style(|s| {
                    s.padding(12.0)
                        .border_radius(8.0)
                        .background(Color::from_rgb8(34, 197, 94))
                        .cursor(sig::event::Cursor::Pointer)
                })
                .child(text("Button 2").style(|s| s.color(Color::WHITE)))
                .on_focus(|_e| {
                    println!("✅ Button 2 got focus!");
                })
                .on_blur(|_e| {
                    println!("❌ Button 2 lost focus");
                }),
            
            text("Click buttons to see focus events")
                .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 100, 100))),
        ))
}

fn main() -> anyhow::Result<()> {
    sig::render::run(
        sig::AppConfig {
            title: "Focus Events Demo".to_string(),
            width: 600.0,
            height: 400.0,
        },
        app_view,
    )
}
