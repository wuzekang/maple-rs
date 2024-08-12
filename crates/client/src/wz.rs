use glam::Vec2;
use image::DynamicImage;
use indexmap::{Equivalent, IndexMap};
use std::collections::{HashMap, VecDeque};
use std::num::ParseIntError;
use std::ops::Not;
use std::sync::{Arc, Mutex, OnceLock};
use wz_reader::node::Error;
use wz_reader::{property::Vector2D, WzNodeArc};
use wz_reader::{WzNodeCast, WzNodeName};

pub fn resolve_base() -> Result<Node, std::io::Error> {
    let wz_node = wz_reader::util::resolve_base("./Data/Base.wz", None)?;
    Ok(wz_node.into())
}

#[derive(Clone)]
pub struct Node {
    pub wz_node: WzNodeArc,
}

impl Into<Node> for WzNodeArc {
    fn into(self) -> Node {
        Node { wz_node: self }
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

impl Into<NodeName> for WzNodeName {
    fn into(self) -> NodeName {
        NodeName { wz_name: self }
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

        let mut paths = path.split("/").collect::<Vec<_>>();

        if paths.len() == 1 && !path.ends_with(".img") {
            return Ok(self.get(path));
        }

        let mut paths = paths
            .into_iter()
            .fold(VecDeque::from(["".to_string()]), |mut paths, v| {
                let last = paths.back_mut().unwrap();
                if last.len() > 0 {
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
            wz_reader::util::node_util::parse_node(&current)?;
        }
        for path in paths {
            let node = current
                .read()
                .unwrap()
                .at_path(&path)
                .ok_or(Error::NodeNotFound)?;
            if path.ends_with(".img") {
                wz_reader::util::node_util::parse_node(&node)?;
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
        node.children
            .iter()
            .map(|(k, v)| (k.clone().into(), v.clone().into()))
            .collect()
    }
    pub fn parse(&self) -> &Self {
        wz_reader::util::node_util::parse_node(&self.wz_node).unwrap();
        self
    }
    pub fn has(&self, name: &str) -> bool {
        self.wz_node.read().unwrap().children.contains_key(name)
    }
    pub fn path(&self) -> String {
        self.wz_node.read().unwrap().get_full_path().to_string()
    }
}

impl From<Node> for Vec2 {
    fn from(node: Node) -> Vec2 {
        let node = node.wz_node.read().unwrap();
        let Vector2D(x, y) = node.try_as_vector2d().unwrap();
        Vec2 {
            x: *x as f32,
            y: *y as f32,
        }
    }
}

impl From<Node> for i32 {
    fn from(node: Node) -> i32 {
        node.wz_node
            .read()
            .unwrap()
            .try_as_int()
            .unwrap()
            .to_owned()
    }
}

impl From<Node> for String {
    fn from(node: Node) -> String {
        node.wz_node
            .read()
            .unwrap()
            .try_as_string()
            .unwrap()
            .to_owned()
            .get_string()
            .unwrap()
    }
}

impl From<Node> for DynamicImage {
    fn from(node: Node) -> DynamicImage {
        node.wz_node
            .read()
            .unwrap()
            .try_as_png()
            .unwrap()
            .to_owned()
            .extract_png()
            .unwrap()
    }
}

impl From<Node> for Arc<DynamicImage> {
    fn from(node: Node) -> Self {
        static CACHE: OnceLock<Mutex<HashMap<String, Arc<DynamicImage>>>> = OnceLock::new();
        let path = node.wz_node.read().unwrap().get_full_path();
        CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap()
            .entry(path)
            .or_insert_with(|| Arc::new(node.into()))
            .clone()
    }
}

impl From<Node> for bool {
    fn from(node: Node) -> Self {
        let value: i32 = node.into();
        value != 0
    }
}

impl<T: From<Node>> From<Node> for Vec<T> {
    fn from(value: Node) -> Self {
        value
            .children()
            .into_iter()
            .filter(|(key, _)| key.to_string().parse::<u32>().is_ok())
            .map(|(_, node)| node.into())
            .collect()
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

impl<T: From<Node>, K: TryFrom<NodeName>> From<Node> for Vec<(K, T)> {
    fn from(node: Node) -> Self {
        node.children()
            .into_iter()
            .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.into())))
            .collect()
    }
}

impl<T: From<Node>, K: TryFrom<NodeName> + std::hash::Hash + std::cmp::Eq> From<Node>
    for HashMap<K, T>
{
    fn from(node: Node) -> Self {
        node.children()
            .into_iter()
            .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.into())))
            .collect()
    }
}

impl<T: From<Node>, K: TryFrom<NodeName> + std::hash::Hash + std::cmp::Eq> From<Node>
    for IndexMap<K, T>
{
    fn from(node: Node) -> Self {
        node.children()
            .into_iter()
            .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.into())))
            .collect()
    }
}
