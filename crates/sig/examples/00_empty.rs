//! Minimal empty app - for testing window focus issues
//! 
//! Test scenarios:
//! 1. Does the window automatically get focus after app startup
//! 2. Can the window regain focus after losing it
//! 3. Can it activate and respond normally

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    run(AppConfig {
        title: "Empty App - Focus Test".to_string(),
        width: 400.0,
        height: 300.0,
    }, || {
        eprintln!("🚀 Empty application starting");
        
        // Create the simplest view - just a white background
        let root = view()
            .style(|s| {
                s.width(400.0)
                    .height(300.0)
                    .background(Color::WHITE)
                    .flex()
                    .justify_center()
                    .items_center()
            })
            .child(
                view()
                    .style(|s| {
                        s.width(200.0)
                            .height(100.0)
                            .background(Color::from_rgb8(100, 150, 255))
                            .flex()
                            .justify_center()
                            .items_center()
                    })
            );

        eprintln!("✅ Empty view created");
        root
    })
}
