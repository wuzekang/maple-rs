//! WZ Editor implemented with Sig framework
//!
//! Features:
//! - Left panel: Tree node browser with virtual scrolling
//! - Right panel: Node detail viewer (images, JSON, properties)
//! - Resizable panels
//! - Search functionality
//! - Async WZ file loading with use_resource to prevent UI blocking
//!
//! # Design Compliance
//!
//! This application uses the Sig design system tokens for consistent spacing,
//! sizing, and styling. See `sig/docs/DESIGN_GUIDELINES.md` for details.

use clipboard_rs::{Clipboard, ClipboardContext};
use serde_json::{Map, Value};
use sig::prelude::*;
use sig::render::{run, AppConfig};
use sig::theme::{Border, FontSize, Radius, Size, Spacing};
use sig::{button, dynamic, Image, Signal, TreeNode, TreeView, Popper, Placement};
use sig::event::Cursor;
use std::rc::Rc;
use vello::peniko::Color;
use wz_parser::util::{node_util, resolve_base};
use wz_parser::{
  property::{png, WzSubProperty, WzValue},
  WzNode, WzNodeArc, WzObjectType,
};

// ============================================================================
// Constants (Using Design Tokens)
// ============================================================================

const DEFAULT_LEFT_WIDTH: f32 = 280.0;

// Colors (Consider moving to theme system in future)
const BG_COLOR: Color = Color::from_rgb8(250, 250, 250); // gray-50
const PANEL_BG: Color = Color::from_rgb8(255, 255, 255); // white
const PANEL_HEADER_BG: Color = Color::from_rgb8(248, 250, 252); // slate-50
const BORDER_COLOR: Color = Color::from_rgb8(226, 232, 240); // slate-200
const TEXT_PRIMARY: Color = Color::from_rgb8(30, 30, 30); // gray-900
const TEXT_SECONDARY: Color = Color::from_rgb8(100, 100, 100); // gray-500
const ERROR_COLOR: Color = Color::from_rgb8(220, 38, 38); // red-600
const SUCCESS_COLOR: Color = Color::from_rgb8(34, 197, 94); // green-500

// ============================================================================
// TreeNode Wrapper for WzNodeArc
// ============================================================================

/// Wrapper for WzNodeArc to implement TreeNode
#[derive(Clone)]
struct WzTreeNode(WzNodeArc);

impl PartialEq for WzTreeNode {
  fn eq(&self, other: &Self) -> bool {
    std::sync::Arc::ptr_eq(&self.0, &other.0)
  }
}

impl Eq for WzTreeNode {}

impl std::hash::Hash for WzTreeNode {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    // Hash using Arc pointer address
    (std::sync::Arc::as_ptr(&self.0) as usize).hash(state);
  }
}

impl TreeNode for WzTreeNode {
  fn label(&self) -> String {
    self.0.read().unwrap().name.to_string()
  }

  fn children(&self) -> Vec<Self> {
    self
      .0
      .read()
      .unwrap()
      .children
      .values()
      .map(|child| WzTreeNode(child.clone()))
      .collect()
  }

  fn has_children(&self) -> bool {
    let node = self.0.read().unwrap();
    !node.children.is_empty()
      || matches!(
        node.object_type,
        WzObjectType::Image(_) | WzObjectType::Directory(_)
      )
  }

  fn load_children(&self) -> Result<(), Box<dyn std::error::Error>> {
    node_util::parse_node(&self.0)?;
    Ok(())
  }
}

// ============================================================================
// Data Structures for Search
// ============================================================================

/// Search result item
#[derive(Clone)]
struct SearchResult {
    path: String,           // 节点完整路径
    node_name: String,      // 节点名称
    value_type: String,     // 值类型（"Int", "String", "Vector" 等）
    value_display: String,  // 值的可读表示
    node: WzNodeArc,       // 节点引用（用于定位）
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        // Compare by Arc pointer equality for the node
        std::sync::Arc::ptr_eq(&self.node, &other.node)
    }
}

/// Serializable value representation
#[derive(Clone)]
struct SerializableValue {
    type_name: String,
    display: String,
}

// ============================================================================
// Application State
// ============================================================================

