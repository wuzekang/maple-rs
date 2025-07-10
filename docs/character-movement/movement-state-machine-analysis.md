# 冒险岛角色移动状态机完整分析

基于 IDA Pro 逆向分析结果

## 1. 核心状态定义

### 1.1 移动动作状态码 (m_nMoveAction)

根据 `CUser::OnResolveMoveAction` 函数分析，移动状态由以下部分组成：
- **最低位（& 1）**：角色朝向（0=右，1=左）
- **高位（>> 1）**：实际动作状态

### 1.2 主要移动状态列表

| 状态码 | 状态名称 | 描述 | 条件 |
|--------|----------|------|------|
| 1 | Walk | 行走 | 在地面上，有水平输入 |
| 2 | Stand | 站立 | 在地面上，无输入 |
| 3 | Fall/Float | 空中下落 | 不在地面上，默认状态 |
| 4 | ProneStab | 趴下 | 在地面上，有垂直输入（下） |
| 5 | Jump | 跳跃 | 在地面上，有垂直输入（上） |
| 6 | Swim | 游泳 | 在游泳区域内 |
| 7 | Ladder | 爬梯子 | 在梯子上 |
| 8 | Rope | 爬绳子 | 在绳子上 |
| 17 | Flying (稳定) | 飞行状态（稳定） | 使用飞行技能，已停止 |
| 18 | Flying (移动) | 飞行状态（移动） | 使用飞行技能，正在移动 |
| 19 | Dash | 冲刺 | 使用冲刺技能（如4321000） |
| 20 | RocketBooster | 火箭推进 | 使用火箭靴技能 |

### 1.3 OnResolveMoveAction 函数详细分析

基于实际反编译代码的准确分析：

```cpp
int __thiscall CUser::OnResolveMoveAction(
    CUser *this,
    int nInputX,      // 水平输入 (-1=左, 0=无, 1=右)
    int nInputY,      // 垂直输入 (-1=上, 0=无, 1=下)
    int nCurMoveAction,
    CVecCtrl *pvc)
{
    int nAction = 0;
    char bDirection = nCurMoveAction & 1;  // 保存当前朝向
    
    // 特殊动作处理（OneTimeAction）
    if (特殊动作状态) {
        // 处理特殊动作如坐下、趴下等
    }
    
    // 主要状态判断逻辑
    if (nInputX != 0) {  // 有水平输入
        if (pvc->m_pfh) {  // 在地面上
            if (IsDashing && GetDashingSkill() == 4321000) {
                nAction = 19;  // 冲刺状态
                bDirection = (nInputX < 0);
            } else {
                nAction = 1;   // 行走状态
                // 处理从冲刺转为普通行走
                if (m_pSpectrumAniState == 19) {
                    SetOneTimeAction(81);  // 停止冲刺动画
                    // 设置冲刺减速参数
                    pvc->m_dShortDrag = 0.4;
                    pvc->m_nSlideCount = 400;
                }
                bDirection = (nInputX < 0);
            }
        }
    } else if (pvc->m_pfh) {  // 在地面上，无水平输入
        if (nInputY > 0) {
            nAction = 5;  // 跳跃准备
        } else {
            nAction = (m_bDelayedLoad > 0) ? 4 : 2;  // 趴下或站立
        }
    }
    
    // 特殊地形状态
    if (CVecCtrl::IsOnLadder(pvc)) {
        // 特殊道具检查（1902040, 1902041, 1902042）
        if (!IsSpecialItem()) {
            nAction = 7;  // 梯子状态
        }
    } else if (CVecCtrl::IsOnRope(pvc)) {
        if (!IsRidingEvanDragon()) {
            nAction = 8;  // 绳子状态
        }
    }
    
    // 空中状态
    if (!pvc->m_pfh) {  // 不在地面上
        nAction = 3;  // 默认空中状态
        
        if (m_bRocketBoosterStart) {
            nAction = 20;  // 火箭推进状态
        }
        
        if (CVecCtrl::IsSwimming(pvc)) {
            nAction = 6;  // 游泳状态
        }
        
        // 飞行状态处理（关键逻辑）
        if (IsFlying() && GetOneTimeAction() <= -1) {
            if (m_bForcedInvisible / 10000 == 193) {  // 特殊飞行道具
                if (IsStopped() || IsRidingWildHunterJaguar() || 
                    m_bForcedInvisible == 1932016) {
                    nAction = 6;  // 特殊游泳动作
                } else {
                    nAction = 2;  // 特殊站立动作
                }
            } else {
                // 重要：这里使用 18 - (IsStopped() != 0)
                // 如果停止：nAction = 18 - 1 = 17（飞行静止）
                // 如果移动：nAction = 18 - 0 = 18（飞行移动）
                nAction = 18 - (CVecCtrl::IsStopped(pvc) != 0);
            }
        }
        
        // 更新方向
        if (nInputX < 0) {
            bDirection = 1;
        } else if (nInputX > 0) {
            bDirection = 0;
        }
    }
    
    // 特殊跳跃动作处理（OneTimeAction == 207）
    if (GetOneTimeAction() == 207 && (nAction == 7 || nAction == 8)) {
        // 从梯子/绳子跳跃的特殊处理
        ClearActionLayer(1);
        SetAction(6, 100, 0);
    }
    
    // 保存状态
    m_pSpectrumAniState = nAction;
    
    // 返回完整的移动动作码（动作 * 2 + 方向）
    return (nAction * 2) | (bDirection & 1);
}
```

## 2. 辅助状态判断函数

