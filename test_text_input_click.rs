//! 简化的 TextInput 点击测试

use sig::prelude::*;
use sig::render::{AppConfig, run};
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), || {
        let root = view()
            .style(|s| {
                s.width(400.0)
                    .height(300.0)
                    .background(Color::from_rgb8(243, 244, 246))
                    .flex()
                    .justify_center()
                    .items_center()
            })
            .child(
                text_input()
                    .placeholder("Click me!")
                    .width(200.0)
                    .build()
            );

        root
    })
}
