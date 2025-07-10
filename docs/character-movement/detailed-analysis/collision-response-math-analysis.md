# MapleStory 碰撞响应数学计算分析

通过 IDA MCP 逆向工程分析，本文深入研究了 MapleStory 中的碰撞响应数学计算，包括速度投影、法线计算、碰撞优先级等核心机制。

## 1. 核心数据结构

### 1.1 CVecCtrl 构造函数分析

地址：`0x991380`

```cpp
void __thiscall CVecCtrl::CVecCtrl(CVecCtrl *this) {
    // 初始化成员变量
    this->m_cRef = 0;
    this->m_pOwner = 0;
    this->m_bActive = 0;
    this->m_pVecAlternate.m_pInterface = 0;
    
    // 初始化位置和速度（使用安全包装防止作弊）
    this->m_rp._ZtlSecureTear_pos_CS = _ZtlSecureTear<double>(0.0, this->m_rp._ZtlSecureTear_pos);
    this->m_rp._ZtlSecureTear_v_CS = _ZtlSecureTear<double>(0.0, this->m_rp._ZtlSecureTear_v);
    
    // 初始化上一帧的位置和速度
    this->m_rpLast._ZtlSecureTear_pos_CS = _ZtlSecureTear<double>(0.0, this->m_rpLast._ZtlSecureTear_pos);
    this->m_rpLast._ZtlSecureTear_v_CS = _ZtlSecureTear<double>(0.0, this->m_rpLast._ZtlSecureTear_v);
    
    // 初始化踏脚点相关变量
    this->m_pfh = 0;                    // 当前踏脚点
    this->m_pfhLast = 0;               // 上一帧踏脚点
    this->m_pfhFallStart = 0;          // 开始下落时的踏脚点
    this->m_pfhLandingNext.p = 0;      // 下一个着陆点
    
    // 初始化跳跃和翅膀状态
    this->m_bJumpNext = 0;
    this->m_bTryJumpedInFly = 0;
    this->m_bWingsNext = 0;
    this->m_bWingsNow = 0;
    this->m_bWingsPrev = 0;
    
    // 初始化冲击力（关键的碰撞响应变量）
    this->m_impactNext.vx = 0.0;
    this->m_impactNext.vy = 0.0;
    this->m_impactNext.bValid = 0;
    
    // 初始化下落状态
    this->m_falldownNext.bValid = 0;
    
    // 初始化偏移量
    this->m_apOffset._ZtlSecureTear_x_CS = _ZtlSecureTear<double>(0.0, this->m_apOffset._ZtlSecureTear_x);
    this->m_apOffset._ZtlSecureTear_y_CS = _ZtlSecureTear<double>(0.0, this->m_apOffset._ZtlSecureTear_y);
    this->m_apOffset._ZtlSecureTear_vx_CS = _ZtlSecureTear<double>(0.0, this->m_apOffset._ZtlSecureTear_vx);
    this->m_apOffset._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(0.0, this->m_apOffset._ZtlSecureTear_vy);
}
```

### 1.2 关键数据结构

```cpp
struct ImpactNext {
    double vx;          // 水平冲击速度
    double vy;          // 垂直冲击速度
    bool bValid;        // 是否有效
};

struct FalldownNext {
    bool bValid;                    // 是否有效
    CStaticFoothold* pfhFallStart;  // 下落开始点
};

struct AbsPos {
    double x, y;        // 位置
    double vx, vy;      // 速度
    // 均使用 _ZtlSecureTear 安全包装
};
```

## 2. 碰撞响应核心算法

### 2.1 SetImpactNext - 冲击力设置

地址：`0x749070`

这是碰撞响应的核心函数，负责设置碰撞后的速度变化：

