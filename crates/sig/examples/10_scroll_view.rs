use sig::dynamic;
use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    view()
      .style(|s| {
        s.size_full()
          .flex()
          .flex_col()
          .background(Color::from_rgb8(248, 250, 252)) // slate-50
      })
      .child((
        // Top header bar
        view()
          .style(|s| {
            s.height(80.0)
              .background(Color::from_rgb8(30, 41, 59)) // slate-800
              .padding(24.0)
              .flex()
              .w_full()
              .items_center()
          })
          .child(
            text("Scroll View Demo (with overflow style)")
              .style(|s| s.font_size(28.0).color(Color::WHITE)),
          ),
        // Main content area - scrollable
        view()
          .style(|s| {
            s.flex_grow(1.0)
              .padding(32.0)
              .flex_col()
              .gap(8.0)
              .w_full()
              .h_full()
              .overflow_y_scroll()
          })
          .child({
            let mut sections = Vec::new();
            sections.push(example_section(
              "1. Vertical Scroll List",
              "A scrollable list with multiple items (use overflow_y_scroll)",
              vertical_scroll_example(),
            ));
            sections.push(spacer(32.0));
            sections.push(example_section(
              "2. Horizontal Scroll Gallery",
              "Scroll horizontally through colored boxes (use overflow_x_scroll)",
              horizontal_scroll_example(),
            ));
            sections.push(spacer(32.0));
            sections.push(example_section(
              "3. Both Directions Scroll",
              "Scroll in both vertical and horizontal directions (use overflow_scroll)",
              both_directions_example(),
            ));
            sections.push(spacer(32.0));
            sections.push(example_section(
              "4. Nested Scroll (🔥 New!)",
              "Inner scroll reaches boundary → continues to outer scroll",
              nested_scroll_example(),
            ));
            sections.push(spacer(32.0));
            sections.push(example_section(
              "5. Dynamic Content List",
              "Add or remove items dynamically",
              dynamic_content_example(),
            ));
            sections.push(spacer(40.0));
            sections
          }),
      ))
  })
}

/// Create an example section
fn example_section(
  title: &'static str,
  description: &'static str,
  content: impl ViewTuple,
) -> View {
  view()
    .style(|s| {
      s.flex()
        .flex_col()
        .w_full()
        .gap(16.0)
        .padding(24.0)
        .background(Color::WHITE)
        .border_radius(12.0)
        .border_all(1.0, Color::from_rgb8(226, 232, 240)) // slate-200
    })
    .child((
      // Title
      text(title).style(|s| {
        s.font_size(20.0).color(Color::from_rgb8(15, 23, 42)) // slate-900
      }),
      // Description
      text(description).style(|s| {
        s.font_size(14.0).color(Color::from_rgb8(100, 116, 139)) // slate-500
      }),
      // Content
      content,
    ))
}

/// Example 1: Vertical scroll list
fn vertical_scroll_example() -> impl ViewTuple {
  // Create 20 list items
  let items: Vec<_> = (1..=20)
    .map(|i| {
      view()
        .style(move |s| {
          let bg = if i % 2 == 0 {
            Color::from_rgb8(248, 250, 252) // slate-50
          } else {
            Color::WHITE
          };

          s.w_full()
            .height(60.0)
            .padding(16.0)
            .background(bg)
            .flex()
            .items_center()
            .border_all(1.0, Color::from_rgb8(226, 232, 240))
        })
        .child(text(format!("List Item #{}", i)).style(|s| {
          s.font_size(16.0).color(Color::from_rgb8(51, 65, 85)) // slate-700
        }))
    })
    .collect();

  view()
    .style(|s| {
      s.width(taffy::Dimension::Percent(1.0))
        .height(300.0)
        .flex_col() // Set flex_col directly, content_size will automatically calculate total height of all child elements
        .overflow_y_scroll()
        .border_radius(8.0)
        .border_all(2.0, Color::from_rgb8(148, 163, 184)) // slate-400
    })
    .child(items)
}

