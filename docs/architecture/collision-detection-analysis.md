# MapleStory 碰撞检测系统分析

通过 IDA 逆向分析，我发现了 MapleStory 的碰撞检测系统实现。

## 1. 核心数据结构

### 1.1 TRSTree - 空间分割树
```cpp
// TRSTree 是一个四叉树/八叉树结构，用于空间分割和快速查询
template<typename T, typename V, int D1, int D2, int D3>
class TRSTree {
    struct I2 {  // 2D 矩形区域
        long l;  // left
        long t;  // top  
        long r;  // right
        long b;  // bottom
    };
    
    struct NODE {
        struct ENTRY {
            I2 i;           // 区域边界
            NODE* pChild;   // 子节点指针
            void* pData;    // 数据指针
        };
        int t;  // 节点类型
    };
    
    NODE* m_pRoot;  // 根节点
};
```

### 1.2 CStaticFoothold - 静态踏脚点
```cpp
class CStaticFoothold : public ZRefCounted {
    // 踏脚点表示地形中的一条线段
    // 包含起点、终点、斜率等信息
};
```

### 1.3 CWvsPhysicalSpace2D - 物理空间管理器
```cpp
class CWvsPhysicalSpace2D {
    TRSTree<long, ZRef<CStaticFoothold>, 2, 4, 2> m_rtFoothold;  // 踏脚点空间树
    ZMap<unsigned long, ZRef<CStaticFoothold>, unsigned long> m_mFoothold;  // 踏脚点映射表
};
```

## 2. 碰撞检测算法

### 2.1 空间查询 - raw_Search
在 `0x5157a0` 地址找到了 TRSTree 的搜索实现：

```cpp
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
                // 添加到结果集（避免重复）
                if (!results->Find(entry->pData)) {
                    results->AddTail(entry->pData);
                }
            }
        }
    }
}
```

### 2.2 获取交叉候选项 - GetCrossCandidate
在 `0x516610` 地址找到了获取线段与踏脚点交叉候选项的函数：

```cpp
void CWvsPhysicalSpace2D::GetCrossCandidate(int x1, int y1, int x2, int y2, 
                                           ZList<ZRef<CStaticFoothold>>* candidates) {
    I2 searchRect;
    
    // 构建搜索矩形（确保 min/max 正确）
    searchRect.l = min(x1, x2);
    searchRect.r = max(x1, x2);
    searchRect.t = min(y1, y2);
    searchRect.b = max(y1, y2);
    
    // 在空间树中搜索
    m_rtFoothold.raw_Search(m_rtFoothold.m_pRoot, &searchRect, candidates);
}
```

### 2.3 获取踏脚点 - GetFoothold
在 `0x517d10` 地址找到了通过 ID 获取踏脚点的函数：

```cpp
CStaticFoothold* CWvsPhysicalSpace2D::GetFoothold(unsigned int dwSN) {
    if (dwSN == 0) return nullptr;
    
    ZRef<CStaticFoothold> foothold;
    if (m_mFoothold.GetAt(dwSN, foothold)) {
        return foothold.p;
    }
    
    return nullptr;
}
```

## 3. 碰撞检测流程

### 3.1 矩形区域碰撞（AABB）
系统使用简单的 AABB 算法进行初步碰撞检测：
```cpp
bool isColliding = (rect1.right >= rect2.left && 
                   rect1.bottom >= rect2.top && 
                   rect1.left <= rect2.right && 
                   rect1.top <= rect2.bottom);
```

### 3.2 核心碰撞检测函数

#### CVecCtrl::CollisionDetectFloat (0x994740)
主要碰撞检测函数，处理角色在空中的碰撞：
```cpp
void CVecCtrl::CollisionDetectFloat(CVecCtrl *this, double tSec) {
    // 1. 边界检查
    if (x < m_rcMBR.left) {
        x = m_rcMBR.left;
        vx = 0.0;
    }
    
    // 2. 获取碰撞候选踏脚点
    CWvsPhysicalSpace2D::GetCrossCandidate(x1, y1, x2, y2, &candidates);
    
    // 3. 对每个候选踏脚点进行碰撞测试
    for (auto& foothold : candidates) {
        // 使用叉积判断相交
        int cross = get_cross_product(x1, y1, fh->x1, fh->y1, fh->x2, fh->y2);
        if (cross != 0) {
            // 计算碰撞时间和位置
            tCollide = calculate_collision_time(...);
        }
    }
}
```

#### 叉积计算函数 (内联)
```cpp
int get_cross_product(int xOrg, int yOrg, int x1, int y1, int x2, int y2) {
    return (y2 - yOrg) * (x1 - xOrg) - (y1 - yOrg) * (x2 - xOrg);
}
```

### 3.3 斜面碰撞处理

