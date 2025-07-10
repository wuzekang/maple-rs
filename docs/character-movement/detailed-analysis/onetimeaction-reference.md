# MapleStory OneTimeAction 完整参考手册

本文档基于对 MapleStory v95 客户端的逆向工程分析，提供 OneTimeAction 系统的完整参考。

## OneTimeAction 系统概述

OneTimeAction 是冒险岛中的一次性动作系统，用于播放不循环的动画效果，如攻击、技能施放、特殊动作等。这些动作具有最高的优先级，可以打断常规的移动状态。

## 核心函数

### CAvatar::GetOneTimeAction
**地址**: `0x45f3c0`  
**作用**: 获取当前的 OneTimeAction，并根据角色状态（变身、幽灵等）进行映射转换

### CAvatar::SetOneTimeAction  
**地址**: `0x46aba0`  
**作用**: 设置新的 OneTimeAction

```cpp
void CAvatar::SetOneTimeAction(int nAction) {
    m_nOneTimeAction = -1;           // 重置当前动作
    m_nTamingMobOneTimeAction = -1;  // 重置坐骑动作
    ClearActionLayer(1);             // 清除动作层
    m_nOneTimeAction = nAction;      // 设置新动作
    PrepareActionLayer(6, 100, 0);   // 准备动作层，优先级100
}
```

## OneTimeAction 编码表

### 普通角色动作

| OneTimeAction | 实际动作ID | 动作类型 | 描述 |
|---------------|------------|----------|------|
| **9** | 16 | 攻击 | 基础攻击动作1 |
| **11** | 17 | 攻击 | 基础攻击动作2 |
| **16** | 21 | 技能 | 技能施放动作1 |
| **17** | 22 | 技能 | 技能施放动作2 |
| **31** | 18 | 防御 | 防御/格挡动作 |
| **36** | 19 | 攻击 | 特殊攻击动作1 |
| **41** | 20 | 攻击 | 特殊攻击动作2 |

### 移动相关动作

| OneTimeAction | 实际动作ID | 动作类型 | 描述 |
|---------------|------------|----------|------|
| **45** | - | 移动 | 向后移动（'-'）|
| **46** | - | 移动 | 向后移动（'.'）|
| **81** | - | 特殊 | 停止冲刺效果 |
| **207** | - | 特殊 | 梯子/绳子跳跃 |

### 状态动作

| OneTimeAction | 实际动作ID | 动作类型 | 描述 |
|---------------|------------|----------|------|
| **58** | 4 | 坐下 | 坐下变化1 |
| **59** | 5 | 坐下 | 坐下变化2 |
| **60** | 6 | 坐下 | 坐下变化3 |
| **61** | 3 | 趴下 | 趴下动作 |
| **64** | 27 | 站立 | 警戒站立（'@'）|
| **65** | 28 | 站立 | 警戒站立（'A'）|

### 技能动作

| OneTimeAction | 实际动作ID | 动作类型 | 描述 |
|---------------|------------|----------|------|
| **101** | 31 | 连击 | 连击技能1 |
| **102** | 32 | 连击 | 连击技能2 |
| **103** | 25 | 蓄力 | 蓄力技能 |
| **104** | 35 | 技能 | 特殊技能1 |
| **105** | 36 | 技能 | 特殊技能2 |
| **106** | 33 | 技能 | 特殊技能3 |
| **107** | 34 | 技能 | 特殊技能4 |
| **108** | 38 | 技能 | 特殊技能5 |
| **109** | 30 | 技能 | 特殊技能6 |

### 高级技能动作

| OneTimeAction | 实际动作ID | 动作类型 | 描述 |
|---------------|------------|----------|------|
| **113** | 26 | 高级 | 高级技能1 |
| **117** | 39 | 高级 | 高级技能2 |
| **120** | 37 | 高级 | 高级技能3 |
| **121** | 24 | 高级 | 高级技能4 |
| **137** | 40 | 终结 | 终结技能 |
| **142** | 29 | 特殊 | 特殊动作 |

### 变身动作

| OneTimeAction | 实际动作ID | 动作类型 | 描述 |
|---------------|------------|----------|------|
| **265** | 44 | 变身 | 变身基础动作 |
| **266** | 45 | 变身 | 变身动作1 |
| **267** | 46 | 变身 | 变身动作2 |
| **268** | 47 | 变身 | 变身动作3 |
| **269** | 48 | 变身 | 变身动作4 |
| **270** | 41 | 变身 | 变身动作5 |
| **271** | 42 | 变身 | 变身动作6 |
| **272** | 43 | 变身 | 变身动作7 |

## 特殊角色状态映射

### 幽灵模式（m_nGhostIndex > 0）