/// Example 2: Horizontal scroll gallery
fn horizontal_scroll_example() -> impl ViewTuple {
  let colors = vec![
    (239, 68, 68),  // red-500
    (249, 115, 22), // orange-500
    (234, 179, 8),  // yellow-500
    (34, 197, 94),  // green-500
    (59, 130, 246), // blue-500
    (168, 85, 247), // purple-500
    (236, 72, 153), // pink-500
    (99, 102, 241), // indigo-500
  ];

  let boxes: Vec<_> = colors
    .into_iter()
    .enumerate()
    .map(|(i, (r, g, b))| {
      view()
        .style(move |s| {
          s.width(180.0)
            .height(200.0)
            .background(Color::from_rgb8(r, g, b))
            .border_radius(12.0)
            .flex()
            .items_center()
            .justify_center()
        })
        .child(text(format!("Box {}", i + 1)).style(|s| s.font_size(20.0).color(Color::WHITE)))
    })
    .collect();

  view()
    .style(|s| {
      s.width(700.0)
        .height(240.0)
        .overflow_x_scroll()
        .border_radius(8.0)
        .border_all(2.0, Color::from_rgb8(148, 163, 184))
        .padding(8.0)
    })
    .child(view().style(|s| s.flex().gap(16.0)).child(boxes))
}

/// Example 3: Bidirectional scrolling
fn both_directions_example() -> impl ViewTuple {
  let content = view()
    .style(|s| {
      s.padding(40.0).flex().flex_col().gap(20.0)
      // 🔑 Don't set fixed width/height, let content expand naturally
    })
    .child((
      text("Large Content Area").style(|s| {
        s.font_size(24.0).color(Color::from_rgb8(7, 89, 133)) // sky-900
      }),
      text("This content is larger than the viewport.").style(|s| {
        s.font_size(16.0).color(Color::from_rgb8(12, 74, 110)) // sky-800
      }),
      text("You can scroll both horizontally and vertically.")
        .style(|s| s.font_size(16.0).color(Color::from_rgb8(12, 74, 110))),
      // Add some grid content
      view()
        .style(|s| {
          s.flex()
            .flex_wrap()
            .width(1200.0)
            .gap(16.0)
            .margin_top(20.0)
        })
        .child({
          let cells: Vec<_> = (1..=50)
            .map(|i| {
              view()
                .style(|s| {
                  s.width(150.0)
                    .height(100.0)
                    .background(Color::from_rgb8(125, 211, 252)) // sky-300
                    .border_radius(8.0)
                    .flex()
                    .items_center()
                    .justify_center()
                })
                .child(
                  text(format!("Cell {}", i))
                    .style(|s| s.font_size(14.0).color(Color::from_rgb8(7, 89, 133))),
                )
            })
            .collect();
          cells
        }),
    ));

  view()
    .style(|s| {
      s.width(700.0)
        .height(400.0)
        .overflow_scroll()
        .border_radius(8.0)
        .border_all(2.0, Color::from_rgb8(148, 163, 184))
        .background(Color::from_rgb8(224, 242, 254))
    })
    .child(content)
}

/// Example 4: Dynamic content list
fn dynamic_content_example() -> impl ViewTuple {
  let count = Signal::new(5);

  view().style(|s| s.flex().flex_col().gap(16.0)).child((
    // Control buttons
    view().style(|s| s.flex().gap(12.0)).child((
      primary_button("Add Item").on_click(move |_| {
        *count.write() += 1;
      }),
      danger_button("Remove Item").on_click(move |_| {
        let current = *count.read();
        if current > 0 {
          *count.write() = current - 1;
        }
      }),
      dynamic_text(move || format!("Items: {}", *count.read())).style(|s| {
        s.font_size(14.0)
          .color(Color::from_rgb8(100, 116, 139))
          .margin_left(12.0)
      }),
    )),
    // Dynamic list
    view()
      .style(|s| {
        s.width(taffy::Dimension::Percent(1.0))
          .height(300.0)
          .flex_col() // content_size will automatically calculate total height of all dynamic child elements
          .overflow_y_scroll()
          .border_radius(8.0)
          .border_all(2.0, Color::from_rgb8(148, 163, 184))
          .background(Color::WHITE)
          .padding(8.0)
      })
      .child(dynamic(move || {
        let items: Vec<_> = (1..=*count.read())
          .map(|i| {
            view()
              .style(move |s| {
                s.width(taffy::Dimension::Percent(1.0))
                  .height(50.0)
                  .padding(12.0)
                  .margin_bottom(8.0)
                  .background(Color::from_rgb8(241, 245, 249)) // slate-100
                  .border_radius(6.0)
                  .flex()
                  .items_center()
              })
              .child(
                text(format!("Dynamic Item #{}", i))
                  .style(|s| s.font_size(14.0).color(Color::from_rgb8(51, 65, 85))),
              )
          })
          .collect();
        fragment(items)
      })),
  ))
}

