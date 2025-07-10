# MapleStory 角色移动系统不清楚状态转换条件分析报告

## 1. ProneStab (趴下) 状态的触发条件

### 1.1 m_bDelayedLoad 变量的含义

根据 `CUser::OnResolveMoveAction` 函数的分析（地址：0x8e5800）：

```cpp
// 第127行
v12 = 2 * (this->m_bDelayedLoad > 0) + 2;
```

这表明：
- 当 `m_bDelayedLoad > 0` 时，`v12 = 4`（趴下状态）
- 当 `m_bDelayedLoad <= 0` 时，`v12 = 2`（站立状态）

**推测**：`m_bDelayedLoad` 可能与以下情况相关：
1. 角色受到攻击后的保护状态（被击倒后的无敌时间）
2. 使用特定技能后的延迟状态
3. 地图传送或复活后的加载状态

但具体的含义需要进一步分析设置这个变量的代码。

## 2. Jump 状态（状态码5）的详细转换逻辑

根据 `CVecCtrl::JustJump` 函数分析（地址：0x993ea0），跳跃状态的转换逻辑包括：

### 2.1 飞行能力检查
```cpp
BOOL canFly = true;
if (!this->CanFly() && !CVecCtrl::IsSwimming(this)) {
    p = this->m_pCurAttrShoe.p;
    if (!p || TSecType<double>::GetData(&p->flyAcc) <= 0.0)
        canFly = false;
}
```

### 2.2 不同情况下的跳跃处理

1. **梯子/绳子上的跳跃**：
   - 需要有水平输入才能跳下
   - 跳跃高度为正常的30%（可飞行）或50%（不可飞行）
   - 水平速度增加30%

2. **地面跳跃**：
   - 正常跳跃速度计算：`vy = -(jumpSpeed * walkJump / gravity)`
   - 如果可以飞行，跳跃高度降低为70%

3. **空中二段跳（飞行）**：
   - 需要飞行能力
   - 游泳时：`vy = swimSpeedV * swimSpeed * fly * 5.0`
   - 飞行时：`vy = flySpeed * flySpeed * fly * 5.0 * flyJumpDec`
   - 如果有水平输入，水平速度加倍

## 3. 游泳状态的进入条件

根据 `CVecCtrl::IsSwimming` 函数分析（地址：0x6a0160）：

```cpp
BOOL __thiscall CVecCtrl::IsSwimming(CVecCtrl *this)
{
    CAttrField *m_pAttrField = this->m_pAttrField;
    
    // 检查地图类型是否为水中地图
    if (m_pAttrField && TSecType<long>::GetData(&m_pAttrField->nMapType) == 1)
        return 1;
    
    // 检查角色是否在游泳区域内
    if (!this->m_pAttrField)
        return 0;
        
    double y = _ZtlSecureFuse<double>((int)this->m_ap._ZtlSecureTear_y, this->m_ap._ZtlSecureTear_y_CS);
    double x = _ZtlSecureFuse<double>((int)&this->m_ap, this->m_ap._ZtlSecureTear_x_CS);
    
    return Geometry::InclusionChecker::IsInArea(&this->m_pAttrField->icSwimArea, (int)x, (int)y);
}
```

**游泳状态进入条件**：
1. **地图类型判断**：当 `nMapType == 1` 时，整个地图都是游泳区域
2. **区域判断**：否则检查角色坐标是否在 `icSwimArea`（游泳区域）内

## 4. 飞行状态（FlyingStable vs FlyingMove）的切换条件

根据 `CUser::OnResolveMoveAction` 函数的第200行：

```cpp
v12 = 18 - (CVecCtrl::IsStopped(pvc) != 0);
```

**飞行状态切换逻辑**：
- 状态17（FlyingStable）：当 `IsStopped() == true` 时（静止飞行）
- 状态18（FlyingMove）：当 `IsStopped() == false` 时（移动飞行）

**特殊飞行道具处理**（道具ID前缀193）：
- 如果停止、骑乘野猎手美洲豹或使用道具1932016：使用状态6（游泳动画）
- 否则：使用状态2（站立动画）

