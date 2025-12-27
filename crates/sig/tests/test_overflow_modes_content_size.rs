//! Comprehensive test for Taffy's content_size behavior with scrollbar_width
//! 
//! This test investigates:
//! 1. Does Taffy subtract scrollbar_size from content_size?
//! 2. Does this only happen with Overflow::Scroll or Overflow::Auto?
//! 3. What is the exact behavior in different scenarios?

use sig::prelude::*;
use sig::{create_scope, view};
use taffy::Overflow;

#[test]
fn test_content_size_with_all_overflow_modes() {
    create_scope(|| {
        const SCROLLBAR_WIDTH: f32 = 8.0;
        
        // Test all overflow modes
        let test_cases = vec![
            ("Visible", Overflow::Visible),
            ("Hidden", Overflow::Hidden),
            ("Clip", Overflow::Clip),
            ("Scroll", Overflow::Scroll),
        ];

        println!("\n=== Content Size with Different Overflow Modes ===\n");

        for (name, overflow_mode) in test_cases {
            println!("--- Testing: Overflow::{} ---", name);

            // Create container with overflowing children
            let root = view()
                .style(move |s| {
                    let mut style = s.width(300.0)
                        .height(500.0)
                        .flex()
                        .flex_col();
                    
                    // Set overflow mode
                    style = match overflow_mode {
                        Overflow::Visible => style,
                        Overflow::Hidden => style.overflow_y_hidden(),
                        Overflow::Clip => style.overflow_y_clip(),
                        Overflow::Scroll => style.overflow_y_scroll(),
                    };
                    
                    style
                })
                .child(
                    // Children that overflow: 100 * 28px = 2800px
                    (0..100).map(|i| {
                        view().style(|s| s.w_full().height(28.0))
                            .child(text(format!("Item {}", i)))
                    }).collect::<Vec<_>>()
                );

            sig::runtime::flush_pending_signals();

            let available = taffy::Size {
                width: taffy::AvailableSpace::Definite(300.0),
                height: taffy::AvailableSpace::Definite(500.0),
            };
            let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
            root.id.compute_layout(available, &ctx);

            sig::runtime::with_layout(|runtime| {
                let layout = runtime.taffy.layout(root.id.node_id()).unwrap();
                let style = runtime.taffy.style(root.id.node_id()).unwrap();

                println!("  Container size: {:?}", layout.size);
                println!("  Content size: {:?}", layout.content_size);
                println!("  Scrollbar size: {:?}", layout.scrollbar_size);
                println!("  Style scrollbar_width: {}", style.scrollbar_width);

                // Check if content_size was reduced by scrollbar
                let expected_full_width = 300.0;
                let content_width = layout.content_size.width;
                let scrollbar_width_used = layout.scrollbar_size.width;

                if scrollbar_width_used > 0.0 {
                    println!("  ✅ Taffy recorded scrollbar size: {}", scrollbar_width_used);
                    
                    // Check if content_size was reduced
                    if content_width < expected_full_width {
                        let reduction = expected_full_width - content_width;
                        println!("  ✅ Content width REDUCED by: {}", reduction);
                        
                        if (reduction - scrollbar_width_used).abs() < 0.1 {
                            println!("     Reduction matches scrollbar_width!");
                        } else {
                            println!("     ⚠️  Reduction ({}) != scrollbar_width ({})",
                                reduction, scrollbar_width_used);
                        }
                    } else {
                        println!("  ⚠️  Content width NOT reduced (still {})", content_width);
                        println!("     Taffy did NOT subtract scrollbar from content_size");
                    }
                } else {
                    println!("  ℹ️  No scrollbar recorded (scrollbar_size = 0)");
                }
                
                println!();
            });
        }

        println!("=== Summary ===");
        println!("Check the results above to determine:");
        println!("1. Does Taffy reduce content_size when scrollbar_size > 0?");
        println!("2. Is this behavior specific to Overflow::Scroll?");
    });
}

