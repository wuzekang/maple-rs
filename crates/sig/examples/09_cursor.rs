use sig::prelude::*;
use sig::Cursor;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    view()
      .style(|s| {
        s.width(900.0)
          .height(700.0)
          .padding(40.0)
          .flex()
          .flex_col()
          .gap(30.0)
          .background(Color::from_rgb8(248, 250, 252)) // slate-50
      })
      .child((
        // Title
        view()
          .style(|s| s.font_size(28.0).color(Color::from_rgb8(15, 23, 42)))
          .child(text("Cursor Inheritance Demo")),

        view()
          .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139)))
          .child(text("Move your mouse over different areas to see cursor changes")),

        // Example 1: Inherit cursor
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .border_all(2.0, Color::from_rgb8(226, 232, 240))
              .flex()
              .flex_col()
              .gap(12.0)
              .cursor(Cursor::Help) // 🎯 Set parent container cursor
          })
          .child((
            text("Section with cursor: Help")
              
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            text("Hover over this text - it inherits Help cursor from parent")
              
              .style(|s| s.color(Color::from_rgb8(71, 85, 105))),
            
            view()
              .style(|s| {
                s.padding(12.0)
                  .background(Color::from_rgb8(241, 245, 249))
                  .border_radius(6.0)
              })
              .child(text("Child element also inherits Help cursor")),
          )),

        // Example 2: Override inherited cursor
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .border_all(2.0, Color::from_rgb8(226, 232, 240))
              .flex()
              .flex_col()
              .gap(12.0)
              .cursor(Cursor::Move) // 🎯 Parent container sets Move cursor
          })
          .child((
            text("Section with cursor: Move")
              
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            text("This inherits Move cursor")

              .style(|s| s.color(Color::from_rgb8(71, 85, 105))),

            // Child element overrides cursor
            view()
              .style(|s| {
                s.padding(12.0)
                  .background(Color::from_rgb8(239, 246, 255))
                  .border_radius(6.0)
                  .cursor(Cursor::Text) // 🎯 Child element overrides to Text cursor
              })
              .child(text("This child overrides with Text cursor")),
          )),

        // Example 3: Button automatically sets Pointer
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .border_all(2.0, Color::from_rgb8(226, 232, 240))
              .flex()
              .flex_col()
              .gap(12.0)
              .cursor(Cursor::NotAllowed) // 🎯 Parent container sets NotAllowed
          })
          .child((
            text("Section with cursor: NotAllowed")
              
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            text("Hover over this area shows NotAllowed cursor")

              .style(|s| s.color(Color::from_rgb8(71, 85, 105))),

            // Button overrides to Pointer
            primary_button("Button overrides to Pointer")
              .on_click(|_| println!("Clicked!")),
          )),

        // Example 4: Different cursor type display
        view()
          .style(|s| {
            s.padding(20.0)
              .background(Color::WHITE)
              .border_radius(12.0)
              .border_all(2.0, Color::from_rgb8(226, 232, 240))
              .flex()
              .flex_col()
              .gap(12.0)
          })
          .child((
            text("Different Cursor Types")
              
              .style(|s| s.font_size(16.0).color(Color::from_rgb8(30, 41, 59))),
            
            view()
              .style(|s| s.flex().flex_wrap().gap(12.0))
              .child(cursor_box("Default", Cursor::Default))
              .child(cursor_box("Pointer", Cursor::Pointer))
              .child(cursor_box("Text", Cursor::Text))
              .child(cursor_box("Move", Cursor::Move))
              .child(cursor_box("Grab", Cursor::Grab))
              .child(cursor_box("Grabbing", Cursor::Grabbing))
              .child(cursor_box("Wait", Cursor::Wait))
              .child(cursor_box("Help", Cursor::Help))
              .child(cursor_box("NotAllowed", Cursor::NotAllowed))
              .child(cursor_box("Crosshair", Cursor::Crosshair))
              .child(cursor_box("Resize ↔", Cursor::ResizeHorizontal))
              .child(cursor_box("Resize ↕", Cursor::ResizeVertical)),
          )),
      ))
  })
}

// Helper function: Create a box to display a specific cursor
fn cursor_box(label: &str, cursor: Cursor) -> View {
  view()
    .name(&format!("CursorBox_{}", label))
    .style(move |s| {
      s.padding(16.0)
        .background(Color::from_rgb8(241, 245, 249))
        .border_radius(8.0)
        .border_all(1.0, Color::from_rgb8(203, 213, 225))
        .cursor(cursor)
        .min_width(120.0)
        .flex()
        .justify_center()
        .items_center()
    })
    .child(text(label))
}
