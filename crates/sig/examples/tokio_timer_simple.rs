//! 简化版 Tokio Timer Demo
//!
//! 测试 sig 与 tokio::time::sleep 的集成

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), || {
        let counter = Signal::new(0);
        
        view()
            .style(|s| {
                s.width(400.0)
                    .height(300.0)
                    .background(Color::WHITE)
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .gap(20.0)
            })
            .child((
                dynamic_text({
                    let counter = counter.clone();
                    move || format!("计数: {}", *counter.read())
                }).style(|s| s.font_size(24.0)),
                
                button("点击 (+1 延迟 1秒)").on_click({
                    let counter = counter.clone();
                    move |_| {
                        let counter = counter.clone();
                        spawn(async move {
                            eprintln!("任务开始");
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            eprintln!("Sleep 完成，更新计数器");
                            *counter.write() += 1;
                            eprintln!("计数器已更新到: {}", *counter.read());
                        });
                    }
                }),
            ))
    })
}
