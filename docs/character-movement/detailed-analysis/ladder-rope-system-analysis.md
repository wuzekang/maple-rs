# MapleStory 梯子/绳子系统分析报告

## 概述
本报告基于 MapleStory v95 客户端的 IDA 分析，深入研究了梯子/绳子系统的实现机制。通过逆向工程分析，发现了关键的物理控制类和移动状态管理系统。

## 核心数据结构和类

### 1. CWvsPhysicalSpace2D 类
**主要职责：** 管理地图的物理空间，包括地面、平台、梯子和绳子等物理对象。

#### 关键函数：
- **GetCrossCandidate** (地址: 0x516610)
  - 功能：获取指定区域内的交叉碰撞候选对象
  - 参数：x1, y1, x2, y2 (区域坐标), lCrossCandidate (输出列表)
  - 实现：使用 TRSTree 数据结构进行高效的空间查询

- **GetFoothold** (地址: 0x517d10)
  - 功能：通过序列号获取 foothold 对象
  - 参数：dwSN (foothold 序列号)
  - 返回：CStaticFoothold 指针

#### 数据结构：
```cpp
class CWvsPhysicalSpace2D {
    TRSTree<long, ZRef<CStaticFoothold>, 2, 4, 2> m_rtFoothold;  // foothold 空间索引树
    ZMap<unsigned long, ZRef<CStaticFoothold>, unsigned long> m_mFoothold;  // foothold 映射表
};
```

### 2. CVecCtrl 和 CVecCtrlUser 类
**主要职责：** 管理角色的移动控制和状态检测。

#### 关键状态检测函数：
- **IsOnLadder(pvc)** - 检测是否在梯子上
- **IsOnRope(pvc)** - 检测是否在绳子上
- **IsSwimming(pvc)** - 检测是否在游泳
- **IsStopped(pvc)** - 检测是否停止移动
- **IsPermitMapFlyingSkill** (地址: 0x8e4090) - 检测是否允许飞行技能

#### CVecCtrlUser::IsPermitMapFlyingSkill 分析：
```cpp
BOOL CVecCtrlUser::IsPermitMapFlyingSkill(CVecCtrlUser *this) {
    CAttrField *m_pAttrField = this->m_pAttrField;
    if (m_pAttrField) {
        if (TSecType<long>::GetData(&m_pAttrField->nMapType) == 2) {
            if (this->m_pAttrField->bNeedSkillForFlying) {
                return 1;
            }
        }
    }
    return 0;
}
```

### 3. CUser 移动动作解析
**OnResolveMoveAction** (地址: 0x8e5800) 是核心的移动状态处理函数。

#### 关键移动状态码：
- **状态 1**: 地面行走
- **状态 2**: 静止站立
- **状态 3**: 空中状态
- **状态 5**: 向上跳跃
- **状态 6**: 游泳
- **状态 7**: 攀爬梯子
- **状态 8**: 攀爬绳子
- **状态 18**: 空中飞行
- **状态 19**: 冲刺技能
- **状态 20**: 火箭靴推进

#### 梯子/绳子状态处理逻辑：
```cpp
if (CVecCtrl::IsOnLadder(pvc)) {
    // 特殊技能检查 (1902040, 1902041, 1902042)
    if (m_bForcedInvisible != 1902040 && 
        m_bForcedInvisible != 1902041 && 
        m_bForcedInvisible != 1902042) {
        v12 = 7;  // 梯子攀爬状态
    } else {
        v12 = 2 - (v6 != 0);  // 特殊情况处理
    }
} else if (CVecCtrl::IsOnRope(pvc)) {
    if (!CAvatar::IsRidingEvanDragon()) {
        v12 = 8;  // 绳子攀爬状态
    } else {
        v12 = 2 - (v6 != 0);  // 骑龙状态特殊处理
    }
}
```

## 物理系统分析

