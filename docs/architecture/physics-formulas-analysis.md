# MapleStory 物理公式详细分析

## 1. 物理常量

### 1.1 CWvsPhysicalSpace2D 构造函数
- **地址**: `0xa17432`
- **符号**: `??0CWvsPhysicalSpace2D@@QAE@XZ`

从 IDA 分析中发现，CWvsPhysicalSpace2D 在构造函数中从 Map/Physics.img 文件加载了以下物理常量：

```cpp
// 构造函数中加载的常量（通过字符串引用确认）
struct CONSTANTS {
    double dWalkForce[14];    // 14种不同地形的行走力量参数
    double dJumpSpeed;        // 跳跃速度
    double dMaxFriction;      // 最大摩擦力
    double dMinFriction;      // 最小摩擦力  
    double dSwimSpeedDec;     // 游泳速度衰减
    double dFlyJumpDec;       // 飞行跳跃衰减
};
```

### 1.2 dWalkForce[14] 数组详细分析

#### 1.2.1 数组加载位置
从 `CWvsPhysicalSpace2D` 构造函数（地址：`0xa17432`）中，dWalkForce[14] 数组从 Map/Physics.img 文件加载：

| 索引 | 参数名 | 存储位置 | 描述 |
|------|--------|----------|------|
| 0 | "walkForce" | dWalkForce[0] | 基础行走力量 |
| 1 | "walkSpeed" | dWalkForce[1] | 行走速度（注：实际使用 CAttrShoe::walkSpeed） |
| 2 | "walkDrag" | dWalkForce[2] | 行走阻力 |
| 3 | "slipForce" | dWalkForce[3] | 滑行力量 |
| 4 | "slipSpeed" | dWalkForce[4] | 滑行速度 |
| 5 | "floatSpeed1" | dWalkForce[5] | 浮动速度1（未找到直接使用） |
| 6 | "floatSpeed2" | dWalkForce[6] | 浮动速度2（未找到直接使用） |
| 7 | "floatDrag1" | dWalkForce[7] | 浮动阻力1 |
| 8 | "floatDrag2" | dWalkForce[8] | 浮动阻力2 |
| 9 | "swimSpeed" | dWalkForce[9] | 游泳速度 |
| 10 | "swimSpeedDec" | dWalkForce[10] | 游泳速度衰减 |
| 11 | "flySpeed" | dWalkForce[11] | 飞行速度 |
| 12 | "gravityAcc" | dWalkForce[12] | 重力加速度 |
| 13 | "fallSpeed" | dWalkForce[13] | 下落速度 |

#### 1.2.2 其他独立加载的物理参数
- "jumpSpeed" → dJumpSpeed
- "maxFriction" → dMaxFriction  
- "minFriction" → dMinFriction
- "swimSpeedDec" → dSwimSpeedDec（注：这个参数名在数组中重复）
- "flyJumpDec" → dFlyJumpDec

#### 1.2.3 参数使用情况总结

**在 CalcWalk 函数中使用的参数**：
- dWalkForce[0] (walkForce) - 计算行走力量
- dWalkForce[2] (walkDrag) - 计算行走阻力
- dWalkForce[3] (slipForce) - 计算滑坡力量
- dWalkForce[4] (slipSpeed) - 计算滑坡速度
- dWalkForce[10] (swimSpeedDec) - 游泳状态速度衰减

**在 CalcFloat 函数中使用的参数**：
- dWalkForce[7] (floatDrag1) - 空中阻力系数1
- dWalkForce[8] (floatDrag2) - 空中阻力系数2  
- dWalkForce[9] (swimSpeed) - 游泳速度系数
- dWalkForce[11] (flySpeed) - 飞行速度系数
- dWalkForce[12] (gravityAcc) - 重力加速度
- dWalkForce[13] (fallSpeed) - 最大下落速度系数

**特殊说明**：
1. walkSpeed (dWalkForce[1]) 虽然加载到数组中，但实际速度限制使用的是 CAttrShoe::walkSpeed
2. floatSpeed1 和 floatSpeed2 (dWalkForce[5-6]) 在当前分析的代码中未找到直接使用

## 2. 时间系统

### 2.1 时间步长计算

从 CVecCtrl::WorkUpdateActive 函数中发现：
- **地址**: `0x994460`
- **符号**: `?WorkUpdateActive@CVecCtrl@@MAEHJ@Z`
- **大小**: `0x2d8`

