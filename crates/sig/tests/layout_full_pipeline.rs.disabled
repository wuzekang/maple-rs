/// 全链路布局测试
/// 
/// 测试目标：验证从 view 树构建到 taffy 布局计算的完整流程
/// 重点：height(0) + flex_grow(1.0) 是否正确工作

use sig::prelude::*;

/// 辅助函数：递归打印 view 树结构
fn print_view_tree(view_id: &sig::ViewId, depth: usize) {
    let indent = "  ".repeat(depth);
    let children = view_id.get_children();
    
    sig::runtime::with_layout(|runtime| {
        let name = runtime.view_states.get(view_id)
            .map(|s| s.name.as_str())
            .unwrap_or("unnamed");
        println!("{}├─ {} (ViewId: {:?}, children: {})", indent, name, view_id, children.len());
    });
    
    for child in children.iter() {
        print_view_tree(child, depth + 1);
    }
}

/// 辅助函数：递归打印 taffy 树结构
fn print_taffy_tree(node_id: taffy::NodeId, depth: usize) {
    let indent = "  ".repeat(depth);
    
    sig::runtime::with_layout(|runtime| {
        if let Ok(layout) = runtime.taffy.layout(node_id) {
            if let Ok(style) = runtime.taffy.style(node_id) {
                println!("{}📦 NodeId({:?})", indent, node_id);
                println!("{}   尺寸: {:.1} x {:.1}", indent, layout.size.width, layout.size.height);
                println!("{}   位置: ({:.1}, {:.1})", indent, layout.location.x, layout.location.y);
                println!("{}   Flex: grow={:.1}, shrink={:.1}, basis={:?}", 
                    indent, style.flex_grow, style.flex_shrink, style.flex_basis);
                println!("{}   Height: {:?}", indent, style.size.height);
                
                if let Ok(children) = runtime.taffy.children(node_id) {
                    println!("{}   Taffy children: {}", indent, children.len());
                    for child in children {
                        print_taffy_tree(child, depth + 1);
                    }
                }
            }
        }
    });
}

/// 辅助函数：手动同步 view 树到 taffy 树
/// 这是关键函数，模拟渲染时的树构建过程
/// 
/// 注意：View 在创建时已经通过 create_effect 设置了自动同步
/// 但 effect 只在 Signal 被读取时才会触发
/// 所以我们需要强制刷新所有待处理的 Signal
fn sync_view_to_taffy(view_id: &sig::ViewId) {
    // 强制刷新所有待处理的 Signal 通知
    // 这会触发 View::new() 中的 create_effect
    sig::runtime::flush_pending_signals();
    
    // 递归处理所有子节点
    let children = view_id.get_children();
    for child_id in children.iter() {
        sync_view_to_taffy(child_id);
    }
}

#[test]
fn test_full_layout_pipeline_basic() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  测试 1: 基础布局 - 验证 view 树到 taffy 树的同步      ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        // Step 1: 创建 view 树
        println!("Step 1: 创建 view 树");
        let root = view()
            .name("Root")
            .style(|s| s.width(800.0).height(600.0).flex().flex_col())
            .child((
                view()
                    .name("Header")
                    .style(|s| s.height(100.0).w_full().flex_shrink(0.0)),
                view()
                    .name("Content")
                    .style(|s| s.flex_grow(1.0).min_height(0.0).w_full()),
            ));

        // Step 2: 打印 view 树结构
        println!("\nStep 2: View 树结构：");
        print_view_tree(&root.id, 0);
        
        // Step 3: 同步 view 树到 taffy 树
        println!("\nStep 3: 同步 view 树到 taffy 树");
        sync_view_to_taffy(&root.id);
        
        // Step 4: 验证 taffy 树结构
        println!("\nStep 4: Taffy 树结构（同步后）：");
        sig::runtime::with_layout(|runtime| {
            if let Ok(children) = runtime.taffy.children(root.id.node_id()) {
                println!("Taffy 子节点数: {}", children.len());
                assert_eq!(children.len(), 2, "❌ Taffy 树应该有 2 个子节点");
            }
        });
        
        // Step 5: 计算布局
        println!("\nStep 5: 计算布局");
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(800.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (800.0, 600.0));
        root.id.compute_layout(available_space, &render_ctx);
        
        // Step 6: 验证布局结果
        println!("\nStep 6: 布局结果：");
        print_taffy_tree(root.id.node_id(), 0);
        
        println!("\nStep 7: 验证高度");
        sig::runtime::with_layout(|runtime| {
            let children = runtime.taffy.children(root.id.node_id()).unwrap();
            let header_layout = runtime.taffy.layout(children[0]).unwrap();
            let content_layout = runtime.taffy.layout(children[1]).unwrap();
            
            println!("Header 高度: {:.1}px (期望: 100px)", header_layout.size.height);
            println!("Content 高度: {:.1}px (期望: 500px)", content_layout.size.height);
            
            assert_eq!(header_layout.size.height, 100.0, "❌ Header 高度错误");
            assert_eq!(content_layout.size.height, 500.0, "❌ Content 高度错误 - height(0) + flex_grow 失败！");
        });
        
        println!("\n✅ 测试通过！height(0) + flex_grow(1.0) 工作正常\n");
    });
}