struct AppState {
  root_node: Signal<Option<WzTreeNode>>,
  selected_node: Signal<Option<WzNodeArc>>, // Keep as WzNodeArc for compatibility with rendering
  left_width: Signal<f32>,
  error_message: Signal<Option<String>>,
  search_text: Signal<String>, // Search text for filtering

  // New fields for search feature
  tree_view: Signal<Option<Rc<TreeView<WzTreeNode>>>>,
  search_results: Signal<Vec<SearchResult>>,
  show_search_dropdown: Signal<bool>,
  selected_result_index: Signal<Option<usize>>,
  saved_open_states: Signal<std::collections::HashMap<u64, bool>>,
  location_success: Signal<Option<String>>,
}

impl AppState {
  fn new() -> Rc<Self> {
    Rc::new(Self {
      root_node: Signal::new(None),
      selected_node: Signal::new(None),
      left_width: Signal::new(DEFAULT_LEFT_WIDTH),
      error_message: Signal::new(None),
      search_text: Signal::new(String::new()),

      // Initialize new fields
      tree_view: Signal::new(None),
      search_results: Signal::new(Vec::new()),
      show_search_dropdown: Signal::new(false),
      selected_result_index: Signal::new(None),
      saved_open_states: Signal::new(std::collections::HashMap::new()),
      location_success: Signal::new(None),
    })
  }
}

// ============================================================================
// WZ Search Functions
// ============================================================================

/// 搜索 WZ 树中的所有可序列化值（最多返回 50 条）
///
/// 遍历整个 WZ 树，查找所有包含搜索文本的节点值。
/// 使用异步方式避免阻塞 UI。
/// 
/// **注意**：此函数会解析所有节点（包括未展开的），可能耗时较长。
fn search_wz_values(root: &WzNodeArc, search_term: &str) -> Vec<SearchResult> {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::collections::HashSet;

    let results = Mutex::new(Vec::new());
    let search_term = search_term.to_lowercase();
    let nodes_visited = AtomicUsize::new(0);
    let visited_ids = Mutex::new(HashSet::new());
    let parse_errors = AtomicUsize::new(0);

    println!("  🔎 [Search] Walking tree for term: '{}' (parsing all nodes...)", search_term);

    // 手动实现非递归遍历，避免栈溢出和循环引用
    let mut stack = vec![root.clone()];
    
    while let Some(node_arc) = stack.pop() {
        // 检查是否已访问（通过指针地址判断）
        let node_id = std::sync::Arc::as_ptr(&node_arc) as usize;
        {
            let mut visited = visited_ids.lock().unwrap();
            if visited.contains(&node_id) {
                // println!("  ⚠️  [Search] Detected circular reference, skipping node");
                continue;
            }
            visited.insert(node_id);
        }

        let count = nodes_visited.fetch_add(1, Ordering::Relaxed);
        if count % 5000 == 0 && count > 0 {
            let matches = results.lock().unwrap().len();
            // println!("  📊 [Search] Visited {} nodes, found {} matches so far...", count, matches);
        }

        // 限制最大访问节点数，防止无限循环或超长搜索
        // Base.wz 通常有 10-20 万个节点
        if count > 500000 {
            eprintln!("  ⚠️  [Search] Exceeded maximum node limit (500000), stopping search");
            break;
        }

        // 🔑 关键修复：在访问节点前先解析它（懒加载）
        // 这样未展开的节点也能被搜索到
        if let Err(_e) = node_util::parse_node(&node_arc) {
            // 解析失败不是致命错误，继续处理
            // 大多数失败是因为节点已经被解析过了
            parse_errors.fetch_add(1, Ordering::Relaxed);
        }

        let node_read = node_arc.read().unwrap();

        // 提取可序列化的值
        let value_info = extract_serializable_value(&node_read);

        if let Some(info) = value_info {
            // 不区分大小写的文本匹配
            let value_lower = info.display.to_lowercase();

            if value_lower.contains(&search_term) {
                let mut results = results.lock().unwrap();
                if results.len() < 50 {
                    results.push(SearchResult {
                        path: node_read.get_full_path(),
                        node_name: node_read.name.to_string(),
                        value_type: info.type_name,
                        value_display: info.display,
                        node: node_arc.clone(),
                    });
                    
                    // 早停：如果已经找到 50 个结果，可以提前结束
                    // 但继续处理当前栈中的节点，以防遗漏更相关的结果
                    if results.len() >= 50 {
                        println!("  ✅ [Search] Found 50 matches, continuing to process remaining nodes in stack...");
                    }
                }
            }
        }

        // 添加子节点到栈中（即使已经找到 50 个结果也继续，以获得更全面的搜索）
        for child in node_read.children.values() {
            stack.push(child.clone());
        }
    }

    // 限制结果数量为 50
    let mut results = results.into_inner().unwrap();
    let total_visited = nodes_visited.load(Ordering::Relaxed);
    let total_errors = parse_errors.load(Ordering::Relaxed);
    println!("  📊 [Search] Complete: visited {} nodes, {} parse errors, found {} matches", 
             total_visited, total_errors, results.len());
    
    results.truncate(50);

    results
}