/// Example 4: Nested scrolling (3 layers)
fn nested_scroll_example() -> impl ViewTuple {
  // Outermost container (800px)
  view()
    .style(|s| {
      s.width(taffy::Dimension::Percent(1.0))
        .height(800.0)
        .background(Color::from_rgb8(254, 242, 242)) // red-50
        .border_radius(8.0)
        .border_all(3.0, Color::from_rgb8(239, 68, 68)) // red-500
        .padding(16.0)
        .flex_col()
        .gap(12.0)
        .overflow_y_scroll()
    })
    .child((
      // Outermost description
      view().style(|s| s.flex_col().gap(4.0).w_full()).child((
        text("🔴 Layer 1 (Outer) - Height: 800px").style(|s| {
          s.font_size(16.0)
            .font_weight(700)
            .color(Color::from_rgb8(153, 27, 27))
        }), // red-900
        text("Start scrolling from the innermost blue box →")
          .style(|s| s.font_size(12.0).color(Color::from_rgb8(127, 29, 29))),
      )),
      // Middle layer container (600px)
      view()
        .style(|s| {
          s.width(taffy::Dimension::Percent(1.0))
            .height(600.0)
            .background(Color::from_rgb8(254, 249, 195)) // yellow-100
            .border_radius(8.0)
            .border_all(3.0, Color::from_rgb8(234, 179, 8)) // yellow-500
            .padding(16.0)
            .flex_col()
            .gap(12.0)
            .overflow_y_scroll()
        })
        .child((
          // Middle layer description
          view().style(|s| s.flex_col().gap(4.0).w_full()).child((
            text("🟡 Layer 2 (Middle) - Height: 600px").style(|s| {
              s.font_size(15.0)
                .font_weight(700)
                .color(Color::from_rgb8(113, 63, 18))
            }), // yellow-900
            text("When blue box reaches bottom, this layer scrolls")
              .style(|s| s.font_size(12.0).color(Color::from_rgb8(120, 53, 15))),
          )),
          // Innermost container (400px)
          view()
            .style(|s| {
              s.width(taffy::Dimension::Percent(1.0))
                .height(400.0)
                .background(Color::from_rgb8(239, 246, 255)) // blue-50
                .border_radius(8.0)
                .border_all(3.0, Color::from_rgb8(59, 130, 246)) // blue-500
                .padding(12.0)
                .flex_col()
                .gap(8.0)
                .overflow_y_scroll()
            })
            .child((
              // Innermost description
              text("🔵 Layer 3 (Inner) - Height: 400px - Scroll here first!").style(|s| {
                s.font_size(14.0)
                  .font_weight(700)
                  .color(Color::from_rgb8(30, 64, 175))
              }), // blue-800
              // Innermost content
              view().style(|s| s.flex_col().w_full()).child({
                let items: Vec<_> = (1..=20)
                  .map(|i| {
                    view()
                      .style(move |s| {
                        s.w_full()
                          .height(50.0)
                          .padding(12.0)
                          .margin_bottom(4.0)
                          .background(if i % 2 == 0 {
                            Color::from_rgb8(219, 234, 254) // blue-100
                          } else {
                            Color::WHITE
                          })
                          .border_radius(4.0)
                          .flex()
                          .items_center()
                      })
                      .child(
                        text(format!("🔵 Inner Item #{}", i))
                          .style(|s| s.font_size(13.0).color(Color::from_rgb8(30, 64, 175))),
                      )
                  })
                  .collect();
                items
              }),
            )),
        })),
      // Outermost content
      view().style(|s| s.flex_col().gap(8.0).w_full()).child({
        let items: Vec<_> = (1..=10)
          .map(|i| {
            view()
              .style(move |s| {
                s.w_full()
                  .height(50.0)
                  .padding(12.0)
                  .background(Color::from_rgb8(254, 202, 202)) // red-200
                  .border_radius(4.0)
                  .flex()
                  .items_center()
                  .border_all(1.0, Color::from_rgb8(239, 68, 68))
              })
              .child(
                text(format!("🔴 Outer Item #{}", i))
                  .style(|s| s.font_size(13.0).color(Color::from_rgb8(153, 27, 27))),
              )
          })
          .collect();
        items
      }),
    ))
}

/// Create a vertical spacer
fn spacer(height: f32) -> View {
  view().style(move |s| s.height(height))
}
