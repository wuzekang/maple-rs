use sig::prelude::*;
use sig::render::{AppConfig, run};
use sig::text::{dynamic_text, text};

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let count = Signal::new(0);

    view()
      .style(|s| {
        s.width(500.0)
          .height(400.0)
          .padding(20.0)
          .flex()
          .flex_col()
          .gap(10.0)
      })
      .child(text("Static Text Example"))
      .child(view().style(|s| s.width(460.0).height(2.0)))
      .child(dynamic_text(move || {
        format!("Dynamic Count: {}", *count.read())
      }))
      .child(
        view()
          .style(|s| s.width(200.0).height(40.0).justify_center().items_center())
          .on_click(move |_| {
            let c = *count.read();
            *count.write() = c + 1;
            println!("Clicked! Count: {}", c + 1);
          })
          .child(text("Click Me")),
      )
  })
}