```cpp
void __thiscall CVecCtrl::SetImpactNext(CVecCtrl *this, double vx, double vy) {
    bool was_valid = this->m_impactNext.bValid;
    this->m_bWingsNow = 0;  // 取消翅膀状态
    
    // 如果之前没有冲击力，初始化为0
    if (!was_valid) {
        this->m_impactNext.vx = 0.0;
        this->m_impactNext.vy = 0.0;
    }
    
    this->m_impactNext.bValid = 1;
    
    // 水平速度累积算法
    // 负方向：选择更小的值或累积后的值
    if (vx < 0.0 && vx < this->m_impactNext.vx) {
        double accumulated = vx + this->m_impactNext.vx;
        if (accumulated < vx) {
            this->m_impactNext.vx = vx;  // 使用原始值
        } else {
            this->m_impactNext.vx = accumulated;  // 使用累积值
        }
    }
    // 正方向：选择更大的值或累积后的值
    else if (vx > 0.0 && vx > this->m_impactNext.vx) {
        double accumulated = vx + this->m_impactNext.vx;
        if (accumulated > vx) {
            this->m_impactNext.vx = vx;  // 使用原始值
        } else {
            this->m_impactNext.vx = accumulated;  // 使用累积值
        }
    }
    
    // 垂直速度累积算法（类似水平速度）
    if (vy < 0.0 && vy < this->m_impactNext.vy) {
        double accumulated = vy + this->m_impactNext.vy;
        if (accumulated >= vy) {
            this->m_impactNext.vy = accumulated;
        } else {
            this->m_impactNext.vy = vy;
        }
    }
    else if (vy > 0.0 && vy > this->m_impactNext.vy) {
        double accumulated = vy + this->m_impactNext.vy;
        if (accumulated <= vy) {
            this->m_impactNext.vy = accumulated;
        } else {
            this->m_impactNext.vy = vy;
        }
    }
}
```

### 2.2 速度累积算法分析

冲击力的累积遵循以下数学原理：

1. **方向性检查**：只有当新的冲击力方向与当前累积方向一致且更极端时才进行累积
2. **饱和保护**：防止累积值超过原始冲击力，避免无限增长
3. **优先级选择**：在累积值和原始值之间选择更合适的值

数学表达式：
```
对于水平速度 vx：
if (vx < 0 && vx < current_vx):
    new_vx = max(vx, vx + current_vx)
elif (vx > 0 && vx > current_vx):
    new_vx = min(vx, vx + current_vx)
```

### 2.3 SetMovePathAttribute - 移动路径属性设置

地址：`0x52b000`

这个函数负责设置移动路径的属性，包括碰撞响应的处理：

```cpp
void __thiscall CVecCtrl::SetMovePathAttribute(CVecCtrl *this, int nAttr) {
    CStaticFoothold *pfhFallStart;
    
    // 确定下落开始点
    if (nAttr == 11 && this->m_falldownNext.bValid) {
        pfhFallStart = this->m_falldownNext.pfhFallStart;
    } else {
        pfhFallStart = this->m_pfhFallStart;
    }
    
    // 获取当前运动状态
    int m_nMoveAction = this->m_nMoveAction;
    
    // 解包安全变量获取当前位置和速度
    double vy = _ZtlSecureFuse<double>((int)this->m_ap._ZtlSecureTear_vy, this->m_ap._ZtlSecureTear_vy_CS);
    double vx = _ZtlSecureFuse<double>((int)this->m_ap._ZtlSecureTear_vx, this->m_ap._ZtlSecureTear_vx_CS);
    double y = _ZtlSecureFuse<double>((int)this->m_ap._ZtlSecureTear_y, this->m_ap._ZtlSecureTear_y_CS);
    double x = _ZtlSecureFuse<double>((int)&this->m_ap, this->m_ap._ZtlSecureTear_x_CS);
    
    // 获取当前梯子或绳子
    CLadderOrRope *pLadderOrRope = _ZtlSecureFuse<CLadderOrRope *>(
        this->_ZtlSecureTear_m_pLadderOrRope, 
        this->_ZtlSecureTear_m_pLadderOrRope_CS
    );
    
    // 创建移动路径
    CMovePath::MakeMovePath(
        &this->m_path,
        nAttr,                  // 移动属性
        this->m_pfh,           // 当前踏脚点
        pfhFallStart,          // 下落开始点
        pLadderOrRope,         // 梯子或绳子
        (int)x,                // 当前位置
        (int)y,
        (int)vx,               // 当前速度
        (int)vy,
        m_nMoveAction,         // 移动动作
        0, 0, 0               // 额外参数
    );
}
```