/// 提取节点的可序列化值
///
/// 尝试从 WzNode 中提取所有可序列化的值类型。
/// 如果节点不包含可序列化的值，返回 None。
fn extract_serializable_value(node: &wz_parser::WzNode) -> Option<SerializableValue> {
    use wz_parser::WzNodeCast;

    // 尝试获取 WzValue
    if let Some(wz_value) = node.try_as_value() {
        match wz_value {
            // 跳过的类型
            WzValue::Null => None,
            WzValue::RawData(_) => None,
            WzValue::Lua(_) => None,

            // 数字类型
            WzValue::Short(v) => Some(SerializableValue {
                type_name: "Short".to_string(),
                display: v.to_string(),
            }),

            WzValue::Int(v) => Some(SerializableValue {
                type_name: "Int".to_string(),
                display: v.to_string(),
            }),

            WzValue::Long(v) => Some(SerializableValue {
                type_name: "Long".to_string(),
                display: v.to_string(),
            }),

            WzValue::Float(v) => Some(SerializableValue {
                type_name: "Float".to_string(),
                display: v.to_string(),
            }),

            WzValue::Double(v) => Some(SerializableValue {
                type_name: "Double".to_string(),
                display: v.to_string(),
            }),

            // Vector 类型
            WzValue::Vector(v) => Some(SerializableValue {
                type_name: "Vector".to_string(),
                display: format!("({}, {})", v.0, v.1),
            }),

            // 字符串类型
            WzValue::String(s) | WzValue::UOL(s) => {
                s.get_string().ok().map(|text| SerializableValue {
                    type_name: if matches!(wz_value, WzValue::String(_)) {
                        "String"
                    } else {
                        "UOL"
                    }.to_string(),
                    display: text,
                })
            }

            WzValue::ParsedString(s) => Some(SerializableValue {
                type_name: "String".to_string(),
                display: s.clone(),
            }),
        }
    } else {
        None
    }
}
/// 设置搜索防抖效果
fn setup_search_effect(state: &Rc<AppState>) {
    let search_text = state.search_text.clone();
    let root_node = state.root_node.clone();
    let search_results = state.search_results.clone();
    let show_dropdown = state.show_search_dropdown.clone();
    let selected_index = state.selected_result_index.clone();

    sig::create_effect(move || {
        let text = search_text.read().clone();

        // 空搜索或太短时不搜索
        if text.len() < 2 {
            *show_dropdown.write() = false;
            *search_results.write() = Vec::new();
            *selected_index.write() = None;
            return;
        }

        *show_dropdown.write() = true;
        *selected_index.write() = None; // 重置选择

        let root = root_node.read();

        if let Some(tree_root) = root.as_ref() {
            let term = text.clone();
            let results_signal = search_results.clone();
            let tree_root_arc = tree_root.0.clone();

            println!("🔍 [Search] Starting search for term: '{}'", term);

            // 使用 spawn 异步搜索，避免阻塞 UI
            sig::spawn(async move {
                println!("🚀 [Search] sig::spawn task started");
                
                // 在后台线程执行耗时的搜索操作
                let start = std::time::Instant::now();
                let results = tokio::task::spawn_blocking(move || {
                    println!("⚙️  [Search] spawn_blocking: executing search in background thread");
                    let results = search_wz_values(&tree_root_arc, &term);
                    println!("✅ [Search] spawn_blocking: found {} results", results.len());
                    results
                }).await.unwrap_or_else(|e| {
                    eprintln!("❌ [Search] spawn_blocking failed: {:?}", e);
                    Vec::new()
                });
                
                let elapsed = start.elapsed();
                println!("⏱️  [Search] Total search time: {:?}", elapsed);
                
                // 在主线程更新 signal
                *results_signal.write() = results;
                println!("📝 [Search] Results written to signal");
            });
        }
    });
}