```cpp
// 时间步长转换为秒
tSec = (double)tElapse / DOUBLE_1000_0;
```

这表明：
- tElapse 参数是毫秒为单位的时间步长
- 物理计算使用秒为单位，所以需要除以 1000.0

### 2.2 物理更新流程

从反编译代码中提取的实际物理更新逻辑：

```cpp
// CVecCtrl::WorkUpdateActive (0x994460)
int __thiscall CVecCtrl::WorkUpdateActive(CVecCtrl *this, int tElapse) {
    if (this->m_pfh) {  // 如果在踏脚点上
        // 地址 0x9944df: 调用 CalcWalk
        CVecCtrl::CalcWalk(this, tElapse);
        
        // 地址 0x99451a-0x994564: 位置更新公式
        tSec = (double)tElapse / DOUBLE_1000_0;
        // 使用 _ZtlSecureFuse 安全读取速度值
        velocity = _ZtlSecureFuse<double>((int)&rpl[3], LODWORD(rpl[5]));
        // 更新相对位置
        RelPos::_ZtlSecurePut_pos(&this->m_rp, velocity * tSec + pos_old);
    } else {  // 在空中
        // 地址 0x99463b: 调用 CalcFloat
        CVecCtrl::CalcFloat(this, tElapse);
        
        // 地址 0x9945a6-0x994607: 绝对坐标更新
        // X 坐标更新
        vx = _ZtlSecureFuse<double>((int)&apl[6], LODWORD(apl[8]));
        AbsPos::_ZtlSecurePut_x(&this->m_ap, vx * tSec + x_old);
        
        // Y 坐标更新
        vy = _ZtlSecureFuse<double>((int)&apl[9], LODWORD(apl[11]));
        AbsPos::_ZtlSecurePut_y(&this->m_ap, vy * tSec + y_old);
    }
}
```

## 3. 物理公式

### 3.1 位置更新公式

基于反编译代码分析，MapleStory 使用的是标准的物理公式：

**地面移动（在踏脚点上）**：
```
// 地址 0x994564 的实现
position = position + velocity * dt
```
具体代码：
- `RelPos::_ZtlSecurePut_pos(&this->m_rp, velocity * tSec + pos_old)`

**空中移动**：
```
// 地址 0x9945ca 和 0x994607 的实现
x = x + vx * dt
y = y + vy * dt
```
具体代码：
- X轴：`AbsPos::_ZtlSecurePut_x(&this->m_ap, vx * tSec + x_old)`
- Y轴：`AbsPos::_ZtlSecurePut_y(&this->m_ap, vy * tSec + y_old)`

### 3.2 加速度计算细节

从分析中发现的关键信息：

#### 3.2.1 虚函数调用机制
物理计算通过虚函数表调用，主要函数包括：
- CalcWalk: 计算地面行走物理（通过虚表调用）
- CalcFloat: 计算空中浮动物理（通过虚表调用）
- CollisionDetectFloat: 碰撞检测和速度调整（地址 0x994740）

#### 3.2.2 CollisionDetectFloat 中的速度计算
在 `CVecCtrl::CollisionDetectFloat` (0x994740) 函数中发现了速度调整的代码：

```cpp
// 地址 0x994d99-0x994deb: 碰撞时的速度插值计算
vxa = (v74 - _ZtlSecureFuse<double>(p1->_ZtlSecureTear_vx)) * vya + vxb;
vy = (v75 - _ZtlSecureFuse<double>(p1->_ZtlSecureTear_vy)) * vya + v107;

// 地址 0x994e28: 根据踏脚点法线调整速度
this->m_rp._ZtlSecureTear_v = pfhFirstCollideCW->m_uvy * vy + pfhFirstCollideCW->m_uvx * vxa;

// 地址 0x99504d-0x995063: 碰撞后的速度反弹计算
AbsPos::_ZtlSecurePut_vx(v39, vd * v41->m_uvx);  // X轴速度
AbsPos::_ZtlSecurePut_vy(v39, v41->m_uvy * vd);  // Y轴速度
```

#### 3.2.3 输入处理和加速度转换
从 `CVecCtrlUser::WorkUpdateActive` (0x9a1390) 中的输入处理：

```cpp
// 地址 0x9a1594-0x9a15da: 键盘输入转换为方向值
lInputX = IsKeyPressed(VK_RIGHT) - IsKeyPressed(VK_LEFT);  // 结果为 -1, 0, 1
lInputY = IsKeyPressed(VK_DOWN) - IsKeyPressed(VK_UP);      // 结果为 -1, 0, 1

// 地址 0x9a1700: 将输入传递给物理系统
CVecCtrl::SetInput(this, lInputX, lInputY);
```

