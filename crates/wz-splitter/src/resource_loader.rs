use crate::{Manifest, SplitReaderError};
use anyhow::Result;
use std::path::{Path, PathBuf};

// ResourceLoader trait - 资源加载抽象
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait ResourceLoader: Send + Sync {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError>;
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError>;
    async fn object_exists(&self, hash: &str) -> bool;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait]
pub trait ResourceLoader: Send + Sync {
    async fn load_manifest(&self) -> Result<Manifest, SplitReaderError>;
    async fn load_object(&self, hash: &str) -> Result<Vec<u8>, SplitReaderError>;
    async fn object_exists(&self, hash: &str) -> bool;
}

// LocalResourceLoader - 本地文件系统加载器
pub struct LocalResourceLoader {
    split_dir: PathBuf,
}

// 在 wasm32 环境中单线程，因此是安全的
unsafe impl Send for LocalResourceLoader {}
unsafe impl Sync for LocalResourceLoader {}

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

// HttpResourceLoader - 基于 web-fetch 的网络加载器
pub struct HttpResourceLoader {
    base_url: String,
}

// 在 wasm32 环境中单线程，因此是安全的
unsafe impl Send for HttpResourceLoader {}
unsafe impl Sync for HttpResourceLoader {}

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