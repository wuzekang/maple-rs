//! TextInput Element Trait Example
//!
//! Demonstrates that TextInput now implements the Element trait,
//! which provides access to styling and event handling methods.

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), || {
        let value = Signal::new(String::new());
        
        let root = view()
            .style(|s| {
                s.width(800.0)
                    .height(600.0)
                    .background(Color::from_rgb8(243, 244, 246))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .gap(20.0)
                    .padding(40.0)
            })
            .child(
                text("TextInput with Element Trait")
                    .style(|s| {
                        s.font_size(24.0)
                            .color(Color::from_rgb8(17, 24, 39))
                            .margin_bottom(20.0)
                    }),
            )
            .child(
                // TextInput now implements Element trait, so it has:
                // - All styling methods via Styleable
                // - All event handling methods via Interactive
                text_input()
                    .placeholder("Enter text here...")
                    .value(value.clone())
                    .width(400.0)
                    // Direct styling methods from Element/Styleable trait:
                    .style(|s| s
                        .margin_top(20.0)
                        .margin_bottom(20.0)
                        .font_size(16.0)
                    )
                    // Direct event handling from Element/Interactive trait:
                    .on_mouse_enter(|_| {
                        println!("Mouse entered text input!");
                    })
                    .on_mouse_leave(|_| {
                        println!("Mouse left text input!");
                    }),
                    // Note: No need to call .build() - Element trait provides ViewTuple automatically!
            )
            .child(
                dynamic_text(move || format!("Current value: {}", value.read().clone()))
                    .style(|s| {
                        s.font_size(14.0)
                            .color(Color::from_rgb8(107, 114, 128))
                    }),
            )
            .child(
                text("Notice: Mouse enter/leave events are logged to console!")
                    .style(|s| {
                        s.font_size(12.0)
                            .color(Color::from_rgb8(156, 163, 175))
                            .margin_top(20.0)
                    }),
            );
        
        root
    })
}
