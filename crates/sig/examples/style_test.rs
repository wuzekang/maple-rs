//! Style test
use sig::{create_scope, view, Element, Styleable};
use vello::peniko::Color;

fn main() {
  create_scope(|| {
    let v = view()
      .style(|s| {
        s.width(200.0)
          .height(100.0)
          .background(Color::from_rgb8(255, 0, 0)) // Red background
      })
      .name("TestView");

    println!("\n=== View ID: {:?} ===", v.id());
    
    // Manually trigger style computation
    println!("\n📍 Computing styles...");
    let mut style_ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&v.id(), &mut style_ctx);

    // Check computed style
    println!("\n📍 Checking computed style...");
    sig::runtime::with_layout(|runtime| {
      if let Some(style) = runtime.styles.get(&v.id().node_id()) {
        println!("✅ Style found!");
        println!("  Background: {:?}", style.background);
        println!("  Font size: {:?}", style.font_size);
      } else {
        println!("❌ No style found!");
      }

      // Check taffy style
      if let Ok(taffy_style) = runtime.taffy.style(v.id().node_id()) {
        println!("\n✅ Taffy style found!");
        println!("  Width: {:?}", taffy_style.size.width);
        println!("  Height: {:?}", taffy_style.size.height);
      }
    });
  });
}