#### 3.2.4 CalcFloat 函数的具体实现

找到了 `CVecCtrl::CalcFloat` 的完整实现，这是空中物理计算的核心函数：

```cpp
void __thiscall CVecCtrl::CalcFloat(CVecCtrl *this, int tElapse)
```

**关键物理计算**：

1. **时间步长转换**：
```cpp
tSec = (double)tElapse / DOUBLE_1000_0;  // 毫秒转秒
```

2. **自由落体状态**（普通空中状态）：
```cpp
// 重力加速度应用
v15 = TSecType<double>::GetData(&this->m_pAttrField->g);  // 地形重力系数
AccSpeed(&vy, v15 * v14->dGravityAcc * mass, mass, vyMax, tSec);

// 垂直速度最大值
vyMax = TSecType<double>::GetData(&this->m_pAttrField->g) * v9->dFallSpeed;

// 水平移动（空中控制）
if (inputX != 0) {
    AccSpeed(&vx, inputX * drag2 * 2, mass, accShoe, tSec);
} else {
    // 空中阻力
    if (vyMax > vy) {
        DecSpeed(&vx, dFloatCoefficient * drag2, mass, 0.0, tSec);
    } else {
        DecSpeed(&vx, drag2, mass, 0.0, tSec);
    }
}
```

3. **飞行状态**：
```cpp
// 飞行加速度
fx = inputX * dFlyForce * accShoe;
fy = inputY * dFlyForce * accShoe;

// 飞行速度限制
vyMax = accShoe * (dFlySpeed * flySpeed);

// 应用加速度
AccSpeed(&vx, fx, mass, vyMax, tSec);
AccSpeed(&vy, fy, mass, vyMax, tSec);
```

4. **游泳状态**：
```cpp
// 游泳使用不同的参数
if (IsSwimming()) {
    accShoe = swimAcc;
    vMax = swimSpeedH * dSwimSpeed * vyMax;
    force = dSwimForce;
} else {
    accShoe = flyAcc;
    vMax = flySpeed * dFlySpeed * vyMax;
    force = dFlyForce;
}

// 特殊的垂直速度处理
if (inputY < 0) {  // 向上
    vMax = vMaxa * 0.3;  // 上游速度为30%
} else if (inputY > 0) {  // 向下
    vMax = vMaxa * 1.5;  // 下潜速度为150%
}
```

5. **位置更新公式**（使用平均速度）：
```cpp
// X坐标更新
x_new = x_old + (v_old + v_new) * 0.5 * tSec;

// Y坐标更新  
y_new = y_old + (v_old + v_new) * 0.5 * tSec;
```

#### 3.2.5 速度计算公式（从代码提取）

**碰撞时的速度调整**：
```cpp
// 速度根据踏脚点法线进行投影
v_parallel = (vx * foothold.uvx + vy * foothold.uvy)  // 平行于踏脚点的速度分量
vx_new = v_parallel * foothold.uvx  // 新的X速度
vy_new = v_parallel * foothold.uvy  // 新的Y速度
```

**摩擦力处理**（从代码模式推断）：
```cpp
// 地址 0x9951b6-0x9951e9: 斜坡上的速度衰减
if (foothold.uvx > 0.0) {  // 斜坡
    if (v * inputX > 0) {  // 速度与输入同向
        v = v * 0.5;  // 速度减半（摩擦效果）
    }
}
```

### 3.3 虚函数表地址映射

| 虚表地址 | 类名 | 描述 |
|----------|------|------|
| 0xbabd70 | CVecCtrl | 基类虚函数表 |
| 0xbac4b0 | CVecCtrlUser | 用户控制虚函数表 |
| 0x994740 | CollisionDetectFloat | 虚表索引 11 |

### 3.4 AccSpeed 和 DecSpeed 函数的具体实现

#### 3.4.1 AccSpeed 函数

```cpp
void __cdecl AccSpeed(long double *v, double f, long double m, double vMax, long double tSec)
{
  if ( vMax >= 0.0 )
  {
    if ( f <= 0.0 )  // 负向力（向左/向上）
    {
      if ( -vMax < *v )  // 还没达到负向最大速度
      {
        *v = f / m * tSec + *v;  // v = v + (f/m) * dt
        if ( *v < -vMax )
          *v = -vMax;  // 限制在负向最大速度
      }
    }
    else  // 正向力（向右/向下）
    {
      if ( vMax > *v )  // 还没达到正向最大速度
      {
        *v = f / m * tSec + *v;  // v = v + (f/m) * dt
        if ( *v > vMax )
          *v = vMax;  // 限制在正向最大速度
      }
    }
  }
}
```

