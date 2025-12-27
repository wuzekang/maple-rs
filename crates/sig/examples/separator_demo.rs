//! Separator demonstration example
//!
//! Showcases the use of horizontal and vertical separators

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), || {
        view()
            .style(|s| s
                .width(800.0)
                .height(600.0)
                .padding(40.0)
                .flex()
                .flex_col()
                .items_center()
                .background(Color::from_rgb8(248, 250, 252)) // slate-50
            )
            .child((
                // Title
                view()
                    .style(|s| s
                        .font_size(28.0)
                        .color(Color::from_rgb8(15, 23, 42)) // slate-900
                        .margin_bottom(32.0)
                    )
                    .child(text("Separator Component Demo")),

                // Container
                view()
                    .style(|s| s
                        .width(700.0)
                        .background(Color::WHITE)
                        .padding(32.0)
                        .border_radius(12.0)
                        .border_all(1.0, Color::from_rgb8(229, 231, 235))
                        .flex()
                        .flex_col()
                    )
                    .child((
                        // Example 1: Horizontal separator
                        create_horizontal_example(),

                        view().style(|s| s.height(32.0)), // Spacing

                        // Example 2: Vertical separator
                        create_vertical_example(),

                        view().style(|s| s.height(32.0)), // Spacing

                        // Example 3: Custom styles
                        create_custom_example(),
                    ))
            ))
    })
}

/// Example 1: Horizontal separator
fn create_horizontal_example() -> View {
    view()
        .style(|s| s.flex().flex_col())
        .child((
            view()
                .style(|s| s
                    .font_size(18.0)
                    .color(Color::from_rgb8(71, 85, 105)) // slate-600
                    .margin_bottom(12.0)
                )
                .child(text("1. Horizontal Separator")),
            
            view()
                .style(|s| s.color(Color::from_rgb8(100, 116, 139))) // slate-500
                .child(text("Section 1")),
            
            separator().style(|s| s.margin_top(8.0).margin_bottom(8.0)),
            
            view()
                .style(|s| s.color(Color::from_rgb8(100, 116, 139)))
                .child(text("Section 2")),
            
            separator().style(|s| s.margin_top(8.0).margin_bottom(8.0)),
            
            view()
                .style(|s| s.color(Color::from_rgb8(100, 116, 139)))
                .child(text("Section 3")),
        ))
}

/// Example 2: Vertical separator
fn create_vertical_example() -> View {
    view()
        .style(|s| s.flex().flex_col())
        .child((
            view()
                .style(|s| s
                    .font_size(18.0)
                    .color(Color::from_rgb8(71, 85, 105))
                    .margin_bottom(12.0)
                )
                .child(text("2. Vertical Separator")),
            
            view()
                .style(|s| s
                    .flex()
                    .flex_row()
                    .items_center()
                )
                .child((
                    view()
                        .style(|s| s
                            .padding(12.0)
                            .color(Color::from_rgb8(100, 116, 139))
                        )
                        .child(text("Left")),
                    
                    vertical_separator().style(|s| s.height(40.0)),
                    
                    view()
                        .style(|s| s
                            .padding(12.0)
                            .color(Color::from_rgb8(100, 116, 139))
                        )
                        .child(text("Middle")),
                    
                    vertical_separator().style(|s| s.height(40.0)),
                    
                    view()
                        .style(|s| s
                            .padding(12.0)
                            .color(Color::from_rgb8(100, 116, 139))
                        )
                        .child(text("Right")),
                ))
        ))
}

/// Example 3: Custom styles
fn create_custom_example() -> View {
    view()
        .style(|s| s.flex().flex_col())
        .child((
            view()
                .style(|s| s
                    .font_size(18.0)
                    .color(Color::from_rgb8(71, 85, 105))
                    .margin_bottom(12.0)
                )
                .child(text("3. Custom Styles")),
            
            separator()
                .style(|s| s
                    .height(3.0)
                    .background(Color::from_rgb8(59, 130, 246)) // blue-500
                    .margin_top(8.0)
                    .margin_bottom(8.0)
                ),
            
            separator()
                .style(|s| s
                    .height(2.0)
                    .background(Color::from_rgb8(16, 185, 129)) // emerald-500
                    .margin_top(8.0)
                    .margin_bottom(8.0)
                ),
            
            separator()
                .style(|s| s
                    .height(4.0)
                    .background(Color::from_rgb8(239, 68, 68)) // red-500
                    .border_radius(2.0)
                    .margin_top(8.0)
                ),
        ))
}