#### 碰撞响应计算
```cpp
// 碰撞时的速度调整
if (pfh->m_uvx > 0.0) {
    // 水平踏脚点，速度减半
    v = v * 0.5;
} else {
    // 斜面踏脚点，保留沿斜面的速度分量
    v = vx * pfh->m_uvx + vy * pfh->m_uvy;
}

// 更新位置到碰撞点
this->m_rp._ZtlSecureTear_v = pfh->m_uvy * vy + pfh->m_uvx * vx;
```

### 3.4 特殊碰撞检测

#### is_blocked_area 函数
检测两个相邻踏脚点之间的阻挡区域：
```cpp
int is_blocked_area(CStaticFoothold *pfh1, CStaticFoothold *pfh2, int x, int y) {
    // 计算叉积判断点的位置关系
    int cross = (pfh1->m_x2 - pfh1->m_x1) * (pfh2->m_y2 - pfh1->m_y1) 
              - (pfh2->m_x2 - pfh1->m_x1) * (pfh1->m_y2 - pfh1->m_y1);
    
    // 根据叉积符号和其他条件判断是否被阻挡
    if (cross > 0) {
        // 额外的位置检查逻辑
        return check_point_position(...);
    }
    return 0;
}

## 4. 性能优化

### 4.1 空间分割
TRSTree 使用四叉树结构将空间分割成多个区域，减少需要检测的对象数量。

### 4.2 两阶段检测
1. 粗检测：使用 AABB 快速排除不可能碰撞的对象
2. 精检测：对可能碰撞的对象进行详细检测

### 4.3 缓存机制
使用 ZMap 缓存踏脚点，通过 ID 快速访问。

## 5. 实现建议

基于分析结果，在 Rust 中实现类似系统的建议：

### 5.1 数据结构
```rust
// 空间四叉树
pub struct QuadTree<T> {
    bounds: Rectangle,
    objects: Vec<T>,
    children: Option<Box<[QuadTree<T>; 4]>>,
    max_objects: usize,
    max_levels: usize,
    level: usize,
}

// 踏脚点
pub struct Foothold {
    id: u32,
    start: Point2<f32>,
    end: Point2<f32>,
    normal: Vector2<f32>,
}

// 物理空间
pub struct PhysicalSpace2D {
    quad_tree: QuadTree<Foothold>,
    footholds: HashMap<u32, Foothold>,
}
```

### 5.2 碰撞检测接口
```rust
impl PhysicalSpace2D {
    // 获取与矩形相交的踏脚点
    pub fn query_rect(&self, rect: &Rectangle) -> Vec<&Foothold> {
        self.quad_tree.query(rect)
    }
    
    // 获取与线段相交的踏脚点
    pub fn query_line(&self, start: Point2<f32>, end: Point2<f32>) -> Vec<&Foothold> {
        let bounds = Rectangle::from_points(start, end);
        let candidates = self.query_rect(&bounds);
        
        candidates.into_iter()
            .filter(|fh| line_intersects_line(start, end, fh.start, fh.end))
            .collect()
    }
    
    // 射线检测
    pub fn raycast(&self, origin: Point2<f32>, direction: Vector2<f32>, max_distance: f32) 
        -> Option<RaycastHit> {
        let end = origin + direction * max_distance;
        let hits = self.query_line(origin, end);
        
        // 找到最近的碰撞点
        hits.into_iter()
            .filter_map(|fh| {
                line_intersection(origin, end, fh.start, fh.end)
                    .map(|point| RaycastHit {
                        point,
                        normal: fh.normal,
                        foothold: fh,
                        distance: (point - origin).magnitude(),
                    })
            })
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }
}
```

### 5.3 边界情况处理
```rust
// 防止卡墙的检测
pub fn is_blocked_area(fh1: &Foothold, fh2: &Foothold, pos: Point2<f32>) -> bool {
    // 使用叉积判断点是否在阻挡区域内
    let cross = (fh1.end.x - fh1.start.x) * (fh2.end.y - fh1.start.y) 
              - (fh2.end.x - fh1.start.x) * (fh1.end.y - fh1.start.y);
    // 根据叉积符号和点的位置判断
    cross != 0 && /* 额外的位置判断逻辑 */
}

