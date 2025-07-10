# CVecCtrl 相关函数分析

## 1. CVecCtrl::Jump 函数
地址: `0x994430`
```cpp
void __thiscall CVecCtrl::Jump(CVecCtrl *this)
{
    this->m_bJumpNext = 0;
    this->m_bTryJumpedInFly = 1;
    if ( CVecCtrl::JustJump(this) )
        CVecCtrl::SetMovePathAttribute(this, 1);
}
```

关键点：
- 设置 `m_bJumpNext = 0` 表示不需要再次跳跃
- 设置 `m_bTryJumpedInFly = 1` 表示尝试在空中跳跃
- 调用 `JustJump()` 执行实际的跳跃逻辑
- 如果跳跃成功，调用 `SetMovePathAttribute(this, 1)` 设置移动路径属性

## 2. CVecCtrl::FallDown 函数
地址: `0x993d80`
```cpp
void __thiscall CVecCtrl::FallDown(CVecCtrl *this)
{
    // 保存下落开始的立足点
    pfhFallStart = this->m_falldownNext.pfhFallStart;
    this->m_falldownNext.bValid = 0;
    this->m_pfhFallStart = pfhFallStart;
    
    // 如果在梯子或绳子上
    if ( _ZtlSecureFuse<CLadderOrRope *>(this->_ZtlSecureTear_m_pLadderOrRope, t_4) )
    {
        // 调用虚函数表中的第5个函数（可能是脱离梯子/绳子的函数）
        this->ZRefCounted::vfptr[5].__vecDelDtor(this, 0, 0, 0);
    }
    else
    {
        if ( !this->m_pfh )
            return;
            
        // 脱离立足点
        CVecCtrl::DetachFromFoothold(this);
        
        // 如果是玩家，播放跳跃音效
        if ( !this->m_pOwner->vfptr->GetType(this->m_pOwner) )
        {
            play_game_sound("Jump", 0x64u);
        }
    }
    
    // 设置下落速度
    CONSTANTS *p = TSingleton<CWvsPhysicalSpace2D>::ms_pInstance->m_constants.p;
    double g = TSecType<double>::GetData(&this->m_pAttrField->g);
    this->m_ap._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(
        p->dJumpSpeed * DOUBLE_N0_35355339 / g,
        this->m_ap._ZtlSecureTear_vy
    );
    this->m_ap._ZtlSecureTear_vx_CS = _ZtlSecureTear<double>(0.0, this->m_ap._ZtlSecureTear_vx);
}
```

关键点：
- 处理从立足点或梯子/绳子下落的逻辑
- 设置垂直速度为正值（向下）
- 水平速度设为0

## 3. CVecCtrl::JustJump 函数
地址: `0x993ea0`

这是实际执行跳跃逻辑的核心函数，处理了多种情况：

### 3.1 飞行/游泳跳跃
```cpp
// 检查是否可以飞行
BOOL v3 = 1;
if ( !((int (__thiscall *)(CVecCtrl *))this->ZRefCounted::vfptr[2].__vecDelDtor)(this) && !CVecCtrl::IsSwimming(this) )
{
    p = this->m_pCurAttrShoe.p;
    if ( !p || TSecType<double>::GetData(&p->flyAcc) <= DOUBLE_0_0 )
        v3 = 0;
}
```

### 3.2 在梯子/绳子上的跳跃
```cpp
if ( _ZtlSecureFuse<CLadderOrRope *>(this->_ZtlSecureTear_m_pLadderOrRope, this->_ZtlSecureTear_m_pLadderOrRope_CS) )
{
    if ( _ZtlSecureFuse<long>(this->_ZtlSecureTear_m_nInputX, this->_ZtlSecureTear_m_nInputX_CS) )
    {
        // 如果按住方向键，从梯子/绳子上跳下
        this->ZRefCounted::vfptr[5].__vecDelDtor(this, 0, 0, 0);
        
        // 设置跳跃速度（减小的跳跃高度）
        double vMax = v3 ? DOUBLE_0_3 : DOUBLE_0_5;
        this->m_ap._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(
            -(v5->dJumpSpeed * Data / v6 * vMax),
            this->m_ap._ZtlSecureTear_vy
        );
        
        // 设置水平速度
        if ( this->m_bEscortMob )
        {
            // 护送怪物时的特殊速度
            this->m_ap._ZtlSecureTear_vx_CS = _ZtlSecureTear<double>(
                (double)fly_4a * DOUBLE_125_0 * DOUBLE_1_3,
                this->m_ap._ZtlSecureTear_vx
            );
        }
        else
        {
            // 正常水平速度
            this->m_ap._ZtlSecureTear_vx_CS = _ZtlSecureTear<double>(
                v8->dWalkSpeed * Data * (double)fly_4b * DOUBLE_1_3,
                this->m_ap._ZtlSecureTear_vx
            );
        }
    }
    return 1;
}
```

