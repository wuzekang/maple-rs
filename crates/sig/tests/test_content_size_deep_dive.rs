//! Additional test for content_size with actual children

use sig::prelude::*;
use sig::{create_scope, view};

#[test]
fn test_content_size_with_children() {
    create_scope(|| {
        // Create a container with children that overflow
        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .padding(20.0)
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
            })
            .child((
                // Child 1
                view().style(|s| s.w_full().height(300.0)),
                // Child 2
                view().style(|s| s.w_full().height(300.0)),
                // Child 3
                view().style(|s| s.w_full().height(300.0)),
                // Total: 900px > 460px (container content area)
            ));

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();

            println!("\n=== Content Size with Overflowing Children ===");
            println!("Container size: {:?}", layout.size);
            println!("Content size: {:?}", layout.content_size);
            println!("Padding: {:?}", layout.padding);

            // Container size (border-box)
            assert_eq!(layout.size.width, 300.0);
            assert_eq!(layout.size.height, 500.0);

            // Content area: 300 - 40 = 260, 500 - 40 = 460
            let content_area_width = 300.0 - 20.0 * 2.0;
            let content_area_height = 500.0 - 20.0 * 2.0;

            println!("\nExpected content area: {}x{}", content_area_width, content_area_height);
            
            // Actual children total: 3 * 300px = 900px height
            println!("Actual children height: 900px (3 * 300px)");
            println!("Taffy's content_size.height: {}", layout.content_size.height);

            // Key question: Does content_size reflect the actual overflow?
            if layout.content_size.height > content_area_height {
                println!("✅ content_size reflects overflowing content!");
                println!("   This is what we want for scrollbar calculations");
            } else {
                println!("⚠️  content_size does NOT reflect overflow");
                println!("   content_size: {}", layout.content_size.height);
                println!("   expected (900px) or at least > {} (content area)",
                    content_area_height);
            }

            // Check if content_size is close to actual children size
            assert!(
                layout.content_size.height >= 890.0,
                "content_size should be at least 890px (close to 900px), got {}",
                layout.content_size.height
            );
        });
    });
}

#[test]
fn test_scrollbar_calculation_with_real_content() {
    create_scope(|| {
        // Simulate the actual left panel scenario
        let items: Vec<i32> = (0..100).collect();

        let root = view()
            .style(|s| {
                s.width(280.0)
                    .height(600.0)
                    .flex()
                    .flex_col()
            })
            .child((
                // Search box (fixed height)
                view().style(|s| s.w_full().height(48.0).flex_shrink(0.0)),
                // List container (should scroll)
                view()
                    .style(|s| {
                        s.w_full()
                            .flex_grow(1.0)
                            .min_height(0.0)
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                    })
                    .child(
                        items.iter().map(|i| {
                            view().style(|s| s.w_full().height(28.0))
                                .child(text(format!("Item {}", i)))
                        }).collect::<Vec<_>>()
                    ),
            ));

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(280.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (280.0, 600.0));
        root.id.compute_layout(available, &ctx);

        let children = root.id.get_children();
        println!("\n=== Real Left Panel Scenario ===");
        println!("Root children count: {}", children.len());

        // Get list container (second child)
        if children.len() >= 2 {
            let list_container = children[1];

            sig::runtime::with_layout(|runtime| {
                let layout = runtime.taffy.layout(list_container.node_id()).unwrap();

                println!("\nList container layout:");
                println!("  size: {:?}", layout.size);
                println!("  content_size: {:?}", layout.content_size);

                // List container should be: 600 - 48 = 552px height
                println!("\nExpected list height: {} (600 - 48)", 600.0 - 48.0);
                println!("Actual list height: {}", layout.size.height);

                // Content should be: 100 items * 28px = 2800px
                println!("\nExpected content height: 2800 (100 * 28)");
                println!("Actual content_size.height: {}", layout.content_size.height);

                // Check if Taffy tracks the overflow correctly
                if layout.content_size.height >= 2800.0 {
                    println!("✅ Taffy correctly tracks overflowing content");
                } else {
                    println!("⚠️  Taffy's content_size ({}) < actual content (2800)",
                        layout.content_size.height);
                    println!("    This confirms scrollbar calculations need manual tracking!");
                }

                // Scrollbar position calculation
                let scrollbar_x = layout.size.width - 8.0 - 2.0;  // width - scrollbar - padding
                println!("\nScrollbar X position:");
                println!("  Container width: {}", layout.size.width);
                println!("  Calculated X: {}", scrollbar_x);
                println!("  Should be visible: {}", scrollbar_x >= 0.0 && scrollbar_x < layout.size.width);
            });
        }
    });
}
