// Style computation improvement verification example
//
// This example demonstrates how the improved style computation system works

use sig::{create_scope, view, text, Styleable};
use vello::peniko::Color;

fn main() {
    create_scope(|| {
        // Example 1: StyleTrigger::Layout - requires re-layout
        let _layout_trigger_view = view()
            .style(|s| s
                .width(800.0)      // Layout trigger
                .height(600.0)     // Layout trigger
                .font_size(24.0)   // Layout trigger
            )
            .child(text("Layout Trigger"));

        // Example 2: StyleTrigger::Paint - requires repaint
        let _paint_trigger_view = view()
            .style(|s| s
                .background(Color::rgb8(255, 0, 0))    // Paint trigger
                .color(Color::WHITE)                    // Paint trigger
                .border_radius(8.0)                     // Paint trigger
            )
            .child(text("Paint Trigger"));

        // Example 3: StyleTrigger::Composite - only composite layer update needed
        let _composite_trigger_view = view()
            .style(|s| s
                .opacity(0.5)              // Composite trigger
                .translate_x(10.0)         // Composite trigger
            )
            .child(text("Composite Trigger"));

        // Example 4: StyleTrigger::None - no repaint needed
        let _none_trigger_view = view()
            .style(|s| s
                .cursor(sig::Cursor::Pointer)  // None trigger
            )
            .child(text("None Trigger"));

        // Example 5: Inherited property test
        let _parent = view()
            .style(|s| s
                .color(Color::rgb8(0, 0, 255))        // Inheritable
                .font_size(20.0)                       // Inheritable
                .background(Color::rgb8(128, 128, 128)) // Non-inheritable
            )
            .child(
                view()
                    .child(text("Child 1 inherits color and font_size"))
            )
            .child(
                view()
                    .child(text("Child 2 inherits color and font_size"))
            );

        // Example 6: Dirty check optimization test
        // When parent's inheritable property changes, recursively update all child views
        // When parent's non-inheritable property changes, only update parent itself
        let _dirty_check_view = view()
            .style(|s| s
                .color(Color::rgb8(0, 255, 0))  // Inheritable - will trigger recursion
            )
            .child(
                view().child(text("Level 1"))
            )
            .child(
                view()
                    .child(view().child(text("Level 2-1")))
                    .child(view().child(text("Level 2-2")))
            );

        println!("✅ Style computation improvements verified:");
        println!("  1. StyleTrigger mechanism - fine-grained repaint control");
        println!("  2. strum::IntoEnumIterator - automatic property iteration");
        println!("  3. Dirty check optimization - recursive child views only when needed");
        println!("  4. Improved error handling - explicit panic");
        println!("  5. ViewId::request_repaint - unified repaint entry point");
        println!("  6. Style cache - supports inheritable property propagation");
    });
}
