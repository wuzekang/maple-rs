//! Test Taffy's scrollbar_width handling

use sig::prelude::*;
use sig::{create_scope, view};

#[test]
fn test_taffy_scrollbar_width_property() {
    create_scope(|| {
        // Create a container with overflow
        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y_scroll()
            });

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let taffy_style = runtime.taffy.style(root.id.node_id()).unwrap();
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();

            println!("\n=== Taffy scrollbar_width Test ===");
            println!("Taffy style.scrollbar_width: {}", taffy_style.scrollbar_width);
            println!("Taffy layout.scrollbar_size: {:?}", layout.scrollbar_size);

            if taffy_style.scrollbar_width == 0.0 {
                println!("⚠️  scrollbar_width is 0.0 (not set by Sig)");
                println!("    This means Taffy doesn't reserve space for scrollbars");
                println!("    Scrollbars will overlap content!");
            } else {
                println!("✅ scrollbar_width is set: {}", taffy_style.scrollbar_width);
                println!("   Taffy will reserve this space in layout");
            }

            // Check if Taffy calculated scrollbar_size
            if layout.scrollbar_size.width == 0.0 && layout.scrollbar_size.height == 0.0 {
                println!("⚠️  layout.scrollbar_size is zero");
                println!("    Taffy didn't reserve space for scrollbars");
            } else {
                println!("✅ layout.scrollbar_size: {:?}", layout.scrollbar_size);
            }
        });
    });
}

#[test]
fn test_layout_with_scrollbar_width() {
    create_scope(|| {
        // Test what happens if we manually set scrollbar_width
        const SCROLLBAR_WIDTH: f32 = 8.0;

        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y_scroll()
            });

        sig::runtime::flush_pending_signals();

        // Manually set scrollbar_width in Taffy style
        sig::runtime::with_layout_mut(|runtime| {
            let node_id = root.id.node_id();
            if let Ok(mut style) = runtime.taffy.style(node_id).cloned() {
                style.scrollbar_width = SCROLLBAR_WIDTH;
                let _ = runtime.taffy.set_style(node_id, style);
                println!("\n=== Manual scrollbar_width Test ===");
                println!("Set style.scrollbar_width = {}", SCROLLBAR_WIDTH);
            }
        });

        // Re-compute layout
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();

            println!("After re-compute:");
            println!("  layout.size: {:?}", layout.size);
            println!("  layout.content_size: {:?}", layout.content_size);
            println!("  layout.scrollbar_size: {:?}", layout.scrollbar_size);

            if layout.scrollbar_size.width > 0.0 || layout.scrollbar_size.height > 0.0 {
                println!("✅ Taffy reserved space for scrollbar!");
                println!("   scrollbar_size: {:?}", layout.scrollbar_size);

                // With vertical scroll, should reserve width
                assert!(
                    layout.scrollbar_size.width > 0.0,
                    "Should reserve width for vertical scrollbar"
                );
            } else {
                println!("⚠️  Even after setting scrollbar_width, Taffy didn't reserve space");
            }
        });
    });
}

#[test]
fn test_content_size_with_scrollbar_width() {
    create_scope(|| {
        const SCROLLBAR_WIDTH: f32 = 8.0;

        // Create container with children
        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
            })
            .child(
                (0..100).map(|i| {
                    view().style(|s| s.w_full().height(28.0))
                        .child(text(format!("Item {}", i)))
                }).collect::<Vec<_>>()
            );

        sig::runtime::flush_pending_signals();

        // First: compute without scrollbar_width
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        let (content_size_without, size_without) = sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();
            (layout.content_size, layout.size)
        });

        println!("\n=== Content Size with vs without scrollbar_width ===");
        println!("Without scrollbar_width:");
        println!("  size: {:?}", size_without);
        println!("  content_size: {:?}", content_size_without);

        // Now set scrollbar_width and re-compute
        sig::runtime::with_layout_mut(|runtime| {
            if let Ok(mut style) = runtime.taffy.style(root.id.node_id()).cloned() {
                style.scrollbar_width = SCROLLBAR_WIDTH;
                let _ = runtime.taffy.set_style(root.id.node_id(), style);
            }
        });

        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();

            println!("\nWith scrollbar_width = {}:", SCROLLBAR_WIDTH);
            println!("  size: {:?}", layout.size);
            println!("  content_size: {:?}", layout.content_size);
            println!("  scrollbar_size: {:?}", layout.scrollbar_size);

            // Key question: Does content_size.width change?
            // If Taffy reserves space, content_size.width should be less
            let width_diff = content_size_without.width - layout.content_size.width;
            println!("\nContent width difference: {}", width_diff);

            if width_diff > 0.0 {
                println!("✅ Taffy reserved space! Content width reduced by {}", width_diff);
                println!("   This should equal scrollbar_width ({})", SCROLLBAR_WIDTH);

                // Verify it's approximately equal to scrollbar_width
                assert!(
                    (width_diff - SCROLLBAR_WIDTH).abs() < 0.1,
                    "Content width should be reduced by scrollbar_width"
                );
            } else {
                println!("⚠️  Content width unchanged - Taffy didn't reserve space");
            }
        });
    });
}