| 移动状态 | 映射到的动作ID |
|----------|----------------|
| 行走(1) | 124 |
| 站立(2) | 125（默认）|
| 下落(3) | 126 |
| 跳跃(5) | 127 |
| 游泳(6) | 128 |
| 梯子(7) | 129 |
| 绳子(8) | 130 |
| 攻击(9) | 47 |
| 坐下(10) | 131 |

### 机械师模式

| 机械师技能ID | 映射的动作 | 特殊处理 |
|--------------|-----------|----------|
| 35111004 | 217 | 固定返回 |
| 35121005 | 221/223 | 根据状态：行走=223，跳跃=56，其他=221 |
| 35121005（火箭）| 210 | 当 m_bRocketBoosterStart 或 m_bRocketBoosterLoop 时 |
| 35121013 | 227 | 固定返回 |
| 35001001 | 234 | 固定返回 |
| 35101009 | 237 | 固定返回 |

### 变身状态映射

**怪物变身（IsMonsterMorphed）和隐藏变身（IsHideMorphed）**：
- 使用简化的动作集（1-12, 41-42）
- 某些 OneTimeAction 会映射到基础动作

**超人变身（IsSuperMan）**：
- 保留大部分原始动作
- 特殊处理变身相关的 265-272

**可攻击变身（IsAttackableMorphed）**：
- 完整的动作映射
- 支持所有战斗相关动作

## OneTimeAction 的特殊处理

### 状态转换中的特殊处理

在 `CUser::OnResolveMoveAction` 中，某些 OneTimeAction 有特殊处理：

```cpp
// 影响站立状态判断的 OneTimeAction
case '-':  // 45
case '.':  // 46  
case '@':  // 64
case 'A':  // 65
    // 这些动作会影响是否显示站立动画
    break;

// 梯子/绳子跳跃
if (GetOneTimeAction() == 207 && (state == 7 || state == 8)) {
    ClearActionLayer(1);
    SetAction(6, 100, 0);  // 强制设置跳跃动作
}
```

### 冲刺停止特效

```cpp
if (m_pSpectrumAniState.p == 19) {  // 当前在冲刺状态
    SetOneTimeAction(81);  // 设置停止冲刺动作
    // 播放停止特效
    Effect_General("Skill/432.img/skill/4321000/stopEffect", ...);
}
```

## 使用建议

### 1. 动作优先级

OneTimeAction 具有最高优先级，会覆盖常规移动状态。在实现时应该：

```rust
pub struct Avatar {
    one_time_action: Option<i32>,
    one_time_action_frame: u32,  // 当前帧数
    one_time_action_duration: u32,  // 总帧数
}

impl Avatar {
    pub fn update(&mut self) {
        // 优先处理 OneTimeAction
        if let Some(action) = self.one_time_action {
            self.one_time_action_frame += 1;
            if self.one_time_action_frame >= self.one_time_action_duration {
                self.clear_one_time_action();
            }
            return;  // OneTimeAction 期间不处理其他状态
        }
        
        // 处理常规状态
        self.update_regular_state();
    }
}
```

### 2. 动作映射

不同角色状态需要不同的动作映射：

```rust
pub fn map_one_time_action(raw_action: i32, avatar: &Avatar) -> i32 {
    // 幽灵模式
    if avatar.ghost_index > 0 {
        return map_ghost_action(raw_action);
    }
    
    // 变身状态
    if let Some(morph_id) = avatar.morph_template_id {
        if is_superman(morph_id) {
            return map_superman_action(raw_action);
        }
        // ... 其他变身类型
    }
    
    // 机械师模式
    if let Some(mode) = avatar.mechanic_mode {
        return map_mechanic_action(raw_action, mode);
    }
    
    // 普通映射
    map_normal_action(raw_action)
}
```

### 3. 动作层管理

OneTimeAction 使用独立的动作层（层级1），优先级100：

```rust
pub struct ActionLayer {
    layer_id: u8,
    priority: u8,
    action: Option<Action>,
}

impl Avatar {
    pub fn set_one_time_action(&mut self, action: i32) {
        // 清除旧动作
        self.clear_action_layer(1);
        
        // 设置新动作
        self.one_time_action = Some(action);
        self.prepare_action_layer(1, 100, action);
    }
}
```

## 注意事项

1. **动作中断**：OneTimeAction 可以被新的 OneTimeAction 中断，但不能被常规移动状态中断
2. **动作完成**：动作完成后需要调用 `ClearActionLayer` 清除动作层
3. **坐骑兼容**：设置 OneTimeAction 时会同时重置 `m_nTamingMobOneTimeAction`
4. **网络同步**：OneTimeAction 需要同步到服务器，确保其他玩家看到相同的动作

## 调试技巧

1. 监控 `m_nOneTimeAction` 值的变化
2. 在 `PrepareActionLayer` 处设置断点查看动作层的准备
3. 使用 `GetOneTimeAction` 的返回值来验证映射是否正确
4. 检查动作资源文件（.wz）中对应动作ID的动画是否存在