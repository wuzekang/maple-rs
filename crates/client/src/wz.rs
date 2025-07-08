use glam::Vec2;
use image::DynamicImage;
use indexmap::{Equivalent, IndexMap};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::num::ParseIntError;
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};
use weak_table::PtrWeakKeyHashMap;
use wz_parser::node::Error;
use wz_parser::{property::Vector2D, WzNodeArc};
use wz_parser::{WzNode, WzNodeCast, WzNodeName};

#[derive(Clone)]
pub struct Node {
  pub wz_node: WzNodeArc,
}

impl From<WzNodeArc> for Node {
  fn from(val: WzNodeArc) -> Self {
    Node { wz_node: val }
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NodeName {
  pub wz_name: WzNodeName,
}

impl Equivalent<NodeName> for str {
  fn equivalent(&self, key: &NodeName) -> bool {
    self == key.as_str()
  }
}

impl From<WzNodeName> for NodeName {
  fn from(val: WzNodeName) -> Self {
    NodeName { wz_name: val }
  }
}

impl NodeName {
  pub fn to_string(&self) -> String {
    self.wz_name.to_string()
  }
  pub fn as_str(&self) -> &str {
    self.wz_name.as_str()
  }
}

impl Node {
  pub fn at_path(&self, path: &str) -> Result<Node, Error> {
    if path.is_empty() {
      return Err(Error::NodeNotFound);
    }

    let paths = path.split("/").collect::<Vec<_>>();

    if paths.len() == 1 && !path.ends_with(".img") {
      return Ok(self.get(path));
    }

    let mut paths = paths
      .into_iter()
      .fold(VecDeque::from(["".to_string()]), |mut paths, v| {
        let last = paths.back_mut().unwrap();
        if !last.is_empty() {
          *last += "/";
        }
        *last += v;
        if v.ends_with(".img") {
          paths.push_back("".to_string());
        }
        paths
      });

    if paths.back().unwrap() == "" {
      paths.pop_back();
    }

    let Self { wz_node } = self;
    let first = paths.pop_front().unwrap();
    let mut current = wz_node
      .read()
      .unwrap()
      .at_path(&first)
      .ok_or(Error::NodeNotFound)?;
    if first.ends_with(".img") {
      wz_parser::util::node_util::parse_node(&current)?;
    }
    for path in paths {
      let node = current
        .read()
        .unwrap()
        .at_path(&path)
        .ok_or(Error::NodeNotFound)?;
      if path.ends_with(".img") {
        wz_parser::util::node_util::parse_node(&node)?;
      }
      current = node;
    }

    Ok(current.into())
  }
  pub fn get(&self, name: &str) -> Node {
    self.try_get(name).unwrap()
  }
  pub fn try_get(&self, name: &str) -> Option<Node> {
    let node = self.wz_node.read().unwrap();
    let node: Node = node.children.get(name)?.clone().into();
    Some(node)
  }
  pub fn children(&self) -> IndexMap<NodeName, Node> {
    let node = self.wz_node.read().unwrap();
    node
      .children
      .iter()
      .map(|(k, v)| (k.clone().into(), v.clone().into()))
      .collect()
  }
  pub fn parse(&self) -> &Self {
    wz_parser::util::node_util::parse_node(&self.wz_node).unwrap();
    self
  }
  pub fn has(&self, name: &str) -> bool {
    self.wz_node.read().unwrap().children.contains_key(name)
  }
  pub fn path(&self) -> String {
    self.wz_node.read().unwrap().get_full_path().to_string()
  }
}

impl TryFrom<Node> for Vec2 {
  type Error = ();

  fn try_from(node: Node) -> Result<Self, Self::Error> {
    let node = node.wz_node.read().unwrap();
    let Vector2D(x, y) = node.try_as_vector2d().ok_or(())?;
    Ok(Vec2 {
      x: *x as f32,
      y: *y as f32,
    })
  }
}

impl TryFrom<Node> for i32 {
  type Error = ();

  fn try_from(node: Node) -> Result<Self, Self::Error> {
    let v = node.wz_node.read().unwrap();
    v.try_as_int()
      .copied()
      .or_else(|| v.try_as_string()?.get_string().ok()?.parse().ok())
      .ok_or(())
  }
}

impl TryFrom<Node> for f32 {
  type Error = ();

  fn try_from(node: Node) -> Result<Self, Self::Error> {
    let value: i32 = node.try_into()?;
    Ok(value as f32)
  }
}

impl TryFrom<Node> for String {
  type Error = ();

  fn try_from(node: Node) -> Result<Self, Self::Error> {
    node
      .wz_node
      .read()
      .unwrap()
      .try_as_string()
      .ok_or(())?
      .get_string()
      .or(Err(()))
  }
}

impl TryFrom<Node> for DynamicImage {
  type Error = ();

  fn try_from(node: Node) -> Result<Self, Self::Error> {
    node
      .wz_node
      .read()
      .unwrap()
      .try_as_png()
      .ok_or(())?
      .extract_png()
      .or(Err(()))
  }
}

impl TryFrom<Node> for Arc<DynamicImage> {
  type Error = ();

  fn try_from(node: Node) -> Result<Self, Self::Error> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<DynamicImage>>>> = OnceLock::new();
    let path = node.wz_node.read().unwrap().get_full_path();
    let value = CACHE
      .get_or_init(|| Mutex::new(HashMap::new()))
      .lock()
      .unwrap()
      .entry(path)
      .or_insert_with(|| Arc::new(node.try_into().unwrap()))
      .clone();

    Ok(value)
  }
}

impl TryFrom<Node> for bool {
  type Error = ();
  fn try_from(node: Node) -> Result<Self, Self::Error> {
    let value: i32 = node.try_into()?;
    Ok(value != 0)
  }
}

impl<T: TryFrom<Node>> TryFrom<Node> for Vec<T> {
  type Error = ();

