/// 测试不同内容高度场景下的布局表现

use sig::prelude::*;

#[test]
fn test_content_less_than_container() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  场景 1: 内容高度 < 容器高度                           ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        let root = view()
            .name("Root")
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().name("Header").style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                view()
                    .name("ContentContainer")
                    .style(|s| s.flex_grow(1.0).min_height(0.0).w_full().overflow_y_scroll())
                    .child(
                        view().name("SmallContent").style(|s| s.height(200.0).w_full()),
                    ),
            ));

        sig::runtime::flush_pending_signals();
        
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(400.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (400.0, 600.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let children = runtime.taffy.children(root.id.node_id()).unwrap();
            let container_layout = runtime.taffy.layout(children[1]).unwrap();
            let container_children = runtime.taffy.children(children[1]).unwrap();
            let content_layout = runtime.taffy.layout(container_children[0]).unwrap();
            
            println!("容器高度: {:.1}px", container_layout.size.height);
            println!("内容高度: {:.1}px", content_layout.size.height);
            
            assert!((container_layout.size.height - 540.0).abs() < 50.0);
            assert_eq!(content_layout.size.height, 200.0);
            
            println!("✅ 容器占满空间，内容保持自身高度\n");
        });
    });
}

#[test]
fn test_content_exceeds_container() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  场景 2: 内容高度 > 容器高度                           ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        let root = view()
            .name("Root")
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().name("Header").style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                view()
                    .name("ScrollContainer")
                    .style(|s| s.flex_grow(1.0).min_height(0.0).w_full().overflow_y_scroll())
                    .child(
                        view().name("LargeContent").style(|s| s.height(800.0).w_full()),
                    ),
            ));

        sig::runtime::flush_pending_signals();
        
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(400.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (400.0, 600.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let children = runtime.taffy.children(root.id.node_id()).unwrap();
            let container_layout = runtime.taffy.layout(children[1]).unwrap();
            let container_children = runtime.taffy.children(children[1]).unwrap();
            let content_layout = runtime.taffy.layout(container_children[0]).unwrap();
            
            println!("容器高度: {:.1}px", container_layout.size.height);
            println!("内容高度: {:.1}px", content_layout.size.height);
            
            assert!((container_layout.size.height - 540.0).abs() < 50.0);
            assert_eq!(content_layout.size.height, 800.0);
            
            println!("✅ 容器限制高度，内容超出可滚动\n");
        });
    });
}

#[test]
fn test_sidebar_detail_equal_height() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  场景 3: Sidebar 和 Detail 高度是否一致                ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        let root = view()
            .name("Root")
            .style(|s| s.width(1200.0).height(800.0).flex().flex_col())
            .child((
                view().name("Header").style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                view()
                    .name("MiddleContainer")
                    .style(|s| s.flex().flex_row().flex_grow(1.0).min_height(0.0).w_full())
                    .child((
                        view()
                            .name("Sidebar")
                            .style(|s| s.width(300.0).flex_shrink(0.0).h_full().overflow_y_scroll())
                            .child(view().style(|s| s.height(2000.0).w_full())),
                        view()
                            .name("Detail")
                            .style(|s| s.flex_grow(1.0).h_full().overflow_y_scroll())
                            .child(view().style(|s| s.height(100.0).w_full())),
                    )),
            ));

        sig::runtime::flush_pending_signals();
        
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(1200.0),
            height: taffy::AvailableSpace::Definite(800.0),
        };
        let ctx = sig::RenderContext::new(1.0, (1200.0, 800.0));
        root.id.compute_layout(available, &ctx);

        sig::runtime::with_layout(|runtime| {
            let root_children = runtime.taffy.children(root.id.node_id()).unwrap();
            let middle_children = runtime.taffy.children(root_children[1]).unwrap();
            
            let sidebar_layout = runtime.taffy.layout(middle_children[0]).unwrap();
            let detail_layout = runtime.taffy.layout(middle_children[1]).unwrap();
            
            println!("Sidebar: {:.1}x{:.1}", sidebar_layout.size.width, sidebar_layout.size.height);
            println!("Detail:  {:.1}x{:.1}", detail_layout.size.width, detail_layout.size.height);
            
            assert_eq!(sidebar_layout.size.width, 300.0);
            assert_eq!(detail_layout.size.width, 900.0);
            assert_eq!(sidebar_layout.size.height, detail_layout.size.height);
            assert_eq!(sidebar_layout.size.height, 740.0);
            
            println!("✅ Sidebar 和 Detail 高度一致\n");
        });
    });
}

#[test]
fn test_window_resize_impact() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  场景 4: 窗口大小变化的影响                            ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        let root = view()
            .name("Root")
            .style(|s| s.w_full().h_full().flex().flex_col())
            .child((
                view().style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                view()
                    .style(|s| s.flex_grow(1.0).min_height(0.0).w_full().overflow_y_scroll())
                    .child(view().style(|s| s.height(500.0).w_full())),
            ));

        sig::runtime::flush_pending_signals();
        
        let test_sizes = vec![
            (800.0, 600.0),
            (1200.0, 800.0),
            (1600.0, 1000.0),
        ];
        
        for (w, h) in test_sizes {
            let available = taffy::Size {
                width: taffy::AvailableSpace::Definite(w),
                height: taffy::AvailableSpace::Definite(h),
            };
            let ctx = sig::RenderContext::new(1.0, (w, h));
            root.id.compute_layout(available, &ctx);
            
            sig::runtime::with_layout(|runtime| {
                let children = runtime.taffy.children(root.id.node_id()).unwrap();
                let content_layout = runtime.taffy.layout(children[1]).unwrap();
                let expected_height = h - 60.0;
                
                println!("窗口 {:.0}x{:.0}: Content = {:.1}px", w, h, content_layout.size.height);
                
                assert!((content_layout.size.height - expected_height).abs() < 50.0);
            });
        }
        
        println!("✅ 布局响应窗口变化\n");
    });
}
