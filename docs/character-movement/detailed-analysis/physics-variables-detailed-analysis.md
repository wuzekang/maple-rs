# MapleStory 物理计算变量深度分析

## 1. 概述

基于对现有逆向工程分析文档的深入研究，本文详细分析了MapleStory角色移动系统中关键物理计算变量的实现细节、计算方法和使用场景。

## 2. 地形交互变量详细分析

### 2.1 踏脚点单位向量 (m_uvx, m_uvy)

#### 2.1.1 基本定义
```cpp
class CStaticFoothold : public ZRefCounted {
    int m_x1, m_y1, m_x2, m_y2;     // 踏脚点线段的起点和终点坐标
    double m_len;                    // 踏脚点线段长度
    double m_uvx, m_uvy;             // 单位方向向量
    ZRef<CAttrFoothold> m_pAttrFoothold;  // 地形属性引用
};
```

#### 2.1.2 计算方法
**位置**: CStaticFoothold构造函数中计算
**公式**:
```cpp
// 1. 计算踏脚点向量
double dx = m_x2 - m_x1;
double dy = m_y2 - m_y1;

// 2. 计算长度
m_len = sqrt(dx * dx + dy * dy);

// 3. 计算单位向量
m_uvx = dx / m_len;    // 水平分量
m_uvy = dy / m_len;    // 垂直分量
```

#### 2.1.3 物理意义
- **m_uvx**: 踏脚点沿X轴的单位分量，表示踏脚点的水平倾斜程度
- **m_uvy**: 踏脚点沿Y轴的单位分量，表示踏脚点的垂直倾斜程度
- **关系**: m_uvx² + m_uvy² = 1 (单位向量性质)

#### 2.1.4 使用场景
1. **碰撞响应**: 速度投影到踏脚点方向
2. **斜坡计算**: 确定上坡/下坡方向
3. **摩擦力计算**: 确定法向力方向

### 2.2 斜率值 (sin1)

#### 2.2.1 计算方法
**函数**: CVecCtrl::CalcWalk
**公式**:
```cpp
sin1 = fabs(foothold->m_uvy);  // 取踏脚点垂直分量的绝对值
```

#### 2.2.2 物理意义
- **定义**: 踏脚点与水平面的夹角的正弦值
- **范围**: [0, 1]
  - 0: 完全水平的踏脚点
  - 1: 完全垂直的踏脚点
- **用途**: 判断踏脚点的倾斜程度

#### 2.2.3 数据来源
- **直接来源**: 踏脚点的m_uvy值
- **间接来源**: 踏脚点的几何坐标 (x1,y1) 到 (x2,y2)
- **计算时机**: 每次物理更新时从当前踏脚点获取

### 2.3 斜坡方向 (hd)

#### 2.3.1 计算方法
```cpp
hd = (foothold->m_uvy >= 0.0) ? -1 : 1;
```

#### 2.3.2 判断逻辑
- **hd = -1**: 下坡方向 (m_uvy >= 0)
  - 从左到右是向下的斜坡
- **hd = 1**: 上坡方向 (m_uvy < 0)
  - 从左到右是向上的斜坡

#### 2.3.3 物理意义
- **重力分量**: 确定重力在斜坡上的分解方向
- **滑坡效应**: 决定滑坡力的方向
- **移动难度**: 影响上坡/下坡的移动速度

## 3. 速度和力的计算变量

### 3.1 斜率平方 (vMaxL)

#### 3.1.1 计算方法
```cpp
vMaxL = sin1 * sin1;  // 斜率的平方
```

#### 3.1.2 物理作用
**在斜坡影响因子计算中使用**:
```cpp
if (hd <= 0)  // 下坡
    factor = vMaxL + 1.0;
else          // 上坡
    factor = 1.0 - vMaxL;
```

#### 3.1.3 数学含义
- **下坡**: factor = sin²θ + 1 ∈ [1, 2]
  - 斜坡越陡，移动越容易
- **上坡**: factor = 1 - sin²θ = cos²θ ∈ [0, 1]
  - 斜坡越陡，移动越困难

