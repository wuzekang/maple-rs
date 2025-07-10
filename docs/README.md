# MapleStory 技术文档库

本文档库包含了对 MapleStory 客户端核心系统的深入技术分析，主要聚焦于移动系统、物理引擎、怪物AI等关键组件的实现机制。

## 文档结构概览

### 📁 [architecture](./architecture/) - 核心架构
系统底层架构和基础组件的技术分析。

| 文档 | 描述 |
|------|------|
| [collision-detection-analysis.md](./architecture/collision-detection-analysis.md) | 碰撞检测系统：TRSTree空间分割、CStaticFoothold踏脚点系统 |
| [physics-formulas-analysis.md](./architecture/physics-formulas-analysis.md) | 物理公式详解：dWalkForce数组、速度计算、阻力系统 |
| [time-framerate-system-analysis.md](./architecture/time-framerate-system-analysis.md) | 时间系统：帧率控制、物理时间步长、网络同步机制 |

### 👤 [character-movement](./character-movement/) - 角色移动系统
玩家角色的完整移动控制系统分析。

| 文档 | 描述 |
|------|------|
| [character-movement-system.md](./character-movement/character-movement-system.md) | **核心文档** - 角色移动系统完整分析 |
| [movement-system-analysis.md](./character-movement/movement-system-analysis.md) | CVecCtrlUser类深度分析：物理计算、状态管理 |
| [input-system-analysis.md](./character-movement/input-system-analysis.md) | 输入系统：CInputSystem、按键处理、连击识别 |
| [movement-keys-analysis.md](./character-movement/movement-keys-analysis.md) | 移动按键映射和功能键分析 |
| [movement-state-machine-analysis.md](./character-movement/movement-state-machine-analysis.md) | 状态机：Walk、Jump、Fall、Swim等状态转换 |
| [combo-system-analysis.md](./character-movement/combo-system-analysis.md) | 连击系统：CSequencedKeyMan、按键序列识别 |

#### 📁 [detailed-analysis](./character-movement/detailed-analysis/) - 深度技术分析
角色移动系统的底层实现细节。

| 文档 | 描述 |
|------|------|
| [collision-response-math-analysis.md](./character-movement/detailed-analysis/collision-response-math-analysis.md) | 碰撞响应数学计算：速度投影、法线向量 |
| [ladder-rope-system-analysis.md](./character-movement/detailed-analysis/ladder-rope-system-analysis.md) | 梯子/绳子系统：状态检测、移动控制 |
| [movement-constraints-analysis.md](./character-movement/detailed-analysis/movement-constraints-analysis.md) | 移动约束：地图边界、状态限制、优先级 |
| [physics-parameters-analysis.md](./character-movement/detailed-analysis/physics-parameters-analysis.md) | 物理参数相互作用：装备属性、地形影响 |
| [special-movement-states-analysis.md](./character-movement/detailed-analysis/special-movement-states-analysis.md) | 特殊移动状态：冲刺、击飞、传送技能 |
| [state-duration-system.md](./character-movement/detailed-analysis/state-duration-system.md) | 状态持续时间系统：各状态的时间管理 |
| [state-priority-rules.md](./character-movement/detailed-analysis/state-priority-rules.md) | 状态优先级规则：冲突解决、转换规则 |
| [state-transition-analysis.md](./character-movement/detailed-analysis/state-transition-analysis.md) | 状态转换分析：地面↔空中↔游泳↔飞行 |
| [timestep-handling-analysis.md](./character-movement/detailed-analysis/timestep-handling-analysis.md) | 时间步长处理：数值稳定性、积分算法 |
| [onetimeaction-reference.md](./character-movement/detailed-analysis/onetimeaction-reference.md) | OneTimeAction完整参考：动作编码表 |

### 🤖 [mob-ai](./mob-ai/) - 怪物AI系统
怪物移动AI和行为控制系统。

| 文档 | 描述 |
|------|------|
| [mob-movement-analysis.md](./mob-ai/mob-movement-analysis.md) | **核心文档** - 怪物移动系统完整分析 |
| [mob-move-ability-analysis.md](./mob-ai/mob-move-ability-analysis.md) | 移动能力分类：静止、行走、跳跃、飞行、护送模式 |
| [flying-mob-movement-analysis.md](./mob-ai/flying-mob-movement-analysis.md) | 飞行型怪物：随机巡逻、围绕攻击、区域控制 |
| [random-movement-analysis.md](./mob-ai/random-movement-analysis.md) | 随机移动机制：概率分布、时间控制 |

### 🔧 [technical-analysis](./technical-analysis/) - 逆向工程分析
基于IDA Pro的底层技术分析。

| 文档 | 描述 |
|------|------|
| [ida-analysis-cmob.md](./technical-analysis/ida-analysis-cmob.md) | CMob类逆向分析：虚函数表、继承关系 |
| [ida-analysis-cvecctrl-escort.md](./technical-analysis/ida-analysis-cvecctrl-escort.md) | 护送模式AI：路径导航、碰撞检测 |
| [ida-analysis-cvecctrl.md](./technical-analysis/ida-analysis-cvecctrl.md) | CVecCtrl基础函数：Jump、FallDown实现 |
| [death-state-analysis.md](./technical-analysis/death-state-analysis.md) | 死亡状态分析 |
| [missing-information-analysis.md](./technical-analysis/missing-information-analysis.md) | 待研究信息清单 |

### 📚 [reference](./reference/) - 参考资料
技术规范和格式文档。

| 文档 | 描述 |
|------|------|
| [wz-format.md](./reference/wz-format.md) | WZ文件格式规范：结构、压缩、加密 |

## 阅读建议

### 🎯 如果你想了解...

**角色移动系统实现**
1. 先读 [character-movement-system.md](./character-movement/character-movement-system.md) - 获取整体架构
2. 深入 [movement-system-analysis.md](./character-movement/movement-system-analysis.md) - 了解CVecCtrlUser实现
3. 参考 [detailed-analysis](./character-movement/detailed-analysis/) 下的具体技术细节

**怪物AI行为**
1. 先读 [mob-movement-analysis.md](./mob-ai/mob-movement-analysis.md) - 全面了解怪物移动
2. 根据兴趣深入特定移动模式的分析文档

**底层技术实现**
1. 查看 [architecture](./architecture/) 下的基础架构文档
2. 参考 [technical-analysis](./technical-analysis/) 下的逆向分析

**实现自己的服务器**
1. 重点关注物理公式和算法实现
2. 理解网络同步机制
3. 参考状态机和约束系统的设计