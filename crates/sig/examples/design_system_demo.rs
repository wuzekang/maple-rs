// Example: Design System Consistency Demo
//
// This example demonstrates the visual consistency achieved through
// the design guidelines and token system.

use sig::prelude::*;
use sig::theme::{Spacing, Size, Radius, FontSize};

fn main() {
    let config = AppConfig {
        title: "Design System Consistency Demo".to_string(),
        width: 800.0,
        height: 900.0,
    };
    
    run(config, || {
        view()
            .name("Root")
            .style(|s| {
                s.width(800.0)
                    .height(900.0)
                    .padding(Spacing::XXL)  // 24px padding
                    .background(vello::peniko::Color::from_rgb8(249, 250, 251)) // gray-50
                    .flex()
                    .flex_col()
                    .gap(Spacing::XXL)  // 24px gap between sections
            })
                .child((
                    // Header
                    text("Design System Consistency Demo")
                        .style(|s| s
                            .font_size(FontSize::XXL)  // 20px
                            .font_weight(700)
                            .margin_bottom(Spacing::SM)),  // 8px
                    
                    text("All components use design tokens for consistent sizing, spacing, and styling")
                        .style(|s| s
                            .font_size(FontSize::MD)  // 14px
                            .color(vello::peniko::Color::from_rgb8(107, 114, 128))  // gray-500
                            .margin_bottom(Spacing::XL)),  // 20px

                    // Section 1: Size Variants Alignment
                    section("Size Variants - Buttons & Inputs Aligned", || {
                        view()
                            .style(|s| s.flex().flex_col().gap(Spacing::LG))  // 16px gap
                            .child((
                                // Small size row
                                row("Small (32px height)", || {
                                    view()
                                        .style(|s| s.flex().gap(Spacing::SM))  // 8px gap
                                        .child((
                                            button("Small Button")
                                                .size(ButtonSize::Small),
                                            text_input()
                                                .placeholder("Small Input")
                                                .width(150.0)
                                                .style(|s| s.height(Size::SM)),  // 32px - aligned!
                                        ))
                                }),

                                // Medium size row
                                row("Medium (40px height - Default)", || {
                                    view()
                                        .style(|s| s.flex().gap(Spacing::SM))
                                        .child((
                                            button("Medium Button")
                                                .size(ButtonSize::Medium),
                                            text_input()
                                                .placeholder("Medium Input")
                                                .width(150.0),
                                            // Default height is Size::MD (40px) - perfectly aligned!
                                        ))
                                }),

                                // Large size row
                                row("Large (48px height)", || {
                                    view()
                                        .style(|s| s.flex().gap(Spacing::SM))
                                        .child((
                                            button("Large Button")
                                                .size(ButtonSize::Large),
                                            text_input()
                                                .placeholder("Large Input")
                                                .width(150.0)
                                                .style(|s| s
                                                    .height(Size::LG)      // 48px - aligned!
                                                    .font_size(FontSize::LG)),  // 16px
                                 ))
                                }),
                            ))
                    }),

                    // Section 2: Border Radius Consistency
                    section("Border Radius Scale", || {
                        view()
                            .style(|s| s.flex().gap(Spacing::MD))  // 12px gap
                            .child((
                                button("SM Radius (4px)")
                                    .size(ButtonSize::Small),
                                    // Small buttons use Radius::SM
                                
                                button("MD Radius (6px)")
                                    .size(ButtonSize::Medium),
                                    // Medium buttons use Radius::MD
                                
                                button("MD Radius (6px)")
                                    .size(ButtonSize::Large),
                                    // Large buttons keep Radius::MD (not too rounded)
                            ))
                    }),

                    // Section 3: Spacing Consistency
                    section("Spacing Scale in Action", || {
                        view()
                            .style(|s| s.flex().flex_col().gap(Spacing::MD))  // 12px gap
                            .child((
                                // Button group with SM gap
                                view()
                                    .style(|s| s
                                        .flex()
                                        .gap(Spacing::SM)  // 8px - tight spacing for button group
                                        .padding(Spacing::MD)  // 12px padding
                                        .background(vello::peniko::Color::WHITE)
                                        .border_radius(Radius::LG)  // 8px - larger radius for containers
                                        .border_all(1.0, vello::peniko::Color::from_rgb8(229, 231, 235)))
                                    .child((
                                        button("Action 1"),
                                        button("Action 2"),
                                        button("Action 3"),
                                    )),

                                // Form layout with LG gap
                                view()
                                    .style(|s| s
                                        .flex()
                                        .flex_col()
                                        .gap(Spacing::LG)  // 16px - standard form field spacing
                                        .padding(Spacing::LG)  // 16px padding
                                        .background(vello::peniko::Color::WHITE)
                                        .border_radius(Radius::LG)
                                        .border_all(1.0, vello::peniko::Color::from_rgb8(229, 231, 235)))
                                    .child((
                                        text_input()
                                            .placeholder("Email")
                                            .width(300.0),
                                        text_input()
                                            .placeholder("Password")
                                            .width(300.0),
                                        button("Submit")
                                            .variant(ButtonVariant::Primary),
                                    )),
                            ))
                    }),

                    // Section 4: Button Variants
                    section("Button Style Variants", || {
                        view()
                            .style(|s| s.flex().gap(Spacing::SM))  // 8px gap
                            .child((
                                primary_button("Primary"),
                                secondary_button("Secondary"),
                                outline_button("Outline"),
                                danger_button("Danger"),
                                text_button("Text"),
                            ))
                    }),

                    // Footer note
                    view()
                        .style(|s| s
                            .padding(Spacing::LG)  // 16px
                            .background(vello::peniko::Color::from_rgba8(59, 130, 246, 26))
                            .border_radius(Radius::MD)  // 6px
                            .border_all(1.0, vello::peniko::Color::from_rgb8(59, 130, 246)))
                        .child(
                            text("✓ All spacing, sizing, and radius values use design tokens from theme::")
                                .style(|s| s
                                    .font_size(FontSize::SM)  // 12px
                                    .color(vello::peniko::Color::from_rgb8(29, 78, 216)))
                        ),
                ))
        },
    );
}

// Helper: Section container
fn section(title: &'static str, content: impl FnOnce() -> View) -> View {
    view()
        .style(|s| s
            .flex()
            .flex_col()
            .gap(Spacing::MD)  // 12px gap
            .padding(Spacing::LG)  // 16px padding
            .background(vello::peniko::Color::WHITE)
            .border_radius(Radius::LG)  // 8px
            .border_all(1.0, vello::peniko::Color::from_rgb8(229, 231, 235)))
        .child((
            text(title)
                .style(|s| s
                    .font_size(FontSize::LG)  // 16px
                    .font_weight(600)
                    .margin_bottom(Spacing::SM)),  // 8px
            content(),
        ))
}

// Helper: Row with label
fn row(label: &'static str, content: impl FnOnce() -> View) -> View {
    view()
        .style(|s| s
            .flex()
            .items_center()
            .gap(Spacing::LG))  // 16px gap
        .child((
            text(label)
                .style(|s| s
                    .font_size(FontSize::SM)  // 12px
                    .color(vello::peniko::Color::from_rgb8(107, 114, 128))  // gray-500
                    .width(200.0)),
            content(),
        ))
}