  fn try_from(value: Node) -> Result<Self, Self::Error> {
    Ok(
      value
        .children()
        .into_iter()
        .filter(|(key, _)| key.to_string().parse::<u32>().is_ok())
        .filter_map(|(_, node)| node.try_into().ok())
        .collect(),
    )
  }
}

impl TryFrom<NodeName> for i32 {
  type Error = ParseIntError;
  fn try_from(key: NodeName) -> Result<Self, Self::Error> {
    key.wz_name.to_string().parse::<i32>()
  }
}

impl From<NodeName> for String {
  fn from(key: NodeName) -> Self {
    key.wz_name.to_string()
  }
}

impl<T: TryFrom<Node>, K: TryFrom<NodeName>> TryFrom<Node> for Vec<(K, T)> {
  type Error = ();

  fn try_from(value: Node) -> Result<Self, Self::Error> {
    Ok(
      value
        .children()
        .into_iter()
        .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.try_into().ok()?)))
        .collect(),
    )
  }
}

impl<T: TryFrom<Node>, K: TryFrom<NodeName> + Hash + Eq> TryFrom<Node> for HashMap<K, T> {
  type Error = ();

  fn try_from(value: Node) -> Result<Self, Self::Error> {
    Ok(
      value
        .children()
        .into_iter()
        .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.try_into().ok()?)))
        .collect(),
    )
  }
}

impl<T: TryFrom<Node>, K: TryFrom<NodeName> + Hash + Eq> TryFrom<Node> for IndexMap<K, T> {
  type Error = ();

  fn try_from(value: Node) -> Result<Self, Self::Error> {
    Ok(
      value
        .children()
        .into_iter()
        .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.try_into().ok()?)))
        .collect(),
    )
  }
}

impl From<&wz_splitter::reader::NodeHandle> for Node {
  fn from(handle: &wz_splitter::reader::NodeHandle) -> Self {
    Node {
      wz_node: handle.raw_node().clone(),
    }
  }
}

impl From<wz_splitter::reader::NodeHandle> for Node {
  fn from(handle: wz_splitter::reader::NodeHandle) -> Self {
    Node {
      wz_node: handle.raw_node().clone(),
    }
  }
}

// WzSplitReader 扩展 trait
pub trait WzSplitReaderExt {
  async fn get_node(&self, path: &str) -> Result<Node, wz_splitter::reader::SplitReaderError>;
}

impl WzSplitReaderExt for Arc<wz_splitter::reader::SplitWzReader> {
  async fn get_node(&self, path: &str) -> Result<Node, wz_splitter::reader::SplitReaderError> {
    let handle = self.get(path).await?;
    Ok(Node::from(handle))
  }
}
