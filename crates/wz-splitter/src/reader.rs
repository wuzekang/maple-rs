use crate::{Manifest, resource_loader::ResourceLoader};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use wz_parser::{WzNode, WzNodeArc, WzObjectType, WzReader, WzImage, WzNodeName, WzDirectory};
use std::sync::RwLock;
use lru::LruCache;
use std::collections::HashMap;

// 错误类型定义
#[derive(Debug, thiserror::Error)]
pub enum SplitReaderError {
    #[error("Manifest not found: {0}")]
    ManifestNotFound(PathBuf),
    
    #[error("IMG not found in manifest: {0}")]
    ImgNotInManifest(String),
    
    #[error("Object file not found: {0}")]
    ObjectNotFound(String),
    
    #[error("Path not found: {0}")]
    PathNotFound(String),
    
    #[error("Node is not a directory: {0}")]
    NotADirectory(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Manifest parse error: {0}")]
    ManifestParseError(anyhow::Error),
    
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    
    #[error(transparent)]
    WzParseError(#[from] wz_parser::wz_image::Error),
    
    #[error("WZ node parse error")]
    WzNodeParseError(wz_parser::node::Error),
}


// 主读取器结构
pub struct SplitWzReader {
    /// 资源加载器
    loader: Box<dyn ResourceLoader>,

    
    /// 从 manifest.json 加载的文件映射
    manifest: Manifest,
    
    /// LRU 缓存，缓存已加载的 WzImage
    /// Key: IMG 路径 (如 "UI/Basic.img")
    /// Value: 解析后的 WzNodeArc
    img_cache: Arc<Mutex<LruCache<String, WzNodeArc>>>,
    
    /// 可选的 WZ 版本信息（用于解密）
    wz_iv: Option<[u8; 4]>,
    
    /// 目录节点映射
    /// Key: 目录路径 (如 "Map", "Map/Map0")
    /// Value: 目录节点
    directory_nodes: HashMap<String, WzNodeArc>,
}

// 在非 wasm32 平台上，确保 SplitWzReader 是 Send + Sync
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Send for SplitWzReader {}
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Sync for SplitWzReader {}

// 节点句柄
pub struct NodeHandle {
    /// 当前节点的引用
    node: WzNodeArc,
    
    /// 对 reader 的引用，用于继续加载子节点
    reader: Arc<SplitWzReader>,
    
    /// 当前节点的完整路径（用于调试和错误信息）
    path: String,
}

// 节点值包装
pub enum NodeValue {
    /// 目录节点，可以继续导航
    Directory(NodeHandle),
    
    /// 叶子节点，包含实际数据
    Property(WzNodeArc),
    
    /// IMG 节点，需要加载后才能访问内部
    Image(NodeHandle),
}


// SplitWzReader 实现
impl SplitWzReader {
    /// 创建新的 reader，使用自定义加载器
    pub async fn new(loader: Box<dyn ResourceLoader>) -> Result<Self, SplitReaderError> {
        let manifest = loader.load_manifest().await?;
        
        // 构建目录节点树
        let directory_nodes = Self::build_directory_tree(&manifest);
        
        Ok(Self {
            loader,
            manifest,
            img_cache: Arc::new(Mutex::new(LruCache::new(100.try_into().unwrap()))),
            directory_nodes,
            wz_iv: None,
        })
    }
    
    /// 从本地目录创建
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn from_local(split_dir: impl AsRef<Path>) -> Result<Self, SplitReaderError> {
        let loader = Box::new(crate::resource_loader::LocalResourceLoader::new(split_dir));
        Self::new(loader).await
    }
    
    /// 从 HTTP URL 创建
    pub async fn from_http(base_url: impl Into<String>) -> Result<Self, SplitReaderError> {
        let loader = Box::new(crate::resource_loader::HttpResourceLoader::new(base_url));
        Self::new(loader).await
    }
    
    /// 设置 WZ 版本信息（用于解密）
    pub fn with_iv(mut self, iv: [u8; 4]) -> Self {
        self.wz_iv = Some(iv);
        self
    }
    
    /// 获取根节点或指定路径的节点
    pub async fn get(self: &Arc<Self>, path: &str) -> Result<NodeHandle, SplitReaderError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return Err(SplitReaderError::PathNotFound("Empty path".to_string()));
        }
        
        // 查找第一个 .img 文件的位置
        let img_index = parts.iter().position(|p| p.ends_with(".img"));
        
        if let Some(idx) = img_index {
            // 构建 IMG 路径
            let img_path = parts[..=idx].join("/");

            // 获取或加载 IMG
            let img_node = self.get_or_load_img(&img_path).await?;

            // 如果还有剩余路径，继续导航
            if idx + 1 < parts.len() {
                let remaining_path = parts[idx + 1..].join("/");
                self.navigate_in_node(img_node, &remaining_path, &img_path)
            } else {
                Ok(NodeHandle {
                    node: img_node,
                    reader: Arc::clone(self),
                    path: img_path,
                })
            }
        } else {
            // 没有 .img 文件，这是一个目录路径
            Err(SplitReaderError::PathNotFound(
                format!("No .img file found in path: {}", path)
            ))
        }
    }
    
