# Sig 组件库设计准则

> **目标**：确保所有组件的用户体验一致，特别是在视觉反馈方面（尺寸、间距、圆角等）。
> 
> **优化方向**：为代码编辑器、游戏内嵌调试UI、MacOS配置界面等桌面应用场景设计，采用更紧凑的尺寸。

## 目录

- [架构概览](#架构概览)
- [Foundation Layer（基础层）](#foundation-layer基础层)
- [Component Layer（组件层）](#component-layer组件层)
- [Variant Layer（变体层）](#variant-layer变体层)
- [实施指南](#实施指南)

---

## 架构概览

本设计准则采用**三层架构**，确保组件视觉体验的一致性和可扩展性：

### 1. Foundation Layer（基础层）
定义最底层的设计 tokens，包括：
- **Spacing Scale**：统一的间距体系（基于 8px 基准）
- **Size Scale**：组件尺寸体系（height、width 的标准值）
- **Radius Scale**：圆角半径体系
- **Border Scale**：边框宽度体系

### 2. Component Layer（组件层）
定义组件如何消费 Foundation tokens：
- 组件的默认尺寸映射（如 Button.medium 使用 size.md）
- 组件的内边距规则（如 Button 的 padding-x 使用 spacing.lg）
- 组件的外边距建议（如 form 场景下的 margin-bottom）

### 3. Variant Layer（变体层）
定义组件变体的差异化规则：
- Size 变体（small/medium/large）如何调整尺寸和间距
- Style 变体的视觉差异（primary/secondary/outline/text）
- State 变体的交互反馈（hover/focus/press/disabled）

### 设计原则

- **Token 优先**：组件优先使用 Foundation tokens，避免硬编码数值
- **灵活覆盖**：允许通过 `.style()` 方法自定义，但需文档说明偏离标准的原因
- **渐进增强**：现有组件逐步迁移到新准则，新组件强制遵循

---

## Foundation Layer（基础层）

### 1. Spacing Scale（间距体系）

采用 **8px 基准系统**，确保所有间距都是 4px 的倍数（兼容高分屏和可访问性）。

| Token | 值 | 使用场景 |
|-------|-----|----------|
| `spacing.xxs` | 2px | 极小间距，用于非常紧密的元素 |
| `spacing.xs` | 4px | 极小间距，用于紧密元素 |
| `spacing.sm` | 8px | 小间距，用于相关元素 |
| `spacing.md` | 12px | 中等间距，默认内边距 |
| `spacing.lg` | 16px | 大间距，默认外边距 |
| `spacing.xl` | 20px | 超大间距，区块分隔 |
| `spacing.2xl` | 24px | 特大间距，主要区域分隔 |
| `spacing.3xl` | 32px | 巨大间距，页面级分隔 |

**推荐用法：**
- `padding`: 优先使用 `md`/`lg`
- `margin`: 优先使用 `lg`/`xl`
- `gap`（Stack/Grid）: 优先使用 `sm`/`md`/`lg`

---

### 2. Size Scale（尺寸体系）

定义组件高度和关键尺寸的标准值（优化为桌面应用的紧凑尺寸）。

| Token | 值 | 使用场景 |
|-------|-----|----------|
| `size.sm` | 24px | 小尺寸组件（紧凑UI、树形项） |
| `size.md` | 32px | 中等尺寸（默认，代码编辑器标准） |
| `size.lg` | 40px | 大尺寸（强调操作） |
| `size.xl` | 48px | 超大尺寸（英雄按钮，少用） |

**推荐用法：**
- Button、Input 等交互组件的 `height`
- Icon 容器的尺寸
- 可点击区域的最小尺寸（遵循可访问性标准）

**设计理念：**
- 比传统 Web UI（40/48/56px）更紧凑
- 适合代码编辑器、配置界面等桌面应用
- 保持足够的可点击性（最小 24px）

---

### 3. Radius Scale（圆角体系）

定义边框圆角的标准值（更微妙的圆角适合专业桌面应用）。

| Token | 值 | 使用场景 |
|-------|-----|----------|
| `radius.none` | 0px | 无圆角（特殊场景） |
| `radius.sm` | 2px | 小圆角（紧凑组件，微妙圆润） |
| `radius.md` | 4px | 中等圆角（默认，专业感） |
| `radius.lg` | 6px | 大圆角（卡片、容器） |
| `radius.xl` | 8px | 超大圆角（强调场景） |
| `radius.full` | 9999px | 完全圆形（徽章、头像） |

**推荐用法：**
- Button、Input: 优先使用 `md` (4px)
- Card、Modal: 优先使用 `lg` (6px)
- Badge、Avatar: 使用 `full`

**设计理念：**
- 比传统设计（4/6/8px）更微妙
- 保持专业、清爽的外观
- 避免过度圆润（不适合专业工具）

---

### 4. Border Scale（边框宽度）

| Token | 值 | 使用场景 |
|-------|-----|----------|
| `border.thin` | 1px | 标准边框（默认） |
| `border.medium` | 2px | 强调边框（focus 状态） |
| `border.thick` | 3px | 粗边框（特殊强调） |

---

### 5. Token 命名约定

- 使用 `.<scale>` 格式，避免具体数值（如用 `spacing.md` 而非 `spacing.12`）
- 保持语义化：`size` 用于尺寸，`spacing` 用于间距，`radius` 用于圆角
- 预留扩展空间：可增加 `4xl`、`5xl` 等更大的 scale

---

## Component Layer（组件层）

### 1. 交互组件规范

#### Button 组件

```
默认尺寸映射：
┌─────────────┬──────────┬────────────┬────────────┬─────────────────┐
│ Size        │ Height   │ Padding-X  │ Font Size  │ Border Radius   │
├─────────────┼──────────┼────────────┼────────────┼─────────────────┤
│ Small       │ 24px     │ 8px (sm)   │ 12px       │ 2px (sm)        │
│ Medium      │ 32px     │ 12px (md)  │ 14px       │ 4px (md)        │
│ Large       │ 40px     │ 16px (lg)  │ 16px       │ 4px (md)        │
└─────────────┴──────────┴────────────┴────────────┴─────────────────┘

边框宽度：
- default/filled: border.thin (1px)
- outline: border.thin (1px)
- focus 状态: border.medium (2px)

最小宽度：
- 文本按钮：无最小宽度限制
- 图标按钮：最小宽度 = height（确保正方形）
- 普通按钮：建议最小宽度 = height * 2

间距建议：
- 按钮组内 gap: spacing.sm (8px)
- 表单场景 margin-bottom: spacing.lg (16px)
```

#### Input 组件

```
默认尺寸映射：
┌─────────────┬──────────┬────────────┬────────────┬─────────────────┐
│ Size        │ Height   │ Padding-X  │ Font Size  │ Border Radius   │
├─────────────┼──────────┼────────────┼────────────┼─────────────────┤
│ Small       │ 24px     │ 12px (md)  │ 12px       │ 4px (md)        │
│ Medium      │ 32px     │ 12px (md)  │ 14px       │ 4px (md)        │
│ Large       │ 40px     │ 12px (md)  │ 16px       │ 4px (md)        │
└─────────────┴──────────┴────────────┴────────────┴─────────────────┘

边框宽度：
- default: border.thin (1px)
- focus: border.medium (2px)
- error: border.medium (2px)

特殊规则：
- 带前缀/后缀图标：图标与文本间距 spacing.sm (8px)
- Label 与 Input 间距：spacing.xs (4px)
- Helper text 间距：spacing.xs (4px)
```

---

### 2. 布局组件规范

#### VStack / HStack

```
默认 gap 映射：
- 紧密布局: spacing.sm (8px)
- 常规布局: spacing.md (12px) - 默认
- 宽松布局: spacing.lg (16px)

对齐规则：
- 默认 align-items: stretch（填充）
- 内容对齐: start/center/end 保持语义化
```

#### Container / Card

```
内边距规范：
- compact: spacing.md (12px)
- default: spacing.lg (16px)
- relaxed: spacing.xl (20px)

边框和圆角：
- border-radius: radius.lg (8px) - 比 Button 更圆润
- border-width: border.thin (1px)
```

---

### 3. 展示组件规范

#### Text

```
行高规则：
- 标题文本: line-height = font-size * 1.2
- 正文文本: line-height = font-size * 1.5
- 紧凑文本: line-height = font-size * 1.0

段落间距：
- 默认 margin-bottom: spacing.md (12px)
```

#### Icon

```
尺寸映射：
- 与文本内联：匹配 font-size
- 独立图标：16px (sm), 20px (md), 24px (lg)
- 与 Button 组合：Button height * 0.5
```

---

### 4. 复合组件规范

```
继承规则：
- 内部使用的 Button/Icon 遵循交互组件规范
- 层级缩进：spacing.lg (16px) 或 spacing.xl (20px)
- 项目间距：spacing.xs (4px) - spacing.sm (8px)

特殊调整：
- TreeView 展开/折叠图标：与文本间距 spacing.sm (8px)
- Collapsible trigger 与 content 间距：spacing.sm (8px)
- VirtualList 项目高度：应使用 size scale 或其倍数
```

---

## Variant Layer（变体层）

### 1. Size 变体规范

**核心原则：**
- Size 变体主要影响 **height** 和 **horizontal padding**
- **border-radius** 和 **font-size** 按比例缩放
- **vertical padding** 自动计算以保持视觉平衡

**具体映射规则：**

```rust
// Button / Input / Select 等交互组件

Small 变体：
- height: size.sm (32px)
- padding-x: spacing.md (12px)
- font-size: 12px
- border-radius: radius.sm (4px) - 比例缩小

Medium 变体（默认）：
- height: size.md (40px)
- padding-x: spacing.lg (16px)
- font-size: 14px
- border-radius: radius.md (6px)

Large 变体：
- height: size.lg (48px)
- padding-x: spacing.xl (20px)
- font-size: 16px
- border-radius: radius.md (6px) - 保持一致，避免过圆
```

**布局组件的 Size 变体：**

```rust
// Stack / Container 等布局组件

Compact：
- gap: spacing.sm (8px)
- padding: spacing.md (12px)

Default：
- gap: spacing.md (12px)
- padding: spacing.lg (16px)

Relaxed：
- gap: spacing.lg (16px)
- padding: spacing.xl (20px)
```

---

### 2. Style 变体规范

**视觉差异规则：**

```
Filled 类型（Primary / Secondary / Danger）：
├─ background: 实色填充
├─ border-width: border.thin (1px) - 可选，可与背景同色
├─ padding: 标准 padding（不调整）
├─ hover: 背景加深 10-15%
└─ press: 背景加深 20-25%

Outline 类型：
├─ background: transparent
├─ border-width: border.thin (1px) - 必须可见
├─ border-color: 使用主题色
├─ padding: 标准 padding（border 不占用内容空间）
├─ hover: 背景使用主题色 10% opacity
└─ press: 背景使用主题色 20% opacity

Text 类型：
├─ background: transparent
├─ border-width: 0
├─ padding: 减少 padding-x 到 spacing.sm (8px) - 更紧凑
├─ hover: 背景使用中性色 10% opacity
└─ press: 背景使用中性色 20% opacity
```

---

### 3. State 变体规范

**交互状态的视觉反馈：**

```
Hover 状态：
├─ 背景颜色变化：实色背景加深 10-15%，透明背景添加 10% opacity
├─ border 保持不变
├─ 光标：cursor.pointer
└─ 过渡动画：150ms ease-in-out（建议）

Focus 状态：
├─ border-width: 增加到 border.medium (2px)
├─ border-color: 使用焦点指示色（通常是主题色）
├─ outline: 可选，使用 2-4px 的外轮廓（可访问性）
└─ 其他样式保持不变

Press 状态：
├─ 背景颜色变化：实色背景加深 20-25%，透明背景添加 20% opacity
├─ 可选：scale(0.98) 轻微缩放效果
└─ 过渡动画：100ms ease-out

Disabled 状态：
├─ opacity: 0.5-0.6（整体降低透明度）
├─ cursor: cursor.not-allowed
├─ pointer-events: none
└─ 移除所有 hover/focus 效果
```

---

### 4. 组合变体优先级

**当多个变体组合时的优先级：**

```
优先级顺序（从高到低）：
1. State 变体（disabled 最高优先级）
2. Style 变体（决定基础视觉风格）
3. Size 变体（调整尺寸）

示例：<Button size="large" variant="outline" disabled>
1. 应用 Large size 的尺寸规则
2. 应用 Outline 的边框和背景规则
3. 最后应用 Disabled 的透明度和交互禁用
```

---

### 5. 特殊变体调整

```
Icon-only Button：
├─ width = height（强制正方形）
├─ padding: 0（使用 flexbox 居中）
└─ border-radius: 可选使用 radius.full 形成圆形按钮

Loading 状态：
├─ 内容 opacity: 0.6
├─ 添加 spinner（尺寸 = height * 0.5）
├─ disabled: true（阻止交互）
└─ 保持原有 size 和 variant 样式
```

---

## 实施指南

### 1. 新组件开发检查清单

开发新组件时，请按以下清单逐项检查：

#### ✓ Foundation Tokens 使用
```
□ 所有 padding/margin 使用 spacing scale，避免硬编码
□ 组件高度使用 size scale（或明确说明为什么需要自定义）
□ border-radius 使用 radius scale
□ border-width 使用 border scale
□ 颜色使用主题系统（本次准则未详述，但需遵循）
```

#### ✓ 变体完整性
```
□ 实现 size 变体（至少 small/medium/large）
□ 定义清晰的默认值（通常是 medium）
□ 如需 style 变体，至少支持 primary/secondary
□ 实现交互状态（hover/focus/press/disabled）
□ 状态之间的过渡动画平滑（建议 150ms）
```

#### ✓ 视觉一致性
```
□ 组件尺寸与同类组件对齐（如 Button 和 Input 同高）
□ 间距比例协调（内间距 < 外间距）
□ focus 状态有明确的视觉指示（可访问性）
□ disabled 状态清晰可辨（opacity 0.5-0.6）
```

#### ✓ API 设计
```
□ 变体使用枚举类型（ButtonSize、ButtonVariant）
□ 提供链式调用方法（.size().variant().style()）
□ 默认值符合准则（如默认 medium size）
□ 文档中说明与准则的关系
```

---

### 2. 现有组件迁移策略

**分阶段迁移：**

```
阶段 1：核心交互组件（优先级：高）
├─ Button ✓（已基本符合）
├─ Input / TextInput
└─ Select / Dropdown
目标：建立标准参考实现

阶段 2：布局组件（优先级：中）
├─ VStack / HStack
├─ Container / Card
└─ Grid
目标：统一间距系统

阶段 3：复合组件（优先级：中）
├─ TreeView
├─ VirtualList
└─ Collapsible
目标：继承基础组件的规范

阶段 4：特殊组件（优先级：低）
├─ Image
├─ Separator
└─ Dynamic
目标：按需适配准则
```

**迁移方法：**
1. 对比组件现状与准则差异
2. 创建迁移 checklist（哪些需要调整）
3. 保持 API 兼容性（添加新方法，deprecate 旧方法）
4. 更新文档和示例代码
5. 如有 breaking change，在版本说明中标注

---

### 3. 例外情况处理

**何时可以偏离准则：**

✓ **允许的例外情况：**
- 特定业务场景有明确的设计需求（需文档说明原因）
- 可访问性要求（如需要更大的点击区域）
- 性能优化（如 VirtualList 需要精确控制高度）
- 平台限制（如某些渲染引擎的限制）

✗ **不建议的例外：**
- "看起来更好" 的主观判断
- 复制其他组件的随意数值
- 未经讨论的自定义尺寸

**例外的文档化要求：**

```rust
// ✓ 良好的例外说明
impl SpecialComponent {
  pub fn new() -> Self {
    // 注意：此组件使用 36px 高度而非标准 size.md (40px)
    // 原因：需要与第三方库的固定布局对齐
    // 参考：issue #123
    Self { height: 36.0 }
  }
}

// ✗ 缺少说明的例外
impl BadComponent {
  pub fn new() -> Self {
    Self { height: 37.0 } // 为什么是 37？
  }
}
```

---

### 4. Token 系统扩展指南

**何时需要添加新的 token：**

✓ **应该添加新 token：**
- 多个组件需要相同的非标准值
- 发现现有 scale 有明显缺口（如需要 14px 间距但最近的是 12/16）
- 新的设计需求形成了模式

**添加流程：**
1. 在设计准则中提案（说明用途和场景）
2. 与团队讨论命名和位置（是否需要新的 scale？）
3. 更新文档
4. 在组件中使用并验证

✗ **不应该添加 token：**
- 仅一个组件使用的特殊值
- 可以通过组合现有 token 实现
- 与现有 token 差异过小（如已有 12px，又加 13px）

---

### 5. 组件文档化要求

**组件文档中必须包含的设计信息：**

```rust
//! Button component module
//!
//! # Design Specification
//!
//! ## Tokens Used
//! - Height: `size.sm/md/lg` (32px/40px/48px)
//! - Padding-X: `spacing.md/lg/xl` (12px/16px/20px)
//! - Border-Radius: `radius.md` (6px)
//!
//! ## Variants
//! - Size: small | medium (default) | large
//! - Style: primary (default) | secondary | outline | danger | text
//!
//! ## States
//! - hover: background darken 10-15%
//! - focus: border-width = 2px
//! - press: background darken 20-25%
//! - disabled: opacity = 0.5
//!
//! ## Deviations from Standard
//! - None (fully compliant with design guidelines)
```

---

### 6. 持续改进机制

```
准则评审周期：
├─ 每季度回顾一次设计准则
├─ 收集组件开发中的痛点
├─ 评估 token 使用频率（哪些过多/过少）
└─ 根据反馈调整 scale 和规则

反馈渠道：
├─ 在组件代码中添加 TODO 注释标记不合理之处
├─ 团队会议讨论设计一致性问题
└─ 文档中提供 "准则讨论" 专区
```

---

## 附录

### 快速参考表

**Spacing Scale:**
```
xxs: 2px | xs: 4px  | sm: 8px  | md: 12px | lg: 16px | xl: 20px | 2xl: 24px | 3xl: 32px
```

**Size Scale:**
```
sm: 24px | md: 32px | lg: 40px | xl: 48px
```

**Radius Scale:**
```
none: 0px | sm: 2px | md: 4px | lg: 6px | xl: 8px | full: 9999px
```

**Border Scale:**
```
thin: 1px | medium: 2px | thick: 3px
```

---

### 相关文档

- [组件模式](./PATTERNS.md) - 组件实现模式
- [组件文档](./COMPONENTS.md) - [测试指南](./TESTING.md) - 组件测试策略
