//! Test scrollbar positioning with borders

use sig::prelude::*;
use sig::{create_scope, view};

#[test]
fn test_scrollbar_with_border() {
    create_scope(|| {
        // Container with border_right
        let root = view()
            .style(|s| {
                s.width(280.0)
                    .height(600.0)
                    .border_right(1.0)  // ← 右侧有 1px 边框
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
            })
            .child(
                // Content that overflows
                (0..100).map(|i| {
                    view().style(|s| s.w_full().height(28.0))
                        .child(text(format!("Item {}", i)))
                }).collect::<Vec<_>>()
            );

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(280.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (280.0, 600.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();

            println!("\n=== Scrollbar with Border Test ===");
            println!("Container size: {:?}", layout.size);
            println!("Content size: {:?}", layout.content_size);
            println!("Border: {:?}", layout.border);
            println!("Padding: {:?}", layout.padding);

            // Calculate scrollbar position
            const SCROLLBAR_WIDTH: f64 = 8.0;
            const SCROLLBAR_PADDING: f64 = 2.0;

            let container_width = layout.size.width as f64;
            let scrollbar_x = 0.0 + container_width - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

            println!("\nScrollbar calculation:");
            println!("  Container width (border-box): {}", container_width);
            println!("  Scrollbar X: {}", scrollbar_x);
            println!("  Border right: {}", layout.border.right);

            // The issue: If scrollbar_x doesn't account for border,
            // it will be drawn over the border area!

            // Correct calculation should be:
            let correct_scrollbar_x = container_width - layout.border.right as f64 
                - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

            println!("\n  Correct scrollbar X (inside border): {}", correct_scrollbar_x);
            println!("  Difference: {}", scrollbar_x - correct_scrollbar_x);

            if layout.border.right > 0.0 {
                println!("\n⚠️  With border_right, scrollbar may overlap border!");
                println!("   Current calculation doesn't account for border");
            }
        });
    });
}

#[test]
fn test_scrollbar_positioning_formula() {
    // Test the correct formula for scrollbar positioning
    
    let scenarios = vec![
        ("No border, no padding", 280.0, 0.0, 0.0),
        ("With right border", 280.0, 1.0, 0.0),
        ("With padding", 280.0, 0.0, 12.0),
        ("With both", 280.0, 1.0, 12.0),
    ];

    const SCROLLBAR_WIDTH: f64 = 8.0;
    const SCROLLBAR_PADDING: f64 = 2.0;

    println!("\n=== Scrollbar Positioning Formula Test ===\n");

    for (name, container_width, border_right, padding_right) in scenarios {
        println!("Scenario: {}", name);
        println!("  Container width: {}", container_width);
        println!("  Border right: {}", border_right);
        println!("  Padding right: {}", padding_right);

        // Current formula (in sig)
        let current_x = container_width - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

        // Correct formula (should account for border)
        let correct_x = container_width - border_right - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

        println!("  Current formula X: {}", current_x);
        println!("  Correct formula X: {}", correct_x);
        println!("  Difference: {}\n", current_x - correct_x);

        if border_right > 0.0 {
            assert!(
                correct_x < current_x,
                "With border, scrollbar should be positioned further left"
            );
        }
    }

    println!("✅ Scrollbar positioning formula analysis complete");
}