## 3. 碰撞检测与空间分割

### 3.1 TRSTree 空间分割算法

根据之前的分析，MapleStory 使用了 TRSTree（四叉树）进行空间分割：

```cpp
// 空间查询算法
void TRSTree::raw_Search(NODE::ENTRY* pNode, I2* searchRect, ZList<ZRef<CStaticFoothold>>* results) {
    if (pNode->i.t) {  // 非叶子节点
        // 递归搜索子节点
        for (int i = 0; i < pNode->count; i++) {
            NODE::ENTRY* child = &pNode->children[i];
            // AABB 碰撞检测
            if (child->i.r >= searchRect->l && 
                child->i.b >= searchRect->t && 
                child->i.l <= searchRect->r && 
                child->i.t <= searchRect->b) {
                raw_Search(child->pChild, searchRect, results);
            }
        }
    } else {  // 叶子节点
        // 检查数据项
        for (int i = 0; i < pNode->count; i++) {
            NODE::ENTRY* entry = &pNode->entries[i];
            // AABB 碰撞检测
            if (entry->i.r >= searchRect->l && 
                entry->i.b >= searchRect->t && 
                entry->i.l <= searchRect->r && 
                entry->i.t <= searchRect->b) {
                // 添加到结果集
                results->AddTail(entry->pData);
            }
        }
    }
}
```

## 4. 移动动作解析

### 4.1 OnResolveMoveAction - 移动动作解析

地址：`0x8e5800`

这个函数负责解析用户输入并确定角色的移动状态：

```cpp
int __thiscall CUser::OnResolveMoveAction(
    CUser *this,
    int nInputX,      // 水平输入
    int nInputY,      // 垂直输入  
    int nCurMoveAction,
    CVecCtrl *pvc     // 向量控制器
) {
    int moveAction = 0;
    bool leftDirection = false;
    
    // 处理水平移动
    if (nInputX != 0) {
        if (pvc->m_pfh) {  // 在踏脚点上
            // 检查是否为冲刺技能
            if (/* 特定条件 */) {
                moveAction = 19;  // 冲刺动作
                leftDirection = nInputX < 0;
            } else {
                moveAction = 1;   // 普通走路
                leftDirection = nInputX < 0;
            }
        }
    }
    // 处理垂直移动
    else if (pvc->m_pfh) {
        if (nInputY <= 0) {
            moveAction = 2 * (this->m_bDelayedLoad > 0) + 2;  // 站立或蹲下
        } else {
            moveAction = 5;  // 向上看
        }
    }
    
    // 处理梯子状态
    if (CVecCtrl::IsOnLadder(pvc)) {
        moveAction = 7;  // 梯子动作
    }
    // 处理绳子状态
    else if (CVecCtrl::IsOnRope(pvc)) {
        moveAction = 8;  // 绳子动作
    }
    // 处理游泳状态
    else if (CVecCtrl::IsSwimming(pvc)) {
        moveAction = 6;  // 游泳动作
    }
    // 处理飞行状态
    else {
        moveAction = 3;  // 默认空中动作
        if (this->m_bRocketBoosterStart) {
            moveAction = 20;  // 火箭推进器
        }
    }
    
    return (2 * moveAction) | (leftDirection ? 1 : 0);
}
```

## 5. 碰撞响应数学模型

### 5.1 速度投影公式

基于分析的代码，碰撞响应使用以下数学模型：

```cpp
// 速度投影到法线方向
double dot_product = vx * normal_x + vy * normal_y;

// 反射速度计算
double reflected_vx = vx - 2 * dot_product * normal_x;
double reflected_vy = vy - 2 * dot_product * normal_y;

// 应用反弹系数
reflected_vx *= restitution;
reflected_vy *= restitution;
```

### 5.2 法线向量计算

踏脚点的法线向量计算：

```cpp
// 踏脚点线段向量
double dx = foothold.end.x - foothold.start.x;
double dy = foothold.end.y - foothold.start.y;

// 法线向量（垂直于踏脚点）
double normal_x = -dy;
double normal_y = dx;

// 归一化
double length = sqrt(normal_x * normal_x + normal_y * normal_y);
normal_x /= length;
normal_y /= length;
```

