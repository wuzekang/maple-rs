//! Tokio Timer Demo
//!
//! 展示如何在 sig 中使用 tokio::time::sleep 实现定时器
//! 
//! ✅ 现在可以正常工作了！
//!
//! 关键改进：
//! 1. sig-async 现在使用 channel-based waker，支持跨线程唤醒
//! 2. 不再需要手动管理 tokio 运行时
//! 3. tokio::time::sleep 开箱即用

use sig::prelude::*;
use vello::peniko::Color;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), || {
        // 计数器
        let counter = Signal::new(0);
        let status = Signal::new(String::from("就绪"));
        
        let root = view()
            .style(|s| {
                s.width(500.0)
                    .height(400.0)
                    .background(Color::from_rgb8(243, 244, 246))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .gap(20.0)
            })
            .child((
                view()
                    .style(|s| s.flex().flex_col().items_center().gap(10.0))
                    .child((
                        text("✅ Tokio Timer Demo").style(|s| {
                            s.font_size(28.0).color(Color::from_rgb8(17, 24, 39))
                        }),
                        
                        text("(使用优化后的 sig-async)").style(|s| {
                            s.font_size(14.0).color(Color::from_rgb8(107, 114, 128))
                        }),
                        
                        dynamic_text({
                            let counter = counter.clone();
                            move || format!("计数: {}", *counter.read())
                        }).style(|s| {
                            s.font_size(20.0).color(Color::from_rgb8(59, 130, 246))
                        }),
                        
                        dynamic_text({
                            let status = status.clone();
                            move || status.read().clone()
                        }).style(|s| {
                            s.font_size(14.0).color(Color::from_rgb8(107, 114, 128))
                        }),
                    )),
                
                view()
                    .style(|s| s.flex().flex_col().gap(12.0))
                    .child((
                        // 1秒定时器
                        button("1秒定时器 (tokio)").on_click({
                            let counter = counter.clone();
                            let status = status.clone();
                            move |_| {
                                let counter = counter.clone();
                                let status = status.clone();
                                
                                *status.write() = "等待 1 秒...".to_string();
                                
                                // ✅ 现在可以直接使用 tokio::time::sleep！
                                spawn(async move {
                                    eprintln!("⏱️  定时器开始");
                                    
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    
                                    eprintln!("✅ 定时器完成");
                                    *counter.write() += 1;
                                    *status.write() = "完成！".to_string();
                                });
                            }
                        }),
                        
                        // 循环定时器
                        button("循环 5 次 (500ms)").on_click({
                            let counter = counter.clone();
                            let status = status.clone();
                            move |_| {
                                let counter = counter.clone();
                                let status = status.clone();
                                
                                spawn(async move {
                                    for i in 1..=5 {
                                        *status.write() = format!("第 {} 次", i);
                                        
                                        tokio::time::sleep(Duration::from_millis(500)).await;
                                        
                                        *counter.write() += 1;
                                        eprintln!("📊 迭代 {} 完成", i);
                                    }
                                    
                                    *status.write() = "全部完成！".to_string();
                                });
                            }
                        }),
                        
                        // 使用 futures-timer 的示例
                        button("futures-timer (500ms)").on_click({
                            let counter = counter.clone();
                            let status = status.clone();
                            move |_| {
                                let counter = counter.clone();
                                let status = status.clone();
                                
                                *status.write() = "使用 futures-timer...".to_string();
                                
                                spawn(async move {
                                    use futures_timer::Delay;
                                    
                                    Delay::new(Duration::from_millis(500)).await;
                                    
                                    *counter.write() += 1;
                                    *status.write() = "futures-timer 完成！".to_string();
                                });
                            }
                        }),
                        
                        // 重置
                        button("重置").on_click({
                            let counter = counter.clone();
                            let status = status.clone();
                            move |_| {
                                *counter.write() = 0;
                                *status.write() = "已重置".to_string();
                            }
                        }),
                    )),
            ));

        root
    })
}
