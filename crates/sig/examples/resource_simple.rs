// use_resource UI demo - shows automatic dependency tracking and manual controls in a GUI
//
// For a comprehensive console-based demo showing all features, run:
//   cargo run --example resource_demo
//
use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    println!("=== use_resource Demo ===\n");
    
    run(AppConfig::default(), || {
        let counter = Signal::new(0);
        
        // Create a resource
        let data_resource = use_resource(move || async move {
            let count = *counter.read();
            println!("🔄 Fetching data for counter = {}", count);
            format!("Data-{}", count)
        });
        
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
                view()
                    .style(|s| s.font_size(26.0).color(Color::from_rgb8(30, 41, 59)))
                    .child(text("use_resource Demo")),
                
                // Counter display
                view()
                    .style(|s| {
                        s.padding(20.0)
                            .background(Color::WHITE)
                            .border_radius(10.0)
                    })
                    .child(
                        dynamic_text(move || format!("Counter: {}", *counter.read()))
                            .style(|s| s.font_size(20.0))
                    ),
                
                // Resource state display
                view()
                    .style(|s| {
                        s.padding(20.0)
                            .background(Color::from_rgb8(240, 253, 244))
                            .border_radius(10.0)
                            .flex()
                            .flex_col()
                            .gap(10.0)
                    })
                    .child((
                        view()
                            .style(|s| s.font_size(14.0).font_weight(600).color(Color::from_rgb8(22, 101, 52)))
                            .child(text("Resource State:")),
                        
                        {
                            let res = data_resource.clone();
                            dynamic_text(move || {
                                match res.state() {
                                    ResourceState::Pending => "⏳ Pending...".to_string(),
                                    ResourceState::Ready => {
                                        match res.value() {
                                            Some(val) => format!("✅ Ready: {}", val),
                                            None => "❌ No data".to_string(),
                                        }
                                    }
                                    ResourceState::Paused => "⏸️ Paused".to_string(),
                                    ResourceState::Stopped => "🛑 Stopped".to_string(),
                                }
                            })
                            .style(|s| s.font_size(16.0).color(Color::from_rgb8(21, 128, 61)))
                        },
                    )),
                
                // Controls
                view().style(|s| s.flex().flex_col().gap(12.0)).child((
                    view().style(|s| s.flex().gap(10.0)).child((
                        primary_button("Increment Counter")
                            .on_click(move |_| {
                                *counter.write() += 1;
                            }),
                        
                        {
                            let res = data_resource.clone();
                            secondary_button("Refetch (Restart)")
                                .on_click(move |_| {
                                    // 手动触发重新获取
                                    res.restart();
                                })
                        },
                    )),
                    
                    view().style(|s| s.flex().gap(10.0)).child((
                        {
                            let res = data_resource.clone();
                            outline_button("Pause")
                                .on_click(move |_| {
                                    res.pause();
                                })
                        },
                        
                        {
                            let res = data_resource.clone();
                            outline_button("Resume")
                                .on_click(move |_| {
                                    res.resume();
                                })
                        },
                        
                        {
                            let res = data_resource.clone();
                            danger_button("Cancel")
                                .on_click(move |_| {
                                    res.cancel();
                                })
                        },
                    )),
                )),
                
                // Info panel
                view()
                    .style(|s| {
                        s.margin_top(20.0)
                            .padding(20.0)
                            .background(Color::from_rgb8(254, 243, 199))
                            .border_radius(10.0)
                    })
                    .child(
                        view().style(|s| s.flex().flex_col().gap(8.0)).child((
                            view()
                                .child(text("💡 How it works:"))
                                .style(|s| s.font_size(14.0).font_weight(600)),
                            view()
                                .child(text("1. Click 'Increment Counter' to change the counter"))
                                .style(|s| s.font_size(13.0)),
                            view()
                                .child(text("2. The resource AUTOMATICALLY restarts when counter changes! ✨"))
                                .style(|s| s.font_size(13.0)),
                            view()
                                .child(text("3. You can also manually 'Refetch' or use Pause/Resume/Cancel"))
                                .style(|s| s.font_size(13.0)),
                            view()
                                .child(text("4. Check console for 'Fetching data for counter = X' messages"))
                                .style(|s| s.font_size(13.0)),
                        ))
                    ),
            ))
    })
}
