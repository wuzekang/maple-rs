//! TextInput focus test

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
            text("TextInput Focus Test")
                .style(|s| s.font_size(24.0).color(Color::from_rgb8(30, 30, 30))),
            
            text_input()
                .placeholder("First input")
                .style(|s| s.width(300.0)),
            
            text_input()
                .placeholder("Second input")
                .style(|s| s.width(300.0)),
            
            text("Click inputs to see focus changes")
                .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 100, 100))),
        ))
}

fn main() -> anyhow::Result<()> {
    sig::render::run(
        sig::AppConfig {
            title: "TextInput Focus Test".to_string(),
            width: 600.0,
            height: 400.0,
        },
        app_view,
    )
}
