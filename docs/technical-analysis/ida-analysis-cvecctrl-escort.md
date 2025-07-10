# CVecCtrlMob 护送模式 AI 逻辑分析

## 1. 护送模式概述

护送模式是怪物AI的一种特殊模式，怪物会按照预定义的路径点（EscortDest）移动。主要涉及两个核心函数：
- `CVecCtrlMob::CtrlUpdateActiveEscort` (0x99b8d0) - 更新护送移动逻辑
- `CVecCtrlMob::CollisionDetectEscortDest` (0x997630) - 检测是否到达护送点

## 2. EscortDest 数据结构

从反编译代码分析，`EscortDest` 结构体包含以下成员：

```cpp
struct EscortDest {
    int x;              // +0x00 护送点X坐标
    int y;              // +0x04 护送点Y坐标  
    int nAttr;          // +0x08 属性类型（1=梯子/绳子点, 2=停留点）
    int ZMass;          // +0x0C Z轴层级
    int nStopDuration;  // +0x10 停留时间（毫秒）
};
```

护送点存储在 `CVecCtrlMob::m_aEscorDest` 数组中，使用 `m_nCurrentDestIndex` 跟踪当前目标点。

## 3. CtrlUpdateActiveEscort 函数分析

### 3.1 目标跟随逻辑
```cpp
// 如果有跟随目标（m_pTarget）
if ( this->m_pTarget )
{
    // 检查目标是否在不同的Z层
    if ( m_lZMass != m_pTarget->vfptr->GetZMass(m_pTarget) )
    {
        // 停止追逐
        TSecType<int>::SetData(&this->m_bChasing, 0);
        CVecCtrlMob::ChaseTargetImp(this, Data, 0, 0);
    }
    
    // 根据目标位置调整移动方向
    if ( target_x > mob_x )
        CVecCtrl::SetInput(this, 1, 0);   // 向右移动
    else if ( target_x < mob_x )
        CVecCtrl::SetInput(this, -1, 0);  // 向左移动
    else
        CVecCtrl::SetInput(this, 0, 0);   // 停止
}
```

### 3.2 返回起始点逻辑
```cpp
if ( this->m_bNeedReturnToStartTarget )
{
    if ( this->m_lZMass == this->m_nStartTargetPositionZmass )
    {
        // 向起始点移动
        if ( this->m_ptStartTargetPosition.x >= current_x )
            CVecCtrl::SetInput(this, 1, m_nInputY);  // 向右
        else
            CVecCtrl::SetInput(this, -1, m_nInputY); // 向左
    }
    else
    {
        // 不同Z层，取消返回
        this->m_bNeedReturnToStartTarget = 0;
    }
}
```

### 3.3 护送路径移动逻辑

#### 3.3.1 直角跳跃处理
```cpp
// 检查是否需要直角跳跃（垂直向下跳跃）
if ( this->m_Old_Dest.dp.x == v16->dp.x && 
     this->m_Old_Dest.dp.y > v16->dp.y && 
     this->m_bRightAngleJump )
{
    CVecCtrlMob::MoveMobOnRightAngleX(this);
    CVecCtrl::SetInput(this, 0, v17);  // 停止水平移动
    CVecCtrlMob::SetJumpNext(this, 0);
    this->m_bRightAngleJump = 0;
}
```

#### 3.3.2 水平移动控制
```cpp
// 根据目标点调整水平移动方向
if ( (double)destPoint->x > current_x )
    CVecCtrl::SetInput(this, 1, m_nInputY);   // 向右
else if ( (double)destPoint->x < current_x )
    CVecCtrl::SetInput(this, -1, m_nInputY);  // 向左
```

#### 3.3.3 跳跃触发条件
```cpp
// 需要跳跃的条件：
// 1. 不是特殊属性点（nAttr != 1）
// 2. 在相同Z层或目标点Y坐标更高
// 3. 在目标点的X坐标范围内
if ( destPoint->nAttr != 1 &&
     (this->m_pfh->m_lZMass == destPoint->ZMass ||
      AbsPos::_ZtlSecureGet_y(&this->m_ap) > (double)destPoint->y) &&
     IsInXRange(destPoint) )
{
    CVecCtrlMob::ClearMoveContext(this);
    
    // 调整移动方向防止越过目标点
    if ( destPoint->x < current_x && m_nInputX > 0 )
        CVecCtrl::SetInput(this, -1, m_nInputY);
    else if ( destPoint->x > current_x && m_nInputX < 0 )
        CVecCtrl::SetInput(this, 1, m_nInputY);
        
    CVecCtrlMob::SetJumpNext(this, 1);  // 设置跳跃标记
}
```

#### 3.3.4 梯子/绳子移动
```cpp
// 当前护送点是梯子/绳子点（nAttr == 1）
if ( destPoint->nAttr == 1 )
{
    if ( destPoint->y <= current_y )
        CVecCtrl::SetInput(this, m_nInputX, -1);  // 向上爬
    else
        CVecCtrl::SetInput(this, m_nInputX, 1);   // 向下爬
}
```

## 4. CollisionDetectEscortDest 函数分析

### 4.1 碰撞检测条件
护送点碰撞检测仅在以下条件下进行：
- 没有跟随目标（`!m_pTarget`）
- 没有在停留状态（`!m_nStopDuration`）
- 护送点数组不为空
- 在梯子/绳子上或有立足点

