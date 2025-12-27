//! Collapsible Component Demo Example
//!
//! Demonstrates the basic usage of the collapsible component

use sig::prelude::*;
use sig::Cursor;
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
                .background(Color::from_rgb8(248, 250, 252))
            )
            .child((
                // Title
                view()
                    .style(|s| s
                        .font_size(28.0)
                        .color(Color::from_rgb8(15, 23, 42))
                        .margin_bottom(24.0)
                    )
                    .child(text("Collapsible Component Demo")),

                // Content container
                view()
                    .style(|s| s
                        .background(Color::WHITE)
                        .padding(24.0)
                        .border_radius(12.0)
                        .border_all(1.0, Color::from_rgb8(229, 231, 235))
                        .flex()
                        .flex_col()
                    )
                    .child((
                        create_example1(),
                        view().style(|s| s.height(24.0)),
                        separator(),
                        view().style(|s| s.height(24.0)),
                        create_example2(),
                    ))
            ))
    })
}

/// Example 1: Basic usage
fn create_example1() -> View {
    let open = Signal::new(false);
    
    view()
        .style(|s| s.flex().flex_col())
        .child((
            view()
                .style(|s| s
                    .font_size(18.0)
                    .color(Color::from_rgb8(51, 65, 85))
                    .margin_bottom(12.0)
                )
                .child(text("1. Basic Collapsible Panel")),

            view().child(
                collapsible(open)
                    .trigger(
                        button("Click to Expand/Collapse")
                            .variant(ButtonVariant::Outline)
                            .style(|s| s.w_full())
                    )
                    .content(
                        view()
                            .style(|s| s
                                .padding(16.0)
                                .margin_top(8.0)
                                .background(Color::from_rgb8(241, 245, 249))
                                .border_radius(8.0)
                            )
                            .child(
                                view()
                                    .style(|s| s.color(Color::from_rgb8(71, 85, 105)))
                                    .child(text("This is the collapsible content area"))
                            )
                    )
            )
        ))
}

/// Example 2: FAQ List
fn create_example2() -> View {
    view()
        .style(|s| s.flex().flex_col())
        .child((
            view()
                .style(|s| s
                    .font_size(18.0)
                    .color(Color::from_rgb8(51, 65, 85))
                    .margin_bottom(12.0)
                )
                .child(text("2. FAQ List")),

            view()
                .style(|s| s.flex().flex_col())
                .child((
                    create_faq("What is sig?", "sig is a reactive UI framework"),
                    view().style(|s| s.height(8.0)),
                    create_faq("Does it support nesting?", "Yes! Arbitrary nesting is supported"),
                ))
        ))
}

fn create_faq(question: &str, answer: &str) -> View {
    let open = Signal::new(false);
    let q = question.to_string();
    let a = answer.to_string();
    
    view().child(
        collapsible(open.clone())
            .trigger(
                view()
                    .style(|s| s
                        .w_full()
                        .padding(12.0)
                        .background(Color::from_rgb8(249, 250, 251))
                        .border_radius(6.0)
                        .cursor(Cursor::Pointer)
                    )
                    .child(
                        view().child(
                            dynamic_text(move || {
                                let icon = if *open.read() { "▼" } else { "▶" };
                                format!("{} {}", icon, q)
                            })
                        )
                        .style(|s| s.color(Color::from_rgb8(51, 65, 85)))
                    )
            )
            .content(
                view()
                    .style(|s| s
                        .padding(12.0)
                        .margin_top(4.0)
                        .background(Color::from_rgb8(241, 245, 249))
                        .border_radius(6.0)
                    )
                    .child(
                        view()
                            .style(|s| s.color(Color::from_rgb8(100, 116, 139)))
                            .child(text(a))
                    )
            )
    )
}
