/// 对比测试：验证不同高度策略在嵌套布局中的表现

use sig::prelude::*;

#[test]
fn test_nested_height_strategies() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  对比测试：嵌套布局中的不同高度策略                    ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        // 策略 A: 使用 height(0)
        let testA = view()
            .name("TestA_height0")
            .style(|s| s.width(800.0).height(600.0).flex().flex_col())
            .child((
                view().name("Header").style(|s| s.height(60.0).w_full()),
                view()
                    .name("Middle")
                    .style(|s| s.flex().flex_row().flex_grow(1.0).height(0.0).w_full())  // ⚠️ height(0)
                    .child((
                        view().name("Sidebar").style(|s| s.width(300.0).flex_grow(1.0).height(0.0)),  // ⚠️ height(0)
                        view().name("Detail").style(|s| s.flex_grow(1.0).height(0.0)),
                    )),
            ));
        
        // 策略 B: 使用 min_height(0)
        let testB = view()
            .name("TestB_minheight0")
            .style(|s| s.width(800.0).height(600.0).flex().flex_col())
            .child((
                view().name("Header").style(|s| s.height(60.0).w_full()),
                view()
                    .name("Middle")
                    .style(|s| s.flex().flex_row().flex_grow(1.0).min_height(0.0).w_full())  // ✅ min_height(0)
                    .child((
                        view().name("Sidebar").style(|s| s.width(300.0).flex_grow(1.0).min_height(0.0)),  // ✅ min_height(0)
                        view().name("Detail").style(|s| s.flex_grow(1.0).min_height(0.0)),
                    )),
            ));
        
        // 策略 C: 只用 flex_grow，不设置任何高度
        let testC = view()
            .name("TestC_flexonly")
            .style(|s| s.width(800.0).height(600.0).flex().flex_col())
            .child((
                view().name("Header").style(|s| s.height(60.0).w_full()),
                view()
                    .name("Middle")
                    .style(|s| s.flex().flex_row().flex_grow(1.0).w_full())  // ✅ 不设置高度
                    .child((
                        view().name("Sidebar").style(|s| s.width(300.0).flex_grow(1.0)),  // ✅ 不设置高度
                        view().name("Detail").style(|s| s.flex_grow(1.0)),
                    )),
            ));
        
        // 同步并计算布局
        sig::runtime::flush_pending_signals();
        
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(800.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (800.0, 600.0));
        
        testA.id.compute_layout(available_space, &ctx);
        testB.id.compute_layout(available_space, &ctx);
        testC.id.compute_layout(available_space, &ctx);
        
        // 对比结果
        sig::runtime::with_layout(|runtime| {
            let get_sizes = |test_root: taffy::NodeId, name: &str| {
                let children = runtime.taffy.children(test_root).unwrap();
                let middle = runtime.taffy.layout(children[1]).unwrap();
                let middle_children = runtime.taffy.children(children[1]).unwrap();
                let sidebar = runtime.taffy.layout(middle_children[0]).unwrap();
                let detail = runtime.taffy.layout(middle_children[1]).unwrap();
                
                println!("\n{}", name);
                println!("  Middle: {:.1}x{:.1}", middle.size.width, middle.size.height);
                println!("  Sidebar: {:.1}x{:.1}", sidebar.size.width, sidebar.size.height);
                println!("  Detail: {:.1}x{:.1}", detail.size.width, detail.size.height);
                
                (middle.size.height, sidebar.size.height, detail.size.height)
            };
            
            let (m1, s1, d1) = get_sizes(testA.id.node_id(), "策略 A (height(0))");
            let (m2, s2, d2) = get_sizes(testB.id.node_id(), "策略 B (min_height(0))");
            let (m3, s3, d3) = get_sizes(testC.id.node_id(), "策略 C (无高度设置)");
            
            println!("\n╔══════════════════════════════════════════════════════════╗");
            println!("║                      结论                                ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            
            if s1 == 0.0 {
                println!("❌ height(0) 策略失败: Sidebar 高度是 0");
            } else {
                println!("✅ height(0) 策略成功: Sidebar 高度是 {:.1}px", s1);
            }
            
            if s2 == 540.0 {
                println!("✅ min_height(0) 策略成功: Sidebar 高度是 {:.1}px", s2);
            } else {
                println!("⚠️  min_height(0) 策略: Sidebar 高度是 {:.1}px (期望 540)", s2);
            }
            
            if s3 == 540.0 {
                println!("✅ 无高度设置策略成功: Sidebar 高度是 {:.1}px", s3);
            } else {
                println!("⚠️  无高度设置策略: Sidebar 高度是 {:.1}px (期望 540)", s3);
            }
            
            println!("\n推荐策略: ");
            if s2 == 540.0 && s3 == 540.0 {
                println!("✅ 使用 min_height(0) 或不设置高度都可以");
            } else if s2 == 540.0 {
                println!("✅ 使用 min_height(0)");
            } else if s3 == 540.0 {
                println!("✅ 不设置高度，只用 flex_grow");
            }
        });
        
        println!();
    });
}