### 3.2 斜坡影响因子 (factor)

#### 3.2.1 计算公式
基于斜坡方向和斜率的复合计算:
```cpp
if (hd <= 0) {  // 下坡
    factor = vMaxL + 1.0;
} else {        // 上坡
    factor = 1.0 - vMaxL;
}
```

#### 3.2.2 物理意义
- **下坡加速**: factor > 1，重力协助移动
- **上坡减速**: factor < 1，重力阻碍移动
- **平地**: factor = 1，重力不影响水平移动

#### 3.2.3 应用场景
```cpp
force = walkForce * factor * inputX;
```
直接影响角色在斜坡上的移动力量。

### 3.3 最终作用力 (finalForce)

#### 3.3.1 累积计算过程
**基础计算**:
```cpp
walkForce = walkAcc * (dWalkForce[0] * drag * terrain_walk);
force = walkForce * factor * inputX;
```

**特殊地形处理**:
```cpp
if (foothold->force == 0.0) {
    // 普通地形：只有输入同向才有力
    if (inputX * force > 0.0)
        finalForce = force;
} else if (inputX != 0) {
    // 传送带地形
    if (inputX * foothold->force <= 0.0)  // 反向
        finalForce = 0.2 / abs(force) * walkForce;
    else  // 同向
        finalForce = 2.0 * abs(force) * walkForce;
} else {
    // 无输入时传送带自动移动
    finalForce = foothold->force * walkForce;
}
```

#### 3.3.2 累积组件
1. **基础行走力**: walkAcc * dWalkForce[0]
2. **地形系数**: terrain_walk
3. **阻力系数**: drag
4. **斜坡影响**: factor
5. **输入方向**: inputX
6. **特殊地形**: foothold->force

## 4. 碰撞检测相关变量

### 4.1 叉积计算 (cross1, cross2)

#### 4.1.1 计算函数
```cpp
int get_cross_product(int xOrg, int yOrg, int x1, int y1, int x2, int y2) {
    return (y2 - yOrg) * (x1 - xOrg) - (y1 - yOrg) * (x2 - xOrg);
}
```

#### 4.1.2 输入参数来源
**用于线段相交检测**:
- **线段1**: 角色移动路径 (x1,y1) → (x2,y2)
- **线段2**: 踏脚点线段 (fh->x1,fh->y1) → (fh->x2,fh->y2)

**具体调用**:
```cpp
// 检测角色路径与踏脚点的交叉
int cross1 = get_cross_product(x1, y1, fh->x1, fh->y1, fh->x2, fh->y2);
int cross2 = get_cross_product(x2, y2, fh->x1, fh->y1, fh->x2, fh->y2);
```

#### 4.1.3 几何意义
- **cross1**: 移动起点相对于踏脚点的位置关系
- **cross2**: 移动终点相对于踏脚点的位置关系
- **符号变化**: cross1 * cross2 < 0 表示路径穿过踏脚点

### 4.2 碰撞时间比例 (tCollide)

#### 4.2.1 精确计算方法
```cpp
double calculate_collision_ratio(int cross1, int cross2) {
    if (cross1 == cross2) return -1.0; // 不会碰撞
    return (double)cross1 / (double)(cross1 - cross2);
}
```

#### 4.2.2 数学原理
**基于线性插值**:
- **tCollide = 0**: 碰撞发生在移动起点
- **tCollide = 1**: 碰撞发生在移动终点  
- **0 < tCollide < 1**: 碰撞发生在路径中间

**碰撞点计算**:
```cpp
collision_x = x1 + (x2 - x1) * tCollide;
collision_y = y1 + (y2 - y1) * tCollide;
```

### 4.3 最早碰撞时间 (tFirstCollide)

#### 4.3.1 选择算法
```cpp
double tFirstCollide = 1.0;  // 初始化为路径终点
CStaticFoothold* pfhFirstCollide = nullptr;

for (auto& foothold : candidates) {
    double tCollide = calculate_collision_time(foothold);
    if (tCollide >= 0 && tCollide < tFirstCollide) {
        tFirstCollide = tCollide;
        pfhFirstCollide = foothold;
    }
}
```

