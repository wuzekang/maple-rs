use sig::prelude::*;
use sig::Cursor;
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
        text("Cursor Debug Test")
          .style(|s| s.font_size(28.0).color(Color::from_rgb8(30, 41, 59))),
        
        text("Hover and observe console output for cursor values")
          .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139))),

        // Test 1: View with explicit cursor
        view()
          .name("ExplicitPointerView")
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .cursor(Cursor::Pointer) // Explicitly set Pointer
          })
          .child(
            text("View with explicit Cursor::Pointer")
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59)))
          ),

        // Test 2: Button (should have Pointer)
        view()
          .name("ButtonTestCard")  // 🔍 Add name for debugging
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .margin_top(12.0)
          })
          .child((
            text("Button Test")
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            view()
              .name("ButtonContainer")  // 🔍 Add name for debugging
              .style(|s| s.margin_top(12.0))
              .child(
                primary_button("Click Me (should be Pointer)")
                  .on_click(|_| println!("Button clicked!"))
              ),
          )),

        // Test 3: Input (should have Text)
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .margin_top(12.0)
          })
          .child((
            text("Input Test")
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            view()
              .style(|s| s.margin_top(12.0))
              .child(
                text_input()
                  .placeholder("Should have Text cursor")
                  .width(300.0)
              ),
          )),

        // Test 4: Inheritance chain
        view()
          .name("InheritParent")
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .margin_top(12.0)
              .cursor(Cursor::Help) // Parent sets Help
          })
          .child((
            text("Parent with Cursor::Help")
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            view()
              .name("InheritChild")
              .style(|s| {
                s.padding(12.0)
                  .margin_top(8.0)
                  .background(Color::from_rgb8(241, 245, 249))
                  .border_radius(6.0)
                  // No cursor set - should inherit Help
              })
              .child(
                text("Child (should inherit Help)")
                  .style(|s| s.font_size(14.0))
              ),
          )),

        // Test 5: Default cursor (no explicit setting)
        view()
          .name("DefaultCursorView")
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(8.0)
              .margin_top(12.0)
              // No cursor set
          })
          .child(
            text("View with no cursor set (should be Default)")
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59)))
          ),
      ))
  })
}