**关键公式**：
- 加速度：`a = f / m`
- 速度更新：`v_new = v_old + a * tSec`
- 速度限制：`-vMax <= v <= vMax`

#### 3.4.2 DecSpeed 函数

```cpp
void __cdecl DecSpeed(long double *v, long double f, long double m, long double vMax, long double tSec)
{
  if ( vMax >= 0.0 )
  {
    if ( vMax < *v )  // 当前速度超过正向限制
    {
      *v = *v - f / m * tSec;  // 减速
      if ( *v < vMax )
        *v = vMax;  // 不要过度减速
    }
    else if ( -vMax > *v )  // 当前速度超过负向限制
    {
      *v = *v + f / m * tSec;  // 向正方向减速
      if ( *v > -vMax )
        *v = -vMax;  // 不要过度减速
    }
  }
}
```

**关键特性**：
1. 减速总是朝向零速度方向
2. 不会使速度反向（不会从正变负或从负变正）
3. vMax 参数用作减速后的目标速度限制

### 3.5 物理常量的实际应用

从 CalcFloat 函数中识别的物理常量使用：

| 常量名 | 用途 | 公式中的应用 |
|--------|------|--------------|
| dGravityAcc | 重力加速度 | `force = g * dGravityAcc * mass` |
| dFallSpeed | 最大下落速度系数 | `vyMax = g * dFallSpeed` |
| dFloatDrag1 | 空中阻力系数1 | `g = drag * dFloatDrag1` |
| dFloatDrag2 | 空中阻力系数2 | `drag2 = drag * dFloatDrag2` |
| dFloatCoefficient | 浮动系数 | 用于空中水平减速 |
| dFlyForce | 飞行力量 | `force = input * dFlyForce * acc` |
| dFlySpeed | 飞行速度系数 | `vMax = acc * dFlySpeed * flySpeed` |
| dSwimForce | 游泳力量 | `force = input * dSwimForce * acc` |
| dSwimSpeed | 游泳速度系数 | `vMax = acc * dSwimSpeed * swimSpeed` |

### 3.6 地形对加速度的影响

### 3.3 地形影响

不同地形通过 dWalkForce 数组的 14 个值来影响移动：
- 索引 0-13 对应不同的地形类型
- 每种地形有不同的力量系数
- 实际力量 = 基础力量 * 地形系数

### 3.4 特殊状态物理

**游泳状态**：
- 使用 swimSpeed 替代 walkSpeed
- 应用 swimSpeedDec 作为水阻

**飞行状态**：
- 使用 flySpeed
- 跳跃力量受 flyJumpDec 影响

## 4. 关键发现

### 4.1 物理更新频率
- 物理计算基于毫秒级时间步长
- 每帧都会调用 WorkUpdateActive 进行物理更新
- 时间步长转换：dt (秒) = tElapse (毫秒) / 1000.0

### 4.2 双重坐标系统
- 相对坐标 (RelPos)：在踏脚点上时使用
- 绝对坐标 (AbsPos)：在空中时使用
- 两个系统独立更新，根据状态切换

### 4.3 安全机制
- 使用 _ZtlSecureFuse 和 _ZtlSecureTear 保护关键物理数据
- 防止内存修改和作弊

## 5. 完整的物理公式总结

基于 CalcFloat 函数的分析，现在可以确认 MapleStory 的物理公式：

### 5.1 基础物理公式

**加速度到速度**：
```
acceleration = force / mass
velocity_new = velocity_old + acceleration * dt
```

**位置更新（使用平均速度法）**：
```
position_new = position_old + (velocity_old + velocity_new) * 0.5 * dt
```

### 5.2 重力系统

**重力加速度**：
```
gravity_force = terrain_gravity * dGravityAcc * mass
```

**最大下落速度**：
```
max_fall_speed = terrain_gravity * dFallSpeed
```

### 5.3 空中控制

**水平空中移动**：
```
if (input_x != 0):
    force_x = input_x * drag2 * 2
else:
    if (vy < vyMax):  // 还在加速下落
        deceleration = dFloatCoefficient * drag2
    else:  // 达到最大下落速度
        deceleration = drag2
```

