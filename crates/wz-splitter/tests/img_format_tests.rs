use wz_splitter::Manifest;

#[cfg(test)]
mod manifest_tests {
    use super::*;

    #[test]
    fn test_manifest_structure() {
        let mut manifest = Manifest::new();
        
        // 添加一些测试路径和哈希
        manifest.add_img("Map/Map0/000000000.img".to_string(), 
            "a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890".to_string());
        manifest.add_img("Map/Map0/000010000.img".to_string(), 
            "b2c3d4e5f6789012345678901234567890123456789012345678901234567890ab".to_string());
        manifest.add_img("UI/Basic.img".to_string(), 
            "c3d4e5f6789012345678901234567890123456789012345678901234567890abcd".to_string());
        
        // 测试查找
        assert_eq!(
            manifest.find_hash("Map/Map0/000000000.img"), 
            Some("a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890")
        );
        assert_eq!(
            manifest.find_hash("UI/Basic.img"), 
            Some("c3d4e5f6789012345678901234567890123456789012345678901234567890abcd")
        );
        assert_eq!(manifest.find_hash("NotExist.img"), None);
    }

    #[test]
    fn test_manifest_json_serialization() {
        let mut manifest = Manifest::new();
        manifest.add_img("Test/file1.img".to_string(), "hash1234".to_string());
        manifest.add_img("Test/file2.img".to_string(), "hash5678".to_string());
        
        // 序列化
        let json = manifest.to_json().unwrap();
        assert!(json.contains("\"version\": \"1.0\""));
        assert!(json.contains("hash1234"));
        assert!(json.contains("hash5678"));
        
        // 反序列化
        let manifest2 = Manifest::from_json(&json).unwrap();
        assert_eq!(manifest2.version, "1.0");
        assert_eq!(manifest2.find_hash("Test/file1.img"), Some("hash1234"));
        assert_eq!(manifest2.find_hash("Test/file2.img"), Some("hash5678"));
    }

    #[test]
    fn test_nested_directory_structure() {
        let mut manifest = Manifest::new();
        
        // 测试深层嵌套结构
        manifest.add_img("A/B/C/D/E/file.img".to_string(), "deep_hash".to_string());
        manifest.add_img("A/B/C/file2.img".to_string(), "mid_hash".to_string());
        manifest.add_img("A/file3.img".to_string(), "top_hash".to_string());
        
        assert_eq!(manifest.find_hash("A/B/C/D/E/file.img"), Some("deep_hash"));
        assert_eq!(manifest.find_hash("A/B/C/file2.img"), Some("mid_hash"));
        assert_eq!(manifest.find_hash("A/file3.img"), Some("top_hash"));
        
        // 测试 JSON 序列化保持结构
        let json = manifest.to_json().unwrap();
        let manifest2 = Manifest::from_json(&json).unwrap();
        
        assert_eq!(manifest2.find_hash("A/B/C/D/E/file.img"), Some("deep_hash"));
    }

    #[test]
    fn test_hash_format_validation() {
        let mut manifest = Manifest::new();
        
        // 测试标准的 SHA-256 哈希格式（64个十六进制字符）
        let valid_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        manifest.add_img("test.img".to_string(), valid_hash.to_string());
        
        assert_eq!(manifest.find_hash("test.img"), Some(valid_hash));
        assert_eq!(valid_hash.len(), 64); // SHA-256 哈希应该是64个字符
    }

    #[test]
    fn test_duplicate_paths() {
        let mut manifest = Manifest::new();
        
        // 添加相同路径但不同哈希（模拟文件更新）
        manifest.add_img("UI/Login.img".to_string(), "hash_v1".to_string());
        manifest.add_img("UI/Login.img".to_string(), "hash_v2".to_string());
        
        // 应该保留最后一次的值
        assert_eq!(manifest.find_hash("UI/Login.img"), Some("hash_v2"));
    }
}