// ============================================================================
// UI Components
// ============================================================================

/// Search box with popper dropdown
fn search_box_with_dropdown(state: Rc<AppState>) -> sig::Node {
  let search_text = state.search_text.clone();
  let show_dropdown = state.show_search_dropdown.clone();
  let search_results = state.search_results.clone();
  let selected_index = state.selected_result_index.clone();

  // Reference element (search input)
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
        })
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

        view()
          .style(move |s| {
            s.w_full()
              .padding(Spacing::SM)
              .background(if is_selected {
                SUCCESS_COLOR.with_alpha(0.2)
              } else {
                PANEL_BG
              })
              .cursor(Cursor::Pointer)
          })
          .child(
            text(format!("{}: {}", result.value_type, result.value_display))
              .style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY))
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
                  println!("⚙️  [Location] spawn_blocking: building node path in background thread");
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
                  
                  println!("✅ [Location] spawn_blocking: path built with {} nodes", path_to_root.len());
                  (path_to_root, node_clone)
                }).await.unwrap_or_else(|e| {
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
                    println!("  📂 [Location] Expanded node {}/{}", idx + 1, path_nodes.len());
                  }
                }

                // 选中目标节点
                let target_tree_node = WzTreeNode(target_node.clone());
                tree_view_clone.select_node(&target_tree_node);

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

/// Left panel with tree view
fn left_panel(state: &Rc<AppState>) -> View {
  let root_signal = state.root_node.clone();
  let selected = state.selected_node.clone();
  let tree_view_ref = state.tree_view.clone();

  view()
    .style(|s| {
      s.w_full()
        .h_full()
        .flex()
        .flex_col()
        .background(PANEL_BG)
        .border_right(Border::THIN) // Using design token
        .border_color(BORDER_COLOR)
    })
    .child((
      // Search box with popper dropdown
      search_box_with_dropdown(state.clone()),
      // Tree view
      dynamic(move || {
        if let Some(root) = root_signal.read().as_ref() {
          let root_clone = root.clone();
          let selected_clone = selected.clone();

          // Create TreeView
          let tree_view = Rc::new(
            TreeView::new(root_clone)
              .show_child_count(true)
              .item_height(Size::SM - 4.0) // 28px - slightly smaller than button height
          );

          // Store TreeView reference
          *tree_view_ref.write() = Some(tree_view.clone());

          let tree_selected = tree_view.selected();

          // Sync tree selection with app state (unwrap WzTreeNode to WzNodeArc)
          sig::create_effect(move || {
            if let Some(node) = tree_selected.read().as_ref() {
              *selected_clone.write() = Some(node.0.clone());
            }
          });

          // Clone the TreeView and call build (since build() takes self)
          (*tree_view).clone().build()
        } else {
          view()
            .style(|s| s.w_full().h_full().flex().items_center().justify_center())
            .child(
              text("Loading WZ file...").style(|s| s.font_size(FontSize::MD).color(TEXT_SECONDARY)),
            )
        }
      }),
    ))
}

