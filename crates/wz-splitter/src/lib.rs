mod splitter;
mod inspector;
pub mod reader;

pub use splitter::WzSplitter;
pub use inspector::WzInspector;
pub use reader::{SplitWzReader, NodeHandle, NodeValue, ResourceLoader, SplitReaderError};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manifest 文件结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub imgs: ManifestNode,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ManifestNode {
    File(String),  // SHA-256 hash (hex)
    Directory(HashMap<String, ManifestNode>),
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            imgs: ManifestNode::Directory(HashMap::new()),
        }
    }
    
    pub fn add_img(&mut self, path: String, hash: String) {
        let parts: Vec<&str> = path.split('/').collect();
        if let ManifestNode::Directory(root) = &mut self.imgs {
            Self::insert_hash(root, &parts, hash);
        }
    }
    
    fn insert_hash(node: &mut HashMap<String, ManifestNode>, parts: &[&str], hash: String) {
        if parts.is_empty() {
            return;
        }
        
        if parts.len() == 1 {
            // 叶子节点，插入文件
            node.insert(parts[0].to_string(), ManifestNode::File(hash));
        } else {
            // 中间节点，确保目录存在
            let dir = node.entry(parts[0].to_string())
                .or_insert_with(|| ManifestNode::Directory(HashMap::new()));
            
            if let ManifestNode::Directory(subdir) = dir {
                Self::insert_hash(subdir, &parts[1..], hash);
            }
        }
    }
    
    pub fn find_hash(&self, path: &str) -> Option<&str> {
        let parts: Vec<&str> = path.split('/').collect();
        Self::find_in_node(&self.imgs, &parts)
    }
    
    fn find_in_node<'a>(node: &'a ManifestNode, parts: &[&str]) -> Option<&'a str> {
        if parts.is_empty() {
            return None;
        }
        
        match node {
            ManifestNode::File(hash) => {
                if parts.len() == 1 {
                    Some(hash)
                } else {
                    None
                }
            }
            ManifestNode::Directory(map) => {
                if let Some(child) = map.get(parts[0]) {
                    if parts.len() == 1 {
                        match child {
                            ManifestNode::File(hash) => Some(hash),
                            _ => None,
                        }
                    } else {
                        Self::find_in_node(child, &parts[1..])
                    }
                } else {
                    None
                }
            }
        }
    }
    
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
    
    /// 获取所有文件的迭代器 (path, hash)
    pub fn iter_files(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();
        Self::collect_files(&self.imgs, String::new(), &mut result);
        result
    }
    
    fn collect_files(node: &ManifestNode, prefix: String, result: &mut Vec<(String, String)>) {
        match node {
            ManifestNode::File(hash) => {
                // prefix 已经包含了文件名
                result.push((prefix, hash.clone()));
            }
            ManifestNode::Directory(map) => {
                for (name, child) in map {
                    let path = if prefix.is_empty() {
                        name.clone()
                    } else {
                        format!("{}/{}", prefix, name)
                    };
                    Self::collect_files(child, path, result);
                }
            }
        }
    }
    
    /// 计算总文件数
    pub fn count_files(&self) -> usize {
        Self::count_in_node(&self.imgs)
    }
    
    fn count_in_node(node: &ManifestNode) -> usize {
        match node {
            ManifestNode::File(_) => 1,
            ManifestNode::Directory(map) => {
                map.values().map(|child| Self::count_in_node(child)).sum()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_structure() {
        let mut manifest = Manifest::new();
        manifest.add_img("Map/Map0/000000000.img".to_string(), "a1b2c3d4".to_string());
        manifest.add_img("Map/Map0/000010000.img".to_string(), "e5f6g7h8".to_string());
        manifest.add_img("UI/Basic.img".to_string(), "i9j0k1l2".to_string());
        
        assert_eq!(manifest.find_hash("Map/Map0/000000000.img"), Some("a1b2c3d4"));
        assert_eq!(manifest.find_hash("UI/Basic.img"), Some("i9j0k1l2"));
        assert_eq!(manifest.find_hash("UI/NotExist.img"), None);
    }
    
    #[test]
    fn test_manifest_serialization() {
        let mut manifest = Manifest::new();
        manifest.add_img("Map/Map0/test.img".to_string(), "hash123".to_string());
        
        let json = manifest.to_json().unwrap();
        let manifest2 = Manifest::from_json(&json).unwrap();
        
        assert_eq!(manifest2.version, "1.0");
        assert_eq!(manifest2.find_hash("Map/Map0/test.img"), Some("hash123"));
    }
}