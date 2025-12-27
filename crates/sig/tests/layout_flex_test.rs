/// 测试 Flex 布局的各种场景
/// 重点验证 height(0) + flex_grow(1.0) 是否能正常工作

use sig::prelude::*;

#[test]
fn test_flex_grow_with_height_zero() {
    create_scope(|| {
        println!("\n=== 测试 flex_grow + height(0) ===");
        
        // 创建布局树
        let root = view()
            .style(|s| s.width(800.0).height(600.0).flex().flex_col())
            .child((
                // Header - 固定 100px
                view().style(|s| s.height(100.0).w_full().flex_shrink(0.0)),
                // Content - 应该自动扩展到 500px
                view().style(|s| s.flex_grow(1.0).height(0.0).w_full()),
            ));

        // 计算布局
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(800.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (800.0, 600.0));
        
        root.id.compute_layout(available_space, &render_ctx);

        // 检查布局结果
        sig::runtime::with_layout(|runtime| {
            let root_layout = runtime.taffy.layout(root.id.node_id()).unwrap();
            println!("Root: {}x{}", root_layout.size.width, root_layout.size.height);
            
            let children = runtime.taffy.children(root.id.node_id()).unwrap();
            assert_eq!(children.len(), 2, "应该有 2 个子节点");
            
            let header_layout = runtime.taffy.layout(children[0]).unwrap();
            let content_layout = runtime.taffy.layout(children[1]).unwrap();
            
            println!("Header: {}x{}", header_layout.size.width, header_layout.size.height);
            println!("Content: {}x{}", content_layout.size.width, content_layout.size.height);
            
            // 验证高度
            assert_eq!(header_layout.size.height, 100.0, "Header 应该是 100px");
            assert_eq!(content_layout.size.height, 500.0, "Content 应该是 500px (600 - 100)");
            
            // ⚠️ 关键验证：如果 height(0) 有问题，content_layout.size.height 会是 0
            assert!(content_layout.size.height > 0.0, "❌ 高度不应该是 0！");
        });
        
        println!("✅ 测试通过\n");
    });
}

#[test]
fn test_nested_flex_containers() {
    create_scope(|| {
        println!("\n=== 测试嵌套 flex 容器 ===");
        
        let root = view()
            .style(|s| s.width(1200.0).height(800.0).flex().flex_col())
            .child((
                // Header
                view().style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                // Middle (应该是 740px)
                view()
                    .style(|s| s.flex().flex_row().flex_grow(1.0).height(0.0).w_full())
                    .child((
                        // Sidebar (300px 宽，740px 高)
                        view().style(|s| s.width(300.0).flex_grow(1.0).height(0.0)),
                        // Detail (900px 宽，740px 高)
                        view().style(|s| s.flex_grow(1.0).height(0.0)),
                    )),
            ));

        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(1200.0),
            height: taffy::AvailableSpace::Definite(800.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (1200.0, 800.0));
        
        root.id.compute_layout(available_space, &render_ctx);

        sig::runtime::with_layout(|runtime| {
            let root_children = runtime.taffy.children(root.id.node_id()).unwrap();
            let header_layout = runtime.taffy.layout(root_children[0]).unwrap();
            let middle_layout = runtime.taffy.layout(root_children[1]).unwrap();
            
            println!("Header: {}x{}", header_layout.size.width, header_layout.size.height);
            println!("Middle: {}x{}", middle_layout.size.width, middle_layout.size.height);
            
            assert_eq!(header_layout.size.height, 60.0);
            assert_eq!(middle_layout.size.height, 740.0, "Middle 应该是 740px (800 - 60)");
            
            // 检查 middle 的子节点
            let middle_children = runtime.taffy.children(root_children[1]).unwrap();
            let sidebar_layout = runtime.taffy.layout(middle_children[0]).unwrap();
            let detail_layout = runtime.taffy.layout(middle_children[1]).unwrap();
            
            println!("Sidebar: {}x{}", sidebar_layout.size.width, sidebar_layout.size.height);
            println!("Detail: {}x{}", detail_layout.size.width, detail_layout.size.height);
            
            assert_eq!(sidebar_layout.size.width, 300.0);
            assert_eq!(sidebar_layout.size.height, 740.0, "Sidebar 高度应该继承自 middle");
            assert_eq!(detail_layout.size.width, 900.0, "Detail 宽度应该是 900px (1200 - 300)");
            assert_eq!(detail_layout.size.height, 740.0, "Detail 高度应该继承自 middle");
        });
        
        println!("✅ 测试通过\n");
    });
}

