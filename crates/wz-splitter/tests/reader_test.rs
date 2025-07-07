#[cfg(test)]
mod tests {
    use wz_splitter::{SplitWzReader, Manifest, ManifestNode};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::TempDir;
    use std::fs;
    
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_reader_basic() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let split_dir = temp_dir.path();
        
        // 创建 manifest.json
        let mut manifest = Manifest::new();
        manifest.add_img("UI/Basic.img".to_string(), "abcdef1234567890".to_string());
        
        let manifest_json = manifest.to_json().unwrap();
        fs::write(split_dir.join("manifest.json"), manifest_json).unwrap();
        
        // 创建 objects 目录结构
        let objects_dir = split_dir.join("objects");
        fs::create_dir(&objects_dir).unwrap();
        fs::create_dir(objects_dir.join("ab")).unwrap();
        
        // 创建一个假的 IMG 文件（只是占位，实际解析会失败）
        fs::write(objects_dir.join("ab").join("abcdef1234567890"), b"fake img data").unwrap();
        
        // 测试创建 reader
        let reader = SplitWzReader::from_local(split_dir).await;
        assert!(reader.is_ok(), "Failed to create reader: {:?}", reader.err());
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_manifest_operations() {
        let mut manifest = Manifest::new();
        
        // 添加一些 IMG 路径
        manifest.add_img("UI/Basic.img".to_string(), "hash1".to_string());
        manifest.add_img("Map/Map0/000000000.img".to_string(), "hash2".to_string());
        manifest.add_img("Character/00002000.img".to_string(), "hash3".to_string());
        
        // 测试查找
        assert_eq!(manifest.find_hash("UI/Basic.img"), Some("hash1"));
        assert_eq!(manifest.find_hash("Map/Map0/000000000.img"), Some("hash2"));
        assert_eq!(manifest.find_hash("Character/00002000.img"), Some("hash3"));
        assert_eq!(manifest.find_hash("NonExistent.img"), None);
        
        // 测试文件计数
        assert_eq!(manifest.count_files(), 3);
        
        // 测试迭代
        let files = manifest.iter_files();
        assert_eq!(files.len(), 3);
    }
}