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

### #3 剖析：style 段瓶颈定位（macOS `sample`，all_paint 1000 行 × 8s）

方法：`sample <pid> 8` 采样 release 二进制（workspace 带 debug=1 符号），
只跑 all_paint 场景。先修掉一个自伤：request_redraw 的计数器每次调用读
`SIG_FRAME_STATS` 环境变量（getenv 带全局锁，占 wall ~6.6%），改 OnceLock 缓存。

主线程采样归组（约 6800 样本）：

| 桶 | 采样占比 | 内容 |
|---|---|---|
| HashMap 机制开销 | **~33%** | `hash_one`（RandomState/SipHash 哈希 fieldless 枚举键！）×3 实例 + `reserve_rehash` + `Iter::next` + insert |
| 分配/内存拷贝 | ~12% | malloc/free 家族、memmove、RawVec 释放、Vec::clone |
| 变更检测全枚举扫描 | ~8% | `StylePropertyKey::value` × 全 variant 遍历 + `PartialEq::ne` + `Option::cloned` |
| 真实属性工作 | ~40% | compute_style 主体、taffy `assign_to`、render_view（scene 段）、taffy layout、samara effect run |

结论：style 段约六成时间是纯机制开销（哈希/分配/枚举扫描），四成是真实工作。
第一优先级：`StylePropertyKey`/`TaffyStylePropertyKey` 均为 fieldless 枚举，
属性容器（`ctx.style`、`style_props`、`taffy_style_props`、`style_cache`）
应从 `HashMap<Key, Value>` 换成 `[Option<Value>; VARIANT_COUNT]` 数组
（strum `EnumCount`，键即下标）——哈希、rehash、迭代分配全部归零。

### #4 属性容器数组化（fieldless 枚举键 → 判别式下标）

依据 #3 剖析（哈希 ~33% + 分配 ~12%）。`StylePropertyKey`（16）与
`TaffyStylePropertyKey`（50）为 fieldless 枚举，strum `EnumCount` 提供编译期
长度。容器变更：

- `StyleComputeContext.style`、compute_style 的两个局部属性映射、
  `ViewState.style_cache`：`HashMap<Key, Value>` →
  `[Option<Value>; Key::COUNT]`（`StylePropertyMap`/`TaffyStylePropertyMap`）
- undo 日志改记下标；style_cache 回填从"清空+逐键 insert"变成数组赋值
- `StyleProperty`/`TextWrap`/`PointerEvents` 补 `Copy`（载荷全是 Copy 类型，
  `Option<StyleProperty>` 靠判别式余量零膨胀）

style_bench（p50）：all_paint style 1218→**780µs（-36%）**（300 行 342→220µs
同幅）；effects 396→333µs；all_layout style 886→783µs；one_paint/clean/
layout 场景不变。frame_bench 1000 行：style 段 1.4ms→**995µs**，帧总耗时
3.2→2.6ms。sig 98/98、sig-reactive 142/142。

对照（自 #1 起累计）：all_paint 单迭代 style+effects+layout 合计
3215µs → **1142µs（-64%）**。

### #5 渲染路径 HashMap 查找换 FxHash（scene 段剖析驱动）

先剖析（clean 场景隔离 scene 段）：**55% 是渲染路径的 HashMap 查找哈希**
（`render_view` 每视图、`get_children` 每节点对 `HashMap<NodeId/ViewId, _>` 做
SipHash——键是 slotmap 的稠密 u64）；vello 编码本体 ~25%；taffy 缓存遍历 ~8%；
**parley 文本布局仅 ~3%**（"文本是大头"的猜测被数据否定）。

修复：`LayoutState.styles`、`view_states`、`style_dirty_nodes` 换
`rustc_hash::FxHashMap/FxHashSet`（u64 键乘法哈希）。

style_bench（1000 行 p50）：clean scene 272→**141µs（-48%）**；all_paint style
780→**548µs（-30%）**（style 段同查这些 map，连带受益）；one_paint style
31→16µs；effects 333→283µs。frame_bench 1000 行：style 995→825µs、scene
550→424µs，帧:写 110/100 正常。复剖析：`hash_one`/`write_u64` 从榜单消失，
scene 段头名变为 `render_view` 编码本体。sig 98/98。

累计（#1 起）：all_paint 单迭代 style+effects+layout+scene 合计
≈4.5ms → **1.5ms（-67%）**。剩余大头为真实编码工作（render_view）与
taffy 布局，下一步才是结构性的 scene 损伤追踪（子树缓存/边界）。