#[test]
fn test_full_layout_pipeline_nested() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  测试 2: 嵌套布局 - 模拟 sig_editor 的三层结构         ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        // 创建三层结构：Root -> (Header, Middle) -> Middle -> (Sidebar, Detail)
        let root = view()
            .name("Root")
            .style(|s| s.width(1200.0).height(800.0).flex().flex_col())
            .child((
                // Header
                view()
                    .name("Header")
                    .style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                // Middle container
                view()
                    .name("MiddleContainer")
                    .style(|s| s.flex().flex_row().flex_grow(1.0).min_height(0.0).w_full())
                    .child((
                        // Sidebar - 固定宽度 300px，高度跟随父容器
                        view()
                            .name("Sidebar")
                            .style(|s| s.width(300.0).flex_shrink(0.0).h_full()),
                        // Detail - 占据剩余宽度，高度跟随父容器
                        view()
                            .name("Detail")
                            .style(|s| s.flex_grow(1.0).h_full()),
                    )),
            ));

        println!("Step 1: View 树结构：");
        print_view_tree(&root.id, 0);
        
        println!("\nStep 2: 同步到 taffy 树");
        sync_view_to_taffy(&root.id);
        
        println!("\nStep 3: 计算布局");
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(1200.0),
            height: taffy::AvailableSpace::Definite(800.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (1200.0, 800.0));
        root.id.compute_layout(available_space, &render_ctx);
        
        println!("\nStep 4: 布局结果：");
        print_taffy_tree(root.id.node_id(), 0);
        
        println!("\nStep 5: 详细验证");
        sig::runtime::with_layout(|runtime| {
            // 验证第一层
            let root_children = runtime.taffy.children(root.id.node_id()).unwrap();
            assert_eq!(root_children.len(), 2, "Root 应该有 2 个子节点");
            
            let header_layout = runtime.taffy.layout(root_children[0]).unwrap();
            let middle_layout = runtime.taffy.layout(root_children[1]).unwrap();
            
            println!("Header: {:.1}x{:.1} (期望: 1200x60)", header_layout.size.width, header_layout.size.height);
            println!("Middle: {:.1}x{:.1} (期望: 1200x740)", middle_layout.size.width, middle_layout.size.height);
            
            assert_eq!(header_layout.size.height, 60.0);
            assert_eq!(middle_layout.size.height, 740.0, "❌ Middle 高度错误");
            
            // 验证第二层
            let middle_children = runtime.taffy.children(root_children[1]).unwrap();
            assert_eq!(middle_children.len(), 2, "Middle 应该有 2 个子节点");
            
            let sidebar_layout = runtime.taffy.layout(middle_children[0]).unwrap();
            let detail_layout = runtime.taffy.layout(middle_children[1]).unwrap();
            
            println!("Sidebar: {:.1}x{:.1} (期望: 300x740)", sidebar_layout.size.width, sidebar_layout.size.height);
            println!("Detail: {:.1}x{:.1} (期望: 900x740)", detail_layout.size.width, detail_layout.size.height);
            
            assert_eq!(sidebar_layout.size.width, 300.0);
            assert_eq!(sidebar_layout.size.height, 740.0, "❌ Sidebar 高度错误 - height(0) + flex_grow 在嵌套中失败！");
            assert_eq!(detail_layout.size.width, 900.0);
            assert_eq!(detail_layout.size.height, 740.0, "❌ Detail 高度错误");
        });
        
        println!("\n✅ 嵌套布局测试通过！\n");
    });
}

