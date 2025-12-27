//! Test button text color inheritance
use sig::{create_scope, text, view, Element, Styleable};
use vello::peniko::Color;

fn main() {
  create_scope(|| {
    let btn = view()
      .style(|s| {
        s.width(200.0)
          .height(60.0)
          .background(Color::from_rgb8(59, 130, 246)) // blue background
          .color(Color::WHITE) // white text
          .flex()
          .items_center()
          .justify_center()
      })
      .name("Button")
      .child(text("Should be white"));
    
    let root = view()
      .style(|s| s.width(800.0).height(600.0).flex().items_center().justify_center())
      .name("Root")
      .child(btn);
    
    println!("\n=== Flushing pending signals to build tree ===");
    sig::runtime::flush_pending_signals();
    
    println!("\n=== Computing styles ===");
    sig::style::compute::batch_bubble_dirty_marks();
    
    let mut ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&root.id(), &mut ctx);
    
    println!("\n=== Checking computed styles ===");
    
    // Check button color
    let btn_id = root.id().get_children()[0];
    sig::runtime::with_layout(|runtime| {
      if let Some(style) = runtime.styles.get(&btn_id.node_id()) {
        println!("✅ Button style:");
        println!("  Color: {:?}", style.color);
      }
      
      // Check text color
      let text_id = btn_id.get_children()[0];
      if let Some(style) = runtime.styles.get(&text_id.node_id()) {
        println!("\n✅ Text style:");
        println!("  Color: {:?}", style.color);
      } else {
        println!("\n❌ Text style not found!");
      }
    });
  });
}
