//! Dirty Bubbling test example
//!
//! Test the Dirty Bubbling mechanism for style updates

use sig::{create_scope, view, Element, Styleable};
use vello::peniko::Color;

fn main() {
  create_scope(|| {
    // Build a simple three-layer tree
    let header = view()
      .style(|s| s.width(400.0).height(60.0).background(Color::from_rgb8(60, 60, 60)))
      .name("Header");
    let header_id = header.id();
      
    let content = view()
      .style(|s| s.width(400.0).height(200.0).background(Color::from_rgb8(100, 150, 200)))
      .name("Content");
    let content_id = content.id();
      
    let footer = view()
      .style(|s| s.width(400.0).height(60.0).background(Color::from_rgb8(80, 80, 80)))
      .name("Footer");

    let root = view()
      .style(|s| {
        s.width(800.0)
          .height(600.0)
          .flex()
          .flex_col()
          .background(Color::from_rgb8(40, 40, 40))
      })
      .name("Root")
      .child(header)
      .child(content)
      .child(footer);

    println!("\n=== View Tree ===");
    root.id().print_tree();
    
    println!("\n=== Children count: {}", root.id().get_children().len());
    
    println!("\n=== Dirty Bubbling Test ===\n");

    // Simulate: manually mark Content as dirty
    println!("📍 Step 1: Manually mark 'Content' as dirty\n");
    content_id.request_style();
    
    println!("\n📍 Step 2: Run compute_style_recursive from root\n");
    let mut style_ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&root.id(), &mut style_ctx);
    
    println!("\n✅ Expected behavior:");
    println!("  - Root: visited (child_dirty=true)");
    println!("  - Header: Bailout! (both dirty flags = false)");
    println!("  - Content: Computed (style_dirty=true)");
    println!("  - Footer: Bailout! (both dirty flags = false)");
    println!("\n🎯 Only 2 nodes computed (Root path + Content)");
  });
}
