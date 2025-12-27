//! Rendering example - Using Vello to render sig views
//!
//! Shows how to use the render module to render sig views to a window

use sig::{
  Interactive, Signal, Styleable,
  app::{AppConfig, run},
  dynamic, fragment, text, view,
};
use taffy::FlexDirection;

#[derive(Clone, Debug, PartialEq)]
struct TodoItem {
  id: usize,
  text: String,
  completed: bool,
}

fn main() -> anyhow::Result<()> {
  println!("╔════════════════════════════════════════════════════════════╗");
  println!("║      05 - Rendering Example (Vello Rendering + Dynamic Logic) ║");
  println!("╚════════════════════════════════════════════════════════════╝\n");

  println!("Starting render window...");
  use std::io::Write;
  let _ = std::io::stdout().flush();

  let config = AppConfig {
    title: "Sig Todo App".to_string(),
    width: 1044.0,
    height: 800.0,
  };

  run(config, || {
    // ==================== Application State ====================
    let app_title = Signal::new("Todo List".to_string());
    let show_completed = Signal::new(false);
    let todos = Signal::new(vec![
      TodoItem {
        id: 1,
        text: "Learn sig framework".to_string(),
        completed: false,
      },
      TodoItem {
        id: 2,
        text: "Write example code".to_string(),
        completed: true,
      },
      TodoItem {
        id: 3,
        text: "Test features".to_string(),
        completed: false,
      },
      TodoItem {
        id: 4,
        text: "Integrate dynamic logic".to_string(),
        completed: false,
      },
    ]);
    let filter = Signal::new("".to_string());

    // Auxiliary views
    // let detail_view = view().style(|s| s.width(200.0).height(50.0).margin(5.0));
    // let counter_view = view().style(|s| s.width(20.0).height(20.0).margin(5.0));

    view()
      .name("TodoApp")
      .style(|s| {
        s.width(800.0)
          .height(800.0)
          .flex()
          .flex_direction(FlexDirection::Column)
          .padding(20.0)
      })
      .child((
        // Title
        view()
          .name("Header")
          .style(|s| s.height(60.0).flex().align_items(taffy::AlignItems::Center))
          .child((
            text((*app_title.read()).clone()),
            // dynamic(move || text(format!(" Total tasks: {}", (*todos.read()).len()))),
          )),
        // Control panel
        view()
          .name("Controls")
          .style(|s| {
            s.height(80.0)
              .flex()
              .flex_direction(FlexDirection::Row)
              .align_items(taffy::AlignItems::Center)
          })
          .child((
            view()
              .style(|s| s.width(100.0).height(40.0).margin(10.0))
              .child((text("Add Task"),))
              .on_click(move |_| {
                println!("Added new todo");
                let mut current = (*todos.read()).clone();
                let id = current.len() + 1;
                current.push(TodoItem {
                  id,
                  text: format!("New Task {}", id),
                  completed: false,
                });
                *todos.write() = current;
              }),
            view()
              .style(|s| s.width(150.0).height(40.0).margin(10.0))
              .child((dynamic(move || {
                if *show_completed.read() {
                  text("Hide Completed")
                } else {
                  text("Show Completed")
                }
              }),))
              .on_click(move |_| {
                let current = *show_completed.read();
                *show_completed.write() = !current;
                println!("Toggled show_completed: {}", !current);
              }),
            // Filter buttons could go here
          )),
        // Task list
        view()
          .name("TodoList")
          .style(|s| {
            s.flex_grow(1.0)
              .flex()
              .flex_direction(FlexDirection::Column)
          })
          .child((
            text("Task List:"),
            dynamic(move || {
              let filter_text = (*filter.read()).clone();
              let show_complete = *show_completed.read();
              let todo_list = (*todos.read()).clone();

              // Filter logic
              let filtered: Vec<_> = todo_list
                .into_iter()
                .filter(|item| {
                  let matches_filter = filter_text.is_empty()
                    || item
                      .text
                      .to_lowercase()
                      .contains(&filter_text.to_lowercase());
                  let matches_status = show_complete || !item.completed;
                  matches_filter && matches_status
                })
                .collect();

              if filtered.is_empty() {
                fragment((view()
                  .style(|s| s.height(50.0).padding(10.0))
                  .child((text("No matching tasks"),)),))
              } else {
                fragment(
                  filtered
                    .into_iter()
                    .map(|item| {
                      let id = item.id;
                      let completed = item.completed;
                      let setter = todos.clone();

                      view()
                        .style(|s| {
                          s.width(700.0)
                            .height(50.0)
                            .margin(5.0)
                            .flex()
                            .flex_direction(FlexDirection::Row)
                            .align_items(taffy::AlignItems::Center)
                        })
                        .on_click(move |_| {
                          println!("Clicked item {}", id);
                          // Toggle completion status
                          let mut current_todos = (*setter.read()).clone();
                          if let Some(todo) = current_todos.iter_mut().find(|t| t.id == id) {
                            todo.completed = !todo.completed;
                          }
                          *setter.write() = current_todos;
                        })
                        .child((
                          view()
                            .style(|s| s.width(20.0).height(20.0).margin(10.0))
                            .child((text(if completed { "☑" } else { "☐" }),)),
                          text(format!("{} - {}", id, item.text)),
                          if completed {
                            fragment((view().style(|s| s.width(20.0).height(20.0).margin(5.0)),))
                          } else {
                            fragment(())
                          },
                        ))
                        .into_node()
                    })
                    .collect::<Vec<_>>(),
                )
              }
            }),
          )),
        // Statistics
        /*
        view()
            .name("Stats")
            .style(|s| s.height(60.0).padding(10.0))
            .child((
                dynamic(move || {
                     let list = (*todos.read()).clone();
                     let total = list.len();
                     let done = list.iter().filter(|t| t.completed).count();
                     text(format!("Total: {} | Completed: {} | Pending: {}", total, done, total - done))
                }),
            )),
            */
      ))
  })
}
