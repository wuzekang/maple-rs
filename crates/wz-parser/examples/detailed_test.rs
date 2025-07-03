use wz_parser::{WzFile, WzObjectType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <wz_file_path>", args[0]);
        return Ok(());
    }
    
    let file_path = &args[1];
    println!("🔍 Loading WZ file: {}", file_path);
    
    // 读取文件到字节数组 (WASM 兼容方式)
    let data = std::fs::read(file_path)?;
    println!("📊 File size: {} bytes", data.len());
    
    // 使用字节数组创建 WzFile
    let mut wz_file = WzFile::from_bytes(data, None, None, None)?;
    
    println!("✅ WZ file loaded successfully!");
    println!("📋 File metadata:");
    println!("   - Patch version: {}", wz_file.wz_file_meta.patch_version);
    println!("   - Version header: {}", wz_file.wz_file_meta.wz_version_header);
    println!("   - Has encrypt header: {}", wz_file.wz_file_meta.wz_with_encrypt_version_header);
    println!("   - Hash: {}", wz_file.wz_file_meta.hash);
    
    // 创建根节点并解析
    let root_node = wz_parser::WzNode::new(&"test".into(), wz_file.clone(), None).into_lock();
    
    match wz_file.parse(&root_node, None) {
        Ok(children) => {
            println!("\n🌳 Found {} child nodes:", children.len());
            
            for (name, node) in children.iter() {
                let node_read = node.read().unwrap();
                let type_name = match &node_read.object_type {
                    WzObjectType::File(_) => "File",
                    WzObjectType::Directory(_) => "Directory",
                    WzObjectType::Image(_) => "Image",
                    WzObjectType::Property(_) => "Property",
                    WzObjectType::Value(_) => "Value",
                };
                
                println!("   📁 {} [{}]", name, type_name);
                
                // 如果是图像文件，尝试显示一些子节点
                if let WzObjectType::Image(_) = &node_read.object_type {
                    drop(node_read); // 释放读锁
                    
                    // 获取写锁来解析
                    if let Ok(mut node_write) = node.write() {
                        if let Ok(_) = node_write.parse(node) {
                            let children_count = node_write.children.len();
                            if children_count > 0 {
                                println!("      └── Contains {} child nodes", children_count);
                                for (child_name, _) in node_write.children.iter().take(3) {
                                    println!("          • {}", child_name);
                                }
                                if children_count > 3 {
                                    println!("          • ... and {} more", children_count - 3);
                                }
                            }
                        }
                    }
                }
            }
            
            println!("\n✨ Successfully parsed WZ structure!");
            
            // 测试特定路径访问
            if let Some((name, node)) = children.iter().find(|(name, _)| name.ends_with(".img")) {
                println!("\n🔍 Testing node access for: {}", name);
                
                if let Ok(mut node_write) = node.write() {
                    if let Err(e) = node_write.parse(node) {
                        println!("   ⚠️  Failed to parse image: {}", e);
                    } else {
                        println!("   ✅ Successfully parsed image with {} children", node_write.children.len());
                    }
                }
            }
            
        }
        Err(e) => {
            println!("❌ Failed to parse WZ structure: {}", e);
        }
    }
    
    println!("\n🎉 Test completed!");
    Ok(())
}