use wz_parser::{WzNode, WzNodeCast};

fn main() {
    println!("加载 Base.wz...");
    let root = WzNode::from_wz_file("./Data/Base.wz", None).unwrap().into_lock();
    
    println!("解析 Base.wz...");
    root.write().unwrap().parse(&root).unwrap();
    
    println!("\n查找 UI.wz...");
    if let Some(ui_wz) = root.read().unwrap().at("UI") {
        println!("找到 UI.wz");
        
        println!("解析 UI.wz...");
        ui_wz.write().unwrap().parse(&ui_wz).unwrap();
        
        println!("\n查找 Logo.img...");
        if let Some(logo_img) = ui_wz.read().unwrap().at("Logo.img") {
            let logo_read = logo_img.read().unwrap();
            println!("找到 Logo.img");
            println!("  名称: {}", logo_read.name);
            println!("  完整路径: {}", logo_read.get_full_path());
            println!("  子节点数量: {}", logo_read.children.len());
            
            if let Some(img) = logo_read.try_as_image() {
                println!("  类型: WzImage");
                println!("  是否已解析: {}", img.is_parsed);
            } else {
                println!("  类型: 不是 WzImage");
            }
            
            drop(logo_read);
            
            println!("\n尝试解析 Logo.img...");
            match logo_img.write().unwrap().parse(&logo_img) {
                Ok(_) => {
                    let logo_read = logo_img.read().unwrap();
                    println!("  解析成功!");
                    println!("  解析后子节点数量: {}", logo_read.children.len());
                    
                    if let Some(img) = logo_read.try_as_image() {
                        println!("  解析后 is_parsed: {}", img.is_parsed);
                    }
                    
                    println!("\n子节点列表 (前10个):");
                    for (name, _) in logo_read.children.iter().take(10) {
                        println!("    - {}", name);
                    }
                }
                Err(e) => {
                    println!("  解析失败: {}", e);
                }
            }
        } else {
            println!("❌ 未找到 Logo.img");
        }
    } else {
        println!("❌ 未找到 UI.wz");
    }
}
