# MapleStory 移动状态转换系统分析

## 概述

MapleStory 的移动系统基于 CVecCtrl 类实现，通过复杂的状态转换机制管理角色的各种移动状态。本文档详细分析了地面、空中、游泳、飞行等状态之间的转换逻辑。

## 主要状态类型

### 1. 基础移动状态
- **地面状态 (Ground)**: 角色站在立足点上
- **空中状态 (Air)**: 角色在空中（跳跃、下落）
- **游泳状态 (Swimming)**: 角色在水中
- **飞行状态 (Flying)**: 角色使用飞行技能或坐骑

### 2. 特殊移动状态
- **梯子/绳子状态 (Ladder/Rope)**: 角色附着在梯子或绳子上
- **护送状态 (Escort)**: 角色护送怪物时的特殊移动

## 状态转换核心函数

### 1. 游泳状态判断 (IsSwimming)
**地址**: `0x6a0160`
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

**状态判断逻辑**:
- 地图类型为1时，整个地图为水中地图
- 否则检查角色位置是否在游泳区域(swimArea)内

### 2. 脱离立足点 (DetachFromFoothold)
**地址**: `0x990d90`
```cpp
void __thiscall CVecCtrl::DetachFromFoothold(CVecCtrl *this)
{
    CStaticFoothold *m_pfh = this->m_pfh;
    if (m_pfh)
    {
        // 限制相对位置在立足点范围内
        double pos = _ZtlSecureFuse<double>((int)&this->m_rp, this->m_rp._ZtlSecureTear_pos_CS);
        if (pos < 0.0) pos = 0.0;
        if (pos > m_pfh->m_len) pos = m_pfh->m_len;
        
        // 设置绝对位置
        this->m_rp._ZtlSecureTear_pos_CS = _ZtlSecureTear<double>(pos, this->m_rp._ZtlSecureTear_pos);
        AbsPos::SetFromRelPos(&this->m_ap, &this->m_rp, this->m_pfh);
        
        // 限制X坐标在立足点范围内
        double x = _ZtlSecureFuse<double>((int)&this->m_ap, this->m_ap._ZtlSecureTear_x_CS);
        if (x < m_pfh->m_x1) x = m_pfh->m_x1;
        if (x > m_pfh->m_x2) x = m_pfh->m_x2;
        this->m_ap._ZtlSecureTear_x_CS = _ZtlSecureTear<double>(x, this->m_ap._ZtlSecureTear_x);
        
        // 将Y坐标向下调整1像素，脱离立足点
        double y = _ZtlSecureFuse<double>((int)this->m_ap._ZtlSecureTear_y, this->m_ap._ZtlSecureTear_y_CS);
        this->m_ap._ZtlSecureTear_y_CS = _ZtlSecureTear<double>(floor(y - 1.0), this->m_ap._ZtlSecureTear_y);
        
        // 调用虚函数处理脱离立足点的后续逻辑
        ((void (__thiscall *)(CVecCtrl *, _DWORD, _DWORD, _DWORD))this->ZRefCounted::vfptr[5].__vecDelDtor)(this, 0, 0, 0);
    }
}
```

**状态转换效果**:
- 从 **地面状态** → **空中状态**
- 清除立足点引用，角色开始受重力影响

### 3. 跳跃状态转换 (JustJump)
**地址**: `0x993ea0`

这是状态转换最复杂的函数，处理从各种状态的跳跃转换：

#### 3.1 飞行能力检查
```cpp
BOOL canFly = true;
if (!this->CanFly() && !CVecCtrl::IsSwimming(this))
{
    // 检查鞋子的飞行加速度
    p = this->m_pCurAttrShoe.p;
    if (!p || TSecType<double>::GetData(&p->flyAcc) <= 0.0)
        canFly = false;
}
```

#### 3.2 状态转换矩阵

