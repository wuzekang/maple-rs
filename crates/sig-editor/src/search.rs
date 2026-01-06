use crate::state::AppState;
use crate::types::{SearchResult, SerializableValue};
use std::rc::Rc;
use wz_parser::{property::WzValue, util::node_util, WzNodeArc, WzNodeCast};

/// 搜索 WZ 树中的所有可序列化值（最多返回 50 条）
///
/// 遍历整个 WZ 树，查找所有包含搜索文本的节点值。
/// 使用异步方式避免阻塞 UI。
///
/// **注意**：此函数会解析所有节点（包括未展开的），可能耗时较长。
pub fn search_wz_values(root: &WzNodeArc, search_term: &str) -> Vec<SearchResult> {
  use std::collections::HashSet;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::Mutex;

  let results = Mutex::new(Vec::new());
  let search_term = search_term.to_lowercase();
  let nodes_visited = AtomicUsize::new(0);
  let visited_ids = Mutex::new(HashSet::new());
  let parse_errors = AtomicUsize::new(0);

  println!(
    "  🔎 [Search] Walking tree for term: '{}' (parsing all nodes...)",
    search_term
  );

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
      // let matches = results.lock().unwrap().len();
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
            println!(
              "  ✅ [Search] Found 50 matches, continuing to process remaining nodes in stack..."
            );
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
  println!(
    "  📊 [Search] Complete: visited {} nodes, {} parse errors, found {} matches",
    total_visited,
    total_errors,
    results.len()
  );

  results.truncate(50);

  results
}

/// 提取节点的可序列化值
///
/// 尝试从 WzNode 中提取所有可序列化的值类型。
/// 如果节点不包含可序列化的值，返回 None。
fn extract_serializable_value(node: &wz_parser::WzNode) -> Option<SerializableValue> {
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
      WzValue::String(s) | WzValue::UOL(s) => s.get_string().ok().map(|text| SerializableValue {
        type_name: if matches!(wz_value, WzValue::String(_)) {
          "String"
        } else {
          "UOL"
        }
        .to_string(),
        display: text,
      }),

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
pub fn setup_search_effect(state: &Rc<AppState>) {
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
          println!(
            "✅ [Search] spawn_blocking: found {} results",
            results.len()
          );
          results
        })
        .await
        .unwrap_or_else(|e| {
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
