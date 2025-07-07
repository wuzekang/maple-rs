use std::fs;
use tempfile::TempDir;
use wz_splitter::Manifest;

#[test]
fn test_manifest_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    
    // 创建一个测试 manifest
    let mut manifest = Manifest::new();
    
    // 添加一些测试路径和哈希
    manifest.add_img(
        "Map/Map0/000000000.img".to_string(),
        "a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890".to_string(),
    );
    
    manifest.add_img(
        "Sound/UI.img".to_string(),
        "b2c3d4e5f6789012345678901234567890123456789012345678901234567890ab".to_string(),
    );
    
    manifest.add_img(
        "Base/String/Mob.img".to_string(),
        "c3d4e5f6789012345678901234567890123456789012345678901234567890abcd".to_string(),
    );
    
    // 保存 manifest
    let manifest_path = temp_dir.path().join("manifest.json");
    let json = manifest.to_json().unwrap();
    fs::write(&manifest_path, json).unwrap();
    
    // 验证文件已创建
    assert!(manifest_path.exists());
    
    // 读取并验证
    let loaded = fs::read_to_string(&manifest_path).unwrap();
    let loaded_manifest = Manifest::from_json(&loaded).unwrap();
    
    // 验证版本
    assert_eq!(loaded_manifest.version, "1.0");
    
    // 验证路径映射
    assert_eq!(
        loaded_manifest.find_hash("Map/Map0/000000000.img"),
        Some("a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890")
    );
    assert_eq!(
        loaded_manifest.find_hash("Sound/UI.img"),
        Some("b2c3d4e5f6789012345678901234567890123456789012345678901234567890ab")
    );
    assert_eq!(
        loaded_manifest.find_hash("Base/String/Mob.img"),
        Some("c3d4e5f6789012345678901234567890123456789012345678901234567890abcd")
    );
    
    // 创建对应的 object 文件
    for (path, _) in vec![
        ("Map/Map0/000000000.img", "a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890"),
        ("Sound/UI.img", "b2c3d4e5f6789012345678901234567890123456789012345678901234567890ab"),
        ("Base/String/Mob.img", "c3d4e5f6789012345678901234567890123456789012345678901234567890abcd"),
    ] {
        if let Some(hash) = loaded_manifest.find_hash(path) {
            let prefix = &hash[..2];
            let object_dir = temp_dir.path().join("objects").join(prefix);
            
            // 创建目录
            fs::create_dir_all(&object_dir).unwrap();
            
            // 创建空文件
            let object_path = object_dir.join(hash);
            fs::write(&object_path, b"test").unwrap();
            
            // 验证文件存在
            assert!(object_path.exists(), "Object file {} should exist", hash);
            
            println!("✓ {} -> objects/{}/{}", path, prefix, hash);
        }
    }
}

#[test]
fn test_manifest_json_structure() {
    let mut manifest = Manifest::new();
    
    // 构建一个有层次的结构
    manifest.add_img("UI/Basic.img".to_string(), "hash1".to_string());
    manifest.add_img("UI/Login.img".to_string(), "hash2".to_string());
    manifest.add_img("Map/Map0/000000000.img".to_string(), "hash3".to_string());
    manifest.add_img("Map/Map1/000010000.img".to_string(), "hash4".to_string());
    
    let json = manifest.to_json().unwrap();
    
    // 验证 JSON 包含正确的结构
    assert!(json.contains("\"version\": \"1.0\""));
    assert!(json.contains("\"imgs\""));
    assert!(json.contains("\"UI\""));
    assert!(json.contains("\"Map\""));
    assert!(json.contains("\"Map0\""));
    assert!(json.contains("\"Map1\""));
    assert!(json.contains("\"Basic.img\": \"hash1\""));
    assert!(json.contains("\"000000000.img\": \"hash3\""));
}

#[test]
fn test_content_addressing() {
    let temp_dir = TempDir::new().unwrap();
    let mut manifest = Manifest::new();
    
    // 模拟相同内容的不同文件（应该有相同的哈希）
    let same_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    
    manifest.add_img("Map/Tile/empty1.img".to_string(), same_hash.to_string());
    manifest.add_img("Map/Tile/empty2.img".to_string(), same_hash.to_string());
    manifest.add_img("UI/blank.img".to_string(), same_hash.to_string());
    
    // 所有文件应该指向同一个哈希
    assert_eq!(manifest.find_hash("Map/Tile/empty1.img"), Some(same_hash));
    assert_eq!(manifest.find_hash("Map/Tile/empty2.img"), Some(same_hash));
    assert_eq!(manifest.find_hash("UI/blank.img"), Some(same_hash));
    
    // 在文件系统中，只需要存储一份
    let prefix = &same_hash[..2];
    let object_path = temp_dir.path().join("objects").join(prefix).join(same_hash);
    
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, b"empty content").unwrap();
    
    // 验证单个文件可以服务多个逻辑路径
    assert!(object_path.exists());
    println!("✓ Content deduplication: 3 files -> 1 object");
}