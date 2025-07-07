use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wz_splitter::{WzSplitter, WzInspector, Manifest};

#[derive(Parser, Debug)]
#[command(name = "wz-splitter")]
#[command(about = "WZ file utilities for splitting and inspecting")]
#[command(version, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Split WZ files into individual IMG files for on-demand loading
    Split {
        /// WZ files to split
        #[arg(required = true)]
        wz_files: Vec<PathBuf>,

        /// Output directory for split files
        #[arg(short, long, default_value = "split")]
        output: PathBuf,

        /// Copy Base.wz to output directory
        #[arg(short, long)]
        copy_base: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Inspect WZ file structure
    Inspect {
        /// WZ file to inspect
        #[arg(required = true)]
        wz_file: PathBuf,

        /// Maximum depth to display (0 for unlimited)
        #[arg(short, long, default_value = "0")]
        depth: usize,

        /// Output format
        #[arg(short, long, default_value = "tree")]
        format: String,

        /// Show only specific node types (file, directory, image)
        #[arg(short = 't', long)]
        filter: Option<String>,

        /// Show statistics
        #[arg(short, long)]
        stats: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Split { wz_files, output, copy_base, verbose } => {
            handle_split_command(wz_files, output, copy_base, verbose)
        }
        Commands::Inspect { wz_file, depth, format, filter, stats } => {
            handle_inspect_command(wz_file, depth, format, filter, stats)
        }
    }
}

fn handle_split_command(
    wz_files: Vec<PathBuf>,
    output: PathBuf,
    copy_base: bool,
    verbose: bool,
) -> Result<()> {
    // 创建输出目录
    std::fs::create_dir_all(&output)?;

    // 复制 Base.wz 如果需要
    if copy_base {
        for wz_file in &wz_files {
            if wz_file.file_name().and_then(|n| n.to_str()) == Some("Base.wz") {
                let dest = output.join("Base.wz");
                if verbose {
                    println!("Copying Base.wz to {:?}", dest);
                }
                std::fs::copy(wz_file, dest)?;
                break;
            }
        }
    }

    // 创建 splitter
    let mut splitter = WzSplitter::new(&output);
    splitter.set_verbose(verbose);

    // 拆分 WZ 文件
    if verbose {
        println!("Splitting {} WZ files to {:?}", wz_files.len(), output);
    }

    splitter.split_multiple(&wz_files)?;

    println!("✅ Successfully split {} WZ files", wz_files.len());
    println!("📁 Output directory: {}", output.display());
    println!("📄 Manifest saved to: {}", output.join("manifest.json").display());

    // 验证文件一致性
    if verbose {
        println!("\n🔍 Verifying file consistency...");
        verify_manifest_consistency(&output)?;
    }

    Ok(())
}

fn handle_inspect_command(
    wz_file: PathBuf,
    depth: usize,
    format: String,
    filter: Option<String>,
    stats: bool,
) -> Result<()> {
    let inspector = WzInspector::new();
    inspector.inspect_file(&wz_file, depth, &format, filter.as_deref(), stats)?;
    Ok(())
}

fn verify_manifest_consistency(output_dir: &PathBuf) -> Result<()> {
    let manifest_path = output_dir.join("manifest.json");
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_content)?;
    
    let mut missing_count = 0;
    let mut found_count = 0;
    
    // 遍历所有文件（新的内容寻址存储）
    for (path, hash) in manifest.iter_files() {
        let prefix = &hash[..2];
        let object_path = output_dir.join("objects").join(prefix).join(&hash);
        if object_path.exists() {
            found_count += 1;
        } else {
            missing_count += 1;
            println!("  ❌ Missing: {} (expected at objects/{}/{})", path, prefix, hash);
        }
    }
    
    if missing_count > 0 {
        println!("  ⚠️  {} files in manifest are missing", missing_count);
    }
    
    println!("  ✅ {} files verified", found_count);
    
    // 检查实际对象文件
    let objects_dir = output_dir.join("objects");
    if objects_dir.exists() {
        let actual_count = count_object_files(&objects_dir)?;
        let manifest_count = manifest.count_files();
        println!("  📊 Manifest entries: {}, Object files: {}", manifest_count, actual_count);
        if actual_count < manifest_count {
            println!("  💡 Content deduplication saved {} files", manifest_count - actual_count);
        }
    }
    
    Ok(())
}

fn count_object_files(dir: &PathBuf) -> Result<usize> {
    let mut count = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // 递归计算子目录中的文件
            count += count_object_files(&path)?;
        } else if path.is_file() {
            // 计算所有文件（object 文件没有扩展名）
            count += 1;
        }
    }
    Ok(count)
}