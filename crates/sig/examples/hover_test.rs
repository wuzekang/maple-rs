//! Hover Test
use sig::{run, AppConfig, view, text, Signal, Element, Styleable, Interactive};
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let hovered = Signal::new(false);
    
    view()
      .style(|s| s.width(800.0).height(600.0).flex().items_center().justify_center())
      .child(
        view()
          .style({
            let hovered = hovered.clone();
            move |s| {
              let bg = if *hovered.read() {
                println!("🎨 Style effect: hovered = true, using GREEN");
                Color::from_rgb8(0, 255, 0) // green
              } else {
                println!("🎨 Style effect: hovered = false, using RED");
                Color::from_rgb8(255, 0, 0) // red
              };
              s.width(200.0)
                .height(200.0)
                .background(bg)
                .flex()
                .items_center()
                .justify_center()
            }
          })
          .name("HoverBox")
          .on_mouse_enter({
            let hovered = hovered.clone();
            move |_| {
              println!("\n🖱️  MOUSE ENTER!");
              *hovered.write() = true;
            }
          })
          .on_mouse_leave({
            let hovered = hovered.clone();
            move |_| {
              println!("\n🖱️  MOUSE LEAVE!");
              *hovered.write() = false;
            }
          })
          .child(text("Hover me!"))
      )
  })
}