#[test]
fn test_min_height_zero_vs_height_zero() {
    create_scope(|| {
        println!("\n=== 对比 min_height(0) vs height(0) ===");
        
        // 测试 1: height(0)
        let test1 = view()
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().style(|s| s.height(100.0).w_full()),
                view().style(|s| s.flex_grow(1.0).height(0.0).w_full()),
            ));
        
        // 测试 2: min_height(0)
        let test2 = view()
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().style(|s| s.height(100.0).w_full()),
                view().style(|s| s.flex_grow(1.0).min_height(0.0).w_full()),
            ));

        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(400.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (400.0, 600.0));
        
        test1.id.compute_layout(available_space, &render_ctx);
        test2.id.compute_layout(available_space, &render_ctx);

        sig::runtime::with_layout(|runtime| {
            let test1_children = runtime.taffy.children(test1.id.node_id()).unwrap();
            let test2_children = runtime.taffy.children(test2.id.node_id()).unwrap();
            
            let test1_content = runtime.taffy.layout(test1_children[1]).unwrap();
            let test2_content = runtime.taffy.layout(test2_children[1]).unwrap();
            
            println!("height(0) 内容高度: {}", test1_content.size.height);
            println!("min_height(0) 内容高度: {}", test2_content.size.height);
            
            // 两者都应该得到 500px
            assert_eq!(test1_content.size.height, 500.0, "height(0) + flex_grow 应该得到 500px");
            assert_eq!(test2_content.size.height, 500.0, "min_height(0) + flex_grow 应该得到 500px");
        });
        
        println!("✅ 两种方式结果一致\n");
    });
}

#[test]
fn test_deeply_nested_flex() {
    create_scope(|| {
        println!("\n=== 测试深度嵌套的 flex 布局 (4 层) ===");
        
        let root = view()
            .style(|s| s.width(1200.0).height(800.0).flex().flex_col())
            .child((
                // Layer 1: Header
                view().style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                // Layer 2: Middle
                view()
                    .style(|s| s.flex().flex_row().flex_grow(1.0).height(0.0).w_full())
                    .child((
                        // Layer 3: Sidebar container
                        view()
                            .style(|s| s.width(300.0).flex_grow(1.0).height(0.0).flex().flex_col())
                            .child(
                                // Layer 4: VirtualList placeholder
                                view().style(|s| s.flex_grow(1.0).height(0.0).w_full()),
                            ),
                        // Layer 3: Detail
                        view().style(|s| s.flex_grow(1.0).height(0.0)),
                    )),
            ));

        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(1200.0),
            height: taffy::AvailableSpace::Definite(800.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (1200.0, 800.0));
        
        root.id.compute_layout(available_space, &render_ctx);

        sig::runtime::with_layout(|runtime| {
            // Layer 1
            let l1_children = runtime.taffy.children(root.id.node_id()).unwrap();
            let middle_node = l1_children[1];
            let middle_layout = runtime.taffy.layout(middle_node).unwrap();
            
            println!("Layer 2 (Middle): {}x{}", middle_layout.size.width, middle_layout.size.height);
            assert_eq!(middle_layout.size.height, 740.0);
            
            // Layer 2
            let l2_children = runtime.taffy.children(middle_node).unwrap();
            let sidebar_container = l2_children[0];
            let sidebar_layout = runtime.taffy.layout(sidebar_container).unwrap();
            
            println!("Layer 3 (Sidebar container): {}x{}", sidebar_layout.size.width, sidebar_layout.size.height);
            assert_eq!(sidebar_layout.size.height, 740.0);
            
            // Layer 3
            let l3_children = runtime.taffy.children(sidebar_container).unwrap();
            let virtuallist_placeholder = l3_children[0];
            let vl_layout = runtime.taffy.layout(virtuallist_placeholder).unwrap();
            
            println!("Layer 4 (VirtualList placeholder): {}x{}", vl_layout.size.width, vl_layout.size.height);
            
            // ⚠️ 关键验证：4 层深度嵌套后，最底层的高度应该仍然是 740px
            assert_eq!(vl_layout.size.height, 740.0, "❌ 深度嵌套后高度丢失！应该是 740px");
            assert!(vl_layout.size.height > 0.0, "❌ 深度嵌套后高度变成了 0！");
        });
        
        println!("✅ 深度嵌套测试通过\n");
    });
}