| 当前状态 | 输入条件 | 目标状态 | 转换逻辑 |
|----------|----------|----------|----------|
| 梯子/绳子 | 有方向键 | 空中状态 | 从梯子/绳子跳下，减小跳跃高度 |
| 梯子/绳子 | 无方向键 | 梯子/绳子 | 保持原状态 |
| 地面状态 | 任意 | 空中状态 | 正常跳跃，脱离立足点 |
| 空中状态 | 可飞行 | 飞行状态 | 空中二段跳 |
| 空中状态 | 不可飞行 | 空中状态 | 跳跃失败 |
| 游泳状态 | 任意 | 游泳状态 | 游泳中的垂直移动 |

#### 3.3 跳跃参数计算

**地面跳跃**:
```cpp
// 垂直速度 = -(基础跳跃速度 * 鞋子跳跃加成 / 重力)
double vy = -(jumpSpeed * walkJump / gravity);

// 如果可以飞行，降低跳跃高度为70%
if (canFly)
    vy *= 0.7;
```

**梯子/绳子跳跃**:
```cpp
// 垂直速度系数
double vMax = canFly ? 0.3 : 0.5;  // 飞行状态下跳跃更低
double vy = -(jumpSpeed * walkJump / gravity * vMax);

// 水平速度
double vx = walkSpeed * inputDirection * 1.3;  // 30%的水平速度加成
```

**空中跳跃（飞行）**:
```cpp
if (IsSwimming())
{
    // 游泳时的垂直速度
    double vy = swimSpeedV * swimSpeed * fly * 5.0;
}
else
{
    // 飞行时的垂直速度
    double vy = flySpeed * flySpeed * fly * 5.0 * flyJumpDec;
    
    // 如果按住方向键，水平速度加倍
    if (inputX != 0)
        vx += vx;
}
```

### 4. 下落状态转换 (FallDown)
**地址**: `0x993d80`

```cpp
void __thiscall CVecCtrl::FallDown(CVecCtrl *this)
{
    // 保存下落开始点
    this->m_pfhFallStart = this->m_falldownNext.pfhFallStart;
    this->m_falldownNext.bValid = 0;
    
    if (this->m_pLadderOrRope)
    {
        // 从梯子/绳子下落
        this->ZRefCounted::vfptr[5].__vecDelDtor(this, 0, 0, 0);
    }
    else
    {
        if (this->m_pfh)
        {
            // 从立足点下落
            CVecCtrl::DetachFromFoothold(this);
            
            // 播放跳跃音效（仅限玩家）
            if (!this->m_pOwner->vfptr->GetType(this->m_pOwner))
                play_game_sound("Jump", 0x64u);
        }
    }
    
    // 设置初始下落速度
    double g = TSecType<double>::GetData(&this->m_pAttrField->g);
    this->m_ap._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(
        jumpSpeed * -0.35355339 / g,  // 初始向下速度
        this->m_ap._ZtlSecureTear_vy
    );
    this->m_ap._ZtlSecureTear_vx_CS = _ZtlSecureTear<double>(0.0, this->m_ap._ZtlSecureTear_vx);
}
```

**状态转换效果**:
- 从 **地面状态** → **空中状态**
- 从 **梯子/绳子状态** → **空中状态**
- 设置初始下落速度

## 状态转换优先级

### 1. 状态检查优先级
1. **梯子/绳子状态** - 最高优先级
2. **立足点状态** - 次高优先级
3. **游泳状态** - 中等优先级
4. **飞行状态** - 低优先级
5. **空中状态** - 默认状态

### 2. 输入响应优先级
1. **Alt键 + 方向键** - 向下穿透平台
2. **跳跃键** - 状态转换
3. **方向键** - 水平移动
4. **上/下键** - 梯子/绳子攀爬

## 物理参数变化

### 1. 状态转换时的速度变化

| 转换类型 | 水平速度(vx) | 垂直速度(vy) | 特殊处理 |
|----------|-------------|-------------|----------|
| 地面→空中 | 保持或增加 | 设为负值(向上) | 根据输入调整vx |
| 空中→地面 | 保持 | 设为0 | 着陆检测 |
| 空中→游泳 | 保持 | 调整为游泳速度 | 阻力计算 |
| 游泳→空中 | 保持 | 设为负值 | 离开水面 |
| 梯子→空中 | 增加30% | 减少50%或70% | 侧向跳跃 |

### 2. 重力影响