#### 4.3.2 优先级规则
1. **时间优先**: 选择最早发生的碰撞
2. **有效性检查**: 只考虑有效的碰撞时间 (tCollide >= 0)
3. **边界处理**: 确保碰撞时间在 [0, 1] 范围内

## 5. 边界处理变量

### 5.1 地图边界矩形 (m_rcMBR)

#### 5.1.1 数据结构
```cpp
struct MapBoundingRect {
    int left;    // 左边界
    int right;   // 右边界
    int top;     // 上边界
    int bottom;  // 下边界
};
```

#### 5.1.2 更新机制
**在CVecCtrl::CollisionDetectFloat中**:
```cpp
// 强制边界限制
if (x < m_rcMBR.left) {
    x = m_rcMBR.left;
    vx = 0.0;  // 撞墙后水平速度归零
}
if (x > m_rcMBR.right) {
    x = m_rcMBR.right;
    vx = 0.0;
}
// 类似处理top和bottom
```

#### 5.1.3 使用场景
- **边界检查**: 防止角色移出地图
- **速度归零**: 撞墙时停止移动
- **位置修正**: 强制拉回到有效范围内

### 5.2 碰撞后速度投影

#### 5.2.1 具体实现
```cpp
// 速度投影到踏脚点方向
if (pfhFirstCollide) {
    // 计算速度在踏脚点方向上的投影
    double v_parallel = vx * pfhFirstCollide->m_uvx + vy * pfhFirstCollide->m_uvy;
    
    // 更新速度到沿踏脚点方向
    vx_new = v_parallel * pfhFirstCollide->m_uvx;
    vy_new = v_parallel * pfhFirstCollide->m_uvy;
    
    // 特殊情况处理
    if (pfhFirstCollide->m_uvx > 0.5) {
        // 接近水平的踏脚点，速度衰减50%
        v_parallel *= 0.5;
    }
}
```

#### 5.2.2 数学原理
**向量投影公式**:
```
v_parallel = v · û = (vx, vy) · (uvx, uvy) = vx*uvx + vy*uvy
v_projected = v_parallel * û = v_parallel * (uvx, uvy)
```

其中 û = (uvx, uvy) 是踏脚点的单位方向向量。

## 6. 变量间的相互关系

### 6.1 数据流向图
```
踏脚点坐标 (x1,y1,x2,y2)
    ↓
单位向量 (m_uvx, m_uvy)
    ↓
斜率值 (sin1 = |m_uvy|)
    ↓
斜坡方向 (hd) + 斜率平方 (vMaxL)
    ↓
影响因子 (factor)
    ↓
最终作用力 (finalForce)
```

### 6.2 碰撞检测流程
```
角色移动路径 (x1,y1) → (x2,y2)
    ↓
获取候选踏脚点 (GetCrossCandidate)
    ↓
叉积计算 (cross1, cross2)
    ↓
碰撞时间计算 (tCollide)
    ↓
最早碰撞选择 (tFirstCollide)
    ↓
速度投影修正
```

## 7. 实现建议

### 7.1 数据结构设计
```rust
#[derive(Debug, Clone)]
pub struct Foothold {
    pub id: u32,
    pub start: Point2<f32>,
    pub end: Point2<f32>,
    pub length: f32,
    pub unit_vector: Vector2<f32>,  // (uvx, uvy)
    pub attributes: FootholdAttributes,
}

impl Foothold {
    pub fn new(start: Point2<f32>, end: Point2<f32>) -> Self {
        let delta = end - start;
        let length = delta.magnitude();
        let unit_vector = delta / length;
        
        Self {
            id: 0,
            start,
            end,
            length,
            unit_vector,
            attributes: FootholdAttributes::default(),
        }
    }
    
    pub fn get_slope_sine(&self) -> f32 {
        self.unit_vector.y.abs()
    }
    
    pub fn get_slope_direction(&self) -> i32 {
        if self.unit_vector.y >= 0.0 { -1 } else { 1 }
    }
    
    pub fn calculate_slope_factor(&self) -> f32 {
        let sin1 = self.get_slope_sine();
        let vmax_l = sin1 * sin1;
        let hd = self.get_slope_direction();
        
        if hd <= 0 {
            vmax_l + 1.0  // 下坡
        } else {
            1.0 - vmax_l  // 上坡
        }
    }
}
```