### 2.1 is_stand_action 函数
```cpp
BOOL is_stand_action(int nAction) {
    return (nAction >= 2 && nAction <= 3) ||  // 基础站立
           (nAction >= 48 && nAction <= 54) || // 扩展站立动作
           (nAction == 125);                   // 特殊站立
}
```

### 2.2 is_back_action 函数
```cpp
BOOL is_back_action(int nAction, int bMorphed) {
    if (bMorphed) {
        return (nAction == 9 || nAction == 10);  // 变身后退动作
    } else {
        return (nAction == 45 || nAction == 46 || 
                nAction == 129 || nAction == 130);  // 普通后退动作
    }
}
```

### 2.3 get_action_from_act_dir 函数
```cpp
int get_action_from_act_dir(int act_dir) {
    return act_dir >> 1;  // 右移1位去除方向位
}
```

## 3. 特殊状态 (OneTimeAction)

OneTimeAction 用于临时动作，不影响基础移动状态：

| OneTimeAction | 描述 |
|---------------|------|
| '-' (45) | 特殊动作1 |
| '.' (46) | 特殊动作2 |
| '@' (64) | 特殊动作3 |
| 'A' (65) | 特殊动作4 |
| 81 | 停止冲刺效果 |
| 207 | 从梯子/绳子跳跃 |

## 4. 状态转换规则

### 4.1 地面状态转换
- **站立 → 行走**：有水平输入
- **站立 → 跳跃**：按跳跃键
- **站立 → 趴下**：按下键
- **行走 → 冲刺**：使用冲刺技能
- **冲刺 → 行走**：停止冲刺

### 4.2 空中状态转换
- **任意地面状态 → 空中**：离开地面
- **空中 → 游泳**：进入游泳区域
- **空中 → 飞行**：使用飞行技能
- **飞行 → 空中**：取消飞行

### 4.3 特殊地形转换
- **任意状态 → 梯子**：碰到梯子 + 上下键
- **任意状态 → 绳子**：碰到绳子 + 上下键
- **梯子/绳子 → 跳跃**：按跳跃键

## 5. 状态机特性

### 5.1 状态优先级
1. 特殊道具状态（最高优先级）
2. 地形限制状态（梯子、绳子）
3. 环境状态（游泳区域）
4. 技能状态（飞行、冲刺）
5. 基础移动状态（最低优先级）

### 5.2 方向处理
- 方向独立于动作状态
- 始终保存在最低位
- 某些状态下方向会被锁定（如梯子、绳子）

### 5.3 状态持续性
- 大部分状态需要持续满足条件
- OneTimeAction 为临时状态，会自动结束
- 某些状态有最小持续时间（如冲刺）

## 6. 实现建议

### 6.1 状态机结构
```rust
pub struct MovementStateMachine {
    current_state: MovementState,
    current_direction: Direction,
    one_time_action: Option<OneTimeAction>,
    state_timer: f32,
}

pub enum MovementState {
    Walk = 1,
    Stand = 2,
    Fall = 3,
    ProneStab = 4,
    Jump = 5,
    Swim = 6,
    Ladder = 7,
    Rope = 8,
    FlyingStable = 17,
    FlyingMove = 18,
    Dash = 19,
    RocketBooster = 20,
}

pub enum Direction {
    Right = 0,
    Left = 1,
}
```

### 6.2 状态更新逻辑
```rust
impl MovementStateMachine {
    pub fn update(&mut self, input: &Input, physics: &PhysicsState) -> u32 {
        // 1. 处理 OneTimeAction
        if let Some(action) = self.one_time_action {
            // 处理特殊动作
        }
        
        // 2. 根据物理状态和输入计算新状态
        let new_state = self.resolve_state(input, physics);
        
        // 3. 处理状态转换
        if new_state != self.current_state {
            self.on_state_exit(self.current_state);
            self.on_state_enter(new_state);
            self.current_state = new_state;
        }
        
        // 4. 更新方向
        if input.horizontal != 0 {
            self.current_direction = if input.horizontal < 0 {
                Direction::Left
            } else {
                Direction::Right
            };
        }
        
        // 5. 返回完整的移动动作码
        (self.current_state as u32) * 2 | (self.current_direction as u32)
    }
}
```

## 7. 重要发现和修正

### 7.1 飞行状态的正确理解
通过仔细分析第200行代码：
```cpp
v12 = 18 - (CVecCtrl::IsStopped(pvc) != 0);
```
- 状态 17：飞行静止（当 IsStopped() == true）
- 状态 18：飞行移动（当 IsStopped() == false）

### 7.2 死亡状态的疑问
之前认为 `(m_nMoveAction & 0xFFFFFFFE) == 18` 是死亡状态的判断是错误的：
- 18 是 `状态9 << 1` 的结果
- 但在 OnResolveMoveAction 中，状态18是飞行移动
- 死亡状态可能通过其他机制处理，不在基础移动状态中

## 8. 注意事项

1. **服务器验证**：所有状态转换都需要服务器验证
2. **动画同步**：状态改变需要同步更新动画系统
3. **物理同步**：某些状态会影响物理参数（如游泳速度）
4. **技能交互**：技能可能会强制改变状态
5. **道具影响**：特殊道具会改变状态行为

## 9. 未完全解析的部分

- 死亡状态的实际处理机制
- 完整的 OneTimeAction 列表
- 所有扩展站立动作（48-54）的具体含义
- 特殊道具 ID（1902040-1902042, 1932016）的确切效果
- 部分职业特有的移动状态
- 坐骑系统对状态机的完整影响