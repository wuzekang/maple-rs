use sig::prelude::*;
use sig::{Dynamic, dynamic};

#[test]
fn test_render_tree_get_children_updates() {
    create_scope(|| {
        let level1_open = Signal::new(false);
        let level2_open = Signal::new(false);

        let root_view = view();
        let root_id = root_view.id;

        // 添加嵌套 Dynamic
        root_view.child(create_nested_dynamic(level1_open, level2_open));

        eprintln!("--- 初始状态（都关闭）---");
        let initial_count = count_all_views(&root_id);
        eprintln!("初始视图数量: {}", initial_count);

        eprintln!("\n--- 展开 LEVEL_1 ---");
        *level1_open.write() = true;
        let level1_count = count_all_views(&root_id);
        eprintln!("LEVEL_1 展开后视图数量: {}", level1_count);
        assert!(level1_count > initial_count, "View count should increase after opening level 1");

        eprintln!("\n--- 展开 LEVEL_2 ---");
        *level2_open.write() = true;
        let final_count = count_all_views(&root_id);
        eprintln!("LEVEL_2 展开后视图数量: {}", final_count);
        assert!(final_count > level1_count, "View count should increase after opening level 2");

        // 验证
        let final_children = root_id.get_children();
        eprintln!("\n=== 最终结果 ===");
        eprintln!("root_view 直接子节点: {} 个", final_children.len());
        eprintln!("总视图数量: {} 个", final_count);

        assert!(final_count >= 6, "Should have at least 6 views when fully expanded");
    });
}

fn create_nested_dynamic(level1_open: Signal<bool>, level2_open: Signal<bool>) -> Dynamic {
    dynamic(move || {
        let open1 = *level1_open.read();
        if !open1 {
            view().child(text("LEVEL_1: 折叠"))
        } else {
            view()
                .child(text("LEVEL_1: 展开"))
                .child(dynamic(move || {
                    let open2 = *level2_open.read();
                    if !open2 {
                        view().child(text("  LEVEL_2: 折叠"))
                    } else {
                        view()
                            .child(text("  LEVEL_2: 展开"))
                            .child(text("    子节点 A"))
                            .child(text("    子节点 B"))
                    }
                }))
        }
    })
}

fn count_all_views(root_id: &sig::ViewId) -> usize {
    let children = root_id.get_children();
    let mut count = children.len();
    for id in children.iter() {
        count += count_all_views(id);
    }
    count
}
