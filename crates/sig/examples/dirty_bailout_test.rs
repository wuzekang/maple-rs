//! Test Dirty Bubbling bailout effect
use sig::{create_scope, view, Signal, Element, Styleable, Interactive};
use vello::peniko::Color;

fn main() {
  create_scope(|| {
    let toggle = Signal::new(false);
    
    let root = view()
      .style(|s| s.width(800.0).height(600.0).flex().flex_row())
      .name("Root")
      .child((
        // Left side: reactive area
        view()
          .style({
            let toggle = toggle.clone();
            move |s| {
              let bg = if *toggle.read() {
                Color::from_rgb8(255, 0, 0)
              } else {
                Color::from_rgb8(0, 255, 0)
              };
              s.width(400.0).height(600.0).background(bg)
            }
          })
          .name("LeftPanel")
          .on_mouse_down({
            let toggle = toggle.clone();
            move |_| {
              println!("\n🖱️  Click! Toggling...");
              *toggle.write() = !*toggle.read();
            }
          }),
        
        // Right side: static area (should be bailout)
        view()
          .style(|s| s.width(400.0).height(600.0).background(Color::from_rgb8(200, 200, 200)))
          .name("RightPanel")
          .child((
            view().style(|s| s.width(100.0).height(100.0).background(Color::from_rgb8(100, 100, 100))).name("RightChild1"),
            view().style(|s| s.width(100.0).height(100.0).background(Color::from_rgb8(150, 150, 150))).name("RightChild2"),
          )),
      ));
    
    println!("\n=== Initial render ===");
    let mut ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&root.id(), &mut ctx);
    
    println!("\n=== Simulating click ===");
    *toggle.write() = true;
    sig::runtime::flush_pending_signals();
    
    println!("\n=== Second render (should bailout RightPanel) ===");
    let mut ctx2 = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&root.id(), &mut ctx2);
    
    println!("\n✅ Expected: RightPanel and its children should bailout in second render");
  });
}
