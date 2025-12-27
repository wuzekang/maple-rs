//! Scroll example - Demonstrates the new scrolling architecture
//!
//! Shows how to use the overflow property to implement scrolling instead of using the scroll_view component

use sig::prelude::*;
use vello::peniko::Color;

fn app() -> View {
    view()
        .style(|s| {
            s.size_full()
                .flex()
                .flex_col()
                .background(Color::from_rgb8(15, 15, 15))
        })
        .child((
            // Title
            view()
                .style(|s| {
                    s.padding(20.0)
                        .background(Color::from_rgb8(51, 102, 153))
                        .width(taffy::Dimension::Percent(1.0))
                })
                .child(
                    text("Scroll Example - New Architecture")
                        .style(|s| {
                            s.font_size(24.0)
                                .color(Color::WHITE)
                        }),
                ),

            // Main content area
            view()
                .style(|s| {
                    s.flex()
                        .gap(20.0)
                        .padding(20.0)
                        .flex_grow(1.0)
                })
                .child((
                    // Left: Vertical scrolling
                    vertical_scroll_demo(),

                    // Right: Horizontal scrolling
                    horizontal_scroll_demo(),
                )),
        ))
}

fn vertical_scroll_demo() -> View {
    view()
        .style(|s| {
            s.flex()
                .flex_col()
                .background(Color::from_rgb8(64, 64, 64))
                .border_radius(8.0)
                .flex_grow(1.0)
        })
        .child((
            // Title
            view()
                .style(|s| {
                    s.padding(10.0)
                        .background(Color::from_rgb8(76, 76, 76))
                        .width(taffy::Dimension::Percent(1.0))
                })
                .child(
                    text("Vertical Scrolling")
                        .style(|s| {
                            s.font_size(18.0)
                                .color(Color::WHITE)
                        }),
                ),
            
            // Scrollable content area - Container
            view()
                .style(|s| {
                    s.flex()
                        .flex_col()
                        .width(taffy::Dimension::Percent(1.0))
                        .height(400.0)
                        .overflow_y_scroll() // 🎯 Key: Auto scrolling
                        .background(Color::from_rgb8(51, 51, 51))
                })
                .child(
                    // Content container - Let it expand naturally
                    view()
                        .style(|s| {
                            s.flex()
                                .flex_col()
                                .width(taffy::Dimension::Percent(1.0))
                        })
                        .child({
                            // Generate many items
                            let items: Vec<_> = (0..50).map(|i| {
                        view()
                            .style(move |s| {
                                let bg = if i % 2 == 0 {
                                    Color::from_rgb8(64, 64, 64)
                                } else {
                                    Color::from_rgb8(76, 76, 76)
                                };
                                
                                s.padding(15.0)
                                    .margin(5.0)
                                    .background(bg)
                                    .width(taffy::Dimension::Percent(1.0))
                            })
                            .child(
                                text(format!("Item {}", i))
                                    .style(|s| s.color(Color::WHITE)),
                            )
                            .on_click(move |_| {
                                println!("Clicked item {}", i);
                            })
                            .into_node()
                    }).collect();
                    items
                        })
                ),
        ))
}

fn horizontal_scroll_demo() -> View {
    view()
        .style(|s| {
            s.flex()
                .flex_col()
                .background(Color::from_rgb8(64, 64, 64))
                .border_radius(8.0)
                .flex_grow(1.0)
        })
        .child((
            // Title
            view()
                .style(|s| {
                    s.padding(10.0)
                        .background(Color::from_rgb8(76, 76, 76))
                        .width(taffy::Dimension::Percent(1.0))
                })
                .child(
                    text("Horizontal Scrolling")
                        .style(|s| {
                            s.font_size(18.0)
                                .color(Color::WHITE)
                        }),
                ),
            
            // Scrollable content area - Container
            view()
                .style(|s| {
                    s.flex()
                        .height(taffy::Dimension::Percent(1.0))
                        .width(400.0)
                        .overflow_x_scroll() // 🎯 Key: Auto scrolling
                        .background(Color::from_rgb8(51, 51, 51))
                })
                .child(
                    // Content container - Let it expand naturally
                    view()
                        .style(|s| {
                            s.flex()
                                .height(taffy::Dimension::Percent(1.0))
                        })
                        .child({
                            // Generate many columns
                            let items: Vec<_> = (0..20).map(|i| {
                        view()
                            .style(move |s| {
                                s.padding(15.0)
                                    .margin(5.0)
                                    .background(Color::from_rgb8(102, 128, 153))
                                    .width(150.0)
                                    .height(200.0)
                            })
                            .child(
                                text(format!("Col {}", i))
                                    .style(|s| s.color(Color::WHITE)),
                            )
                            .on_click(move |_| {
                                println!("Clicked column {}", i);
                            })
                            .into_node()
                    }).collect();
                    items
                        })
                ),
        ))
}

fn main() -> anyhow::Result<()> {
    let config = AppConfig {
        title: "Scroll Example - New Architecture".to_string(),
        width: 1200.0,
        height: 800.0,
    };
    
    run(config, || app())
}
