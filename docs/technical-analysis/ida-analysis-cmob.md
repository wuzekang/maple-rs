# CMob 类分析报告

## 概述
CMob 是游戏中怪物(Monster/Mob)的核心类，继承自多个接口：
- ZRefCounted (引用计数)
- IVecCtrlOwner (向量控制所有者)
- IGObj (游戏对象接口)

## 虚函数表
CMob 类有三个虚函数表：
1. `??_7CMob@@6BZRefCounted@@@` (0xb51ffc) - ZRefCounted 接口
2. `??_7CMob@@6BIVecCtrlOwner@@@` (0xb52000) - IVecCtrlOwner 接口
3. `??_7CMob@@6BIGObj@@@` (0xb52024) - IGObj 接口

## 核心方法

### 1. 更新和AI相关
- **Update** (0x654300) - 主更新函数，大小 0x1abc
  - 虚函数，位于 IGObj 虚函数表中
  - 负责怪物的整体更新逻辑

### 2. 追击和目标相关
- **ChaseTarget** (0x642db0) - 追击目标
  - 参数：目标类型、IVecCtrlOwner*、未知参数
  - 大小：0x135
  
- **IsChaseTargetEscort** (0x63b7b0) - 检查是否为护送追击目标
  - 大小：0x7e
  
- **IsChaseTargetDazzle** (0x63b830) - 检查是否为眩晕追击目标
  - 大小：0x6a

### 3. 攻击范围检测
- **IsTargetInAttackRange** (0x645f50) - 检查目标是否在攻击范围内
  - 参数：包含 TARGETINFO 结构
  - 大小：0xb72 (相当大的函数)

### 4. 移动相关
- **OnResolveMoveAction** (0x63caf0) - 处理移动动作
  - 虚函数
  - 参数：四个长整型和 CVecCtrl*
  - 大小：0x1ea

- **OnLayerZChanged** (0x63b470) - 层级Z改变时的处理
  - 虚函数
  - 参数：CVecCtrl*
  - 大小：0x22

### 5. 基本属性获取
- **GetType** (0x64cf70) - 获取类型
- **GetPos** (0x64cff0) - 获取位置
- **GetPosPrev** (0x64d020) - 获取上一个位置
- **GetZMass** (0x64cfa0) - 获取Z轴质量
- **GetShoeAttr** (0x640c20) - 获取鞋子属性

### 6. RTTI相关
- **GetRTTI** (0x64cf80) - 获取运行时类型信息
- **IsKindOf** (0x64cfc0) - 检查是否为特定类型
- RTTI信息位于：0xc56eb4

## CVecCtrlMob 类 (怪物移动控制)

### 移动控制相关方法
- **SetActive** (0x9987f0) - 设置活动状态
  - 参数：多个坐标参数和 CStaticFoothold*
  - 大小：0x2d8

- **WorkUpdateActive** (0x99d450) - 活动状态工作更新
  - 大小：0x8d1

- **InspectUpdateActive** (0x9996f0) - 检查活动状态更新
  - 大小：0x1e5

- **CtrlUpdateActiveMove** (0x99d090) - 控制活动移动更新
  - 大小：0x3b9

- **CtrlUpdateActiveStop** (0x99b6a0) - 控制活动停止更新
  - 大小：0x221

- **ChaseTargetImp** (0x998ad0) - 追击目标实现
  - 参数：类似 CMob::ChaseTarget
  - 大小：0x1aa

- **CollisionDetectEscortDest** (0x99773c) - 护送目标碰撞检测
  - 大小：0x474

- **IsAbleToClimbLadderOrRope** (0x996e10) - 检查是否能爬梯子或绳子
  - 大小：0x57

- **IsCheatMobMoveRandImp** (0x99a860) - 检查是否为作弊随机移动
  - 大小：0xb3b

## 数据结构
CMob 类使用了多个内部数据结构：
- AFFECTEDSKILLENTRY - 受影响技能条目
- ATTACKENTRY - 攻击条目
- DAMAGEINFO - 伤害信息
- HITEFFECT - 击中效果
- DROPPICKUP - 掉落拾取
- ReservedPacket - 保留数据包
- MobBullet - 怪物子弹
- TARGETINFO - 目标信息

## 相关类
- CMobTemplate (0xb520c8) - 怪物模板类
- CMobPool (0xb52088) - 怪物对象池
- CVecCtrlMob - 怪物向量控制器

## 总结
CMob 类是一个复杂的游戏对象类，负责怪物的所有行为，包括：
1. AI逻辑和目标追击
2. 攻击范围检测和攻击行为
3. 移动控制和碰撞检测
4. 状态更新和动画控制
5. 与其他游戏系统的交互

Update 函数(0x1abc字节)是整个类的核心，协调所有子系统的运行。CVecCtrlMob 类专门处理移动相关的逻辑，实现了复杂的寻路和移动控制。