    /// 内部方法：在节点内导航
    fn navigate_in_node(
        self: &Arc<Self>,
        node: WzNodeArc,
        path: &str,
        base_path: &str,
    ) -> Result<NodeHandle, SplitReaderError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_node = node;
        let mut current_path = base_path.to_string();
        
        for part in parts {
            let next_node = {
                let node_read = current_node.read().unwrap();
                if let Some(child) = node_read.children.get(&WzNodeName::from(part)) {
                    Arc::clone(child)
                } else {
                    return Err(SplitReaderError::PathNotFound(
                        format!("Child '{}' not found in '{}'", part, current_path)
                    ));
                }
            };
            current_node = next_node;
            current_path = format!("{}/{}", current_path, part);
        }
        
        Ok(NodeHandle {
            node: current_node,
            reader: Arc::clone(self),
            path: current_path,
        })
    }
    
    /// 内部方法：查找或加载 IMG
    async fn get_or_load_img(&self, img_path: &str) -> Result<WzNodeArc, SplitReaderError> {
        // 首先检查缓存
        {
            let mut cache = self.img_cache.lock().unwrap();
            if let Some(node) = cache.get(img_path) {
                return Ok(Arc::clone(node));
            }
        }
        
        // 不在缓存中，需要加载
        let img_node = self.load_img(img_path).await?;
        
        // 存入缓存
        {
            let mut cache = self.img_cache.lock().unwrap();
            cache.put(img_path.to_string(), Arc::clone(&img_node));
        }
        
        Ok(img_node)
    }
    
    /// 内部方法：从 objects 目录加载 IMG
    async fn load_img(&self, img_path: &str) -> Result<WzNodeArc, SplitReaderError> {
        // 从 manifest 查找哈希
        let hash = self.manifest.find_hash(img_path)
            .ok_or_else(|| SplitReaderError::ImgNotInManifest(img_path.to_string()))?;

        // 通过 loader 加载数据
        let img_data = self.loader.load_object(hash).await?;
        
        // 记录数据长度
        let data_len = img_data.len();
        
        // 使用 wz-parser 解析 IMG
        let reader = WzReader::new(img_data);
        let reader = if let Some(iv) = self.wz_iv {
            reader.with_iv(iv)
        } else {
            reader
        };
        
        // 获取父目录路径和 IMG 文件名
        let (parent_path, img_name) = match img_path.rfind('/') {
            Some(pos) => (&img_path[..pos], &img_path[pos + 1..]),
            None => ("", img_path),
        };
        
        // 从已构建的目录树中获取父节点
        let parent_node = if !parent_path.is_empty() {
            self.directory_nodes.get(parent_path)
        } else {
            None
        };
        
        let img = WzImage {
            reader: Arc::new(reader),
            name: img_name.into(),
            offset: 0,
            block_size: data_len,
            is_parsed: false,
        };
        
        // 创建节点并解析
        let img_node = WzNode::from_str(img_name, img, parent_node);
        let img_arc = Arc::new(RwLock::new(img_node));
        
        // 如果有父节点，将 IMG 节点添加到父节点的 children 中
        if let Some(parent) = parent_node {
            let mut parent = parent.write().unwrap();
            parent.children.insert(img_name.into(), Arc::clone(&img_arc));
        }
        
        img_arc.write().unwrap().parse(&img_arc)
            .map_err(SplitReaderError::WzNodeParseError)?;
        
        Ok(img_arc)
    }
    
    /// 从 manifest 构建完整的目录节点树
    fn build_directory_tree(manifest: &crate::Manifest) -> HashMap<String, WzNodeArc> {
        let mut directory_nodes = HashMap::new();
        
        // 递归构建目录树
        Self::build_directory_nodes(
            &manifest.imgs,
            "",
            None,
            &mut directory_nodes
        );
        
        directory_nodes
    }
    
    /// 递归构建目录节点
    fn build_directory_nodes(
        node: &crate::ManifestNode,
        current_path: &str,
        parent: Option<&WzNodeArc>,
        directory_nodes: &mut HashMap<String, WzNodeArc>
    ) {
        match node {
            crate::ManifestNode::Directory(children) => {
                for (name, child) in children {
                    let child_path = if current_path.is_empty() {
                        name.clone()
                    } else {
                        format!("{}/{}", current_path, name)
                    };
                    
                    match child {
                        crate::ManifestNode::Directory(_) => {
                            // 创建目录节点
                            let reader = Arc::new(WzReader::new(vec![]));
                            let dir = WzDirectory::new(0, 0, &reader, false);
                            let dir_node = WzNode::from_str(name, dir, parent).into_lock();
                            
                            // 如果有父节点，添加到父节点的 children
                            if let Some(parent_node) = parent {
                                let mut parent_write = parent_node.write().unwrap();
                                parent_write.children.insert(name.as_str().into(), Arc::clone(&dir_node));
                            }
                            
                            // 保存到映射中
                            directory_nodes.insert(child_path.clone(), Arc::clone(&dir_node));
                            
                            // 递归处理子目录
                            Self::build_directory_nodes(
                                child,
                                &child_path,
                                Some(&dir_node),
                                directory_nodes
                            );
                        }
                        crate::ManifestNode::File(_) => {
                            // IMG 文件节点将在实际加载时创建
                            // 这里不需要处理
                        }
                    }
                }
            }
            crate::ManifestNode::File(_) => {
                // 这种情况不应该出现在顶层
            }
        }
    }
    
    /// 获取目录节点
    pub fn get_directory(&self, path: &str) -> Option<&WzNodeArc> {
        self.directory_nodes.get(path)
    }
    
    /// 获取所有顶级目录
    pub fn get_root_directories(&self) -> Vec<(&String, &WzNodeArc)> {
        self.directory_nodes
            .iter()
            .filter(|(path, _)| !path.contains('/'))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manifest;
    
    #[tokio::test]
    async fn test_directory_tree_building() {
        // 创建测试 manifest
        let mut manifest = Manifest::new();
        manifest.add_img("Map/Map0/000000000.img".to_string(), "hash1".to_string());
        manifest.add_img("Map/Map1/000010000.img".to_string(), "hash2".to_string());
        manifest.add_img("UI/Basic.img".to_string(), "hash3".to_string());
        
        // 构建目录树
        let directory_nodes = SplitWzReader::build_directory_tree(&manifest);
        
        // 验证目录节点存在
        assert!(directory_nodes.contains_key("Map"));
        assert!(directory_nodes.contains_key("Map/Map0"));
        assert!(directory_nodes.contains_key("Map/Map1"));
        assert!(directory_nodes.contains_key("UI"));
        
        // 验证父子关系
        let map_node = directory_nodes.get("Map").unwrap();
        let map0_node = directory_nodes.get("Map/Map0").unwrap();
        
        // 检查 Map0 的父节点是否指向 Map
        {
            let map0_read = map0_node.read().unwrap();
            let parent_weak = &map0_read.parent;
            assert!(parent_weak.upgrade().is_some());
            
            // 验证父节点名称
            if let Some(parent_arc) = parent_weak.upgrade() {
                let parent_read = parent_arc.read().unwrap();
                assert_eq!(parent_read.name.as_str(), "Map");
            }
        }
        
        // 检查 Map 节点是否包含 Map0 子节点
        {
            let map_read = map_node.read().unwrap();
            assert!(map_read.children.contains_key(&WzNodeName::from("Map0")));
            assert!(map_read.children.contains_key(&WzNodeName::from("Map1")));
        }
    }
}

