# MapleStory 时间和帧率系统完整分析

## 1. 时间系统架构

### 1.1 时间获取函数
MapleStory v95 使用以下时间函数：

- **GetTickCount()** (导入地址: `0xb140f8`) - Windows API，返回系统启动以来的毫秒数
- **timeGetTime()** (导入地址: `0xb14354`) - 多媒体定时器API，提供更高精度的时间测量
- **HS_GetTickCount()** (`0xa20833`) - 游戏内部对 GetTickCount 的包装函数
- **HS_timeGetTime()** (`0xa2083e`) - 游戏内部对 timeGetTime 的包装函数
- **get_update_time()** - 获取当前更新时间的内部函数

### 1.2 时间单位转换
从 CVecCtrl::WorkUpdateActive (地址: `0x994460`) 的反编译代码中发现：

```cpp
// 时间步长转换为秒
tSec = (double)tElapse / DOUBLE_1000_0;
```

这表明：
- **tElapse**: 毫秒为单位的时间步长
- **物理计算**: 使用秒为单位，所以需要除以 1000.0
- **DOUBLE_1000_0**: 常量值 1000.0

## 2. 游戏更新循环

### 2.1 主要更新函数
游戏使用典型的 Update 模式，主要更新函数包括：

| 函数名 | 地址 | 描述 |
|--------|------|------|
| CWvsContext::Update | - | 游戏世界上下文的主更新函数 |
| CUserLocal::Update | 0x9383e0 | 本地玩家的更新函数 |
| CVecCtrl::WorkUpdateActive | 0x994460 | 物理系统的更新函数 |
| CInputSystem::Update | - | 输入系统更新 |
| CAnimationDisplayer::Update | - | 动画显示更新 |

### 2.2 更新函数参数
所有 Update 函数都接受一个 `long` 类型参数，这个参数是：
- **deltaTime**: 自上次更新以来经过的时间（毫秒）
- 或 **currentTime**: 当前时间戳

### 2.3 更新频率控制
从代码中发现的更新间隔：

```cpp
// CWvsContext::Update 中的时间间隔
3600000ms (1小时) - 游玩时间提示间隔
60000ms (1分钟) - 道具过期检查间隔  
15000ms (15秒) - 鼠标隐藏超时
10000ms (10秒) - 头像喊话更新间隔

// 使用计数器控制的更新
nCounter_2 % 400 == 0 - 每400次更新执行一次的任务
```

## 3. 物理更新系统

### 3.1 物理更新流程
从 CVecCtrl::WorkUpdateActive 的实现可以看出：

```cpp
int __thiscall CVecCtrl::WorkUpdateActive(CVecCtrl *this, int tElapse) {
    // 转换时间单位
    tSec = (double)tElapse / 1000.0;
    
    if (this->m_pfh) {  // 在踏脚点上
        CVecCtrl::CalcWalk(this, tElapse);
        // 位置更新：position = position + velocity * tSec
        RelPos::_ZtlSecurePut_pos(&this->m_rp, velocity * tSec + pos_old);
    } else {  // 在空中
        CVecCtrl::CalcFloat(this, tElapse);
        // X坐标更新：x = x + vx * tSec
        AbsPos::_ZtlSecurePut_x(&this->m_ap, vx * tSec + x_old);
        // Y坐标更新：y = y + vy * tSec
        AbsPos::_ZtlSecurePut_y(&this->m_ap, vy * tSec + y_old);
    }
}
```

### 3.2 物理时间步长
- 使用毫秒级时间步长作为输入
- 内部转换为秒进行物理计算
- 支持可变时间步长（delta time）

## 4. 帧率推测

虽然没有直接找到 FPS 设置，但基于以下证据可以推测：

### 4.1 目标帧率
- **推测目标**: 60 FPS
- **理由**: 
  - MapleStory 传统上使用 60 FPS
  - 16.67ms 的帧时间对应 60 FPS
  - 常见的 2D 游戏标准