### 5.4 特殊状态物理

**飞行状态**：
```
force_x = input_x * dFlyForce * flyAcc * terrain_fly
force_y = input_y * dFlyForce * flyAcc * terrain_fly
max_speed = terrain_fly * dFlySpeed * flySpeed
```

**游泳状态**：
```
force = input * dSwimForce * swimAcc * terrain_fly
max_speed = terrain_fly * dSwimSpeed * swimSpeed

// 垂直速度特殊处理
if (swimming_up): max_speed *= 0.3
if (swimming_down): max_speed *= 1.5
```

## 6. CalcWalk 函数的完整实现

找到了 `CVecCtrl::CalcWalk` 的完整实现，这是地面移动物理计算的核心函数：

```cpp
void __thiscall CVecCtrl::CalcWalk(CVecCtrl *this, int tElapse)
```

### 6.1 地面移动的关键物理计算

1. **斜坡物理参数**：
```cpp
sin1 = fabs(foothold->m_uvy);  // 踏脚点的垂直分量（斜率）
vMaxL = sin1 * sin1;            // 斜率的平方，用于速度调整
hd = (foothold->m_uvy >= 0.0) ? -1 : 1;  // 斜坡方向
```

2. **行走力量计算**：
```cpp
// 基础行走力量
walkForce = walkAcc * (dWalkForce * drag * terrain_walk);

// 斜坡影响
if (hd <= 0)  // 下坡
    factor = vMaxL + 1.0;
else          // 上坡
    factor = 1.0 - vMaxL;
    
force = walkForce * factor * inputX;
```

3. **特殊地形力量（force属性）**：
```cpp
if (foothold->force == 0.0) {
    // 普通地形：只有输入同向才有力
    if (inputX * force > 0.0)
        finalForce = force;
} else if (inputX != 0) {
    // 传送带地形
    if (inputX * foothold->force <= 0.0)  // 反向
        finalForce = 0.2 / |force| * walkForce;
    else  // 同向
        finalForce = 2.0 * |force| * walkForce;
} else {
    // 无输入时传送带自动移动
    finalForce = foothold->force * walkForce;
}
```

4. **摩擦力系统**：
```cpp
// 摩擦力计算
friction = walkDrag * drag * foothold->drag * terrain->drag;

// 限制在最大最小值之间
if (friction > dMaxFriction) friction = dMaxFriction;
if (friction < dMinFriction) friction = dMinFriction;
if (friction < 1.0) friction *= 0.5;

// 实际阻力
drag = friction * dWalkDrag;
```

5. **滑坡处理**（当斜率大于 walkSlant）：
```cpp
if (sin1 > walkSlant) {
    slipForce = dSlipForce * sin1 * -hd;
    slipSpeed = dSlipSpeed * sin1;
    
    if (hd * inputX <= 0) {  // 试图向上爬
        slipForce = slipForce + walkForce;
        slipSpeed = slipSpeed + vMaxL;
    } else {  // 顺坡滑下
        slipForce = slipForce * 0.5;
        slipSpeed = slipSpeed * 0.5;
    }
}
```

### 6.2 速度限制系统

根据斜坡方向和当前速度方向，使用不同的速度限制：

```cpp
// 平地
if (sin1 == 0.0) {
    maxSpeed = walkSpeed;
}
// 斜坡
else {
    maxSpeedUp = walkSpeed;
    maxSpeedDown = (vMaxL + 1.0) * walkSpeed;
    
    if (hd * velocity <= 0.0)  // 向上
        maxSpeed = maxSpeedDown;
    else  // 向下
        maxSpeed = maxSpeedUp;
}
```

### 6.3 游泳状态影响

```cpp
if (IsSwimming()) {
    force *= dSwimSpeedDec;
    maxSpeed *= dSwimSpeedDec;
}
```

## 7. 待确认信息

基于 CalcFloat 和 CalcWalk 的分析，仍需要确认：

1. **具体的物理常量数值**
   - 需要从 Map/Physics.img 文件中读取

### 7.2 速度相关的其他发现

#### 7.2.1 CAttrShoe 类的速度属性
在地址 `0x50b756` 的 CAttrShoe 构造函数中，walkSpeed 是作为类成员初始化的：
```cpp
TSecType<double>::TSecType<double>(&this->walkSpeed);
```

