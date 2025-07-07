use crate::Manifest;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use wz_parser::{WzNode, WzNodeArc, WzNodeCast, WzNodeName, WzObjectType};
use std::sync::RwLock;

pub struct WzSplitter {
    output_dir: PathBuf,
    manifest: Arc<Mutex<Manifest>>,
    progress: Arc<ProgressBar>,
    verbose: bool,
}

impl WzSplitter {
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        let progress = ProgressBar::new(0);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            manifest: Arc::new(Mutex::new(Manifest::new())),
            progress: Arc::new(progress),
            verbose: false,
        }
    }
    
    /// 设置是否显示详细输出
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
    
    fn is_verbose(&self) -> bool {
        self.verbose
    }
    
    /// 打印节点树结构（用于调试）
    fn print_node_tree(&self, node: &WzNodeArc, indent: usize) {
        let node_read = node.read().unwrap();
        let indent_str = "  ".repeat(indent);
        
        match &node_read.object_type {
            WzObjectType::File(_) => println!("{}📁 {} (File)", indent_str, node_read.name.as_str()),
            WzObjectType::Directory(_) => println!("{}📂 {} (Directory)", indent_str, node_read.name.as_str()),
            WzObjectType::Image(_) => println!("{}📄 {} (Image)", indent_str, node_read.name.as_str()),
            _ => println!("{}❓ {} (Other)", indent_str, node_read.name.as_str()),
        }
        
        for child in node_read.children.values() {
            self.print_node_tree(child, indent + 1);
        }
    }
    
    /// 解析目录结构，处理 Directory 节点的外部 WZ 文件引用
    fn parse_directory_structure(&self, node: &WzNodeArc, wz_folder: &Path, visited_files: &mut HashSet<PathBuf>) -> Result<()> {
        // 获取基础节点的 patch_version 和 keys（用于后续加载）
        let (base_patch_version, base_keys) = if let Some(parent) = node.read().unwrap().parent.upgrade() {
            let parent_read = parent.read().unwrap();
            if let Some(file) = parent_read.try_as_file() {
                (Some(file.wz_file_meta.patch_version), Some(file.reader.keys.clone()))
            } else {
                (None, None)
            }
        } else if let Some(file) = node.read().unwrap().try_as_file() {
            (Some(file.wz_file_meta.patch_version), Some(file.reader.keys.clone()))
        } else {
            (None, None)
        };
        let children: Vec<_> = {
            let node_read = node.read().unwrap();
            node_read.children.values().cloned().collect()
        };

        for child in children {
            {
                let node_read = child.read().unwrap();
                match &node_read.object_type {
                    WzObjectType::Directory(_) => {
                        // Directory 节点：检查是否有对应的 WZ 文件
                        let directory_name = node_read.name.as_str().to_string();
                        drop(node_read);
                        
                        // 尝试查找对应的 WZ 文件
                        let mut target_wz_path = None;
                        
                        // 1. 先尝试直接的 .wz 文件
                        let direct_path = wz_folder.join(format!("{}.wz", directory_name));
                        if direct_path.exists() && !visited_files.contains(&direct_path) {
                            target_wz_path = Some(direct_path);
                        }
                        
                        // 2. 如果是目录，查找目录下的同名 .wz 文件（像 Base/Base.wz）
                        if target_wz_path.is_none() {
                            let dir_path = wz_folder.join(&directory_name);
                            if dir_path.is_dir() {
                                let inner_wz_path = dir_path.join(format!("{}.wz", directory_name));
                                if inner_wz_path.exists() && !visited_files.contains(&inner_wz_path) {
                                    target_wz_path = Some(inner_wz_path);
                                }
                            }
                        }
                        
                        if let Some(wz_path) = target_wz_path {
                            // 标记文件为已访问，防止循环
                            visited_files.insert(wz_path.clone());
                            
                            if self.verbose {
                                println!("Loading external WZ: {}", wz_path.display());
                            }
                            
                            // 使用 base 的 version 和 keys 加载对应的 WZ 文件
                            match WzNode::from_wz_file_full(
                                &wz_path,
                                None, // version - 自动检测
                                base_patch_version,
                                Some(&node), // parent
                                base_keys.as_ref(),
                            ) {
                                Ok(mut target_node) => {
                                    // 设置节点名称为目录名（不带 .wz 后缀）
                                    target_node.name = WzNodeName::from(directory_name.as_str());
                                    
                                    let target_arc = Arc::new(RwLock::new(target_node));
                                    
                                    // 解析目标 WZ 文件
                                    let mut target_write = target_arc.write().unwrap();
                                    if let Ok(_) = target_write.parse(&target_arc) {
                                        drop(target_write);
                                        
                                        // 替换当前 Directory 节点为 File 节点
                                        let parent_node = {
                                            let child_read = child.read().unwrap();
                                            child_read.parent.upgrade()
                                        };
                                        
                                        if let Some(parent) = parent_node {
                                            let mut parent_write = parent.write().unwrap();
                                            parent_write.children.insert(WzNodeName::from(directory_name.as_str()), target_arc.clone());
                                        }
                                        
                                        // 递归解析新的 File 节点
                                        self.parse_directory_structure(&target_arc, wz_folder, visited_files)?;
                                    }
                                }
                                Err(e) => {
                                    if self.verbose {
                                        println!("Warning: Failed to load {}: {}", wz_path.display(), e);
                                    }
                                    // 按普通 Directory 处理
                                    let mut child_write = child.write().unwrap();
                                    if let Err(e) = child_write.parse(&child) {
                                        if self.verbose {
                                            println!("Warning: Failed to parse directory {}: {}", child_write.name.as_str(), e);
                                        }
                                    } else {
                                        drop(child_write);
                                        self.parse_directory_structure(&child, wz_folder, visited_files)?;
                                    }
                                }
                            }
                        } else {
                            // 如果对应的 WZ 文件不存在或已访问，就按普通 Directory 处理
                            let mut child_write = child.write().unwrap();
                            if let Err(e) = child_write.parse(&child) {
                                if self.verbose {
                                    println!("Warning: Failed to parse directory {}: {}", child_write.name.as_str(), e);
                                }
                            } else {
                                drop(child_write);
                                self.parse_directory_structure(&child, wz_folder, visited_files)?;
                            }
                        }
                    }
                    WzObjectType::File(_) => {
                        // File 节点：正常解析
                        drop(node_read);
                        let mut child_write = child.write().unwrap();
                        if let Err(e) = child_write.parse(&child) {
                            if self.verbose {
                                println!("Warning: Failed to parse file {}: {}", child_write.name.as_str(), e);
                            }
                        } else {
                            drop(child_write);
                            self.parse_directory_structure(&child, wz_folder, visited_files)?;
                        }
                    }
                    WzObjectType::Image(_) => {
                        // Image 节点：不需要进一步解析内容，只记录存在
                    }
                    _ => {
                        // 其他类型节点：尝试解析
                        drop(node_read);
                        let mut child_write = child.write().unwrap();
                        if let Err(e) = child_write.parse(&child) {
                            if self.verbose {
                                println!("Warning: Failed to parse node {}: {}", child_write.name.as_str(), e);
                            }
                        } else {
                            drop(child_write);
                            self.parse_directory_structure(&child, wz_folder, visited_files)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
    
    

    /// 拆分单个 WZ 文件
    pub fn split_wz_file(&self, wz_path: &Path) -> Result<()> {
        self.progress.set_message(format!("Loading {}", wz_path.display()));
        
        // 统一使用普通方式加载 WZ 文件
        let root_node = WzNode::from_wz_file(wz_path, None)?;
        let root_arc = Arc::new(RwLock::new(root_node));
        
        let wz_name = wz_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid wz file name"))?;

        // 解析根节点
        let mut root_write = root_arc.write().unwrap();
        root_write.parse(&root_arc)?;
        drop(root_write);
        
        // 获取 WZ 文件所在目录，用于解析 Directory 节点
        let wz_folder = wz_path.parent().unwrap_or(Path::new("."));
        let mut visited_files = HashSet::new();
        visited_files.insert(wz_path.to_path_buf());
        
        // 使用优化的解析逻辑
        self.parse_directory_structure(&root_arc, wz_folder, &mut visited_files)?;
        
        // 打印调试信息
        if self.is_verbose() {
            println!("Loaded WZ file: {}", wz_name);
            self.print_node_tree(&root_arc, 0);
        }

        // 计算总的 IMG 数量
        let img_count = self.count_imgs(&root_arc.read().unwrap());
        self.progress.set_length(img_count as u64);
        self.progress.set_message(format!("Extracting {} images from {}", img_count, wz_name));

        // 提取所有 IMG（现在有智能检查，会跳过无效的 IMG）
        self.extract_imgs_recursive(wz_name, &root_arc)?;

        self.progress.finish_with_message(format!("Extracted {} images from {}", img_count, wz_name));
        Ok(())
    }

    /// 拆分多个 WZ 文件
    pub fn split_multiple(&self, wz_paths: &[PathBuf]) -> Result<()> {
        for wz_path in wz_paths {
            self.split_wz_file(wz_path)?;
        }

        // 保存 manifest
        self.save_manifest()?;
        Ok(())
    }

    /// 递归计算 IMG 数量
    fn count_imgs(&self, node: &WzNode) -> usize {
        match &node.object_type {
            WzObjectType::Image(_) => 1,
            WzObjectType::Directory(_) | WzObjectType::File(_) => {
                node.children.values()
                    .map(|child| self.count_imgs(&child.read().unwrap()))
                    .sum()
            }
            _ => 0,
        }
    }

    /// 递归提取 IMG
    fn extract_imgs_recursive(
        &self,
        wz_name: &str,
        node: &WzNodeArc,
    ) -> Result<()> {
        // 直接使用当前节点作为根 WZ 文件
        self.extract_imgs_recursive_with_root(wz_name, node, node)
    }
    
    /// 递归提取 IMG（带根文件节点）
    fn extract_imgs_recursive_with_root(
        &self,
        wz_name: &str,
        node: &WzNodeArc,
        root_wz_file: &WzNodeArc,
    ) -> Result<()> {
        let node_read = node.read().unwrap();
        
        match &node_read.object_type {
            WzObjectType::Image(img) => {
                // 提取单个 IMG，但先检查数据是否在当前文件范围内
                let img_path = self.build_img_path_with_wz(&node_read, wz_name);
                
                // 检查 IMG 数据是否在当前 WZ 文件范围内
                let file_read = root_wz_file.read().unwrap();
                if let Some(wz_file) = file_read.try_as_file() {
                    let file_size = wz_file.reader.get_ref_slice().len();
                    let img_end = img.offset + img.block_size;
                    
                    if img_end <= file_size {
                        // IMG 数据在当前文件范围内，可以安全提取
                        drop(file_read);
                        self.extract_single_img_with_root(node, wz_name, &img_path, img, root_wz_file)?;
                        self.progress.inc(1);
                    } else {
                        // IMG 数据超出当前文件范围，跳过（可能来自其他文件）
                        if self.verbose {
                            println!("Skipping IMG {} (data outside current file): offset={}, size={}, file_size={}", 
                                   img_path, img.offset, img.block_size, file_size);
                        }
                    }
                } else {
                    if self.verbose {
                        println!("Skipping IMG {} (root is not a WZ file)", img_path);
                    }
                }
            }
            WzObjectType::File(_) => {
                // File 类型：使用自身作为根 WZ 文件
                let children: Vec<_> = node_read.children.values().cloned().collect();
                drop(node_read); // 释放读锁
                
                // 对 File 类型的子节点，使用 File 节点自身作为根
                for child in children {
                    self.extract_imgs_recursive_with_root(wz_name, &child, node)?;
                }
            }
            WzObjectType::Directory(_) => {
                // Directory 类型：继续使用传入的根 WZ 文件
                let children: Vec<_> = node_read.children.values().cloned().collect();
                drop(node_read); // 释放读锁
                
                for child in children {
                    self.extract_imgs_recursive_with_root(wz_name, &child, root_wz_file)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// 构建 IMG 的完整路径，包含 WZ 名称前缀
    fn build_img_path_with_wz(&self, node: &WzNode, _wz_name: &str) -> String {
        self.build_img_path(node)
    }
    
    /// 构建 IMG 的完整路径（包含所有父节点）
    fn build_img_path(&self, node: &WzNode) -> String {
        let mut path_parts = vec![node.name.as_str().to_string()];
        let mut current = node.parent.upgrade();
        let mut skip_root_wz_file = true;
        
        // 构建路径
        while let Some(parent) = current {
            let parent_read = parent.read().unwrap();
            
            // 跳过最顶层的 WzFile 节点（它代表整个 WZ 文件）
            if skip_root_wz_file && parent_read.try_as_file().is_some() && parent_read.parent.upgrade().is_none() {
                skip_root_wz_file = false;
                current = parent_read.parent.upgrade();
                continue;
            }
            
            if !parent_read.name.as_str().is_empty() {
                path_parts.push(parent_read.name.as_str().to_string());
            }
            
            current = parent_read.parent.upgrade();
        }
        
        path_parts.reverse();
        path_parts.join("/")
    }

    /// 提取单个 IMG（带根文件节点）
    fn extract_single_img_with_root(
        &self,
        _node: &WzNodeArc,
        wz_name: &str,
        img_path: &str,
        img: &wz_parser::WzImage,
        root_wz_file: &WzNodeArc,
    ) -> Result<()> {
        // 读取 IMG 数据
        let file_read = root_wz_file.read().unwrap();
        let wz_file = file_read.try_as_file()
            .ok_or_else(|| anyhow::anyhow!("Root node is not a WzFile"))?;
        
        let file_size = wz_file.reader.get_ref_slice().len();
        let img_end = img.offset + img.block_size;
        
        if img_end > file_size {
            return Err(anyhow::anyhow!(
                "IMG data range out of bounds: offset={}, size={}, end={}, file_size={}, path={}",
                img.offset, img.block_size, img_end, file_size, img_path
            ));
        }
        
        let img_data = wz_file.reader.get_slice(img.offset..img_end).to_vec();
        
        // 计算 SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&img_data);
        let hash_bytes = hasher.finalize();
        let hash = hex::encode(hash_bytes);
        
        // 使用前两个字符作为子目录（避免单目录文件过多）
        let subdir = &hash[..2];
        let object_path = self.output_dir
            .join("objects")
            .join(subdir)
            .join(&hash);
        
        // 创建目录并写入文件（如果不存在）
        if !object_path.exists() {
            if let Some(parent) = object_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&object_path, &img_data)?;
            
            if self.verbose {
                println!("Extracted: {} -> objects/{}/{}", img_path, subdir, hash);
            }
        } else if self.verbose {
            println!("Skipped (duplicate): {} -> objects/{}/{}", img_path, subdir, hash);
        }
        
        // 更新 manifest（直接使用 img_path，不包含 wz 文件名）
        let manifest_path = img_path.to_string();
        
        {
            let mut manifest = self.manifest.lock().unwrap();
            manifest.add_img(manifest_path, hash);
        }
        
        Ok(())
    }



    /// 保存 manifest.json
    fn save_manifest(&self) -> Result<()> {
        let manifest_path = self.output_dir.join("manifest.json");
        let manifest = self.manifest.lock().unwrap();
        let json = manifest.to_json()?;
        fs::write(manifest_path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wz_splitter_creation() {
        let temp_dir = TempDir::new().unwrap();
        let splitter = WzSplitter::new(temp_dir.path());
        
        assert_eq!(splitter.output_dir, temp_dir.path());
    }
}