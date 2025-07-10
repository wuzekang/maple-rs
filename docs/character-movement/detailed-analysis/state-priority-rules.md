# 冒险岛角色移动状态转换优先级规则详细分析

本文档基于对 MapleStory v95 客户端的逆向工程分析，详细说明角色移动状态转换的优先级规则。

## 核心函数分析

### CUser::OnResolveMoveAction
**地址**: `0x8e5800`  
**原型**: `int __thiscall CUser::OnResolveMoveAction(CUser *this, int nInputX, int nInputY, int nCurMoveAction, CVecCtrl *pvc)`

这是决定角色移动状态的核心函数，每帧都会调用以确定角色应该处于什么状态。

## 状态转换优先级层次

### 1. OneTimeAction 处理（最高优先级）

OneTimeAction 是一次性动作，具有最高的执行优先级。

```cpp
// 特殊 OneTimeAction 处理
switch (OneTimeAction) {
    case '-':  // 45
    case '.':  // 46
    case '@':  // 64
    case 'A':  // 65
        // 这些动作影响站立状态的判断
        *m_pSpectrumAniState = (GetOneTimeAction() <= -1 && IsStopped() && !nInputY);
        break;
}
```

**特殊处理：OneTimeAction 207**
```cpp
if (GetOneTimeAction() == 207 && (v12 == 7 || v12 == 8)) {
    // 梯子/绳子跳跃
    ClearActionLayer(1);
    SetAction(6, 100, 0);  // 设置跳跃动作
}
```

### 2. 地面状态判断

当角色在地面上时（`pvc->m_pfh` 存在），按以下顺序判断：

#### 2.1 有水平输入（nInputX != 0）
```cpp
if (nInputX != 0) {
    if (GetDashingSkill() == 4321000 && 技能激活) {
        // 冲刺状态
        v12 = 19;
        v11 = nInputX < 0;
        
        // 设置冲刺参数
        pVecCtrl->m_dShortDrag = 0.4;
        pVecCtrl->m_nSlideCount = 400;
        
        // 如果之前在冲刺，播放停止特效
        if (m_pSpectrumAniState.p == 19) {
            SetOneTimeAction(81);
            Effect_General("Skill/432.img/skill/4321000/stopEffect", ...);
        }
    } else {
        // 普通行走
        v12 = 1;
        v11 = nInputX < 0;
    }
}
```

#### 2.2 无水平输入
```cpp
else {
    if (nInputY > 0) {
        // 向下输入 - 跳跃准备
        v12 = 5;
    } else if (m_bDelayedLoad > 0) {
        // 趴下状态
        v12 = 4;
    } else {
        // 站立状态
        v12 = 2;
    }
}
```

### 3. 特殊地形状态（空中时）

#### 3.1 梯子检测
```cpp
if (IsOnLadder(pvc)) {
    // 排除特殊道具
    if (m_bForcedInvisible != 1902040 && 
        m_bForcedInvisible != 1902041 && 
        m_bForcedInvisible != 1902042) {
        v12 = 7;  // 梯子攀爬状态
    } else {
        v12 = 2 - (nInputY != 0);  // 特殊处理
    }
}
```

#### 3.2 绳子检测
```cpp
else if (IsOnRope(pvc)) {
    if (!IsRidingEvanDragon()) {
        v12 = 8;  // 绳子攀爬状态
    } else {
        v12 = 2 - (nInputY != 0);  // 骑龙时特殊处理
    }
}
```

### 4. 空中状态判断

#### 4.1 特殊状态检查
```cpp
// 火箭推进器
if (m_bRocketBoosterStart) {
    v12 = 20;
}

// 游泳状态
if (IsSwimming(pvc)) {
    v12 = 6;
}
```

#### 4.2 飞行状态
```cpp
if (CanFly() && !pvc->m_pfh && GetOneTimeAction() <= -1) {
    if (m_bForcedInvisible / 10000 == 193) {
        // 特殊飞行道具
        if (IsStopped() || IsRidingWildHunterJaguar() || m_bForcedInvisible == 1932016) {
            v12 = 6;
        } else {
            v12 = 2;
        }
    } else {
        // 普通飞行
        v12 = IsStopped() ? 17 : 18;
    }
}
```

#### 4.3 默认空中状态
```cpp
// 如果以上都不满足
v12 = 3;  // 下落状态
```

### 5. 方向处理

方向总是在最后处理，独立于状态判断：

```cpp
// 默认保持当前方向
v11 = nCurMoveAction & 1;

// 根据输入更新方向
if (nInputX >= 0) {
    v11 = nInputX > 0 ? 0 : v11;  // 0 = 右
} else {
    v11 = 1;  // 1 = 左
}

// 最终返回值
return (2 * v12) | (v11 & 1);
```

## 优先级总结

1. **OneTimeAction**（最高）
   - 特殊动作码处理
   - 梯子/绳子跳跃（207）

2. **地面状态**
   - 冲刺（需要技能4321000）
   - 行走
   - 跳跃准备
   - 趴下
   - 站立

3. **特殊地形**
   - 梯子
   - 绳子

4. **空中特殊状态**
   - 火箭推进器
   - 游泳
   - 飞行

5. **默认空中**（最低）
   - 下落

## 状态冲突解决

当多个条件同时满足时，按照上述优先级顺序，高优先级的状态会覆盖低优先级的状态。例如：

- 即使在地面上，如果有 OneTimeAction 207 且在梯子/绳子上，会执行跳跃
- 在梯子上但有特殊道具（1902040-1902042）时，会使用普通移动状态
- 飞行状态下的特殊道具（193xxxx）有独特的处理逻辑

## 实现建议

在实现状态机时，建议：

1. 使用分层的状态检查，按优先级顺序处理
2. 将 OneTimeAction 作为独立的系统，可以覆盖常规状态
3. 特殊道具和技能应该有独立的标志位
4. 方向和状态应该分开处理，最后合并

```rust
pub fn resolve_move_action(
    input_x: i32,
    input_y: i32,
    current_action: i32,
    vec_ctrl: &CVecCtrl,
    player: &Player,
) -> i32 {
    let mut state = 0;
    let mut direction = current_action & 1;
    
    // 1. OneTimeAction 优先级最高
    if let Some(one_time) = player.get_one_time_action() {
        // 处理特殊动作
    }
    
    // 2. 地面状态
    if vec_ctrl.is_on_ground() {
        // 地面状态逻辑
    }
    // 3. 特殊地形
    else if vec_ctrl.is_on_ladder() {
        // 梯子逻辑
    }
    // ... 其他状态
    
    // 更新方向
    if input_x != 0 {
        direction = if input_x < 0 { 1 } else { 0 };
    }
    
    (state << 1) | direction
}
```