// NodeHandle 实现
impl NodeHandle {
    /// 获取子节点
    pub async fn get(&self, path: &str) -> Result<NodeHandle, SplitReaderError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return Err(SplitReaderError::PathNotFound("Empty path".to_string()));
        }
        
        // 检查是否需要跨越 .img 边界
        if parts[0].ends_with(".img") {
            // 需要加载新的 IMG
            let full_path = format!("{}/{}", self.path, path);
            self.reader.get(&full_path).await
        } else {
            // 在当前节点内导航
            self.reader.navigate_in_node(
                Arc::clone(&self.node),
                path,
                &self.path,
            )
        }
    }
    
    /// 获取节点名称
    pub fn name(&self) -> String {
        let node_read = self.node.read().unwrap();
        node_read.name.as_str().to_string()
    }
    
    /// 获取节点类型
    pub fn node_type(&self) -> WzObjectType {
        let node_read = self.node.read().unwrap();
        node_read.object_type.clone()
    }
    
    /// 获取所有子节点名称
    pub fn children_names(&self) -> Vec<String> {
        let node_read = self.node.read().unwrap();
        node_read.children.keys()
            .map(|k| k.as_str().to_string())
            .collect()
    }
    
    /// 尝试获取节点的值
    pub fn value(&self) -> Result<NodeValue, SplitReaderError> {
        let node_read = self.node.read().unwrap();
        
        match &node_read.object_type {
            WzObjectType::Directory(_) => Ok(NodeValue::Directory(NodeHandle {
                node: Arc::clone(&self.node),
                reader: Arc::clone(&self.reader),
                path: self.path.clone(),
            })),
            WzObjectType::Image(_) => Ok(NodeValue::Image(NodeHandle {
                node: Arc::clone(&self.node),
                reader: Arc::clone(&self.reader),
                path: self.path.clone(),
            })),
            _ => Ok(NodeValue::Property(Arc::clone(&self.node))),
        }
    }
    
    /// 获取原始的 WzNode（用于高级用途）
    pub fn raw_node(&self) -> &WzNodeArc {
        &self.node
    }
    
    /// 获取节点的完整路径
    pub fn path(&self) -> &str {
        &self.path
    }
}