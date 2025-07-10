# MapleStory 物理参数相互作用关系分析

## 分析概述

本分析基于 MapleStory v95 客户端 (MD5: 600b1c2dda171684007f080aed6947eb) 的逆向工程，重点研究物理参数的相互作用关系和计算机制。

## 核心发现

### 1. dWalkForce[14] 数组参数分析

通过对 `CWvsPhysicalSpace2D::CWvsPhysicalSpace2D` 构造函数 (地址: 0xa173e0) 的分析，发现物理参数从 `Physics.img` 文件中加载，包含以下14个核心参数：

#### 1.1 dWalkForce 数组结构
```cpp
struct CONSTANTS {
    double dWalkForce[14];  // 核心移动力参数数组
    double dJumpSpeed;      // 跳跃速度
    double dMaxFriction;    // 最大摩擦力
    double dMinFriction;    // 最小摩擦力
    double dSwimSpeedDec;   // 游泳速度衰减
    double dFlyJumpDec;     // 飞行跳跃衰减
};
```

#### 1.2 参数加载序列
从构造函数可以看出参数加载顺序和标识符：

1. **dWalkForce[0]** - 标识符: "wa_1" (基础行走力)
2. **dWalkForce[1]** - 标识符: "wa_0" (修正行走力)
3. **dWalkForce[2]** - 标识符: "wa_3" (第三行走力参数)
4. **dWalkForce[3]** - 标识符: "sl" (斜坡参数)
5. **dWalkForce[4]** - 标识符: "sl_0" (斜坡修正参数)
6. **dWalkForce[5]** - 标识符: "fl_0" (浮动参数1)
7. **dWalkForce[6]** - 标识符: "fl" (浮动参数2)
8. **dWalkForce[7]** - 标识符: "fl_7" (浮动参数3)
9. **dWalkForce[8]** - 标识符: "sw_0" (游泳参数1)
10. **dWalkForce[9]** - 标识符: "sw_1" (游泳参数2)
11. **dWalkForce[10]** - 标识符: "fl_3" (浮动参数4)
12. **dWalkForce[11]** - 标识符: "fl_5" (浮动参数5)
13. **dWalkForce[12]** - 标识符: "gr" (重力参数)
14. **dWalkForce[13]** - 标识符: "fa_0" (衰减参数)

#### 1.3 其他关键物理参数
- **dJumpSpeed** - 标识符: "ju" (跳跃速度)
- **dMaxFriction** - 标识符: "ma_5" (最大摩擦力)
- **dMinFriction** - 标识符: "mi_3" (最小摩擦力)
- **dSwimSpeedDec** - 标识符: "sw" (游泳速度衰减)
- **dFlyJumpDec** - 标识符: "fl_6" (飞行跳跃衰减)

### 2. 地形属性系统 (CAttrFoothold)

#### 2.1 CAttrFoothold 结构分析
```cpp
class CAttrFoothold : public ZRefCounted {
    TSecType<double> walk;           // 行走系数 (默认: 1.0)
    TSecType<double> drag;           // 阻力系数 (默认: 1.0)
    TSecType<double> force;          // 额外力系数 (默认: 0.0)
    TSecType<int> forbidfalldown;    // 禁止下落标志 (默认: 0)
    TSecType<int> cantThrough;       // 不可穿越标志 (默认: 0)
};
```

#### 2.2 地形系数计算方法
- **walk**: 影响在该地形上的行走速度，乘法关系
- **drag**: 影响阻力大小，与摩擦力计算相关
- **force**: 额外施加的力（如传送带、风力等）
- **forbidfalldown**: 防止角色从该地形下落
- **cantThrough**: 阻止角色穿越该地形

### 3. 装备属性系统 (CAttrShoe)

#### 3.1 CAttrShoe 结构分析
```cpp
class CAttrShoe : public ZRefCounted {
    TSecType<double> mass;          // 质量 (默认: 100.0)
    TSecType<double> walkAcc;       // 行走加速度 (默认: 1.0)
    TSecType<double> walkSpeed;     // 行走速度 (默认: 1.0)
    TSecType<double> walkDrag;      // 行走阻力 (默认: 1.0)
    TSecType<double> walkSlant;     // 斜坡行走 (默认: 0.9)
    TSecType<double> walkJump;      // 行走跳跃 (默认: 1.0)
    TSecType<double> swimAcc;       // 游泳加速度 (默认: 1.0)
    TSecType<double> swimSpeedH;    // 游泳水平速度 (默认: 1.0)
    TSecType<double> swimSpeedV;    // 游泳垂直速度 (默认: 1.0)
    TSecType<double> flyAcc;        // 飞行加速度 (默认: 1.0 或 0.0)
    TSecType<double> flySpeed;      // 飞行速度 (默认: 1.0 或 0.0)
};
```

