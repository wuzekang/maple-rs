use glam::Vec2;
use image::DynamicImage;
use indexmap::{Equivalent, IndexMap};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::num::ParseIntError;
use std::sync::{Arc, Mutex, OnceLock};
use wz_parser::node::Error;
use wz_parser::{property::Vector2D, WzNodeArc};
use wz_parser::{WzNodeCast, WzNodeName};
use web_fetch::{WebFetch, FetchError, FetchOptions};

#[derive(Debug, Clone)]
pub struct WzConfig {
    pub url: Option<String>,
    pub use_network: bool,
    pub fallback_to_preload: bool,
    pub timeout_ms: Option<u32>,
}

impl Default for WzConfig {
    fn default() -> Self {
        Self {
            url: None,
            use_network: true,
            fallback_to_preload: true,
            timeout_ms: Some(30000),
        }
    }
}

pub fn resolve_base() -> Result<Node, std::io::Error> {
    resolve_base_with_config(&WzConfig::default())
}

pub async fn resolve_base_async() -> Result<Node, std::io::Error> {
    resolve_base_with_config_async(&WzConfig::default()).await
}

pub fn resolve_base_with_config(config: &WzConfig) -> Result<Node, std::io::Error> {
    #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
    {
        log::info!("Running in Emscripten environment with config: {:?}", config);
        
        // Try network loading first if enabled
        if config.use_network {
            log::warn!("Synchronous network loading is not supported in Emscripten. Falling back to preloaded files.");
            
            if !config.fallback_to_preload {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Synchronous network loading is not supported and fallback to preload is disabled. Please use resolve_base_async()"
                ));
            }
        }
        
        // Fall back to preloaded filesystem paths
        let possible_paths = [
            "/Data/Base.wz",
            "Data/Base.wz", 
            "./Data/Base.wz",
            "Base.wz"
        ];
        
        for path in &possible_paths {
            match std::fs::read(path) {
                Ok(bytes) => {
                    log::info!("Successfully read {} bytes from {}", bytes.len(), path);
                    let wz_node = wz_parser::util::resolve_base_from_bytes(&bytes, None)?;
                    return Ok(wz_node.into());
                }
                Err(_) => {
                    log::debug!("Failed to read from {}, trying next path", path);
                    continue;
                }
            }
        }
        
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            "Could not find Base.wz in any expected location. Make sure to preload it with --preload-file or serve it via HTTP"
        ));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        let wz_node = wz_parser::util::resolve_base("./Data/Base.wz", None)?;
        Ok(wz_node.into())
    }
}

pub async fn resolve_base_with_config_async(config: &WzConfig) -> Result<Node, std::io::Error> {
    #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
    {
        log::info!("Running in Emscripten environment with config: {:?}", config);
        
        // Try network loading first if enabled
        if config.use_network {
            let url = config.url.as_deref().unwrap_or("./Data/Base.wz");
            log::info!("Attempting network loading from: {}", url);
            
            if let Ok(node) = resolve_base_from_url_async_with_config(url, config).await {
                return Ok(node);
            }
            
            if !config.fallback_to_preload {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Network loading failed and fallback to preload is disabled"
                ));
            }
            
            log::info!("Network loading failed, falling back to preloaded files");
        }
        
        // Fall back to preloaded filesystem paths
        let possible_paths = [
            "/Data/Base.wz",
            "Data/Base.wz", 
            "./Data/Base.wz",
            "Base.wz"
        ];
        
        for path in &possible_paths {
            match std::fs::read(path) {
                Ok(bytes) => {
                    log::info!("Successfully read {} bytes from {}", bytes.len(), path);
                    let wz_node = wz_parser::util::resolve_base_from_bytes(&bytes, None)?;
                    return Ok(wz_node.into());
                }
                Err(_) => {
                    log::debug!("Failed to read from {}, trying next path", path);
                    continue;
                }
            }
        }
        
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            "Could not find Base.wz in any expected location. Make sure to preload it with --preload-file or serve it via HTTP"
        ));
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 在非 wasm 环境中，直接使用同步版本
        resolve_base_with_config(config)
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub fn resolve_base_from_url(url: &str) -> Result<Node, std::io::Error> {
    resolve_base_from_url_with_config(url, &WzConfig::default())
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub fn resolve_base_from_url_with_config(_url: &str, _config: &WzConfig) -> Result<Node, std::io::Error> {
    // 同步网络请求在 Emscripten 中会阻塞浏览器主线程
    // 请使用异步版本 resolve_base_from_url_async_with_config
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Synchronous network requests are not supported in Emscripten. Please use the async version: resolve_base_from_url_async_with_config"
    ))
}

#[cfg(not(all(target_arch = "wasm32", target_os = "emscripten")))]
pub fn resolve_base_from_url(_url: &str) -> Result<Node, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Network loading is only supported on Emscripten target"
    ))
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub async fn resolve_base_from_url_async(url: &str) -> Result<Node, std::io::Error> {
    resolve_base_from_url_async_with_config(url, &WzConfig::default()).await
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub async fn resolve_base_from_url_async_with_config(url: &str, config: &WzConfig) -> Result<Node, std::io::Error> {
    log::info!("Attempting to load WZ file from URL: {}", url);
    
    let mut options = FetchOptions::default();
    if let Some(timeout) = config.timeout_ms {
        options.timeout_ms = Some(timeout);
    }
    
    match WebFetch::fetch(url, Some(options)).await {
        Ok(response) => {
            if response.status != 200 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP error: {} {}", response.status, response.status_text)
                ));
            }
            
            log::info!("Successfully fetched {} bytes from {}", response.data.len(), url);
            
            let wz_node = wz_parser::util::resolve_base_from_bytes(&response.data, None)
                .map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse WZ file: {}", e)
                ))?;
            
            Ok(wz_node.into())
        }
        Err(fetch_error) => {
            let error_msg = match fetch_error {
                FetchError::NetworkError(msg) => format!("Network error: {}", msg),
                FetchError::InvalidUrl(url) => format!("Invalid URL: {}", url),
            };
            
            log::warn!("Failed to fetch WZ file from {}: {}", url, error_msg);
            
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_msg
            ))
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "emscripten")))]
pub async fn resolve_base_from_url_async(_url: &str) -> Result<Node, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Network loading is only supported on Emscripten target"
    ))
}


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
        node.children
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
        node.wz_node
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
        node.wz_node
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

// impl Into<Arc<DynamicImage>> for Node {
//     fn into(self) -> Arc<DynamicImage> {
//         self.try_into().unwrap()
//     }
// }

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
        Ok(value
            .children()
            .into_iter()
            .filter(|(key, _)| key.to_string().parse::<u32>().is_ok())
            .filter_map(|(_, node)| node.try_into().ok())
            .collect())
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
        Ok(value
            .children()
            .into_iter()
            .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.try_into().ok()?)))
            .collect())
    }
}

impl<T: TryFrom<Node>, K: TryFrom<NodeName> + Hash + Eq> TryFrom<Node> for HashMap<K, T> {
    type Error = ();

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        Ok(value
            .children()
            .into_iter()
            .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.try_into().ok()?)))
            .collect())
    }
}

impl<T: TryFrom<Node>, K: TryFrom<NodeName> + Hash + Eq> TryFrom<Node> for IndexMap<K, T> {
    type Error = ();

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        Ok(value
            .children()
            .into_iter()
            .filter_map(|(key, node)| Some((K::try_from(key).ok()?, node.try_into().ok()?)))
            .collect())
    }
}