## 5. RocketBooster 状态的进入和退出条件

根据代码分析，RocketBooster（火箭推进器）状态的判断：

```cpp
if (this->m_bRocketBoosterStart)
    v12 = 20;
```

**推测**：
- `m_bRocketBoosterStart` 是一个布尔变量，表示火箭推进器是否激活
- 可能通过特定技能或道具激活
- 退出条件可能包括：技能持续时间结束、主动取消、碰撞等

## 6. 扩展站立动作（48-54, 125）的触发条件

### 6.1 OneTimeAction 映射
根据 `CAvatar::GetOneTimeAction` 函数，发现了许多OneTimeAction的映射关系，但没有直接找到触发状态48-54和125的条件。

### 6.2 推测的触发条件
这些扩展站立动作可能与以下情况相关：
1. **表情动作**：玩家输入的表情命令
2. **任务动画**：特定任务触发的动画
3. **道具效果**：使用特定道具后的动画
4. **技能动画**：某些技能的待机动画

## 7. OneTimeAction 207 的具体作用

根据代码分析：
```cpp
if (CAvatar::GetOneTimeAction((CAvatar *)&this->m_pLayerNameTag[2]) == 207 && (v12 == 7 || v12 == 8))
{
    // 清除动作层
    CAvatar::ClearActionLayer((CAvatar *)&this->m_pLayerNameTag[2], 1);
    // 设置动作6（可能是跳跃动画）
    SetAction(6, 100, 0);
}
```

**OneTimeAction 207 的作用**：
- 仅在梯子（状态7）或绳子（状态8）上触发
- 清除当前动作层并设置新动作
- 可能是从梯子/绳子上跳跃的特殊动画

## 8. 未完全解析的内容

### 8.1 需要进一步研究的变量
1. `m_bDelayedLoad` 的设置时机和具体含义
2. `m_bRocketBoosterStart` 的激活和取消机制
3. `m_bEscortMob` 护送怪物状态的完整逻辑
4. `m_bForcedInvisible` 特殊道具ID的完整列表

### 8.2 需要查找的函数
1. 设置OneTimeAction的函数
2. 技能激活相关的函数
3. 道具使用相关的函数
4. 状态转换的网络同步函数

### 8.3 建议的后续分析方向
1. 查找引用这些变量的代码，了解它们的设置时机
2. 分析技能系统，找到触发特殊移动状态的技能
3. 分析道具系统，找到影响移动状态的道具
4. 查看服务器数据包处理，了解状态同步机制

## 9. 实现建议

在实现这些状态转换时，建议：

1. **创建状态标志结构体**：
```rust
pub struct MovementFlags {
    pub delayed_load: bool,           // m_bDelayedLoad
    pub rocket_booster_start: bool,   // m_bRocketBoosterStart
    pub escort_mob: bool,             // m_bEscortMob
    pub forced_invisible: u32,        // m_bForcedInvisible (道具ID)
    pub can_fly: bool,                // 飞行能力
    pub is_morphed: bool,             // 变身状态
}
```

2. **实现条件检查函数**：
```rust
impl MovementStateMachine {
    fn check_prone_condition(&self, flags: &MovementFlags) -> bool {
        flags.delayed_load && self.on_ground && !self.has_input
    }
    
    fn check_flying_condition(&self, flags: &MovementFlags) -> bool {
        flags.can_fly && !self.on_ground && self.one_time_action.is_none()
    }
    
    fn check_rocket_booster_condition(&self, flags: &MovementFlags) -> bool {
        flags.rocket_booster_start && !self.on_ground
    }
}
```

3. **处理特殊道具和技能**：
- 维护一个特殊道具ID到行为的映射表
- 实现技能激活对移动状态的影响
- 处理变身状态对移动系统的修改

这份分析提供了对MapleStory移动系统中不清楚状态转换条件的深入理解，虽然还有一些细节需要进一步研究，但已经能够为实现提供重要的指导。