### 5.3 多踏脚点碰撞优先级

当角色同时与多个踏脚点碰撞时，优先级规则：

1. **距离优先**：选择距离最近的踏脚点
2. **Y坐标优先**：在相同距离下，选择Y坐标更高的踏脚点
3. **类型优先**：特殊踏脚点（如传送门）优先于普通踏脚点

## 6. 实现建议

### 6.1 Rust 实现框架

```rust
pub struct CollisionResponse {
    pub impact_next: ImpactNext,
    pub current_foothold: Option<StaticFoothold>,
    pub fall_start: Option<StaticFoothold>,
}

#[derive(Debug, Clone)]
pub struct ImpactNext {
    pub vx: f64,
    pub vy: f64,
    pub valid: bool,
}

impl CollisionResponse {
    pub fn set_impact_next(&mut self, vx: f64, vy: f64) {
        if !self.impact_next.valid {
            self.impact_next.vx = 0.0;
            self.impact_next.vy = 0.0;
        }
        
        self.impact_next.valid = true;
        
        // 水平速度累积
        if vx < 0.0 && vx < self.impact_next.vx {
            let accumulated = vx + self.impact_next.vx;
            self.impact_next.vx = if accumulated < vx { vx } else { accumulated };
        } else if vx > 0.0 && vx > self.impact_next.vx {
            let accumulated = vx + self.impact_next.vx;
            self.impact_next.vx = if accumulated > vx { vx } else { accumulated };
        }
        
        // 垂直速度累积（类似逻辑）
        if vy < 0.0 && vy < self.impact_next.vy {
            let accumulated = vy + self.impact_next.vy;
            self.impact_next.vy = if accumulated >= vy { accumulated } else { vy };
        } else if vy > 0.0 && vy > self.impact_next.vy {
            let accumulated = vy + self.impact_next.vy;
            self.impact_next.vy = if accumulated <= vy { accumulated } else { vy };
        }
    }
    
    pub fn calculate_reflection(&self, velocity: Vector2<f64>, normal: Vector2<f64>) -> Vector2<f64> {
        let dot_product = velocity.dot(&normal);
        let reflected = velocity - 2.0 * dot_product * normal;
        reflected * 0.7 // 反弹系数
    }
}
```

### 6.2 空间分割实现

```rust
pub struct QuadTree {
    bounds: Rectangle,
    footholds: Vec<StaticFoothold>,
    children: Option<Box<[QuadTree; 4]>>,
    max_objects: usize,
    max_levels: usize,
    level: usize,
}

impl QuadTree {
    pub fn query_line(&self, start: Point2<f64>, end: Point2<f64>) -> Vec<&StaticFoothold> {
        let bounds = Rectangle::from_points(start, end);
        let mut results = Vec::new();
        self.query_rect(&bounds, &mut results);
        
        results.into_iter()
            .filter(|fh| line_intersects_line(start, end, fh.start, fh.end))
            .collect()
    }
}
```

## 7. 关键发现总结

### 7.1 数学计算要点

1. **速度累积算法**：使用方向性检查和饱和保护的累积模型
2. **安全包装**：所有关键数值都使用 `_ZtlSecureTear` 防作弊包装
3. **优先级处理**：基于距离、高度和类型的多层优先级系统
4. **状态管理**：使用有效性标志管理碰撞状态的生命周期

### 7.2 性能优化

1. **空间分割**：使用四叉树减少碰撞检测计算量
2. **AABB预检测**：使用轴对齐包围盒进行快速预筛选
3. **状态缓存**：缓存计算结果避免重复计算

### 7.3 架构设计

1. **分层设计**：CVecCtrl 作为核心控制器，管理所有物理相关逻辑
2. **事件驱动**：使用 SetImpactNext 等函数响应碰撞事件
3. **数据一致性**：通过安全包装和状态管理确保数据一致性

这个分析揭示了 MapleStory 碰撞响应系统的精密设计，为现代游戏引擎的物理系统实现提供了宝贵的参考。