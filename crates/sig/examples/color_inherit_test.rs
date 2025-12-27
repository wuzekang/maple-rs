//! Test color inheritance
use sig::{create_scope, view, text, Element, Styleable};
use vello::peniko::Color;

fn main() {
  create_scope(|| {
    let root = view()
      .style(|s| {
        s.width(800.0)
          .height(600.0)
          .background(Color::from_rgb8(50, 50, 50))
          .color(Color::from_rgb8(255, 255, 0)) // yellow text
      })
      .name("Root")
      .child(
        text("This should be yellow (inherited)")
      );
    
    println!("\n=== Testing color inheritance ===");
    
    // Compute styles
    sig::style::compute::batch_bubble_dirty_marks();
    let mut ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&root.id(), &mut ctx);
    
    // Check root node color
    sig::runtime::with_layout(|runtime| {
      if let Some(style) = runtime.styles.get(&root.id().node_id()) {
        println!("\n✅ Root style:");
        println!("  Background: {:?}", style.background);
        println!("  Color: {:?}", style.color);
      }
      
      // Check inheritable properties in ctx.style
      println!("\n📦 Context inherited styles:");
      for (key, value) in &ctx.style {
        println!("  {:?}: {:?}", key, value);
      }
    });
  });
}
