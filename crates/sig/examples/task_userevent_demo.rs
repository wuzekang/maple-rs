// UserEvent-based task system demo
// Following Dioxus's design: tasks wake via UserEvent, not RedrawRequested
use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    println!("=== UserEvent-Based Task System ===\n");
    println!("✅ Tasks are polled via UserEvent::PollTasks");
    println!("✅ Completely decoupled from rendering");
    println!("✅ Works even when window is minimized\n");
    
    run(AppConfig::default(), || {
        let counter = Signal::new(0);
        let status = Signal::new("Ready".to_string());
        
        view()
            .style(|s| {
                s.width(700.0)
                    .height(500.0)
                    .padding(40.0)
                    .flex()
                    .flex_col()
                    .gap(20.0)
                    .background(Color::from_rgb8(248, 250, 252))
            })
            .child((
                // Title
                view()
                    .style(|s| s.font_size(26.0).color(Color::from_rgb8(30, 41, 59)))
                    .child(text("UserEvent Task System")),
                
                view()
                    .style(|s| s.font_size(14.0).color(Color::from_rgb8(100, 116, 139)))
                    .child(text("Following Dioxus's design pattern")),
                
                // Counter display
                view()
                    .style(|s| {
                        s.padding(24.0)
                            .background(Color::WHITE)
                            .border_radius(12.0)
                            .border_all(2.0, Color::from_rgb8(203, 213, 225))
                    })
                    .child((
                        view()
                            .style(|s| s.font_size(16.0).color(Color::from_rgb8(100, 116, 139)))
                            .child(text("Counter Value:")),
                        
                        dynamic_text(move || format!("{}", *counter.read()))
                            .style(|s| {
                                s.font_size(48.0)
                                    .font_weight(700)
                                    .color(Color::from_rgb8(59, 130, 246))
                            }),
                    )),
                
                // Status
                view()
                    .style(|s| {
                        s.padding(16.0)
                            .background(Color::from_rgb8(240, 253, 244))
                            .border_radius(8.0)
                            .border_all(1.0, Color::from_rgb8(134, 239, 172))
                    })
                    .child(
                        dynamic_text(move || format!("Status: {}", (*status.read()).clone()))
                            .style(|s| s.font_size(14.0).color(Color::from_rgb8(22, 163, 74))),
                    ),
                
                // Buttons
                view().style(|s| s.flex().flex_col().gap(12.0)).child((
                    primary_button("Spawn Task (+1)")
                        .on_click(move |_| {
                            // ✅ Task wakes via UserEvent - immediate response!
                            spawn(async move {
                                let val = *counter.read();
                                *counter.write() = val + 1;
                                *status.write() = format!("Task completed! Counter = {}", val + 1);
                            });
                        }),
                    
                    secondary_button("Spawn 10 Tasks")
                        .on_click(move |_| {
                            for _ in 0..10 {
                                spawn(async move {
                                    let val = *counter.read();
                                    *counter.write() = val + 1;
                                });
                            }
                            *status.write() = "Spawned 10 tasks!".to_string();
                        }),
                    
                    outline_button("Test Pause/Resume")
                        .on_click(move |_| {
                            let task = spawn(async move {
                                *status.write() = "Pause/Resume test completed!".to_string();
                            });
                            
                            task.pause();
                            println!("Task paused: {}", task.paused());
                            
                            task.resume();
                            println!("Task resumed!");
                        }),
                    
                    danger_button("Reset")
                        .on_click(move |_| {
                            *counter.write() = 0;
                            *status.write() = "Reset!".to_string();
                        }),
                )),
                
                // Info panel
                view()
                    .style(|s| {
                        s.margin_top(20.0)
                            .padding(20.0)
                            .background(Color::from_rgb8(239, 246, 255))
                            .border_radius(10.0)
                            .border_all(1.0, Color::from_rgb8(191, 219, 254))
                    })
                    .child(
                        view().style(|s| s.flex().flex_col().gap(10.0)).child((
                            view()
                                .child(text("🎯 Key Features:"))
                                .style(|s| s.font_size(15.0).font_weight(600).color(Color::from_rgb8(29, 78, 216))),
                            view()
                                .child(text("✓ Tasks wake via UserEvent (like Dioxus)"))
                                .style(|s| s.font_size(13.0).color(Color::from_rgb8(30, 64, 175))),
                            view()
                                .child(text("✓ Immediate response - no waiting for redraw"))
                                .style(|s| s.font_size(13.0).color(Color::from_rgb8(30, 64, 175))),
                            view()
                                .child(text("✓ Works even when window is minimized"))
                                .style(|s| s.font_size(13.0).color(Color::from_rgb8(30, 64, 175))),
                            view()
                                .child(text("✓ Proper separation of concerns"))
                                .style(|s| s.font_size(13.0).color(Color::from_rgb8(30, 64, 175))),
                        ))
                    ),
            ))
    })
}
