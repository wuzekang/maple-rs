use wz_parser::WzFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取 wz 文件数据 (这里假设有一个测试文件)
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <wz_file_path>", args[0]);
        return Ok(());
    }
    
    let file_path = &args[1];
    println!("Loading WZ file: {}", file_path);
    
    // 读取文件到字节数组 (WASM 兼容方式)
    let data = match std::fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to read file: {}", e);
            return Ok(());
        }
    };
    
    println!("File size: {} bytes", data.len());
    
    // 使用字节数组创建 WzFile (WASM 兼容)
    let mut wz_file = match WzFile::from_bytes(data, None, None, None) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to parse WZ file: {}", e);
            return Ok(());
        }
    };
    
    println!("WZ file loaded successfully!");
    println!("File metadata: {:?}", wz_file.wz_file_meta);
    
    // 创建根节点并解析
    let root_node = wz_parser::WzNode::new(&"test".into(), wz_file.clone(), None).into_lock();
    
    match wz_file.parse(&root_node, None) {
        Ok(children) => {
            println!("Found {} child nodes:", children.len());
            for (name, _node) in children.iter().take(10) {
                println!("  - {}", name);
            }
            if children.len() > 10 {
                println!("  ... and {} more", children.len() - 10);
            }
        }
        Err(e) => {
            println!("Failed to parse WZ structure: {}", e);
        }
    }
    
    Ok(())
}