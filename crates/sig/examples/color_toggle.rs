//! Verify flush_pending_signals fix
use sig::{create_scope, view, Signal, Element, Styleable, Interactive, run, AppConfig};
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let is_red = Signal::new(true);
    
    view()
      .style(|s| s.width(800.0).height(600.0).flex().items_center().justify_center())
      .child(
        view()
          .style({
            let is_red = is_red.clone();
            move |s| {
              let bg = if *is_red.read() {
                Color::from_rgb8(255, 0, 0)
              } else {
                Color::from_rgb8(0, 0, 255)
              };
              s.width(200.0).height(200.0).background(bg).cursor(sig::Cursor::Pointer)
            }
          })
          .on_mouse_down(move |_| {
            println!("Clicked! Toggling color...");
            let current = *is_red.read();
            *is_red.write() = !current;
          })
      )
  })
}
