//! Reactive style test
use sig::{create_scope, view, Element, Signal, Styleable};
use vello::peniko::Color;

fn main() {
  create_scope(|| {
    let color_signal = Signal::new(Color::from_rgb8(255, 0, 0));
    
    let v = view()
      .style({
        let color_signal = color_signal.clone();
        move |s| {
          let c = *color_signal.read();
          println!("🎨 style() closure running, color: {:?}", c);
          s.width(200.0)
            .height(100.0)
            .background(c)
        }
      })
      .name("ReactiveView");

    println!("\n=== After view creation ===");
    
    // Manually trigger style computation
    println!("\n📍 Computing styles...");
    let mut style_ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&v.id(), &mut style_ctx);

    // Check computed style
    println!("\n📍 Checking computed style...");
    sig::runtime::with_layout(|runtime| {
      if let Some(style) = runtime.styles.get(&v.id().node_id()) {
        println!("✅ Background: {:?}", style.background);
      } else {
        println!("❌ No style found!");
      }
    });
    
    // Change color
    println!("\n📍 Changing color to blue...");
    *color_signal.write() = Color::from_rgb8(0, 0, 255);

    // 🔥 Key: flush pending signals to trigger effect
    println!("\n📍 Flushing pending signals...");
    sig::runtime::flush_pending_signals();
    
    println!("\n📍 Computing styles again...");
    let mut style_ctx2 = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&v.id(), &mut style_ctx2);
    
    println!("\n📍 Checking computed style after change...");
    sig::runtime::with_layout(|runtime| {
      if let Some(style) = runtime.styles.get(&v.id().node_id()) {
        println!("✅ Background: {:?}", style.background);
      }
    });
  });
}