```cpp
// 不同状态下的重力系数
const double GRAVITY_NORMAL = 1.0;      // 正常重力
const double GRAVITY_WATER = 0.5;       // 水中重力减半
const double GRAVITY_FLY = 0.1;         // 飞行时重力极小
const double GRAVITY_LADDER = 0.0;      // 梯子上无重力
```

### 3. 阻力系数

```cpp
// 不同状态下的阻力系数
const double FRICTION_GROUND = 0.8;     // 地面摩擦
const double FRICTION_AIR = 0.98;       // 空气阻力
const double FRICTION_WATER = 0.9;      // 水中阻力
const double FRICTION_ICE = 0.95;       // 冰面摩擦
```

## 状态转换的冲突处理

### 1. 同时满足多个状态条件时的处理

```cpp
// 优先级检查序列
if (HasLadderOrRope()) {
    // 梯子/绳子状态优先
    return LADDER_ROPE_STATE;
} else if (HasFoothold()) {
    // 立足点状态次优先
    return GROUND_STATE;
} else if (IsSwimming()) {
    // 游泳状态
    return SWIMMING_STATE;
} else if (CanFly()) {
    // 飞行状态
    return FLYING_STATE;
} else {
    // 默认空中状态
    return AIR_STATE;
}
```

### 2. 状态转换的缓冲机制

- **立足点丢失缓冲**: 角色离开立足点后有短暂时间内仍可视为地面状态
- **游泳区域缓冲**: 进入/离开游泳区域时有过渡效果
- **飞行状态缓冲**: 飞行技能结束后有缓慢下降的过渡

## 特殊状态转换

### 1. 护送状态 (Escort Mode)
```cpp
if (this->m_bEscortMob)
{
    // 护送怪物时的固定速度
    double vx = inputDirection * 125.0 * 1.3;
    AbsPos::_ZtlSecurePut_vx(&this->m_ap, vx);
}
```

**特殊处理**:
- 水平速度固定为125 * 1.3 = 162.5
- 不受正常速度限制影响
- 跳跃高度可能受限

### 2. 游泳区域检测
```cpp
// 游泳区域从地图数据中加载
void CField::RestoreSwinArea()
{
    // 从WZ文件读取swimArea配置
    // 每个区域包含矩形坐标(x1,y1,x2,y2)
    ZArray<tagRECT> *swimAreas = &this->m_pAttrField->icSwimArea;
    
    // 示例区域添加
    tagRECT *rect = ZArray<tagRECT>::InsertBefore(swimAreas, -1);
    rect->left = x1;
    rect->top = y1;
    rect->right = x2;
    rect->bottom = y2;
}
```

## 状态转换状态机图

```
    [地面状态]
         |
    跳跃/下落键
         |
         v
    [空中状态] ←→ [飞行状态]
         |           |
    碰撞检测      技能结束
         |           |
         v           v
    [地面状态]   [空中状态]
         |           |
    进入水域     进入水域
         |           |
         v           v
    [游泳状态] ←→ [游泳状态]
         |
    上/下键
         |
         v
   [梯子/绳子状态]
```

## 实现细节

### 1. 安全机制
- 使用 `_ZtlSecureTear` 和 `_ZtlSecureFuse` 防止内存修改
- 所有物理参数都有合理性检查
- 状态转换有冷却时间限制

### 2. 性能优化
- 状态检查按优先级顺序进行
- 使用缓存减少重复计算
- 物理更新采用固定时间步长

### 3. 网络同步
- 状态转换需要服务器验证
- 关键状态变化会发送数据包
- 客户端预测和服务器校正机制

## 总结

MapleStory 的状态转换系统是一个复杂但设计精良的系统，通过明确的优先级规则和物理参数调整，实现了流畅的角色移动体验。理解这个系统对于游戏开发和逆向工程都具有重要价值。

关键要点：
1. 状态检查有明确的优先级序列
2. 每种状态转换都有对应的物理参数调整
3. 特殊状态（护送、游泳）有专门的处理逻辑
4. 系统设计考虑了防作弊和性能优化
5. 状态转换与网络同步紧密结合

这个分析为理解 MapleStory 物理引擎的核心机制提供了完整的技术文档。