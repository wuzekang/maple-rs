# wz-parser - WASM Compatible MapleStory WZ File Reader

这是一个专为 WebAssembly 优化的 MapleStory .wz 文件读取库，基于 [wz-reader-rs](../wz-reader-rs/) 重构而成。

## 主要特性

- ✅ **WASM 兼容**: 可在 WebAssembly 环境中运行
- ✅ **零文件系统依赖**: 完全基于内存的字节数组处理
- ✅ **完整功能**: 保留所有核心 WZ 解析能力
- ✅ **条件编译**: 自动适配 WASM 和原生环境
- ✅ **向后兼容**: 保持与原有 API 的兼容性

## 与原版的区别

| 特性 | wz-reader-rs | wz-parser (本库) |
|------|-------------|-----------|
| 文件输入 | 文件路径 | 字节数组 |
| 内存映射 | memmap2 | Vec<u8> |
| 并行处理 | 总是使用 rayon | WASM 环境下回退到串行 |
| WASM 支持 | ❌ | ✅ |
| 文件系统 | 需要 | 不需要 |

## 快速开始

```rust
use wz_parser::{WzFile, WzNodeCast};

// 读取 wz 文件到字节数组
let data = std::fs::read("Base.wz")?;

// 从字节数组创建 WzFile
let mut wz_file = WzFile::from_bytes(data, None, None, None)?;

// 解析文件结构
let root_node = wz_parser::WzNode::new(&"Base".into(), wz_file.clone(), None).into_lock();
let children = wz_file.parse(&root_node, None)?;

// 遍历子节点
for (name, node) in children {
    println!("Node: {}", name);
}
```

## WASM 构建

```bash
# 检查 WASM 目标
cargo check --target wasm32-unknown-unknown

# 构建为 WASM
cargo build --target wasm32-unknown-unknown --release
```

## 支持的功能

- [x] .wz 文件解析
- [x] 图片提取 (PNG, DXT3, DXT5, RGB565 等)
- [x] 音频文件提取
- [x] 字符串解密
- [x] Lua 脚本提取
- [x] 所有数据类型解析
- [x] 加密版本检测

## 运行示例

```bash
cargo run --example basic_usage path/to/your/file.wz
```

## 许可证

MIT License