### 4.2 梯子/绳子上的碰撞检测
```cpp
if ( 在梯子或绳子上 )
{
    // X轴范围：护送点 ±25 像素
    bIn = (destPoint->x - 25) <= current_x && 
          (destPoint->x + 25) >= current_x;
    
    // 向下移动时的碰撞
    if ( m_nInputY > 0 && 
         destPoint->y <= current_y &&
         m_ptLastPoint.y <= destPoint->y )
    {
        bCollision = bIn;
    }
    
    // 向上移动时的碰撞
    if ( m_nInputY < 0 &&
         destPoint->y >= current_y &&
         m_ptLastPoint.y >= destPoint->y )
    {
        bCollision = bIn;
    }
}
```

### 4.3 地面上的碰撞检测
```cpp
else  // 在地面上
{
    // Y轴范围：护送点 ±25 像素
    bIn = (destPoint->y + 25) >= current_y &&
          (destPoint->y - 25) <= current_y;
    
    // 向右移动时的碰撞
    if ( m_nInputX > 0 &&
         destPoint->x <= current_x &&
         m_ptLastPoint.x <= destPoint->x )
    {
        bCollision = bIn;
    }
    
    // 向左移动时的碰撞
    if ( m_nInputX < 0 &&
         destPoint->x >= current_x &&
         m_ptLastPoint.x >= destPoint->x )
    {
        bCollision = bIn;
    }
}
```

### 4.4 到达护送点的处理

#### 4.4.1 更新护送点索引
```cpp
// 保存当前护送点信息到 m_Old_Dest
this->m_Old_Dest = destPoints[m_nCurrentDestIndex];
this->m_bRightAngleJump = 1;  // 标记可能需要直角跳跃

// 移动到下一个护送点
m_nCurrentDestIndex++;

// 检查是否完成所有护送点
if ( m_nCurrentDestIndex == destPointCount )
{
    // 清空护送点数组
    ZArray<CVecCtrlMob::EscortDest>::RemoveAll(&this->m_aEscorDest);
    
    // 通知服务器（如果是CMob）
    if ( this->m_pOwner->IsKindOf(CMob) )
        CMob::SendCollisionEscort(m_pOwner, m_nCurrentDestIndex);
}
```

#### 4.4.2 停留点处理
```cpp
// 如果前一个点是停留点（nAttr == 2）
if ( this->m_Old_Dest.nAttr == 2 )
{
    // 如果在梯子上，先脱离
    if ( CVecCtrl::GetLadderOrRope(this) )
        this->vfptr[5].__vecDelDtor(this, 0, 0, 0);
    
    // 设置停留时间
    this->m_nStopDuration = this->m_Old_Dest.nStopDuration + get_update_time();
    this->m_bEscortStop = 1;
    
    // 微调位置并停止移动
    AbsPos::_ZtlSecurePut_x(&this->m_ap, (double)this->m_Old_Dest.x + 0.1);
    AbsPos::_ZtlSecurePut_vx(&this->m_ap, 0.0);
    
    // 更新相对位置
    if ( this->m_pfh )
        RelPos::SetFromAbsPos(&this->m_rp, &this->m_ap, this->m_pfh);
        
    // 设置移动路径属性为33（可能表示停留状态）
    CVecCtrl::SetMovePathAttribute(this, 33);
}
```

#### 4.4.3 方向调整
```cpp
// 根据下一个护送点调整移动方向
if ( nextDestPoint )
{
    // 梯子/绳子上的方向调整
    if ( CVecCtrl::GetLadderOrRope(this) )
    {
        if ( nextDestPoint->y > currentDestPoint->y )
            CVecCtrl::SetInput(this, m_nInputX, 1);   // 向下
        else if ( nextDestPoint->y < currentDestPoint->y )
            CVecCtrl::SetInput(this, m_nInputX, -1);  // 向上
    }
}
```

## 5. 护送模式的移动决策流程

1. **优先级判断**：
   - 如果有跟随目标（m_pTarget），优先跟随目标
   - 如果需要返回起始点，执行返回逻辑
   - 否则按照护送路径移动

2. **路径移动策略**：
   - 水平移动：始终朝向当前目标点
   - 垂直移动：需要跳跃时触发跳跃，在梯子上时上下移动
   - 特殊移动：直角跳跃用于垂直下降

3. **碰撞检测**：
   - 使用25像素的容差范围
   - 考虑移动方向和上一帧位置，防止穿过目标点
   - 到达后立即处理，更新到下一个目标点

4. **状态管理**：
   - `m_nCurrentDestIndex`：当前目标点索引
   - `m_Old_Dest`：上一个到达的护送点
   - `m_bRightAngleJump`：是否需要直角跳跃
   - `m_bEscortStop`：是否在停留状态
   - `m_nStopDuration`：停留结束时间

## 6. 特殊机制

### 6.1 直角跳跃
用于处理垂直向下的路径，当新目标点在正下方时触发。

### 6.2 停留机制
护送点可以设置停留时间，怪物到达后会暂停指定时间。

### 6.3 Z层切换
护送路径可以跨越不同的Z层（通过跳跃或梯子）。

### 6.4 网络同步
到达护送点或完成护送时，会通过 `CMob::SendCollisionEscort` 通知服务器。