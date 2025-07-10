# MapleStory 状态持续时间系统分析

本文档详细分析 MapleStory v95 中各种移动状态的持续时间机制和冷却系统。

## 系统概述

MapleStory 的状态持续时间系统主要通过以下机制实现：

1. **计数器系统**：使用递减计数器跟踪状态持续时间
2. **特殊参数**：某些状态有专门的持续时间参数
3. **物理更新**：在物理更新循环中处理状态转换
4. **二级属性**：通过 SecondaryStat 系统管理 buff 类状态

## 冲刺状态持续时间

### 实现机制

冲刺（Dash）是最典型的有持续时间限制的移动状态。

**相关代码分析**：

```cpp
// CUser::OnResolveMoveAction (地址: 0x8e5800)
// 初始化冲刺参数
if (GetDashingSkill() == 4321000) {
    v12 = 19;  // 设置为冲刺状态
    CVecCtrl *pVecCtrl = GetVecCtrl();
    pVecCtrl->m_dShortDrag = 0.4;    // 冲刺时的摩擦系数
    pVecCtrl->m_nSlideCount = 400;    // 持续时间（毫秒）
}
```

**持续时间更新**：

```cpp
// CVecCtrl::CalcWalk (地址: 0x992ba0)
if (m_nSlideCount > 0) {
    // 使用特殊的摩擦力
    if (m_dShortDrag < friction) {
        friction = m_dShortDrag;  // 0.4
    }
    // 减少剩余时间
    m_nSlideCount -= tElapse;
}
// 当 m_nSlideCount <= 0 时，冲刺效果自动结束
```

### 参数说明

| 参数 | 值 | 说明 |
|------|---|------|
| m_nSlideCount | 400 | 冲刺持续时间（毫秒）|
| m_dShortDrag | 0.4 | 冲刺时的摩擦系数 |
| 技能ID | 4321000 | 冲刺技能ID |
| OneTimeAction | 81 | 停止冲刺动作 |

### 停止机制

1. **自然结束**：m_nSlideCount 递减到 0
2. **主动停止**：设置 OneTimeAction 81
3. **状态转换**：进入其他状态（如跳跃）

## 击飞状态持续机制

### SetImpactNext 分析

**地址**: `0x749070`

```cpp
void CVecCtrl::SetImpactNext(double vx, double vy) {
    m_bWingsNow = 0;  // 取消翅膀状态
    
    if (!m_impactNext.bValid) {
        m_impactNext.vx = 0.0;
        m_impactNext.vy = 0.0;
    }
    
    m_impactNext.bValid = 1;
    
    // 累积算法（防止速度爆炸）
    // 水平方向
    if (vx < 0.0 && vx < m_impactNext.vx) {
        double accumulated = m_impactNext.vx + vx;
        m_impactNext.vx = (accumulated < vx) ? accumulated : vx;
    } else if (vx > 0.0 && vx > m_impactNext.vx) {
        double accumulated = m_impactNext.vx + vx;
        m_impactNext.vx = (accumulated > vx) ? accumulated : vx;
    }
    
    // 垂直方向同理
}
```

### 应用时机

在 `CVecCtrl::WorkUpdateActive` 中：

```cpp
if (m_impactNext.bValid) {
    CVecCtrl::Impact(this);  // 应用击飞效果
}
```

击飞状态没有固定的持续时间，而是通过物理模拟自然结束：
- 水平速度因摩擦力逐渐减小
- 垂直速度因重力影响
- 落地时结束击飞状态

## 其他状态的持续时间

### 跳跃状态

跳跃没有固定持续时间，由物理计算决定：

```cpp
// 跳跃初速度
vy = -(dJumpSpeed * walkJump / gravity);

// 在空中时，每帧应用重力
vy += gravity * tSec;

// 落地时自动结束跳跃状态
```

### 梯子/绳子攀爬

攀爬状态没有时间限制，由以下条件结束：
- 玩家主动离开（跳跃或移动）
- 到达梯子/绳子末端
- 进入其他状态

### 游泳状态

游泳状态由区域决定，没有时间限制：

```cpp
BOOL CVecCtrl::IsSwimming() {
    // 检查地图类型
    if (m_pAttrField->nMapType == 1) return TRUE;
    
    // 检查是否在游泳区域内
    return IsInArea(&m_pAttrField->icSwimArea, x, y);
}
```

## 二级属性系统（SecondaryStat）

某些状态通过二级属性系统管理，有独立的持续时间：

### 相关状态

| 状态 | 地址 | 说明 |
|------|------|------|
| CTS_Stun | 0xc6c980 | 眩晕 |
| CTS_Slow | - | 减速 |
| CTS_Seal | - | 封印 |
| CTS_Dash_Speed | 0xc6ca00 | 冲刺速度 |
| CTS_Dash_Jump | 0xc6ca10 | 冲刺跳跃 |

### 获取函数

```cpp
// 状态检查函数
_ZtlSecureGet_nSpeed (0x721d80)    // 移动速度
_ZtlSecureGet_nJump (0x721dc0)     // 跳跃能力
_ZtlSecureGet_rStun_ (0x721fc0)    // 眩晕状态
_ZtlSecureGet_rSeal_ (0x722020)    // 封印状态
_ZtlSecureGet_rSlow_ (0x7221c0)    // 减速状态
```

