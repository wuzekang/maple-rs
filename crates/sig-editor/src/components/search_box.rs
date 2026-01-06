use crate::constants::{BORDER_COLOR, PANEL_BG, PANEL_HEADER_BG, SUCCESS_COLOR, TEXT_PRIMARY};
use crate::state::AppState;
use crate::types::WzTreeNode;
use sig::event::Cursor;
use sig::prelude::*;
use sig::theme::{Border, FontSize, Radius, Size, Spacing};
use sig::{dynamic, text, Placement, Popper, Signal, TreeNode};
use std::rc::Rc;

/// Search box with popper dropdown
pub fn search_box_with_dropdown(state: Rc<AppState>) -> sig::Node {
  let search_text = state.search_text.clone();
  let show_dropdown = state.show_search_dropdown.clone();
  let search_results = state.search_results.clone();
  let selected_index = state.selected_result_index.clone();

  // Reference element (search input)
  let search_text_for_click = search_text.clone();
  let show_dropdown_for_click = show_dropdown.clone();

  let reference = view()
    .style(|s| {
      s.w_full()
        .padding(Spacing::MD)
        .border_bottom(Border::THIN)
        .border_color(BORDER_COLOR)
        .background(PANEL_HEADER_BG)
        .flex_shrink(0.0)
    })
    .child(
      sig::text_input()
        .placeholder("Search WZ values...")
        .value(search_text.clone())
        .on_click(move |_| {
          if search_text_for_click.read().len() >= 2 {
            *show_dropdown_for_click.write() = true;
          }
        })
        .style(|s| {
          s.w_full()
            .height(Size::SM)
            .padding_left(Spacing::SM)
            .padding_right(Spacing::SM)
            .border_radius(Radius::MD)
            .border_all(Border::THIN, BORDER_COLOR)
            .background(PANEL_BG)
            .font_size(FontSize::SM)
            .color(TEXT_PRIMARY)
        }),
    );

  // Dropdown content as dynamic
  let state_for_content = state.clone();
  let dropdown_content = dynamic(move || {
    let show = *show_dropdown.read();
    let results = search_results.read();
    let selected = *selected_index.read();

    if !show || results.is_empty() {
      return view();
    }

    // Build dropdown items
    let items: Vec<View> = results
      .iter()
      .enumerate()
      .map(|(idx, result)| {
        let is_selected = selected == Some(idx);
        let result_clone = result.clone();
        let state_clone = state_for_content.clone();

        // Hover state for this item
        let hovered = Signal::new(false);

        view()
          .style(move |s| {
            let is_hovered = *hovered.read();
            s.w_full()
              .padding(Spacing::SM)
              .background(if is_selected {
                SUCCESS_COLOR.with_alpha(0.2)
              } else if is_hovered {
                PANEL_HEADER_BG // Use a slightly different color for hover
              } else {
                PANEL_BG
              })
              .cursor(Cursor::Pointer)
          })
          .on_mouse_enter(move |_| *hovered.write() = true)
          .on_mouse_leave(move |_| *hovered.write() = false)
          .child(
            text(format!("{}: {}", result.value_type, result.value_display))
              .style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)),
          )
          .on_click(move |_| {
            let state = state_clone.clone();
            let result = result_clone.clone();

            // Spawn async task for location
            sig::spawn(async move {
              if let Some(tree_view) = state.tree_view.read_untracked().as_ref() {
                let tree_view_clone = tree_view.clone();
                let node_clone = result.node.clone();
                let node_for_error = node_clone.clone();

                let node_name = {
                  let n = node_clone.read().unwrap();
                  format!("{} ({})", n.name, n.get_full_path())
                };
                println!("🎯 [Location] Starting node location for: {}", node_name);

                // 在后台线程执行耗时的定位操作
                println!("🚀 [Location] sig::spawn task started");
                let start = std::time::Instant::now();

                let (path_nodes, target_node) = tokio::task::spawn_blocking(move || {
                  println!(
                    "⚙️  [Location] spawn_blocking: building node path in background thread"
                  );
                  // 构建从目标节点到根节点的路径
                  let mut path_to_root = Vec::new();
                  let mut current = Some(node_clone.clone());

                  while let Some(node) = current {
                    path_to_root.push(node.clone());

                    let node_read = node.read().unwrap();
                    let parent = node_read.parent.upgrade();

                    if parent.is_none() {
                      break;
                    }

                    current = parent;
                  }

                  // 反转路径（从根到目标）
                  path_to_root.reverse();

                  println!(
                    "✅ [Location] spawn_blocking: path built with {} nodes",
                    path_to_root.len()
                  );
                  (path_to_root, node_clone)
                })
                .await
                .unwrap_or_else(|e| {
                  eprintln!("❌ [Location] spawn_blocking failed: {:?}", e);
                  (Vec::new(), node_for_error)
                });

                let elapsed = start.elapsed();
                println!("⏱️  [Location] Path building time: {:?}", elapsed);

                // 在主线程执行需要访问 Signal 的操作
                println!("📂 [Location] Expanding {} nodes in path", path_nodes.len());

                // 保存当前展开状态
                let current_open_states = tree_view_clone.get_open_states();
                *state.saved_open_states.write() = current_open_states;

                // 展开路径中的所有节点
                for (idx, node_arc) in path_nodes.iter().enumerate() {
                  let tree_node = WzTreeNode(node_arc.clone());

                  // 如果节点有子节点且未展开，则展开它
                  if tree_node.has_children() && !tree_view_clone.is_open(&tree_node) {
                    tree_view_clone.toggle_node(&tree_node);
                    println!(
                      "  📂 [Location] Expanded node {}/{}",
                      idx + 1,
                      path_nodes.len()
                    );
                  }
                }

                // 选中目标节点
                let target_tree_node = WzTreeNode(target_node.clone());
                tree_view_clone.select_node(&target_tree_node);
                tree_view_clone.scroll_to_node(&target_tree_node);

                // 更新 selected_node signal
                *state.selected_node.write() = Some(target_node.clone());

                // 显示定位成功提示
                let node_read = target_node.read().unwrap();
                let full_path = node_read.get_full_path();
                *state.location_success.write() = Some(format!("✅ 已定位到: {}", full_path));

                println!("✅ [Location] Node located successfully: {}", full_path);
              }
              *state.show_search_dropdown.write() = false;
            });
          })
      })
      .collect();

    // Create dropdown container
    view()
      .style(|s| {
        s.width(400.0)
          .max_height(300.0)
          .border_all(Border::THIN, BORDER_COLOR)
          .border_radius(Radius::MD)
          .background(PANEL_BG)
          .flex()
          .flex_col()
          .overflow_y(taffy::Overflow::Scroll)
      })
      .child(items)
  });

  // Create Popper and build to portal
  Popper::new(reference)
    .content(dropdown_content)
    .placement(Placement::BottomStart)
    .offset(4.0)
    .open(show_dropdown)
    .build_to_portal()
}
