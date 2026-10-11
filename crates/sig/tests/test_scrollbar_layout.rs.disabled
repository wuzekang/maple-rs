//! Test for scrollbar width calculation and Taffy layout behavior
//!
//! This test verifies:
//! 1. The difference between `layout.size` and `layout.content_size`
//! 2. How Taffy calculates dimensions with padding and overflow
//! 3. Scrollbar positioning logic

use sig::prelude::*;
use sig::{create_scope, view, VirtualList, Signal};

#[test]
fn test_taffy_content_size_vs_size() {
    create_scope(|| {
        // Create a container with explicit size and padding
        let root = view()
            .style(|s| {
                s.width(300.0)
                    .height(500.0)
                    .padding(20.0)
                    .overflow_y_scroll()
            });

        // Flush signals to trigger layout
        sig::runtime::flush_pending_signals();

        // Compute layout
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        // Check layout dimensions
        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();

            println!("=== Taffy Layout Test ===");
            println!("layout.size: {:?}", layout.size);
            println!("layout.content_size: {:?}", layout.content_size);
            println!("layout.padding: {:?}", layout.padding);
            println!("layout.border: {:?}", layout.border);

            // Expected behavior:
            // - layout.size: 300x500 (border-box, the full container size)
            // - layout.content_size: 260x460 (content-box, size - padding)
            //   because padding is 20px on each side: 300 - 20*2 = 260

            assert_eq!(layout.size.width, 300.0, "Border-box width should be 300");
            assert_eq!(layout.size.height, 500.0, "Border-box height should be 500");

            // Content size should be: size - padding (both sides)
            let expected_content_width = 300.0 - 20.0 * 2.0;
            let expected_content_height = 500.0 - 20.0 * 2.0;

            assert_eq!(
                layout.content_size.width, expected_content_width,
                "Content width should exclude padding"
            );
            assert_eq!(
                layout.content_size.height, expected_content_height,
                "Content height should exclude padding"
            );

            println!("✅ content_size correctly excludes padding");
        });
    });
}

#[test]
fn test_virtual_list_scrollbar_dimensions() {
    create_scope(|| {
        // Create a VirtualList with many items
        let items = Signal::new((0..1000).collect::<Vec<_>>());

        let root = view()
            .style(|s| s.width(300.0).height(500.0))
            .child(
                VirtualList::new(items)
                    .item_height(30.0)
                    .buffer_size(5)
                    .build(|item, _| text(format!("Item {}", item))),
            );

        // Flush signals
        sig::runtime::flush_pending_signals();

        // Compute layout
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        // Get VirtualList's layout
        let children = root.id.get_children();
        assert_eq!(children.len(), 1, "Should have one child (VirtualList)");

        let list_id = children[0];
        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(list_id.node_id()).unwrap();

            println!("\n=== VirtualList Layout Test ===");
            println!("layout.size: {:?}", layout.size);
            println!("layout.content_size: {:?}", layout.content_size);

            // VirtualList should fill parent
            assert_eq!(layout.size.width, 300.0);
            assert_eq!(layout.size.height, 500.0);

            // Content size (1000 items * 30px = 30000px height)
            // But VirtualList uses virtual scrolling, so content_size might not be this
            println!("Actual content dimensions in Taffy:");
            println!("  Width: {}", layout.content_size.width);
            println!("  Height: {}", layout.content_size.height);

            // The key question: Does Taffy know about the virtual content size?
            // If content_size.height < 30000, then Taffy doesn't track virtual content
            if layout.content_size.height < 30000.0 {
                println!("⚠️  Taffy does NOT track virtual content size!");
                println!("    This means scrollbar calculations need manual content size");
            } else {
                println!("✅ Taffy tracks full virtual content size");
            }
        });
    });
}

#[test]
fn test_scrollbar_position_calculation() {
    create_scope(|| {
        // Simulate scrollbar positioning logic
        const SCROLLBAR_WIDTH: f64 = 8.0;
        const SCROLLBAR_PADDING: f64 = 2.0;

        // Container dimensions
        let container_x = 0.0;
        let container_width = 300.0;
        let container_height = 500.0;

        // Content dimensions (from Taffy's content_size)
        let content_height = 2000.0; // Scrollable content

        // Calculate scrollbar X position (from render_scrollbars)
        let scrollbar_x = container_x + container_width - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

        println!("\n=== Scrollbar Position Calculation ===");
        println!("Container width: {}", container_width);
        println!("Scrollbar width: {}", SCROLLBAR_WIDTH);
        println!("Scrollbar padding: {}", SCROLLBAR_PADDING);
        println!("Scrollbar X: {}", scrollbar_x);
        println!("Expected: {}", 300.0 - 8.0 - 2.0);

        assert_eq!(scrollbar_x, 290.0, "Scrollbar should be at x=290");

        // Check if scrollbar is within container bounds
        assert!(
            scrollbar_x >= container_x,
            "Scrollbar should be inside container (left)"
        );
        assert!(
            scrollbar_x + SCROLLBAR_WIDTH <= container_x + container_width,
            "Scrollbar should be inside container (right)"
        );

        println!("✅ Scrollbar is positioned within container bounds");

        // Check scrollbar height calculation
        let scrollbar_height =
            ((container_height * container_height / content_height) as f64).max(20.0);
        println!("\nScrollbar height calculation:");
        println!("  Formula: container² / content = {} * {} / {}",
            container_height, container_height, content_height);
        println!("  Result: {}", scrollbar_height);
        println!("  Min size: 20.0");

        assert_eq!(scrollbar_height, 125.0, "Scrollbar height should be 125px");
    });
}

