use serde_json::{Map, Value};
use wz_parser::{util::node_util, WzNode, WzNodeArc, WzObjectType};

/// Recursively walk node tree and convert to JSON
pub fn walk_node_and_to_json(node_arc: &WzNodeArc, json: &mut Map<String, Value>) {
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
pub fn node_to_json(node: &WzNode) -> String {
  let mut json = Map::new();

  for value in node.children.values() {
    walk_node_and_to_json(value, &mut json);
  }

  serde_json::to_string_pretty(&Value::Object(json)).unwrap_or_else(|_| "{}".to_string())
}
