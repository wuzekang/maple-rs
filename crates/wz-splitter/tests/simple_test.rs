use std::fs;
use tempfile::TempDir;

#[test]
fn test_wz_splitter_creates_manifest() {
    // 测试 manifest 文件是否正确创建
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    
    // 创建一个空的 manifest
    let manifest = wz_splitter::Manifest::new();
    let json = manifest.to_json().unwrap();
    fs::write(&manifest_path, json).unwrap();
    
    // 验证文件存在
    assert!(manifest_path.exists());
    
    // 读取并验证内容
    let content = fs::read_to_string(&manifest_path).unwrap();
    let loaded_manifest: wz_splitter::Manifest = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded_manifest.version, "1.0");
}