/// Right panel with node details
fn right_panel(state: &Rc<AppState>) -> View {
  let selected_node = state.selected_node.clone();
  let location_success = state.location_success.clone();

  view()
    .style(|s| {
      s.flex_grow(1.0)
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .background(PANEL_BG)
        .min_width(0.0) // Prevent content from expanding beyond container
    })
    .child((
      // Location success message (if any)
      dynamic(move || {
        if let Some(success_msg) = location_success.read().as_ref() {
          view()
            .style(|s| {
              s.w_full()
                .padding(Spacing::SM)
                .background(SUCCESS_COLOR.with_alpha(0.1))
                .border_bottom(Border::THIN)
                .border_color(SUCCESS_COLOR)
                .flex_shrink(0.0)
            })
            .child(
              text(success_msg.clone())
                .style(|s| s.font_size(FontSize::SM).color(SUCCESS_COLOR))
            )
        } else {
          view().style(|s| s.width(0.0).height(0.0))
        }
      }),
      // Existing content area
      dynamic(move || {
        if let Some(node) = selected_node.read().as_ref() {
          let node_data = node.read().unwrap();
          let full_path = node_data.get_full_path();
          let object_type = node_data.object_type.clone();
          drop(node_data);

          view()
            .style(|s| s.flex().flex_col().w_full().h_full())
            .child((
              // Header with path (fixed at top) - using design tokens
              view()
                .style(|s| {
                  s.w_full()
                    .padding(Spacing::LG) // 16px - consistent padding
                    .border_bottom(Border::THIN) // 1px border
                    .border_color(BORDER_COLOR)
                    .background(PANEL_HEADER_BG)
                    .flex_shrink(0.0)
                })
                .child(
                  text(full_path).style(|s| s.font_size(FontSize::MD).color(TEXT_PRIMARY)), // 14px
                ),
              // Content (scrollable) - using design tokens
              view()
                .style(|s| {
                  s.w_full()
                    .flex_grow(1.0)
                    .min_height(0.0)
                    .min_width(0.0)
                    .overflow_y_scroll()
                    .padding(Spacing::LG) // 16px - consistent padding
                })
                .child(render_node_content(node.clone(), object_type)),
            ))
        } else {
          view()
            .style(|s| s.w_full().h_full().flex().items_center().justify_center())
            .child(
              text("Select a node to view details")
                .style(|s| s.font_size(FontSize::MD).color(TEXT_SECONDARY)), // 14px
            )
        }
      }),
    ))
}

// ============================================================================
// JSON Serialization Helpers
// ============================================================================

/// Recursively walk node tree and convert to JSON
fn walk_node_and_to_json(node_arc: &WzNodeArc, json: &mut Map<String, Value>) {
  // Parse node first
  let _ = node_util::parse_node(node_arc);

  let node = node_arc.read().unwrap();
  match &node.object_type {
    WzObjectType::Value(value_type) => {
      json.insert(node.name.to_string(), value_type.clone().into());
    }
    WzObjectType::Directory(_)
    | WzObjectType::Image(_)
    | WzObjectType::File(_)
    | WzObjectType::Property(_) => {
      let mut child_json = Map::new();
      if !node.children.is_empty() {
        for value in node.children.values() {
          walk_node_and_to_json(value, &mut child_json);
        }
        json.insert(node.name.to_string(), Value::Object(child_json));
      }
    }
  }
}

/// Convert node to formatted JSON string
fn node_to_json(node: &WzNode) -> String {
  let mut json = Map::new();

  for value in node.children.values() {
    walk_node_and_to_json(value, &mut json);
  }

  serde_json::to_string_pretty(&Value::Object(json)).unwrap_or_else(|_| "{}".to_string())
}

