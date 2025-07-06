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
#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
use web_fetch::{WebFetch, FetchError, FetchOptions};

#[derive(Clone)]
pub struct WzConfig {
    pub url: Option<String>,
    pub use_network: bool,
    pub fallback_to_preload: bool,
    pub timeout_ms: Option<u32>,
    pub base_url: Option<String>,
    pub additional_wz_files: Option<Vec<String>>,
    pub max_response_size: Option<usize>,
}

impl Default for WzConfig {
    fn default() -> Self {
        Self {
            url: None,
            use_network: true,
            fallback_to_preload: true,
            timeout_ms: Some(30000),
            base_url: None,
            additional_wz_files: None,
            max_response_size: Some(1024 * 1024 * 1024), // 1GB
        }
    }
}

pub async fn resolve_base() -> Result<Node, std::io::Error> {
    resolve_base_with_config(&WzConfig::default()).await
}

pub async fn resolve_base_with_config(config: &WzConfig) -> Result<Node, std::io::Error> {
    #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
    {
        log::debug!("Running in Emscripten environment");

        // Try network loading first if enabled
        if config.use_network {
            let url = config.url.as_deref().unwrap_or("./Data/Base.wz");
            log::debug!("Attempting network loading from: {}", url);

            match resolve_base_from_url_with_config(url, config).await {
                Ok(node) => {
                    // Successfully loaded Base.wz, now load additional WZ files
                    let base_url = config.base_url.as_deref().unwrap_or_else(|| {
                        // Extract base URL from the Base.wz URL
                        if let Some(last_slash) = url.rfind('/') {
                            &url[..last_slash]
                        } else {
                            "."
                        }
                    });

                    // Get the list of additional WZ files to load
                    // Priority files (smaller, load first)
                    let priority_files = vec![
                        "Etc".to_string(),
                        "TamingMob".to_string(),
                        "Morph".to_string(),
                        "Quest".to_string(),
                        "String".to_string(),
                        "Item".to_string(),
                        "UI".to_string(),
                    ];

                    // Large files (load after priority files)
                    let large_files = vec![
                        "Sound".to_string(),  // 366MB - Make sure Sound is included!
                        "Map".to_string(),    // 638MB
                        "Mob".to_string(),    // 479MB
                        "Character".to_string(), // 206MB
                        "Skill".to_string(),  // 76MB
                        "Effect".to_string(), // 63MB
                        "Npc".to_string(),    // 53MB
                        "Reactor".to_string(), // 54MB
                        "List".to_string(),   // 13KB
                    ];

                    // Combine both lists
                    let mut default_files = priority_files;
                    default_files.extend(large_files);

                    let additional_files = config.additional_wz_files.as_ref().unwrap_or(&default_files);

                    let base_node_arc = node.wz_node.clone();

                    // Check which WZ files are referenced in Base.wz
                    let mut files_to_load = Vec::new();
                    {
                        let base_read = base_node_arc.read().unwrap();
                        for wz_name in additional_files {
                            if base_read.at(wz_name).is_some() {
                                files_to_load.push(wz_name.clone());
                            }
                        }
                    }

                    // Load all additional WZ files concurrently
                    let mut load_futures = Vec::new();
                    for wz_name in files_to_load {
                        let wz_url = format!("{}/{}.wz", base_url, wz_name);
                        let base_node = base_node_arc.clone();
                        let config = config.clone();
                        let wz_name_clone = wz_name.clone();
                        load_futures.push(async move {
                            if let Err(e) = load_additional_wz_from_url(&base_node, &wz_name_clone, &wz_url, &config).await {
                                log::warn!("Failed to load {}.wz from network: {}", wz_name_clone, e);
                            }
                        });
                    }

                    // Wait for all loads to complete
                    futures::future::join_all(load_futures).await;


                    return Ok(node);
                }
                Err(e) => {
                    log::warn!("Base.wz network loading failed: {}", e);

                    if !config.fallback_to_preload {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "Network loading failed and fallback to preload is disabled",
                        ));
                    }

                    log::debug!("Network loading failed, falling back to preloaded files");
                }
            }
        }

        // Fall back to preloaded filesystem paths
        let possible_paths = [
            "/Data/Base.wz",
            "Data/Base.wz",
            "./Data/Base.wz",
            "Base.wz"
        ];

        let mut base_node = None;
        let mut base_dir = None;

        for path in &possible_paths {
            match std::fs::read(path) {
                Ok(bytes) => {
                    let wz_node = wz_parser::util::resolve_base_from_bytes(&bytes, None)?;
                    base_node = Some(wz_node);
                    // Extract base directory from path
                    base_dir = Some(std::path::Path::new(path).parent().unwrap_or(std::path::Path::new(".")));
                    break;
                }
                Err(_) => {
                    log::debug!("Failed to read from {}, trying next path", path);
                    continue;
                }
            }
        }

        let base_node = base_node.ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find Base.wz in any expected location. Make sure to preload it with --preload-file or serve it via HTTP",
        ))?;

        // Load additional WZ files that are referenced in Base.wz
        if let Some(base_dir) = base_dir {
            let additional_wz_files = ["UI", "Map", "Character", "Item", "Mob", "Npc", "Skill", "Sound", "Effect"];

            for wz_name in &additional_wz_files {
                let wz_path = base_dir.join(format!("{}.wz", wz_name));

                // Check if this WZ file is referenced in Base.wz
                {
                    let base_read = base_node.read().unwrap();
                    if base_read.at(wz_name).is_none() {
                        continue; // Skip if not referenced in Base.wz
                    }
                }

                if let Ok(bytes) = std::fs::read(&wz_path) {
                    let _ = wz_parser::util::merge_wz_bytes(&base_node, wz_name, &bytes, None);
                }
            }
        }


        return Ok(base_node.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let wz_node = wz_parser::util::resolve_base("./Data/Base.wz", None)?;
        Ok(wz_node.into())
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub async fn resolve_base_from_url(url: &str) -> Result<Node, std::io::Error> {
    resolve_base_from_url_with_config(url, &WzConfig::default()).await
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
async fn load_additional_wz_from_url(base_node: &WzNodeArc, wz_name: &str, url: &str, config: &WzConfig) -> Result<(), std::io::Error> {

    let mut options = FetchOptions::default();
    if let Some(timeout) = config.timeout_ms {
        options.timeout_ms = Some(timeout);
    }
    if let Some(max_size) = config.max_response_size {
        options.max_response_size = Some(max_size);
    }

    match WebFetch::fetch(url, Some(options)).await {
        Ok(response) => {
            if response.status != 200 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP error for {}: {} {}", wz_name, response.status, response.status_text),
                ));
            }


            match wz_parser::util::merge_wz_bytes(base_node, wz_name, &response.data, None) {
                Ok(_) => {
                    Ok(())
                }
                Err(e) => {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to merge {}: {:?}", wz_name, e),
                    ))
                }
            }
        }
        Err(fetch_error) => {
            let error_msg = match fetch_error {
                FetchError::NetworkError(msg) => {
                    // Check if it's a 404 error
                    if msg.contains("404") {
                        format!("File not found (404) for {}.wz at URL: {}", wz_name, url)
                    } else {
                        format!("Network error for {}: {}", wz_name, msg)
                    }
                }
                FetchError::InvalidUrl(url) => format!("Invalid URL for {}: {}", wz_name, url),
            };


            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_msg,
            ))
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub async fn resolve_base_from_url_with_config(url: &str, config: &WzConfig) -> Result<Node, std::io::Error> {

    let mut options = FetchOptions::default();
    if let Some(timeout) = config.timeout_ms {
        options.timeout_ms = Some(timeout);
    }
    if let Some(max_size) = config.max_response_size {
        options.max_response_size = Some(max_size);
    }

    match WebFetch::fetch(url, Some(options)).await {
        Ok(response) => {
            if response.status != 200 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP error: {} {}", response.status, response.status_text),
                ));
            }


            let wz_node = wz_parser::util::resolve_base_from_bytes(&response.data, None)
                .map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse WZ file: {}", e),
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
                error_msg,
            ))
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "emscripten")))]
pub async fn resolve_base_from_url(_url: &str) -> Result<Node, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Network loading is only supported on Emscripten target",
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
