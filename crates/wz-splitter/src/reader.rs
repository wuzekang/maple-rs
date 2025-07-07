use crate::Manifest;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use wz_parser::{WzNode, WzNodeArc, WzObjectType, WzReader, WzImage, WzNodeName};
use std::sync::RwLock;
use lru::LruCache;

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

// ResourceLoader trait - 资源加载抽象
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait ResourceLoader: Send + Sync {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError>;
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError>;
    async fn object_exists(&self, hash: &str) -> bool;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait ResourceLoader {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError>;
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError>;
    async fn object_exists(&self, hash: &str) -> bool;
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
}

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

// LocalResourceLoader - 本地文件系统加载器
pub struct LocalResourceLoader {
    split_dir: PathBuf,
}

impl LocalResourceLoader {
    pub fn new(split_dir: impl AsRef<Path>) -> Self {
        Self {
            split_dir: split_dir.as_ref().to_path_buf(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl ResourceLoader for LocalResourceLoader {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError> {
        let manifest_path = self.split_dir.join("manifest.json");
        let content = tokio::fs::read_to_string(manifest_path).await?;
        Manifest::from_json(&content)
            .map_err(|e| SplitReaderError::ManifestParseError(e))
    }
    
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError> {
        let subdir = &hash[..2];
        let object_path = self.split_dir
            .join("objects")
            .join(subdir)
            .join(hash);
        
        tokio::fs::read(object_path)
            .await
            .map_err(|_| SplitReaderError::ObjectNotFound(hash.to_string()))
    }
    
    async fn object_exists(&self, hash: &str) -> bool {
        let subdir = &hash[..2];
        let object_path = self.split_dir
            .join("objects")
            .join(subdir)
            .join(hash);
        
        object_path.exists()
    }
}

// 在 WASM 环境下，LocalResourceLoader 不可用
#[cfg(target_arch = "wasm32")]
impl LocalResourceLoader {
    pub fn new(_split_dir: impl AsRef<Path>) -> Self {
        panic!("LocalResourceLoader is not available in WASM environment");
    }
}

// HttpResourceLoader - 基于 web-fetch 的网络加载器
pub struct HttpResourceLoader {
    base_url: String,
}

impl HttpResourceLoader {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl ResourceLoader for HttpResourceLoader {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError> {
        let url = format!("{}/manifest.json", self.base_url);
        
        let response = web_fetch::WebFetch::get(&url)
            .await
            .map_err(|e| SplitReaderError::NetworkError(format!("{:?}", e)))?;
        
        if response.status != 200 {
            return Err(SplitReaderError::NetworkError(
                format!("HTTP {}: {}", response.status, response.status_text)
            ));
        }
        
        let content = String::from_utf8(response.data)
            .map_err(|e| SplitReaderError::ManifestParseError(anyhow::anyhow!("Invalid UTF-8: {}", e)))?;
        
        Manifest::from_json(&content)
            .map_err(|e| SplitReaderError::ManifestParseError(e))
    }
    
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError> {
        let subdir = &hash[..2];
        let url = format!("{}/objects/{}/{}", self.base_url, subdir, hash);
        
        let response = web_fetch::WebFetch::get(&url)
            .await
            .map_err(|e| SplitReaderError::NetworkError(format!("{:?}", e)))?;
        
        if response.status != 200 {
            return Err(SplitReaderError::ObjectNotFound(hash.to_string()));
        }
        
        Ok(response.data)
    }
    
    async fn object_exists(&self, _hash: &str) -> bool {
        // 对于 HTTP，我们依赖 manifest 或在加载时处理 404
        true
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl ResourceLoader for HttpResourceLoader {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError> {
        let url = format!("{}/manifest.json", self.base_url);
        
        let response = web_fetch::WebFetch::get(&url)
            .await
            .map_err(|e| SplitReaderError::NetworkError(format!("{:?}", e)))?;
        
        if response.status != 200 {
            return Err(SplitReaderError::NetworkError(
                format!("HTTP {}: {}", response.status, response.status_text)
            ));
        }
        
        let content = String::from_utf8(response.data)
            .map_err(|e| SplitReaderError::ManifestParseError(anyhow::anyhow!("Invalid UTF-8: {}", e)))?;
        
        Manifest::from_json(&content)
            .map_err(|e| SplitReaderError::ManifestParseError(e))
    }
    
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError> {
        let subdir = &hash[..2];
        let url = format!("{}/objects/{}/{}", self.base_url, subdir, hash);
        
        let response = web_fetch::WebFetch::get(&url)
            .await
            .map_err(|e| SplitReaderError::NetworkError(format!("{:?}", e)))?;
        
        if response.status != 200 {
            return Err(SplitReaderError::ObjectNotFound(hash.to_string()));
        }
        
        Ok(response.data)
    }
    
    async fn object_exists(&self, _hash: &str) -> bool {
        // 对于 HTTP，我们依赖 manifest 或在加载时处理 404
        true
    }
}

// SplitWzReader 实现
impl SplitWzReader {
    /// 创建新的 reader，使用自定义加载器
    pub async fn new(loader: Box<dyn ResourceLoader>) -> Result<Self, SplitReaderError> {
        let manifest = loader.load_manifest().await?;
        
        Ok(Self {
            loader,
            manifest,
            img_cache: Arc::new(Mutex::new(LruCache::new(100.try_into().unwrap()))),
            wz_iv: None,
        })
    }
    
    /// 从本地目录创建
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn from_local(split_dir: impl AsRef<Path>) -> Result<Self, SplitReaderError> {
        let loader = Box::new(LocalResourceLoader::new(split_dir));
        Self::new(loader).await
    }
    
    /// 从 HTTP URL 创建
    pub async fn from_http(base_url: impl Into<String>) -> Result<Self, SplitReaderError> {
        let loader = Box::new(HttpResourceLoader::new(base_url));
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
        
        let img = WzImage {
            reader: Arc::new(reader),
            name: img_path.into(),
            offset: 0,
            block_size: data_len,
            is_parsed: false,
        };
        
        // 创建节点并解析
        let img_node = WzNode::from_str(img_path, img, None);
        let img_arc = Arc::new(RwLock::new(img_node));
        img_arc.write().unwrap().parse(&img_arc)
            .map_err(SplitReaderError::WzNodeParseError)?;
        
        Ok(img_arc)
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