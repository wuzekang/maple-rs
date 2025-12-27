use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    view()
      .style(|s| {
        s.width(800.0)
          .height(600.0)
          .padding(40.0)
          .flex()
          .flex_col()
          .gap(20.0)
          .background(Color::from_rgb8(248, 250, 252))
      })
      .child((
        text("AABB Debug Viewer")
          .style(|s| s.font_size(28.0).color(Color::from_rgb8(30, 41, 59))),
        
        text("Hover over any element to see its bounding box (red border)")
          .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139))),

        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .flex()
              .flex_col()
              .gap(16.0)
          })
          .child((
            text("Input Fields")
              .style(|s| s.font_size(18.0).color(Color::from_rgb8(30, 41, 59))),
            
            text_input()
              .placeholder("Enter your name")
              .width(300.0),
            
            text_input()
              .placeholder("Enter your email")
              .width(300.0),
          )),

        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .flex()
              .flex_col()
              .gap(16.0)
          })
          .child((
            text("Buttons")
              .style(|s| s.font_size(18.0).color(Color::from_rgb8(30, 41, 59))),
            
            view()
              .style(|s| s.flex().gap(12.0))
              .child((
                primary_button("Primary")
                  .on_click(|_| println!("Primary clicked!")),
                
                secondary_button("Secondary")
                  .on_click(|_| println!("Secondary clicked!")),
                
                outline_button("Outline")
                  .on_click(|_| println!("Outline clicked!")),
              )),
          )),

        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .flex()
              .flex_col()
              .gap(16.0)
          })
          .child((
            text("Nested Containers")
              .style(|s| s.font_size(18.0).color(Color::from_rgb8(30, 41, 59))),
            
            view()
              .style(|s| {
                s.padding(16.0)
                  .background(Color::from_rgb8(241, 245, 249))
                  .border_radius(8.0)
              })
              .child(
                view()
                  .style(|s| {
                    s.padding(12.0)
                      .background(Color::from_rgb8(219, 234, 254))
                      .border_radius(6.0)
                  })
                  .child(text("Hover over me to see nested AABBs")),
              ),
          )),
      ))
  })
}
