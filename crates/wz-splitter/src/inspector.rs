use anyhow::Result;
use std::path::Path;
use std::sync::RwLock;
use std::sync::Arc;
use std::collections::HashSet;
use wz_parser::{WzNode, WzNodeArc, WzObjectType};

pub struct WzInspector;

#[derive(Debug, Default)]
pub struct WzStats {
    pub total_nodes: usize,
    pub file_nodes: usize,
    pub directory_nodes: usize,
    pub image_nodes: usize,
    pub other_nodes: usize,
}

impl WzInspector {
    pub fn new() -> Self {
        Self
    }

    pub fn inspect_file(
        &self,
        wz_path: &Path,
        max_depth: usize,
        format: &str,
        filter: Option<&str>,
        show_stats: bool,
    ) -> Result<()> {
        println!("🔍 Inspecting WZ file: {}", wz_path.display());
        
        // 加载 WZ 文件
        let root_node = WzNode::from_wz_file(wz_path, None)?;
        let root_arc = Arc::new(RwLock::new(root_node));

        // 只解析目录结构，不解析 IMG 内容
        let mut root_write = root_arc.write().unwrap();
        root_write.parse(&root_arc)?;
        drop(root_write);
        
        // 获取当前 WZ 文件的目录路径
        let wz_folder = wz_path.parent().unwrap_or(Path::new("."));
        let mut visited_files = HashSet::new();
        visited_files.insert(wz_path.to_path_buf());
        self.parse_directory_structure(&root_arc, wz_folder, &mut visited_files)?;

        // 收集统计信息
        let mut stats = WzStats::default();
        if show_stats {
            self.collect_stats(&root_arc, &mut stats);
        }

        // 显示结构
        println!("\n📊 File Structure:");
        match format {
            "tree" => self.print_tree_format(&root_arc, 0, max_depth, filter),
            "json" => self.print_json_format(&root_arc, max_depth, filter)?,
            _ => {
                println!("❌ Unsupported format: {}", format);
                println!("💡 Supported formats: tree, json");
                return Ok(());
            }
        }

        // 显示统计信息
        if show_stats {
            self.print_stats(&stats);
        }

        Ok(())
    }

    fn parse_directory_structure(&self, node: &WzNodeArc, wz_folder: &Path, visited_files: &mut HashSet<std::path::PathBuf>) -> Result<()> {
        let children: Vec<_> = {
            let node_read = node.read().unwrap();
            node_read.children.values().cloned().collect()
        };

        for child in children {
            {
                let node_read = child.read().unwrap();
                match &node_read.object_type {
                    WzObjectType::Directory(_) => {
                        // Directory 节点：尝试加载对应的 WZ 文件
                        let directory_name = node_read.name.as_str();
                        let target_wz_path = wz_folder.join(format!("{}.wz", directory_name));
                        
                        drop(node_read);
                        
                        if target_wz_path.exists() && !visited_files.contains(&target_wz_path) {
                            // 标记文件为已访问，防止循环
                            visited_files.insert(target_wz_path.clone());
                            
                            // 加载对应的 WZ 文件
                            match WzNode::from_wz_file(&target_wz_path, None) {
                                Ok(target_node) => {
                                    let target_arc = Arc::new(RwLock::new(target_node));
                                    
                                    // 解析目标 WZ 文件
                                    let mut target_write = target_arc.write().unwrap();
                                    if let Ok(_) = target_write.parse(&target_arc) {
                                        drop(target_write);
                                        
                                        // 将目标 WZ 文件的内容合并到当前 Directory 节点
                                        self.merge_wz_content(&child, &target_arc)?;
                                        
                                        // 只递归解析合并后的节点，不再解析整个目标文件
                                        self.parse_directory_structure(&child, wz_folder, visited_files)?;
                                    }
                                }
                                Err(_) => {
                                    // 如果无法加载 WZ 文件，就按普通 Directory 处理
                                    let mut child_write = child.write().unwrap();
                                    if let Ok(_) = child_write.parse(&child) {
                                        drop(child_write);
                                        self.parse_directory_structure(&child, wz_folder, visited_files)?;
                                    }
                                }
                            }
                        } else {
                            // 如果对应的 WZ 文件不存在或已访问，就按普通 Directory 处理
                            let mut child_write = child.write().unwrap();
                            if let Ok(_) = child_write.parse(&child) {
                                drop(child_write);
                                self.parse_directory_structure(&child, wz_folder, visited_files)?;
                            }
                        }
                    }
                    WzObjectType::File(_) => {
                        // File 节点：正常解析
                        drop(node_read);
                        let mut child_write = child.write().unwrap();
                        if let Ok(_) = child_write.parse(&child) {
                            drop(child_write);
                            self.parse_directory_structure(&child, wz_folder, visited_files)?;
                        }
                    }
                    _ => {
                        // 其他类型节点（如 Image）不需要进一步解析
                    }
                }
            }
        }

        Ok(())
    }
    