/// Render node content based on type
fn render_node_content(node: WzNodeArc, object_type: WzObjectType) -> View {
  match object_type {
    WzObjectType::Value(value) => {
      let text_content = match value {
        WzValue::Short(v) => format!("Short: {}", v),
        WzValue::Int(v) => format!("Int: {}", v),
        WzValue::Long(v) => format!("Long: {}", v),
        WzValue::Float(v) => format!("Float: {}", v),
        WzValue::Double(v) => format!("Double: {}", v),
        WzValue::String(v) => format!("String: {:?}", v.get_string().unwrap_or_default()),
        WzValue::ParsedString(v) => format!("String: {}", v),
        WzValue::Vector(v) => format!("Vector: x={}, y={}", v.0, v.1),
        WzValue::UOL(v) => format!("UOL: {:?}", v),
        WzValue::Null => "Null".to_string(),
        WzValue::RawData(_) => "Raw Data".to_string(),
        WzValue::Lua(_) => "Lua Script".to_string(),
      };

      view().child(
        text(text_content).style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px - slightly smaller for data
      )
    }
    WzObjectType::Property(prop) => match prop {
      WzSubProperty::PNG(png_prop) => {
        // Try to extract the image
        let format = png_prop.format();
        let width = png_prop.width;
        let height = png_prop.height;

        view()
          .style(|s| s.flex().flex_col().gap(Spacing::MD)) // 12px gap
          .child((
            // Info text
            view()
              .style(|s| s.flex().flex_col().gap(Spacing::XS)) // 4px gap
              .child((
                text(format!("PNG Image (Format: {})", format)).style(|s| {
                  s.font_size(FontSize::MD)
                    .font_weight(600)
                    .color(TEXT_PRIMARY)
                }), // 14px
                text(format!("Size: {}x{} pixels", width, height))
                  .style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)), // 12px
              )),
            // Try to render the image
            view().child(dynamic(move || {
              // Extract the PNG
              match png::get_image(&node) {
                Ok(dynamic_img) => {
                  // Convert to Image component
                  let img = Image::from_dynamic_image(dynamic_img.clone());

                  // Limit display size to reasonable dimensions
                  let display_width = width.min(600) as f32;
                  let display_height = height.min(600) as f32;

                  // Clone variables for the button handler
                  let node_for_button = node.clone();
                  let dynamic_img_for_button = dynamic_img.clone();

                  view()
                    .style(|s| s.flex().flex_col().gap(Spacing::MD)) // 12px gap
                    .child((
                      // Image display
                      view()
                        .style(|s| {
                          s.padding(Spacing::MD) // 12px padding
                            .background(PANEL_HEADER_BG)
                            .border_radius(Radius::LG) // 8px - container radius
                            .border_all(Border::THIN, BORDER_COLOR) // 1px border
                        })
                        .child(
                          img
                            .size(display_width, display_height)
                            .style(|s| s.border_radius(Radius::SM)), // 4px - inner radius
                        ),
                      // Copy button - using component defaults
                      button(format!("Copy to clipboard [{}]", format))
                        .style(|s| s.width(250.0))
                        .on_click(move |_| {
                          let node_data = node_for_button.read().unwrap();
                          let full_path = node_data.get_full_path();
                          drop(node_data);

                          // 生成文件名：将路径中的 "/" 替换为 "_"
                          let filename = std::env::temp_dir()
                            .join(format!("{}.png", full_path.replace("/", "_")));

                          // 保存 PNG 文件到 tmp 目录
                          let image_rgba = dynamic_img_for_button.clone().into_rgba8();
                          if let Ok(_) = image_rgba.save(&filename) {
                            // 使用 set_files 设置文件路径，这样粘贴时可以带上路径信息
                            if let Ok(clipboard) = ClipboardContext::new() {
                              let _ =
                                clipboard.set_files(vec![filename.to_string_lossy().to_string()]);
                              println!("Image copied to clipboard: {:?}", filename);
                            } else {
                              eprintln!("Failed to create clipboard context");
                            }
                          } else {
                            eprintln!("Failed to save image to temp file");
                          }
                        }),
                    ))
                }
                Err(e) => {
                  view().child(
                    text(format!("Failed to load image: {}", e))
                      .style(|s| s.font_size(FontSize::SM).color(ERROR_COLOR)), // 12px red
                  )
                }
              }
            })),
          ))
      }
      WzSubProperty::Sound(sound_prop) => {
        view()
          .style(|s| s.flex().flex_col().gap(Spacing::SM)) // 8px gap
          .child((
            text(format!("Sound: {:?}", sound_prop.sound_type))
              .style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
            text(format!("Duration: {} ms", sound_prop.duration))
              .style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)), // 12px
            text("Audio playback not yet implemented")
              .style(|s| s.font_size(FontSize::XS).color(TEXT_SECONDARY)), // 11px
          ))
      }
      WzSubProperty::Property => {
        // For Property type, show JSON representation
        let node_data = node.read().unwrap();

        // Only show JSON if it has children
        if node_data.children.is_empty() {
          view().child(
            text("Property Container (Empty)")
              .style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)), // 12px
          )
        } else {
          let json_str = node_to_json(&node_data);
          drop(node_data);

          view()
            .style(|s| {
              s.flex()
                .flex_col()
                .gap(Spacing::SM)
                .w_full() // 确保容器填充父元素宽度
                .min_width(0.0)
            }) // 8px gap
            .child((
              text("Property Container (JSON)").style(|s| {
                s.font_size(FontSize::MD)
                  .font_weight(600)
                  .color(TEXT_PRIMARY)
              }), // 14px
              view()
                .style(|s| {
                  s.padding(Spacing::MD) // 12px padding
                    .background(PANEL_HEADER_BG)
                    .border_radius(Radius::LG) // 8px
                    .border_all(Border::THIN, BORDER_COLOR) // 1px
                    .overflow_y_scroll()
                    .overflow_x_scroll() // Allow horizontal scrolling for long lines
                    .max_height(400.0)
                    .flex() // Use flex layout
                    .w_full()
                    .min_width(0.0) // Allow container to shrink
                })
                .child(text(json_str).style(|s| {
                  s.font_size(FontSize::XS) // 11px for code
                    .color(TEXT_PRIMARY)
                    .line_height(1.5)
                    .flex_grow(1.0) // Make text fill available space
                    .min_width(0.0) // Allow text to shrink
                })),
            ))
        }
      }
      WzSubProperty::Convex => {
        view().child(
          text("Convex Property").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
        )
      }
    },
    WzObjectType::Directory(_) => {
      view().child(
        text("Directory").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
      )
    }
    WzObjectType::Image(_) => {
      view().child(
        text("Image Container").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
      )
    }
    WzObjectType::File(_) => {
      view().child(
        text("File").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
      )
    }
  }
}