#[test]
fn test_taffy_overflow_behavior() {
    create_scope(|| {
        // Test different overflow settings
        let configs = vec![
            ("overflow_y_scroll", true, false),
            ("overflow_scroll", true, true),
            ("no_overflow", false, false),
        ];

        for (name, expect_y, expect_x) in configs {
            println!("\n=== Testing: {} ===", name);

            let root = match name {
                "overflow_y_scroll" => {
                    view().style(|s| s.width(300.0).height(500.0).overflow_y_scroll())
                }
                "overflow_scroll" => {
                    view().style(|s| s.width(300.0).height(500.0).overflow_scroll())
                }
                _ => view().style(|s| s.width(300.0).height(500.0)),
            };

            sig::runtime::flush_pending_signals();

            let available = taffy::Size {
                width: taffy::AvailableSpace::Definite(300.0),
                height: taffy::AvailableSpace::Definite(500.0),
            };
            let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
            root.id.compute_layout(available, &ctx);

            sig::runtime::with_layout(|runtime| {
                let taffy_style = runtime.taffy.style(root.id.node_id()).unwrap();
                println!("Taffy overflow.x: {:?}", taffy_style.overflow.x);
                println!("Taffy overflow.y: {:?}", taffy_style.overflow.y);

                if expect_y {
                    assert_eq!(
                        taffy_style.overflow.y,
                        taffy::Overflow::Scroll,
                        "overflow_y should be Scroll for {}", name
                    );
                }

                if expect_x {
                    assert_eq!(
                        taffy_style.overflow.x,
                        taffy::Overflow::Scroll,
                        "overflow_x should be Scroll for {}", name
                    );
                }
            });
        }
    });
}

#[test]
fn test_nested_containers_with_w_full() {
    create_scope(|| {
        // This tests the w_full() behavior in nested flex containers
        let root = view()
            .style(|s| s.width(300.0).height(500.0).flex().flex_row())
            .child((
                // Left panel (should be fixed width)
                view().style(|s| s.width(280.0).flex_shrink(0.0).h_full()),
                // Right panel (should fill remaining space)
                view().style(|s| s.flex_grow(1.0).h_full()),
            ));

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(300.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (300.0, 500.0));
        root.id.compute_layout(available, &ctx);

        let children = root.id.get_children();
        assert_eq!(children.len(), 2, "Should have 2 children");

        sig::runtime::with_layout(|runtime| {
            let left_layout = runtime.taffy.layout(children[0].node_id()).unwrap();
            let right_layout = runtime.taffy.layout(children[1].node_id()).unwrap();

            println!("\n=== Nested Container Test ===");
            println!("Left panel width: {}", left_layout.size.width);
            println!("Right panel width: {}", right_layout.size.width);

            assert_eq!(left_layout.size.width, 280.0, "Left should be 280px");
            assert_eq!(right_layout.size.width, 20.0, "Right should fill remaining (20px)");

            println!("✅ Flex layout works correctly");
        });
    });
}

#[test]
fn test_w_full_in_resizable_container() {
    create_scope(|| {
        // Test that a child with w_full() inside a fixed-width parent
        // actually fills the parent width
        let parent_width = 280.0;

        let root = view()
            .style(move |s| s.width(parent_width).height(500.0).flex().flex_col())
            .child(
                view()
                    .style(|s| s.w_full().h_full())
                    .child(text("Content")),
            );

        sig::runtime::flush_pending_signals();

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(280.0),
            height: taffy::AvailableSpace::Definite(500.0),
        };
        let ctx = sig::RenderContext::new(1.0, (280.0, 500.0));
        root.id.compute_layout(available, &ctx);

        let children = root.id.get_children();
        assert_eq!(children.len(), 1);

        sig::runtime::with_layout(|runtime| {
            let child_layout = runtime.taffy.layout(children[0].node_id()).unwrap();

            println!("\n=== w_full() Test ===");
            println!("Parent width: {}", parent_width);
            println!("Child width: {}", child_layout.size.width);

            assert_eq!(
                child_layout.size.width, parent_width,
                "Child with w_full() should fill parent"
            );

            println!("✅ w_full() works correctly");
        });
    });
}

#[test]
fn test_scrollbar_visibility_conditions() {
    // Test when scrollbar should be visible
    let container_height = 500.0;
    let test_cases = vec![
        ("Content smaller", 400.0, false),
        ("Content equal", 500.0, false),
        ("Content larger", 600.0, true),
        ("Content much larger", 2000.0, true),
    ];

    for (name, content_height, should_show) in test_cases {
        let needs_scrollbar = content_height > container_height;

        println!("\n{}: content={}, container={}, needs_scrollbar={}",
            name, content_height, container_height, needs_scrollbar);

        assert_eq!(
            needs_scrollbar, should_show,
            "Scrollbar visibility logic for: {}", name
        );
    }

    println!("\n✅ Scrollbar visibility logic is correct");
}