### 7.2 碰撞检测实现
```rust
pub struct CollisionDetector {
    quad_tree: QuadTree<Foothold>,
    map_bounds: Rectangle,
}

impl CollisionDetector {
    pub fn detect_collision(&self, start: Point2<f32>, end: Point2<f32>) -> Option<CollisionResult> {
        let candidates = self.get_collision_candidates(start, end);
        let mut earliest_time = 1.0;
        let mut earliest_foothold = None;
        
        for foothold in candidates {
            if let Some(time) = self.calculate_collision_time(start, end, &foothold) {
                if time >= 0.0 && time < earliest_time {
                    earliest_time = time;
                    earliest_foothold = Some(foothold);
                }
            }
        }
        
        earliest_foothold.map(|fh| CollisionResult {
            time: earliest_time,
            point: start + (end - start) * earliest_time,
            foothold: fh,
        })
    }
    
    fn calculate_collision_time(&self, start: Point2<f32>, end: Point2<f32>, foothold: &Foothold) -> Option<f32> {
        let cross1 = self.cross_product(start, foothold.start, foothold.end);
        let cross2 = self.cross_product(end, foothold.start, foothold.end);
        
        if cross1 == cross2 {
            return None; // 不会碰撞
        }
        
        let time = cross1 as f32 / (cross1 - cross2) as f32;
        Some(time)
    }
    
    fn cross_product(&self, point: Point2<f32>, line_start: Point2<f32>, line_end: Point2<f32>) -> i32 {
        let dx1 = line_start.x - point.x;
        let dy1 = line_start.y - point.y;
        let dx2 = line_end.x - point.x;
        let dy2 = line_end.y - point.y;
        
        (dy2 * dx1 - dy1 * dx2) as i32
    }
}
```

### 7.3 物理计算实现
```rust
pub struct PhysicsCalculator {
    constants: PhysicsConstants,
}

impl PhysicsCalculator {
    pub fn calculate_walk_force(&self, foothold: &Foothold, input_x: f32, character_attrs: &CharacterAttributes) -> f32 {
        let base_force = character_attrs.walk_acceleration * self.constants.walk_force[0];
        let slope_factor = foothold.calculate_slope_factor();
        let terrain_factor = foothold.attributes.walk_coefficient;
        
        let force = base_force * slope_factor * terrain_factor * input_x;
        
        // 处理特殊地形
        if foothold.attributes.force_coefficient != 0.0 {
            self.handle_special_terrain(force, foothold, input_x)
        } else {
            force
        }
    }
    
    pub fn project_velocity_to_foothold(&self, velocity: Vector2<f32>, foothold: &Foothold) -> Vector2<f32> {
        let parallel_component = velocity.dot(&foothold.unit_vector);
        let projected_velocity = foothold.unit_vector * parallel_component;
        
        // 特殊情况处理
        if foothold.unit_vector.x.abs() > 0.5 {
            projected_velocity * 0.5  // 接近水平时速度衰减
        } else {
            projected_velocity
        }
    }
}
```

## 8. 总结

本分析详细解剖了MapleStory物理计算系统中的关键变量，揭示了：

1. **踏脚点单位向量**: 作为所有斜坡计算的基础，从几何坐标精确计算得出
2. **斜率相关变量**: sin1、vMaxL、hd、factor 形成完整的斜坡物理计算链
3. **碰撞检测变量**: cross1/cross2、tCollide、tFirstCollide 实现精确的碰撞时间计算
4. **边界处理**: m_rcMBR 确保角色不会离开地图边界
5. **速度投影**: 基于向量数学的精确碰撞响应

这些变量的相互作用构成了一个精密的物理系统，为实现类似的2D平台游戏物理引擎提供了详细的技术参考。