/// Main application view
fn app_view() -> View {
  // Create portal container (must be at root level)
  let portal_root = sig::portal::provide();

  let state = AppState::new();

  // Use use_resource to load WZ file asynchronously
  let wz_file_path = "./Data/Base.wz";
  let root_node_signal = state.root_node.clone();
  let error_signal = state.error_message.clone();

  println!("📂 [Startup] Loading WZ file: {}", wz_file_path);

  let wz_resource = use_resource(move || {
    let path = wz_file_path.to_string();
    async move {
      println!("🔄 [Resource] Starting WZ file load...");
      let start = std::time::Instant::now();
      
      // This runs in a background task, not blocking the UI
      let result = match resolve_base(&path, None) {
        Ok(node) => {
          let elapsed = start.elapsed();
          println!("✅ [Resource] WZ file loaded successfully in {:?}", elapsed);
          Ok(node)
        },
        Err(e) => {
          let elapsed = start.elapsed();
          eprintln!("❌ [Resource] Failed to load WZ file after {:?}: {}", elapsed, e);
          Err(format!("Failed to load WZ file: {}", e))
        }
      };
      
      result
    }
  });

  // Update state when resource is ready
  create_effect(move || {
    if wz_resource.ready() {
      println!("📝 [State] Resource ready, updating state...");
      if let Some(result) = wz_resource.value() {
        match result {
          Ok(node) => {
            println!("✅ [State] Root node set successfully");
            *root_node_signal.write() = Some(WzTreeNode(node));
            *error_signal.write() = None;
          }
          Err(err) => {
            eprintln!("❌ [State] Setting error: {}", err);
            *error_signal.write() = Some(err);
          }
        }
      }
    }
  });

  // Setup search effect
  println!("🔧 [Startup] Setting up search effect...");
  setup_search_effect(&state);

  let error_display = state.error_message.clone();

  view()
    .style(|s| s.w_full().h_full().flex().flex_col().background(BG_COLOR))
    .child((dynamic(move || {
      // Show error if any
      if let Some(error) = error_display.read().as_ref() {
        view()
          .style(|s| s.w_full().h_full().flex().items_center().justify_center())
          .child(
            view()
              .style(|s| {
                s.flex()
                  .flex_col()
                  .gap(Spacing::MD)
                  .padding(Spacing::XL)
                  .background(PANEL_BG)
                  .border_radius(Radius::LG)
                  .border_all(Border::THIN, ERROR_COLOR)
              })
              .child((
                text("Error Loading WZ File").style(|s| {
                  s.font_size(FontSize::LG)
                    .font_weight(600)
                    .color(ERROR_COLOR)
                }),
                text(error.clone()).style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)),
              )),
          )
      } else {
        view()
          .style(|s| s.w_full().h_full().flex().flex_row())
          .child((
            // Left resizable panel
            Resizable::horizontal(state.left_width.clone())
              .min_size(200.0)
              .max_size(600.0)
              .child(left_panel(&state)),
            // Right panel
            right_panel(&state),
          ))
      }
    }),
    portal_root
  ))
}

fn main() -> anyhow::Result<()> {
  println!("🚀 ============================================");
  println!("🚀 WZ Editor Starting...");
  println!("🚀 ============================================");
  
  let result = run(
    AppConfig {
      title: "WZ Editor (Sig Framework)".to_string(),
      width: 1200.0,
      height: 800.0,
    },
    app_view,
  );
  
  println!("👋 WZ Editor shutting down");
  result
}