#[test]
fn test_full_layout_pipeline_deep_nesting() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  测试 3: 深度嵌套 (4层) - 验证高度传递                 ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        let root = view()
            .name("Root")
            .style(|s| s.width(1200.0).height(800.0).flex().flex_col())
            .child((
                view()
                    .name("Header")
                    .style(|s| s.height(60.0).w_full().flex_shrink(0.0)),
                view()
                    .name("Layer2")
                    .style(|s| s.flex().flex_row().flex_grow(1.0).min_height(0.0).w_full())
                    .child((
                        view()
                            .name("Layer3")
                            .style(|s| s.width(300.0).flex_grow(1.0).min_height(0.0).flex().flex_col())
                            .child(
                                view()
                                    .name("Layer4_Content")
                                    .style(|s| s.flex_grow(1.0).min_height(0.0).w_full()),
                            ),
                        view()
                            .name("Detail")
                            .style(|s| s.flex_grow(1.0).min_height(0.0)),
                    )),
            ));

        sync_view_to_taffy(&root.id);
        
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(1200.0),
            height: taffy::AvailableSpace::Definite(800.0),
        };
        let render_ctx = sig::RenderContext::new(1.0, (1200.0, 800.0));
        root.id.compute_layout(available_space, &render_ctx);
        
        println!("布局结果：");
        print_taffy_tree(root.id.node_id(), 0);
        
        println!("\n验证第 4 层深度的高度："); sig::runtime::with_layout(|runtime| {
            let l1 = runtime.taffy.children(root.id.node_id()).unwrap();
            let l2 = runtime.taffy.children(l1[1]).unwrap();
            let l3 = runtime.taffy.children(l2[0]).unwrap();
            let l4_layout = runtime.taffy.layout(l3[0]).unwrap();
            
            println!("Layer4_Content: {:.1}x{:.1} (期望: 300x740)", l4_layout.size.width, l4_layout.size.height);
            
            assert_eq!(l4_layout.size.height, 740.0, 
                "❌ 4层嵌套后高度丢失！这是 height(0) + flex_grow 在深度嵌套中的问题");
        });
        
        println!("\n✅ 深度嵌套测试通过！\n");
    });
}

#[test]
fn test_compare_height_strategies() {
    create_scope(|| {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  测试 4: 对比不同的高度策略                            ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        // 策略 1: height(0) + flex_grow
        let test1 = view()
            .name("Test1_height0")
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().style(|s| s.height(100.0).w_full()),
                view().name("Content1").style(|s| s.flex_grow(1.0).min_height(0.0).w_full()),
            ));
        
        // 策略 2: min_height(0) + flex_grow
        let test2 = view()
            .name("Test2_minheight0")
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().style(|s| s.height(100.0).w_full()),
                view().name("Content2").style(|s| s.flex_grow(1.0).min_height(0.0).w_full()),
            ));
        
        // 策略 3: 只用 flex_grow
        let test3 = view()
            .name("Test3_flexonly")
            .style(|s| s.width(400.0).height(600.0).flex().flex_col())
            .child((
                view().style(|s| s.height(100.0).w_full()),
                view().name("Content3").style(|s| s.flex_grow(1.0).w_full()),
            ));
        
        sync_view_to_taffy(&test1.id);
        sync_view_to_taffy(&test2.id);
        sync_view_to_taffy(&test3.id);
        
        let available_space = taffy::Size {
            width: taffy::AvailableSpace::Definite(400.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (400.0, 600.0));
        
        test1.id.compute_layout(available_space, &ctx);
        test2.id.compute_layout(available_space, &ctx);
        test3.id.compute_layout(available_space, &ctx);
        
        sig::runtime::with_layout(|runtime| {
            let get_content_height = |test_id: taffy::NodeId| -> f32 {
                let children = runtime.taffy.children(test_id).unwrap();
                runtime.taffy.layout(children[1]).unwrap().size.height
            };
            
            let h1 = get_content_height(test1.id.node_id());
            let h2 = get_content_height(test2.id.node_id());
            let h3 = get_content_height(test3.id.node_id());
            
            println!("策略 1 (height(0) + flex_grow): {:.1}px", h1);
            println!("策略 2 (min_height(0) + flex_grow): {:.1}px", h2);
            println!("策略 3 (只用 flex_grow): {:.1}px", h3);
            
            println!("\n结论：");
            if h1 == 500.0 && h2 == 500.0 && h3 == 500.0 {
                println!("✅ 三种策略结果一致，都是 500px");
            } else {
                println!("⚠️  策略结果不一致！");
                if h1 != 500.0 { println!("   - height(0) 有问题: {}px", h1); }
                if h2 != 500.0 { println!("   - min_height(0) 有问题: {}px", h2); }
                if h3 != 500.0 { println!("   - 只用 flex_grow 有问题: {}px", h3); }
            }
        });
        
        println!();
    });
}
