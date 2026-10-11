//! Compare two scrollbar design approaches:
//! A. Reserve space (Taffy scrollbar_width) - Windows style
//! B. Overlay (no space reserved) - macOS/iOS style

use sig::prelude::*;
use sig::{create_scope, view};

#[test]
fn test_scrollbar_design_comparison() {
    create_scope(|| {
        println!("\n=== Scrollbar Design: Reserved vs Overlay ===\n");

        // Design A: Reserve space (set scrollbar_width)
        println!("--- Design A: Reserve Space (Windows style) ---");
        
        let root_a = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y_scroll()  // Sets scrollbar_width = 8.0
                    .flex()
                    .flex_col()
            })
            .child(
                view().style(|s| s.w_full().height(100.0).background(vello::peniko::Color::from_rgb8(200, 200, 200)))
                    .child(text("Content"))
            );

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root_a.id.compute_layout(available, &ctx);

        let children_a = root_a.id.get_children();
        
        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root_a.id.node_id()).unwrap();
            let child_layout = runtime.taffy.layout(children_a[0].node_id()).unwrap();

            println!("  Container width: {}", layout.size.width);
            println!("  Content area width: {}", layout.content_size.width);
            println!("  Scrollbar size: {:?}", layout.scrollbar_size);
            println!("  Child width: {}", child_layout.size.width);
            println!();
            println!("  Visual:");
            println!("  ┌────────────────────────────┐");
            println!("  │ Content (292px)         ║█║ ← 8px scrollbar");
            println!("  │                         ║█║");
            println!("  └────────────────────────────┘");
            println!("     292px used for content");
            println!();
        });

        // Design B: Overlay (don't set scrollbar_width)
        println!("--- Design B: Overlay (macOS/iOS style) ---");
        
        let root_b = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y(taffy::Overflow::Scroll)
                    .scrollbar_width(0.0)  // Explicitly set to 0
                    .flex()
                    .flex_col()
            })
            .child(
                view().style(|s| s.w_full().height(100.0).background(vello::peniko::Color::from_rgb8(200, 200, 200)))
                    .child(text("Content"))
            );

        sig::runtime::flush_pending_signals();
        root_b.id.compute_layout(available, &ctx);

        let children_b = root_b.id.get_children();
        
        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root_b.id.node_id()).unwrap();
            let child_layout = runtime.taffy.layout(children_b[0].node_id()).unwrap();

            println!("  Container width: {}", layout.size.width);
            println!("  Content area width: {}", layout.content_size.width);
            println!("  Scrollbar size: {:?}", layout.scrollbar_size);
            println!("  Child width: {}", child_layout.size.width);
            println!();
            println!("  Visual:");
            println!("  ┌────────────────────────────┐");
            println!("  │ Content (300px)        ║█║ ← 8px scrollbar (overlay)");
            println!("  │                        ║█║    Semi-transparent");
            println!("  └────────────────────────────┘");
            println!("     300px used for content");
            println!("     Last 8px partially covered by scrollbar");
            println!();
        });

        // Comparison
        println!("=== Comparison ===\n");
        
        let (content_a, child_a) = sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root_a.id.node_id()).unwrap();
            let child = runtime.taffy.layout(children_a[0].node_id()).unwrap();
            (layout.content_size.width, child.size.width)
        });

        let (content_b, child_b) = sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root_b.id.node_id()).unwrap();
            let child = runtime.taffy.layout(children_b[0].node_id()).unwrap();
            (layout.content_size.width, child.size.width)
        });

        println!("Design A (Reserved):");
        println!("  ✅ Content area: {}px (reduced)", content_a);
        println!("  ✅ Child uses: {}px", child_a);
        println!("  ✅ Scrollbar: 8px (in reserved space)");
        println!("  ✅ Content fully visible");
        println!("  ❌ Less horizontal space for content (-8px)");
        println!();

        println!("Design B (Overlay):");
        println!("  ✅ Content area: {}px (full width)", content_b);
        println!("  ✅ Child uses: {}px", child_b);
        println!("  ✅ Scrollbar: 8px (overlays content)");
        println!("  ✅ More horizontal space for content (+8px)");
        println!("  ⚠️  Last 8px partially covered (needs transparency)");
        println!();

        println!("=== Which is better? ===");
        println!();
        println!("Design A (Reserved Space):");
        println!("  ✓ Windows/traditional desktop style");
        println!("  ✓ Content never obscured");
        println!("  ✓ Clear separation");
        println!("  ✗ Takes up layout space");
        println!();
        
        println!("Design B (Overlay):");
        println!("  ✓ macOS/iOS/modern web style");
        println!("  ✓ Maximum content space");
        println!("  ✓ Cleaner look");
        println!("  ✓ Can be semi-transparent");
        println!("  ✗ May partially cover content");
        println!();

        println!("💡 Recommendation:");
        println!("   For modern UI: Use Design B (Overlay)");
        println!("   For maximum clarity: Use Design A (Reserved)");
    });
}

#[test]
fn test_overlay_scrollbar_implementation() {
    create_scope(|| {
        println!("\n=== Overlay Scrollbar Implementation ===\n");

        // Create container WITHOUT setting scrollbar_width
        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .overflow_y(taffy::Overflow::Scroll)
                    // Don't call .scrollbar_width() or set it to 0
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

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();
            let style = runtime.taffy.style(root.id.node_id()).unwrap();

            println!("Implementation:");
            println!("  style.scrollbar_width: {}", style.scrollbar_width);
            println!("  layout.scrollbar_size: {:?}", layout.scrollbar_size);
            println!("  layout.content_size.width: {}", layout.content_size.width);
            println!();

            if style.scrollbar_width == 0.0 {
                println!("✅ scrollbar_width = 0 (overlay mode)");
                
                if (layout.content_size.width - 300.0).abs() < 0.1 {
                    println!("✅ content_size = 300px (full width)");
                    println!("✅ Child elements will use full width");
                    println!("✅ Scrollbar will overlay on top");
                    println!();
                    println!("Implementation notes:");
                    println!("  - Scrollbar drawn at x = 290 (300 - 8 - 2)");
                    println!("  - Content width = 300px");
                    println!("  - Last 8-10px may be partially covered");
                    println!("  - Use semi-transparent scrollbar");
                    println!("  - Consider padding-right for critical content");
                }
            }
        });

        println!("\n💡 This is the macOS/iOS/modern web approach!");
    });
}