// 斜面碰撞响应
pub fn calculate_slope_response(velocity: Vector2<f32>, foothold: &Foothold) -> Vector2<f32> {
    if foothold.normal.x.abs() > 0.5 {
        // 接近水平的踏脚点，速度减半
        velocity * 0.5
    } else {
        // 斜面，计算沿斜面的速度分量
        let slope_dir = Vector2::new(foothold.normal.y, -foothold.normal.x);
        slope_dir * velocity.dot(&slope_dir)
    }
}
```

## 6. 边界情况实现细节

### 6.1 斜面碰撞处理
- **叉积算法**：使用叉积判断线段相交，提供精确的碰撞点
- **单位向量**：踏脚点存储单位方向向量(uvx, uvy)用于速度投影
- **垂直优化**：对接近垂直的踏脚点(|uvx| ≤ 0.5)有特殊处理

### 6.2 角色碰撞系统
- **点碰撞**：采用点碰撞而非碰撞盒，简化计算
- **类型特化**：不同实体类型(User/Mob/Pet)有独立实现
- **双精度**：使用双精度浮点数确保精度

### 6.3 防卡墙机制
- **阻挡检测**：`is_blocked_area`函数检测相邻踏脚点形成的阻挡区域
- **穿墙预防**：`cantThrough`属性阻止穿越特定踏脚点
- **边界限制**：强制限制在地图有效范围内

### 6.4 性能优化
- **R*树索引**：TRSTree提供O(log n)的空间查询
- **范围剪枝**：只检查移动路径矩形范围内的踏脚点
- **质量分层**：不同质量层级的踏脚点分开处理
- **早期退出**：起点终点相同时直接返回

## 7. 关键函数地址汇总

### 7.1 碰撞检测核心函数

| 函数名 | 地址 | 描述 |
|--------|------|------|
| CVecCtrl::CollisionDetectFloat | 0x994740 | 主碰撞检测函数 |
| CWvsPhysicalSpace2D::GetCrossCandidate | 0x516610 | 获取潜在碰撞踏脚点 |
| CWvsPhysicalSpace2D::GetFoothold | 0x517d10 | 通过ID获取踏脚点 |
| TRSTree::raw_Search | 0x5157a0 | 空间树查询 |
| is_blocked_area | 内联函数 | 阻挡区域检测 |

### 7.2 不同角色类型的碰撞检测

| 类/函数 | 地址范围 | 描述 |
|---------|----------|------|
| CVecCtrlUser::CollisionDetectFloat | 派生实现 | 玩家角色碰撞 |
| CVecCtrlMob::CollisionDetectFloat | 派生实现 | 怪物碰撞（含移动能力限制） |
| CVecCtrlNpc::CollisionDetectFloat | 派生实现 | NPC碰撞 |
| CVecCtrlPet::CollisionDetectFloat | 派生实现 | 宠物碰撞 |

### 7.3 辅助函数和常量

| 符号 | 地址/值 | 描述 |
|------|---------|------|
| get_cross_product | 内联 | 叉积计算 |
| DOUBLE_0_5 | 0.5 | 舍入常量 |
| DOUBLE_0_499999999 | 0.499999999 | 精度常量 |
| 边界偏移 | 30.0 | 飞行怪物反弹距离 |

## 8. 算法实现细节

### 8.1 线段相交检测算法
```cpp
// 使用叉积判断两条线段是否相交
bool segments_intersect(int x1, int y1, int x2, int y2,  // 线段1
                       int x3, int y3, int x4, int y4) { // 线段2
    int d1 = get_cross_product(x3, y3, x4, y4, x1, y1);
    int d2 = get_cross_product(x3, y3, x4, y4, x2, y2);
    int d3 = get_cross_product(x1, y1, x2, y2, x3, y3);
    int d4 = get_cross_product(x1, y1, x2, y2, x4, y4);
    
    // 线段相交的充要条件
    return ((d1 * d2) < 0) && ((d3 * d4) < 0);
}
```

### 8.2 碰撞时间计算
```cpp
// 计算精确的碰撞时间（0.0 到 1.0 之间的比例）
double calculate_collision_ratio(int cross1, int cross2) {
    if (cross1 == cross2) return -1.0; // 不会碰撞
    return (double)cross1 / (double)(cross1 - cross2);
}
```

### 8.3 质量层级碰撞规则
- 基础质量（0）：所有角色都能碰撞
- 特殊质量（非0）：只有相同质量的角色才能碰撞
- 飞行/龙骑状态：可以跨越质量层级

## 9. 关键发现总结

1. **TRSTree** - 核心的空间分割数据结构，使用模板参数 `<2,4,2>` 表示二维R*树
2. **两阶段检测** - AABB粗检测 + 叉积精确检测的高效组合
3. **双重存储** - 空间树用于范围查询，哈希表用于ID查询
4. **完善的边界处理**：
   - 防卡墙：is_blocked_area检测
   - 斜面滑动：速度投影算法
   - 边界反弹：特定怪物类型支持
5. **多重优化策略**：
   - 空间索引：O(log n)查询
   - 范围剪枝：只检测路径范围
   - 质量分层：减少不必要的碰撞检测
   - 早期退出：优化特殊情况

这个系统设计精巧，算法高效，特别是边界情况的处理非常完善，为现代2D游戏物理引擎提供了优秀的参考实现。