#### 7.2.2 装备系统的速度增益
发现了与装备系统相关的速度增益属性：
- **incSpeed** - 速度增加值（地址：0xb4fcfc）
- **incSpeedMin** - 最小速度增加（地址：0xb4f68c）
- **incSpeedMax** - 最大速度增加（地址：0xb4f680）
- **psdSpeed** - 物理速度（地址：0xba9768）

这些属性主要用于装备效果（如宝石效果），在 `CItemMakerInfo::RegisterGemEffect`（地址：0x5c7360）中处理。

## 8. 关键函数地址和符号汇总

### 8.1 物理空间类
| 函数名 | 地址 | 符号 | 描述 |
|--------|------|------|------|
| CWvsPhysicalSpace2D 构造函数 | 0xa17432 | ??0CWvsPhysicalSpace2D@@QAE@XZ | 初始化物理常量 |
| TSingleton<CWvsPhysicalSpace2D>::ms_pInstance | 0xc687b0 | ?ms_pInstance@?$TSingleton@VCWvsPhysicalSpace2D@@@@1PAVCWvsPhysicalSpace2D@@A | 单例实例 |

### 8.2 向量控制类
| 函数名 | 地址 | 符号 | 描述 |
|--------|------|------|------|
| CVecCtrl::WorkUpdateActive | 0x994460 | ?WorkUpdateActive@CVecCtrl@@MAEHJ@Z | 主物理更新函数 |
| CVecCtrl::CalcFloat | - | - | 空中物理计算（完整实现已找到） |
| CVecCtrl::CalcWalk | - | - | 地面物理计算（完整实现已找到） |
| CVecCtrl::CollisionDetectFloat | 0x994740 | ?CollisionDetectFloat@CVecCtrl@@UAEHABUAbsPos@@AAJH@Z | 碰撞检测 |
| CVecCtrlUser::WorkUpdateActive | 0x9a1390 | ?WorkUpdateActive@CVecCtrlUser@@MAEHJ@Z | 用户控制的物理更新 |
| CVecCtrlUser::IsAbleToClimbLadderOrRope | 0x9a0c30 | - | 检查攀爬能力 |
| AccSpeed | - | - | 加速函数（完整实现已找到） |
| DecSpeed | - | - | 减速函数（完整实现已找到） |

### 8.3 角色属性类
| 函数名 | 地址 | 符号 | 描述 |
|--------|------|------|------|
| CAttrShoe 构造函数 | 0x50b756 | - | 初始化角色移动属性 |
| CItemMakerInfo::RegisterGemEffect | 0x5c7360 | - | 处理装备速度增益效果 |

### 8.4 位置更新函数
| 操作 | 地址 | 描述 |
|------|------|------|
| 时间转换 | 0x99451a | tElapse / 1000.0 转换为秒 |
| 相对位置更新 | 0x994564 | RelPos::_ZtlSecurePut_pos |
| X坐标更新 | 0x9945ca | AbsPos::_ZtlSecurePut_x |
| Y坐标更新 | 0x994607 | AbsPos::_ZtlSecurePut_y |

### 8.5 物理计算虚函数（通过虚表调用）
| 函数 | 虚表索引 | 描述 |
|------|----------|------|
| CalcWalk | - | 地面行走物理计算 |
| CalcFloat | - | 空中浮动物理计算 |
| CollisionDetect | vfptr[7] | 碰撞检测 |
| 位置更新回调 | vfptr[3]/vfptr[4] | 处理位置更新 |

### 8.6 安全包装函数
| 函数 | 用途 |
|------|------|
| _ZtlSecureFuse | 安全读取加密数据 |
| _ZtlSecurePut | 安全写入加密数据 |
| _ZtlSecureGet | 安全获取加密数据 |

## 9. 实现建议

基于当前分析，实现 MapleStory 物理系统的关键步骤：

1. **建立物理常量系统**
   - 创建类似 CONSTANTS 的结构
   - 从配置文件加载物理参数
   - 支持不同地形的参数配置

2. **实现时间步长系统**
   - 使用固定时间步长（如 16.67ms for 60fps）
   - 时间单位转换（毫秒到秒）
   - 支持可变时间步长的插值

3. **实现基础物理公式**
   - 位置 = 位置 + 速度 * dt
   - 速度 = 速度 + 加速度 * dt
   - 考虑摩擦力和阻力

4. **处理不同移动状态**
   - 地面行走物理
   - 空中物理（重力影响）
   - 特殊状态（游泳、飞行）

这个分析为实现 MapleStory 风格的 2D 平台游戏物理系统提供了坚实的基础。