### 3.3 从立足点跳跃
```cpp
if ( m_pfh )  // 如果有立足点
{
    CVecCtrl::DetachFromFoothold(this);
    
    // 计算跳跃速度
    CONSTANTS *v10 = TSingleton<CWvsPhysicalSpace2D>::ms_pInstance->m_constants.p;
    double walkJump = TSecType<double>::GetData(&this->m_pCurAttrShoe.p->walkJump);
    double g = TSecType<double>::GetData(&this->m_pAttrField->g);
    
    // 设置垂直速度（向上为负）
    this->m_ap._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(
        -(v10->dJumpSpeed * walkJump / g),
        this->m_ap._ZtlSecureTear_vy
    );
    
    // 如果可以飞行，减小跳跃高度
    if ( v3 )
    {
        double vy = _ZtlSecureFuse<double>((int)this->m_ap._ZtlSecureTear_vy, v12);
        this->m_ap._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(vy * DOUBLE_0_7, this->m_ap._ZtlSecureTear_vy);
    }
    
    // 处理水平速度
    if ( _ZtlSecureFuse<long>(this->_ZtlSecureTear_m_nInputX, this->_ZtlSecureTear_m_nInputX_CS) )
    {
        // 计算最大行走速度
        vMax = max_walk_speed(m_pfh, this->m_pCurAttrShoe.p, v14);
        
        if ( this->m_bEscortMob )
        {
            // 护送怪物时的固定速度
            AbsPos::_ZtlSecurePut_vx(&this->m_ap, (double)fly_4c * DOUBLE_125_0);
        }
        else
        {
            // 正常跳跃时的水平速度调整
            // 保持当前速度的80%作为最小值
            if ( current_vx < vMax * DOUBLE_0_8 )
            {
                vx = current_vx + input_direction * vMax * DOUBLE_0_8;
            }
            // 限制最大速度
            if ( current_vx > vMax )
            {
                vx = input_direction * vMax;
            }
        }
    }
    
    // 播放跳跃音效（仅限玩家）
    if ( !this->m_pOwner->vfptr->GetType(this->m_pOwner) )
    {
        play_game_sound("Jump", 0x64u);
    }
}
```

### 3.4 空中跳跃（飞行/游泳）
```cpp
else  // 没有立足点，在空中
{
    if ( !v3 )  // 不能飞行
        return 0;
        
    double fly = fabs(TSecType<double>::GetData(&this->m_pAttrField->fly));
    
    if ( CVecCtrl::IsSwimming(this) )
    {
        // 游泳时的跳跃速度
        v17 = TSecType<double>::GetData(&this->m_pCurAttrShoe.p->swimSpeedV)
            * TSingleton<CWvsPhysicalSpace2D>::ms_pInstance->m_constants.p->dSwimSpeed
            * fly
            * DOUBLE_5_0;
    }
    else
    {
        if ( ((int (__thiscall *)(CVecCtrl *))this->ZRefCounted::vfptr[2].__vecDelDtor)(this) )
        {
            // 飞行时的跳跃速度（有额外的减速因子）
            Data = TSecType<double>::GetData(&this->m_pCurAttrShoe.p->flySpeed)
                * TSingleton<CWvsPhysicalSpace2D>::ms_pInstance->m_constants.p->dFlySpeed
                * fly
                * DOUBLE_5_0
                * TSingleton<CWvsPhysicalSpace2D>::ms_pInstance->m_constants.p->dFlyJumpDec;
                
            // 如果按住方向键，水平速度加倍
            if ( CVecCtrl::_ZtlSecureGet_m_nInputX(this) )
            {
                vx = AbsPos::_ZtlSecureGet_vx(&this->m_ap);
                AbsPos::_ZtlSecurePut_vx(&this->m_ap, vx + vx);
            }
            v17 = Data;
        }
        else
        {
            // 普通飞行速度
            v17 = TSecType<double>::GetData(p_flySpeed)
                * TSingleton<CWvsPhysicalSpace2D>::ms_pInstance->m_constants.p->dFlySpeed
                * fly;
        }
    }
    
    // 设置垂直速度
    this->m_ap._ZtlSecureTear_vy_CS = _ZtlSecureTear<double>(-v17, this->m_ap._ZtlSecureTear_vy);
}
```

## 4. 关于梯子/绳子相关的函数

从代码中可以看到：
- CVecCtrl 类有一个成员变量 `_ZtlSecureTear_m_pLadderOrRope` 用于存储当前附着的梯子或绳子
- 在 Jump 和 FallDown 函数中都会检查是否在梯子/绳子上
- 通过虚函数表的第5个函数（`vfptr[5].__vecDelDtor`）来处理脱离梯子/绳子的逻辑

## 5. 关于 Alt 键跳跃

从当前分析的代码中，没有直接看到处理 Alt 键跳跃的逻辑。Alt 键跳跃（向下跳跃穿过平台）的处理可能在以下位置：
1. 输入处理层（CUser 或 CWvsContext 的 OnKey 函数）
2. FallDown 函数可能被特殊调用来实现向下跳跃
3. 可能有专门的函数处理平台穿透

## 6. 重要常量和参数

- `dJumpSpeed`: 基础跳跃速度
- `walkJump`: 鞋子的跳跃加成
- `g`: 重力加速度
- `DOUBLE_0_7`: 飞行状态下的跳跃高度系数（70%）
- `DOUBLE_0_3` / `DOUBLE_0_5`: 从梯子/绳子跳跃时的高度系数
- `DOUBLE_125_0`: 护送怪物时的固定水平速度
- `DOUBLE_1_3`: 跳跃时的水平速度加成系数

## 7. 物理系统相关

- 使用了 `CWvsPhysicalSpace2D` 单例来获取物理常量
- 速度和位置使用了安全包装（`_ZtlSecureTear` / `_ZtlSecureFuse`）来防止作弊
- 立足点系统（`CStaticFoothold`）用于碰撞检测和站立