## 状态更新频率

### 物理更新循环

在 `CVecCtrl::WorkUpdateActive` 中，状态更新频率由游戏帧率决定：

```cpp
// 目标帧率：60 FPS
// 每帧时间：16.67ms

// 时间步长转换
tSec = (double)tElapse / 1000.0;

// 状态计数器更新
m_nSlideCount -= tElapse;  // tElapse 通常为 16-17ms
```

### 特殊更新间隔

某些检查有特定的更新间隔：

| 更新类型 | 间隔 | 用途 |
|----------|------|------|
| 游戏时间提示 | 3600000ms (1小时) | 防沉迷提示 |
| 道具过期检查 | 60000ms (1分钟) | 检查过期道具 |
| 鼠标隐藏 | 15000ms (15秒) | 自动隐藏鼠标 |
| 头像喊话 | 10000ms (10秒) | 更新聊天气泡 |
| 特殊任务 | 每400帧 | 执行周期性任务 |

## 实现建议

### 1. 状态持续时间结构

```rust
pub struct TimedState {
    state_type: StateType,
    duration: u32,          // 总持续时间（毫秒）
    remaining: u32,         // 剩余时间（毫秒）
    on_expire: Option<Box<dyn Fn(&mut Character)>>,
}

impl TimedState {
    pub fn update(&mut self, delta_ms: u32) -> bool {
        if self.remaining > delta_ms {
            self.remaining -= delta_ms;
            false  // 状态继续
        } else {
            self.remaining = 0;
            true   // 状态结束
        }
    }
}
```

### 2. 冲刺状态实现

```rust
pub struct DashState {
    slide_count: i32,      // 剩余滑行时间
    short_drag: f64,       // 特殊摩擦系数
    stop_effect_played: bool,
}

impl DashState {
    pub fn new() -> Self {
        Self {
            slide_count: 400,
            short_drag: 0.4,
            stop_effect_played: false,
        }
    }
    
    pub fn update(&mut self, physics: &mut Physics, delta_ms: i32) {
        if self.slide_count > 0 {
            // 使用特殊摩擦力
            physics.set_friction_override(self.short_drag);
            self.slide_count -= delta_ms;
            
            // 即将结束时播放特效
            if self.slide_count <= 0 && !self.stop_effect_played {
                play_effect("Skill/432.img/skill/4321000/stopEffect");
                self.stop_effect_played = true;
            }
        }
    }
}
```

### 3. 状态管理器

```rust
pub struct StateManager {
    current_state: MovementState,
    timed_states: Vec<TimedState>,
    secondary_stats: HashMap<StatType, TimedBuff>,
}

impl StateManager {
    pub fn update(&mut self, delta_ms: u32) {
        // 更新持续时间状态
        self.timed_states.retain_mut(|state| {
            if state.update(delta_ms) {
                // 状态结束，执行回调
                if let Some(on_expire) = &state.on_expire {
                    on_expire(&mut self.character);
                }
                false  // 移除
            } else {
                true   // 保留
            }
        });
        
        // 更新二级属性
        for (_, buff) in self.secondary_stats.iter_mut() {
            buff.update(delta_ms);
        }
    }
}
```

### 4. 物理集成

```rust
impl CVecCtrl {
    pub fn calc_walk(&mut self, t_elapse: i32) {
        // 检查滑行状态
        if self.slide_count > 0 {
            let friction = if self.short_drag < normal_friction {
                self.short_drag
            } else {
                normal_friction
            };
            
            // 应用特殊物理
            self.apply_friction(friction);
            self.slide_count -= t_elapse;
        } else {
            // 正常物理计算
            self.apply_normal_physics();
        }
    }
}
```

## 调试和优化

### 监控点

1. **状态计数器**：监控 m_nSlideCount 等计数器的变化
2. **状态转换**：在状态改变时记录日志
3. **物理参数**：跟踪摩擦力、速度等参数的变化

### 性能优化

1. **批量更新**：将相似的状态更新合并处理
2. **事件驱动**：使用事件系统处理状态结束
3. **对象池**：复用状态对象，减少内存分配

### 网络同步

状态持续时间需要考虑网络同步：

```rust
pub struct NetworkedState {
    state: TimedState,
    start_time: u64,        // 服务器时间戳
    predicted_end: u64,     // 预测结束时间
}

impl NetworkedState {
    pub fn validate(&self, server_time: u64) -> bool {
        // 验证客户端预测是否准确
        (self.predicted_end as i64 - server_time as i64).abs() < 100
    }
}
```

## 总结

MapleStory 的状态持续时间系统设计灵活，不同类型的状态使用不同的机制：

1. **计数器机制**：适用于有固定持续时间的状态（如冲刺）
2. **物理模拟**：适用于自然结束的状态（如跳跃、击飞）
3. **区域判定**：适用于环境相关的状态（如游泳）
4. **二级属性**：适用于 buff 类状态（如眩晕、加速）

这种多样化的设计使得游戏能够精确控制各种状态的行为，同时保持良好的可扩展性。