### 1. 地面检测系统
- 使用 **TRSTree** 数据结构进行高效的空间查询
- **CStaticFoothold** 对象表示可站立的地面/平台
- 支持区域查询和精确的碰撞检测

### 2. 梯子/绳子检测机制
基于现有代码分析，梯子/绳子的检测应该包含以下机制：

#### 可能的检测范围：
- 梯子/绳子应该有特定的碰撞区域定义
- 角色必须在特定的 X 坐标范围内才能攀爬
- 垂直方向的检测确保角色能够"抓住"梯子/绳子

#### 对齐机制：
- 当角色开始攀爬时，会自动对齐到梯子/绳子的中心 X 坐标
- 这可能在 **CVecCtrl::SetMovePathAttribute** 中实现

### 3. 从梯子跳跃的物理计算
在 **OnResolveMoveAction** 中发现了梯子跳跃的特殊处理：

```cpp
if (CAvatar::GetOneTimeAction() == 207 && (v12 == 7 || v12 == 8)) {
    // 梯子/绳子跳跃的特殊效果处理
    this->m_nMoveAction = -1;
    this->m_bTamingMobTired = -1;
    CAvatar::ClearActionLayer();
}
```

## 技术实现细节

### 1. 数据结构
- **TRSTree**: 用于高效空间查询的 R-tree 变体
- **ZMap**: 基于键值的映射容器
- **ZRef**: 引用计数智能指针系统

### 2. 状态管理
- 使用状态机模式管理角色的移动状态
- 每个状态都有对应的动画和物理参数
- 状态转换基于输入和物理检测结果

### 3. 坐标系统
- 使用整数坐标系统
- 支持区域查询 (x1, y1, x2, y2)
- 梯子/绳子的位置信息存储在地图数据中

## 潜在的实现细节

### 1. 梯子/绳子数据结构
虽然没有直接找到梯子/绳子的具体数据结构，但基于系统架构推测：

```cpp
struct LadderRopeInfo {
    int nX;          // X 坐标
    int nY1, nY2;    // 垂直范围
    int nType;       // 类型 (梯子/绳子)
    int nPage;       // 所属页面
    // 其他属性...
};
```

### 2. 攀爬物理计算
- 垂直移动速度可能是固定的
- 水平位置锁定在梯子/绳子的中心
- 攀爬过程中可能有特殊的重力处理

### 3. 跳跃计算
- 从梯子/绳子跳跃时，初始速度可能基于输入方向
- 跳跃的水平速度和垂直速度可能有特定的公式

## 关键函数地址总结

| 函数名 | 地址 | 功能 |
|--------|------|------|
| CWvsPhysicalSpace2D::GetCrossCandidate | 0x516610 | 获取碰撞候选对象 |
| CWvsPhysicalSpace2D::GetFoothold | 0x517d10 | 获取 foothold 对象 |
| CVecCtrlUser::IsPermitMapFlyingSkill | 0x8e4090 | 检测飞行技能权限 |
| CUser::OnResolveMoveAction | 0x8e5800 | 处理移动动作 |

## 结论

MapleStory 的梯子/绳子系统是一个复杂的物理模拟系统，涉及：

1. **空间索引**: 使用 TRSTree 进行高效的空间查询
2. **状态管理**: 基于状态机的移动控制系统
3. **物理检测**: 精确的碰撞检测和位置计算
4. **动画系统**: 与移动状态同步的动画播放

虽然没有找到具体的 `GetLadderOrRope` 和 `IsAbleToClimbLadderOrRope` 函数，但通过 `IsOnLadder` 和 `IsOnRope` 函数可以看出系统的基本架构。梯子/绳子的具体实现可能在更底层的物理检测模块中，需要进一步的分析才能完全理解其机制。

## 建议进一步分析的方向

1. 查找梯子/绳子的具体数据结构定义
2. 分析 `CVecCtrl::SetMovePathAttribute` 函数的实现
3. 研究地图数据的解析和梯子/绳子信息的存储方式
4. 分析攀爬状态下的物理计算公式