    fn merge_wz_content(&self, directory_node: &WzNodeArc, wz_content: &WzNodeArc) -> Result<()> {
        let wz_read = wz_content.read().unwrap();
        let children_to_merge: Vec<_> = wz_read.children.values().cloned().collect();
        drop(wz_read);
        
        let mut directory_write = directory_node.write().unwrap();
        for child in children_to_merge {
            let child_read = child.read().unwrap();
            let child_name = child_read.name.clone();
            drop(child_read);
            
            directory_write.children.insert(child_name, child);
        }
        
        Ok(())
    }

    fn collect_stats(&self, node: &WzNodeArc, stats: &mut WzStats) {
        let node_read = node.read().unwrap();
        stats.total_nodes += 1;

        match &node_read.object_type {
            WzObjectType::File(_) => stats.file_nodes += 1,
            WzObjectType::Directory(_) => stats.directory_nodes += 1,
            WzObjectType::Image(_) => stats.image_nodes += 1,
            _ => stats.other_nodes += 1,
        }

        for child in node_read.children.values() {
            self.collect_stats(child, stats);
        }
    }

    fn print_tree_format(
        &self,
        node: &WzNodeArc,
        current_depth: usize,
        max_depth: usize,
        filter: Option<&str>,
    ) {
        if max_depth > 0 && current_depth >= max_depth {
            return;
        }

        let node_read = node.read().unwrap();
        let indent = "  ".repeat(current_depth);

        // 应用过滤器
        if let Some(filter_type) = filter {
            let node_type = match &node_read.object_type {
                WzObjectType::File(_) => "file",
                WzObjectType::Directory(_) => "directory",
                WzObjectType::Image(_) => "image",
                _ => "other",
            };

            if filter_type != node_type {
                // 仍然需要递归检查子节点
                for child in node_read.children.values() {
                    self.print_tree_format(child, current_depth + 1, max_depth, filter);
                }
                return;
            }
        }

        // 显示节点信息
        let (icon, type_name) = match &node_read.object_type {
            WzObjectType::File(_) => ("📁", "File"),
            WzObjectType::Directory(_) => ("📂", "Directory"),
            WzObjectType::Image(_) => ("📄", "Image"),
            _ => ("❓", "Other"),
        };

        println!("{}{} {} ({})", indent, icon, node_read.name.as_str(), type_name);

        // 递归显示子节点
        for child in node_read.children.values() {
            self.print_tree_format(child, current_depth + 1, max_depth, filter);
        }
    }

    fn print_json_format(
        &self,
        node: &WzNodeArc,
        max_depth: usize,
        filter: Option<&str>,
    ) -> Result<()> {
        let json_value = self.node_to_json(node, 0, max_depth, filter)?;
        println!("{}", serde_json::to_string_pretty(&json_value)?);
        Ok(())
    }

    fn node_to_json(
        &self,
        node: &WzNodeArc,
        current_depth: usize,
        max_depth: usize,
        filter: Option<&str>,
    ) -> Result<serde_json::Value> {
        let node_read = node.read().unwrap();

        // 应用过滤器
        if let Some(filter_type) = filter {
            let node_type = match &node_read.object_type {
                WzObjectType::File(_) => "file",
                WzObjectType::Directory(_) => "directory",
                WzObjectType::Image(_) => "image",
                _ => "other",
            };

            if filter_type != node_type {
                return Ok(serde_json::Value::Null);
            }
        }

        let type_name = match &node_read.object_type {
            WzObjectType::File(_) => "file",
            WzObjectType::Directory(_) => "directory",
            WzObjectType::Image(_) => "image",
            _ => "other",
        };

        let mut json_obj = serde_json::Map::new();
        json_obj.insert("name".to_string(), serde_json::Value::String(node_read.name.as_str().to_string()));
        json_obj.insert("type".to_string(), serde_json::Value::String(type_name.to_string()));

        // 如果还没有达到最大深度，添加子节点
        if max_depth == 0 || current_depth < max_depth {
            let mut children = Vec::new();
            for child in node_read.children.values() {
                let child_json = self.node_to_json(child, current_depth + 1, max_depth, filter)?;
                if !child_json.is_null() {
                    children.push(child_json);
                }
            }
            
            if !children.is_empty() {
                json_obj.insert("children".to_string(), serde_json::Value::Array(children));
            }
        }

        Ok(serde_json::Value::Object(json_obj))
    }

    fn print_stats(&self, stats: &WzStats) {
        println!("\n📈 Statistics:");
        println!("  Total nodes: {}", stats.total_nodes);
        println!("  📁 File nodes: {}", stats.file_nodes);
        println!("  📂 Directory nodes: {}", stats.directory_nodes);
        println!("  📄 Image nodes: {}", stats.image_nodes);
        if stats.other_nodes > 0 {
            println!("  ❓ Other nodes: {}", stats.other_nodes);
        }
    }
}