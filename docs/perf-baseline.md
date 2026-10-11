# 性能基准（style 优化前基线）

日期：2026-10-11 · 分支：`wip`（commit `0f9e235`，samara 引擎 + 渲染循环修复后）
机器：Apple Silicon (M-series), macOS, release 构建

## 基准工具

- `crates/sig/examples/style_bench.rs` — 无头 CPU 基准（无窗口/无 GPU），分场景测
  四段：effects（信号写引发的闭包重跑）/ style（bubble + compute）/ layout（taffy）/ scene（vello 编码）。
  运行：`cargo run --release -p sig --example style_bench -- [rows] [iters]`
- `crates/sig/examples/frame_bench.rs` — 端到端窗口基准（含 GPU/vsync）。
  运行：`SIG_FRAME_STATS=1 cargo run --release -p sig --example frame_bench -- [rows] [flips]`

## style_bench 基线（p50/p95，µs）

### 1000 行

| 场景 | effects | style | layout | scene |
|---|---|---|---|---|
| clean（无任何变化） | 0/0 | 0/0 | 22/37 | 236/354 |
| one_paint（1 行改背景色） | 0/2 | 29/47 | 230/363 | 250/469 |
| all_paint（全部改背景色） | 465/630 | 1391/1706 | 794/940 | 324/593 |
| one_layout（1 行改高度） | 0/1 | 29/44 | 227/373 | 241/497 |
| all_layout（全部改高度） | 357/519 | 977/1322 | 769/931 | 283/565 |

### 300 行

| 场景 | effects | style | layout | scene |
|---|---|---|---|---|
| clean | 0/0 | 0/0 | 6/10 | 92/123 |
| one_paint | 0/1 | 10/14 | 68/93 | 93/135 |
| all_paint | 121/158 | 392/497 | 231/283 | 96/175 |
| one_layout | 0/0 | 9/12 | 69/84 | 83/101 |
| all_layout | 94/133 | 285/345 | 222/269 | 86/152 |

## frame_bench 端到端基线（p50，µs）

| 场景 | style | layout | scene | raster | present | total | 帧:写 |
|---|---|---|---|---|---|---|---|
| 300 行 × 300 写 | ~580 | ~290 | ~250 | ~810 | 109 | 2000 | 301/300 |
| 1000 行 × 300 写 | ~1900 | ~850 | ~530 | ~780 | 106 | 4000 | 301/300 |

（空闲渲染为零；每逻辑更新实际工作量对比旧引擎基线：300 行 -45%，1000 行 -33%。）

## 基线揭示的优化点（按预期收益排序）

1. **Paint/Layout 未分流（最大）**：`one_paint` 的 layout 段 230µs ≈ `one_layout`
   的 227µs——改一个背景色触发全量重排。`StyleTrigger::{Paint,Layout}` 枚举
   已存在但未接线。目标：paint 级变更短路 taffy `set_style` 与 layout。
   预期：`one_paint` 总耗时 ~500µs → ~60µs；`all_paint` 省去 ~800µs layout。
2. **scene 无条件全量重建**：`clean` 场景 scene 仍要 236µs。损伤追踪
   （只重编码受影响子树）是独立优化轴。
3. **effects 段 ~0.5µs/行**：属性级样式绑定（background 绑信号）可消掉
   闭包整体重跑，收益约 1/3 个 style 段。
4. **style 段 O(脏) 已成立**（clean=0µs，1 脏=29µs）——脏标记/冒泡机制
   本身健康，不需要"继承依赖登记"这类结构改造（预期收益低于 1、2）。

## 规则

任何 style/layout/scene 相关优化，先跑本基准记录对比，写进本文档的
"优化记录"章节；端到端验收用 frame_bench。

## 优化记录

### #1 等值 set_style 门控（Paint/Layout 分流的第一步）

机制定位（`examples/taffy_setstyle_probe.rs`）：taffy 的 `set_style` 不比较新旧
样式、无条件标脏——写入比特级相同的样式也会触发子树重排（探针数据：缓存命中
11µs，等值 set_style 后 300µs ≈ 冷启动的 75%）。sig 在 `compute_style` 和 style
effect 两处无条件调用 `set_style`，导致 paint-only 变更（如改背景色）照付全价。

修复：两处 `set_style` 均加变更比较门控（taffy 相关属性没变就不写）。

style_bench 对比（1000 行，layout 段 p50）：

| 场景 | 改前 | 改后 | |
|---|---|---|---|
| clean | 22µs | 23µs | 不变（地板） |
| one_paint | 230µs | **23µs** | -90%，落回地板 |
| all_paint | 794µs | **29µs** | -96% |
| one_layout | 227µs | 266µs | 不变（该排照排，噪声内） |
| all_layout | 769µs | 787µs | 不变 |

frame_bench（1000 行）：layout 段 850µs → **122µs（-86%）**，total p50
4.0ms → 3.2ms，帧:写 302/300 保持。sig 测试 98/98。

下一步候选（见上文优化点 2/3）：scene 损伤追踪（clean 仍 249µs）、
属性级样式绑定（effects+style 段仍占大头）。

### #2 style 热路径分配消除

每脏节点/每访问节点的分配清理（语义不变）：

- `compute_style`：`state.styles.clone()`（整个 StyleBuilder 向量深拷贝）→ 在读
  借用内直接合并属性映射；末尾双 `style.clone()` → 单克隆
- `compute_style_recursive`：每节点克隆整个继承上下文 HashMap
  （`ctx.style.clone()`）→ undo 日志（记录被覆盖的继承键，子树走完按日志回放）
- style effect：变更比较后的 `builder.clone()` → move 存储

style_bench（1000 行 p50）：all_paint style 1352→1218µs（-10%）、effects
469→396µs（-16%）；all_layout style 971→886µs。one_paint/clean 不变。
frame_bench 1000 行：style 段 1.6→1.4ms。sig 98/98、sig-reactive 142/142。

剩余成本是结构性的（每节点 HashMap 哈希、`StylePropertyKey::iter()` 全枚举
遍历 ×2、`style_cache` 每节点重建）。下一步候选：fieldless 枚举键 → 定长
数组索引（消掉全部属性映射的哈希与分配）。