#### 3.2 装备属性对物理参数的影响机制
1. **乘法叠加**: 装备属性通过乘法方式影响基础物理参数
2. **分类影响**: 不同装备属性影响不同的移动状态
3. **飞行能力**: 根据 `bUserFlying` 参数决定飞行相关属性的启用

### 4. 物理参数叠加规则

#### 4.1 基础计算公式
```
最终移动力 = dWalkForce[索引] × CAttrShoe.相关属性 × CAttrFoothold.相关属性
```

#### 4.2 不同状态下的参数使用
1. **正常行走**: 使用 dWalkForce[0-2] × walkSpeed × walk
2. **斜坡行走**: 使用 dWalkForce[3-4] × walkSlant × walk
3. **游泳状态**: 使用 dWalkForce[8-9] × swimSpeedH/V × walk
4. **飞行状态**: 使用 dWalkForce[5-7,10-11] × flySpeed × walk

#### 4.3 摩擦力计算
```
摩擦力 = clamp(基础摩擦力 × drag × walkDrag, dMinFriction, dMaxFriction)
```

### 5. 地形系数的计算方法

#### 5.1 CStaticFoothold 结构
```cpp
class CStaticFoothold : public ZRefCounted {
    int m_x1, m_y1, m_x2, m_y2;     // 地形线段坐标
    double m_len;                    // 地形长度
    double m_uvx, m_uvy;             // 单位方向向量
    ZRef<CAttrFoothold> m_pAttrFoothold;  // 地形属性引用
};
```

#### 5.2 地形系数计算
1. **长度计算**: `m_len = sqrt((x2-x1)² + (y2-y1)²)`
2. **方向向量**: `m_uvx = (x2-x1)/m_len`, `m_uvy = (y2-y1)/m_len`
3. **属性应用**: 通过 `m_pAttrFoothold` 应用地形特殊属性

### 6. 相关函数地址和实现细节

#### 6.1 核心函数地址
- **CWvsPhysicalSpace2D::CWvsPhysicalSpace2D**: 0xa173e0 (大小: 0xcfe)
- **CAttrFoothold::CAttrFoothold**: 0xa14c30 (大小: 0x161)
- **CAttrShoe::CAttrShoe**: 0x50b710 (大小: 0x1e2)
- **CStaticFoothold::CStaticFoothold**: 0xa14e80 (大小: 0xd1)

#### 6.2 虚函数表地址
- **CWvsPhysicalSpace2D vtable**: 0xbae514
- **CAttrFoothold vtable**: 0xbae4f8
- **CAttrShoe vtable**: 0xb4b164

#### 6.3 单例实例地址
- **CWvsPhysicalSpace2D::ms_pInstance**: 0xc687b0

### 7. 物理参数单位推测

基于代码分析和游戏机制，推测各参数的单位：

1. **dWalkForce数组**: 像素/帧² (加速度单位)
2. **dJumpSpeed**: 像素/帧 (速度单位)
3. **摩擦力参数**: 像素/帧² (减速度单位)
4. **装备倍率**: 无单位倍数

### 8. 安全机制

#### 8.1 TSecType 模板类
所有关键物理参数都使用 `TSecType<T>` 模板类包装，包含：
- **内存加密**: 使用随机指针和异或加密
- **完整性检查**: 防止内存修改
- **反调试**: 增加逆向工程难度

#### 8.2 数据保护
- 使用 `FakePtr1` 和 `FakePtr2` 进行混淆
- 真实数据存储在 `m_secdata` 结构中
- 通过 `SetData` 和 `GetData` 方法访问

### 9. 总结

MapleStory 的物理系统采用了分层设计：
1. **基础层**: dWalkForce 数组定义基础物理常量
2. **装备层**: CAttrShoe 提供装备属性加成
3. **地形层**: CAttrFoothold 提供地形特殊效果
4. **安全层**: TSecType 提供数据保护

所有参数通过乘法叠加的方式相互作用，形成最终的物理效果。系统设计既保证了游戏的平衡性，又提供了足够的灵活性来实现各种特殊效果。

## 建议的后续研究方向

1. **实时物理计算**: 分析角色移动时的实时物理计算函数
2. **碰撞检测**: 研究角色与地形的碰撞检测算法
3. **技能影响**: 分析各种技能对物理参数的临时修改
4. **网络同步**: 研究物理状态的网络同步机制

---

*本分析基于 MapleStory v95 客户端静态分析，具体数值需要通过动态调试获取。*