### 4.2 物理更新频率
- **可能的配置**:
  - 物理更新频率 = 渲染频率 (60 Hz)
  - 或使用固定时间步长的子步进

### 4.3 帧率独立性
- 使用 delta time 确保帧率独立的游戏逻辑
- 物理计算基于实际经过的时间
- 支持可变帧率而不影响游戏速度

## 5. 实现建议

### 5.1 时间系统实现
```rust
pub struct TimeSystem {
    last_update_time: f64,
    current_time: f64,
    delta_time: f64,
    fixed_timestep: f64,
    accumulator: f64,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            last_update_time: 0.0,
            current_time: 0.0,
            delta_time: 0.0,
            fixed_timestep: 1.0 / 60.0,  // 60 FPS
            accumulator: 0.0,
        }
    }
    
    pub fn update(&mut self, current_time_ms: f64) {
        self.current_time = current_time_ms / 1000.0;  // 转换为秒
        self.delta_time = self.current_time - self.last_update_time;
        self.last_update_time = self.current_time;
    }
}
```

### 5.2 固定时间步长物理
```rust
// 固定时间步长的物理更新
pub fn fixed_update(&mut self, time_system: &mut TimeSystem) {
    time_system.accumulator += time_system.delta_time;
    
    while time_system.accumulator >= time_system.fixed_timestep {
        // 物理更新
        self.physics_update(time_system.fixed_timestep);
        time_system.accumulator -= time_system.fixed_timestep;
    }
    
    // 插值渲染
    let alpha = time_system.accumulator / time_system.fixed_timestep;
    self.interpolate_render(alpha);
}
```

### 5.3 Update 模式实现
```rust
trait Updatable {
    fn update(&mut self, delta_time_ms: i64);
}

struct GameWorld {
    context: WvsContext,
    user: UserLocal,
    pools: GamePools,
}

impl GameWorld {
    fn update(&mut self, delta_time_ms: i64) {
        // 主更新循环
        self.context.update(delta_time_ms);
        self.user.update(delta_time_ms);
        self.pools.update(delta_time_ms);
    }
}
```

## 6. 物理常量系统

### 6.1 物理常量加载
从 CWvsPhysicalSpace2D 构造函数 (地址: `0xa17432`) 的分析中发现，物理常量从 Map/Physics.img 加载：

| 键名 | 描述 | 用途 |
|------|------|------|
| "wa" | walkForce/Speed/Drag | 行走相关参数 |
| "sl" | slipForce/Speed | 滑动相关参数 |
| "fl" | floatSpeed/Drag/JumpDec | 浮空相关参数 |
| "sw" | swimSpeed/SpeedDec | 游泳相关参数 |
| "gr" | gravityAcc | 重力加速度 |
| "fa" | fallSpeed | 下落速度 |
| "ju" | jumpSpeed | 跳跃速度 |
| "ma" | maxFriction | 最大摩擦力 |
| "mi" | minFriction | 最小摩擦力 |

### 6.2 CONSTANTS 结构体
```cpp
struct CONSTANTS {
    double dWalkForce[14];    // 14种不同地形的行走力量参数
    double dJumpSpeed;        // 跳跃速度
    double dMaxFriction;      // 最大摩擦力
    double dMinFriction;      // 最小摩擦力  
    double dSwimSpeedDec;     // 游泳速度衰减
    double dFlyJumpDec;       // 飞行跳跃衰减
};
```

## 7. 总结

MapleStory v95 的时间系统特点：

1. **高精度时间**: 使用 timeGetTime() 获取毫秒级时间
2. **可变时间步长**: 支持不同的帧率而不影响游戏速度
3. **单位转换**: 内部物理计算使用秒，外部接口使用毫秒
4. **更新模式**: 使用标准的 Update(deltaTime) 模式
5. **性能优化**: 不同系统有不同的更新频率

这个时间系统设计确保了游戏在不同性能的机器上都能正常运行，同时保持物理模拟的准确性。