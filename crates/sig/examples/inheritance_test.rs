use sig::prelude::*;
use vello::peniko::Color;

/// Test: Property inheritance
/// 
/// Expected behavior:
/// - Grandparent sets color to RED
/// - Parent does NOT set color (should inherit RED)
/// - Child does NOT set color (should inherit RED)
/// - All three should have RED color
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
        text("Property Inheritance Test")
          .style(|s| s.font_size(28.0).color(Color::from_rgb8(30, 41, 59))),
        
        text("Expected: All three boxes should have RED text (inherited from grandparent)")
          .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139))),

        // Grandparent: Sets RED color
        view()
          .name("Grandparent")
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .color(Color::from_rgb8(255, 0, 0)) // RED
          })
          .child((
            text("Grandparent (RED color set here)")
              .style(|s| s.font_size(16.0)),

            // Parent: Does NOT set color (should inherit RED)
            view()
              .name("Parent")
              .style(|s| {
                s.padding(16.0)
                  .margin_top(12.0)
                  .background(Color::from_rgb8(241, 245, 249))
                  .border_radius(6.0)
                  // NO color set - should inherit RED from grandparent
              })
              .child((
                text("Parent (should inherit RED)")
                  .style(|s| s.font_size(14.0)),

                // Child: Does NOT set color (should inherit RED)
                view()
                  .name("Child")
                  .style(|s| {
                    s.padding(12.0)
                      .margin_top(8.0)
                      .background(Color::from_rgb8(219, 234, 254))
                      .border_radius(4.0)
                      // NO color set - should inherit RED from grandparent
                  })
                  .child(
                    text("Child (should inherit RED)")
                      .style(|s| s.font_size(12.0))
                  ),
              )),
          )),

        text("Test 2: Override in middle")
          .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139)).margin_top(20.0)),

        // Grandparent: Sets RED color
        view()
          .name("Grandparent2")
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .color(Color::from_rgb8(255, 0, 0)) // RED
          })
          .child((
            text("Grandparent (RED)")
              .style(|s| s.font_size(16.0)),

            // Parent: Overrides to BLUE
            view()
              .name("Parent2")
              .style(|s| {
                s.padding(16.0)
                  .margin_top(12.0)
                  .background(Color::from_rgb8(241, 245, 249))
                  .border_radius(6.0)
                  .color(Color::from_rgb8(0, 0, 255)) // BLUE - override
              })
              .child((
                text("Parent (BLUE - overridden)")
                  .style(|s| s.font_size(14.0)),

                // Child: Does NOT set color (should inherit BLUE from parent)
                view()
                  .name("Child2")
                  .style(|s| {
                    s.padding(12.0)
                      .margin_top(8.0)
                      .background(Color::from_rgb8(219, 234, 254))
                      .border_radius(4.0)
                      // NO color set - should inherit BLUE from parent
                  })
                  .child(
                    text("Child (should inherit BLUE from parent)")
                      .style(|s| s.font_size(12.0))
                  ),
              )),
          )),
      ))
  })
}
