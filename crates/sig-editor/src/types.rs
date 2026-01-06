use sig::TreeNode;
use wz_parser::{util::node_util, WzNodeArc, WzObjectType};

/// Wrapper for WzNodeArc to implement TreeNode
#[derive(Clone)]
pub struct WzTreeNode(pub WzNodeArc);

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

/// Search result item
#[derive(Clone)]
pub struct SearchResult {
  pub path: String,          // 节点完整路径
  pub node_name: String,     // 节点名称
  pub value_type: String,    // 值类型（"Int", "String", "Vector" 等）
  pub value_display: String, // 值的可读表示
  pub node: WzNodeArc,       // 节点引用（用于定位）
}

impl PartialEq for SearchResult {
  fn eq(&self, other: &Self) -> bool {
    // Compare by Arc pointer equality for the node
    std::sync::Arc::ptr_eq(&self.node, &other.node)
  }
}

/// Serializable value representation
#[derive(Clone)]
pub struct SerializableValue {
  pub type_name: String,
  pub display: String,
}