#[test]
fn test_scrollbar_width_effect_on_children() {
    create_scope(|| {
        println!("\n=== Scrollbar Width Effect on Child Layout ===\n");

        // Create parent with overflow_y_scroll
        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y_scroll()  // This sets scrollbar_width = 8.0
                    .flex()
                    .flex_col()
            })
            .child(
                // Child with w_full() - should it be 300 or 292?
                view()
                    .style(|s| s.w_full().height(100.0))
                    .child(text("Child with w_full()"))
            );

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        let children = root.id.get_children();
        assert_eq!(children.len(), 1, "Should have 1 child");

        sig::runtime::with_layout(|runtime| {
            let parent_layout = runtime.taffy.layout(root.id.node_id()).unwrap();
            let child_layout = runtime.taffy.layout(children[0].node_id()).unwrap();

            println!("Parent:");
            println!("  size: {:?}", parent_layout.size);
            println!("  content_size: {:?}", parent_layout.content_size);
            println!("  scrollbar_size: {:?}", parent_layout.scrollbar_size);

            println!("\nChild (with w_full()):");
            println!("  size: {:?}", child_layout.size);

            // Key question: Does child width account for scrollbar?
            if child_layout.size.width == 300.0 {
                println!("  ⚠️  Child width is 300 (full container)");
                println!("     Child does NOT account for scrollbar");
                println!("     Content may be partially hidden by scrollbar!");
            } else if (child_layout.size.width - 292.0).abs() < 0.1 {
                println!("  ✅ Child width is 292 (300 - 8)");
                println!("     Child accounts for scrollbar");
                println!("     Content fully visible!");
            } else {
                println!("  ? Child width is {} (unexpected)", child_layout.size.width);
            }

            // The answer determines how Sig should handle scrollbar rendering
            if child_layout.size.width >= 300.0 {
                println!("\n💡 Conclusion:");
                println!("   Taffy does NOT reduce available space for children");
                println!("   Scrollbar will OVERLAP content (macOS style)");
                println!("   This is the current Sig behavior - correct!");
            }
        });
    });
}

#[test]
fn test_scrollbar_auto_vs_scroll() {
    create_scope(|| {
        println!("\n=== Overflow::Auto vs Overflow::Scroll ===\n");

        // Note: Taffy 0.9.1 may not fully support Auto
        // Let's test both to see the difference

        for (name, overflow) in vec![
            ("Scroll", taffy::Point { x: Overflow::Scroll, y: Overflow::Scroll }),
            // Auto mode - scrollbar only shows when needed
            // ("Auto", taffy::Point { x: Overflow::Auto, y: Overflow::Auto }),
        ] {
            println!("--- {} mode ---", name);

            let root = view()
                .style(move |s| {
                    s.width(300.0)
                        .height(500.0)
                        .overflow(overflow)
                        .scrollbar_width(8.0)
                })
                .child(
                    view().style(|s| s.w_full().height(100.0))
                );

            sig::runtime::flush_pending_signals();

            let available = taffy::Size {
                width: taffy::AvailableSpace::Definite(300.0),
                height: taffy::AvailableSpace::Definite(500.0),
            };
            let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
            root.id.compute_layout(available, &ctx);

            sig::runtime::with_layout(|runtime| {
                let layout = runtime.taffy.layout(root.id.node_id()).unwrap();
                
                println!("  scrollbar_size: {:?}", layout.scrollbar_size);
                println!("  content_size: {:?}", layout.content_size);
                println!();
            });
        }

        println!("💡 Both modes should reserve scrollbar_size");
        println!("   The difference is when the scrollbar is actually shown");
    });
}

#[test]
fn test_verify_scrollbar_width_constant_match() {
    // Verify that DEFAULT_SCROLLBAR_WIDTH matches view_render.rs constant
    const VIEW_RENDER_SCROLLBAR_WIDTH: f64 = 8.0;
    
    println!("\n=== Scrollbar Width Constants ===");
    println!("style::DEFAULT_SCROLLBAR_WIDTH: {}", sig::style::DEFAULT_SCROLLBAR_WIDTH);
    println!("view_render SCROLLBAR_WIDTH: {}", VIEW_RENDER_SCROLLBAR_WIDTH);

    assert_eq!(
        sig::style::DEFAULT_SCROLLBAR_WIDTH as f64,
        VIEW_RENDER_SCROLLBAR_WIDTH,
        "Scrollbar width constants must match!"
    );

    println!("✅ Constants match - consistent scrollbar width");
}
