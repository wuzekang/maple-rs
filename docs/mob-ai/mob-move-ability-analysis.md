# 怪物移动能力（MoveAbility）分析报告

## 概述
本文档详细分析了MapleStory客户端中怪物的移动能力系统，特别是 `nMoveAbility` 字段的含义和各种移动模式的实现。

## nMoveAbility 数值定义

根据 `CVecCtrlMob::WorkUpdateActive` 函数（地址：0x99d450）中的 switch 语句分析，`nMoveAbility` 的值定义如下：

| 数值 | 含义 | 对应函数 | 描述 |
|------|------|----------|------|
| 0 | 静止模式 | `CVecCtrlMob::CtrlUpdateActiveStop` | 怪物保持静止，可能会转身面向目标 |
| 1 | 行走模式 | `CVecCtrlMob::CtrlUpdateActiveMove` | 怪物可以左右行走移动 |
| 3 | 跳跃模式 | `CVecCtrlMob::CtrlUpdateActiveJump` | 怪物可以跳跃移动 |
| 4 | 飞行模式 | `CVecCtrlMob::CtrlUpdateActiveFly` | 怪物可以在空中飞行 |
| 6 | 护送模式 | `CVecCtrlMob::CtrlUpdateActiveEscort` | 怪物按照预定路径移动 |

注意：数值 2 和 5 在代码中没有对应的处理，可能是保留值或未使用。

## 各移动模式详细分析

### 1. 静止模式 (nMoveAbility = 0)
**函数**: `CVecCtrlMob::CtrlUpdateActiveStop` (0x99b6a0)

特点：
- 怪物不会主动移动
- 如果有目标，会转身面向目标
- 会判断目标是否在攻击范围内
- 可能触发追击行为

关键逻辑：
```cpp
// 根据目标位置调整朝向
if (target_x >= mob_x)
    direction = 1;  // 面向右
else
    direction = -1; // 面向左

// 检查是否需要追击
if (!IsTargetInAttackRange())
    CVecCtrlMob::ChaseTarget(this, 0, 0, 0);
```

### 2. 行走模式 (nMoveAbility = 1)
**函数**: `CVecCtrlMob::CtrlUpdateActiveMove` (0x99d090)

特点：
- 最常见的移动模式
- 支持随机移动和目标追击
- 可以左右行走，不能跳跃

移动策略：
- **无目标时**：随机移动
  - 移动时间：1000-2000ms（随机）
  - 方向选择：左(-1)、停止(0)、右(1)等概率
- **有目标时**：智能追击
  - 朝向目标移动
  - 考虑边界限制
  - 动态调整移动时间

### 3. 跳跃模式 (nMoveAbility = 3)
**函数**: `CVecCtrlMob::CtrlUpdateActiveJump` (0x99b930)

特点：
- 可以跳跃越过障碍
- 支持平台间移动
- 通常用于地形复杂的地图

### 4. 飞行模式 (nMoveAbility = 4)
**函数**: `CVecCtrlMob::CtrlUpdateActiveFly` (0x99c1f0)

特点：
- 可以在空中自由飞行
- 不受重力影响
- 可以上下左右移动
- 调用前会执行 `FlyCtrlGuardingBefore` 进行预处理

### 5. 护送模式 (nMoveAbility = 6)
**函数**: `CVecCtrlMob::CtrlUpdateActiveEscort` (0x99b8d0)

特点：
- 按照预定义路径点移动
- 支持梯子/绳子移动
- 可以设置停留点
- 支持跨Z层移动

护送点结构：
```cpp
struct EscortDest {
    int x;              // X坐标
    int y;              // Y坐标
    int nAttr;          // 属性（1=梯子/绳子，2=停留点）
    int ZMass;          // Z层
    int nStopDuration;  // 停留时间
};
```

## CMobTemplate 中的相关字段

根据分析，`CMobTemplate` 结构中应该包含：
- `nMoveAbility`: 移动能力类型（0-6）
- `speed`: 移动速度
- `nAttackCount`: 攻击次数
- 其他怪物属性...

## 移动能力的应用场景

1. **静止型怪物** (nMoveAbility = 0)
   - 蘑菇王等BOSS
   - 固定位置的防御型怪物

2. **行走型怪物** (nMoveAbility = 1)
   - 大部分普通怪物
   - 蜗牛、绿水灵等

3. **跳跃型怪物** (nMoveAbility = 3)
   - 需要在平台间移动的怪物
   - 某些精英怪物

4. **飞行型怪物** (nMoveAbility = 4)
   - 蝙蝠类怪物
   - 飞行系怪物

5. **护送型怪物** (nMoveAbility = 6)
   - 任务相关的NPC怪物
   - 需要沿特定路线巡逻的怪物

## 实现建议

在 Rust 实现中，可以定义枚举类型：

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MobMoveAbility {
    Stop = 0,     // 静止
    Move = 1,     // 行走
    Jump = 3,     // 跳跃
    Fly = 4,      // 飞行
    Escort = 6,   // 护送
}

impl MobMoveAbility {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Stop),
            1 => Some(Self::Move),
            3 => Some(Self::Jump),
            4 => Some(Self::Fly),
            6 => Some(Self::Escort),
            _ => None,
        }
    }
}
```

## 相关文档
- [怪物随机移动机制分析](./random-movement-analysis.md)
- [CVecCtrlMob 护送模式 AI 逻辑分析](./ida-analysis-cvecctrl-escort.md)
- [CVecCtrl 分析](./ida-analysis-cvecctrl.md)