# MapleStory 怪物移动系统分析

## 概述
本文档整合了MapleStory客户端中怪物移动系统的完整分析，包括移动能力分类、随机移动机制、目标追击系统以及各种移动模式的详细实现。

## 系统架构概览

### 核心类层次结构
```
CMob (0x654300) - 怪物核心类
├── ZRefCounted - 引用计数
├── IVecCtrlOwner - 向量控制所有者  
├── IGObj - 游戏对象接口
└── CVecCtrlMob - 怪物移动控制器
    ├── CVecCtrl - 基础向量控制
    └── 移动能力专门处理函数
```

### 虚函数表
- `??_7CMob@@6BZRefCounted@@@` (0xb51ffc) - ZRefCounted 接口
- `??_7CMob@@6BIVecCtrlOwner@@@` (0xb52000) - IVecCtrlOwner 接口  
- `??_7CMob@@6BIGObj@@@` (0xb52024) - IGObj 接口

## 移动能力系统 (nMoveAbility)

### 移动能力分类
基于 `CVecCtrlMob::WorkUpdateActive` (0x99d450) 中的分派逻辑：

| 数值 | 含义 | 对应函数 | 地址 | 大小 | 描述 |
|------|------|----------|------|------|------|
| 0 | 静止模式 | `CtrlUpdateActiveStop` | 0x99b6a0 | 0x221 | 怪物保持静止，可转身 |
| 1 | 行走模式 | `CtrlUpdateActiveMove` | 0x99d090 | 0x3b9 | 地面行走移动 |
| 3 | 跳跃模式 | `CtrlUpdateActiveJump` | 0x99b930 | 未知 | 可跳跃越过障碍 |
| 4 | 飞行模式 | `CtrlUpdateActiveFly` | 0x99beb0 | 未知 | 空中自由飞行 |
| 6 | 护送模式 | `CtrlUpdateActiveEscort` | 0x99b8d0 | 未知 | 预定路径移动 |

### 移动能力调度机制
**地址**: 0x99d450 (WorkUpdateActive)
```cpp
switch (this->m_nMoveAbility) {
    case 0: CVecCtrlMob::CtrlUpdateActiveStop(this); break;
    case 1: CVecCtrlMob::CtrlUpdateActiveMove(this); break;
    case 3: CVecCtrlMob::CtrlUpdateActiveJump(this); break;
    case 4: 
        CVecCtrlMob::FlyCtrlGuardingBefore(this);
        CVecCtrlMob::CtrlUpdateActiveFly(this); 
        break;
    case 6: CVecCtrlMob::CtrlUpdateActiveEscort(this); break;
}
```

## 行走型怪物随机移动机制 (nMoveAbility = 1)

### 核心函数
**CVecCtrlMob::CtrlUpdateActiveMove** (0x99d090, 大小: 0x3b9)

### 数据结构
```cpp
// 随机移动管理器 (无目标状态)
struct MoveRndMan_Direction {
    bool m_bGenerated;          // 是否已生成随机数
    unsigned short m_usRandCnt; // 随机计数器
    CRand32 m_randMove;        // 随机数生成器
    int m_nLastRnd;            // 上次随机值
    bool m_bPassed;            // 是否通过验证
};

// 移动上下文
struct MoveContext_WC {
    TSecType<long> nCount;     // 移动持续时间计数
    TSecType<long> lInputX;    // X轴输入方向 (-1/0/1)
};
```

### 双模式移动系统

#### A. 无目标时的随机移动
**地址**: 0x99d34e (CtrlUpdateActiveMove)
```cpp
// 时间生成 (1000-2000ms，以90ms为单位)
v10 = CRand32::Random(&this->m_moveRndMan_Direction.m_randMove);
TSecType<long>::SetData(this->m_moveCtx.wc.nCount, (v10 % 0x7D0 + 1000) / 0x5A);

// 方向生成 (三向等概率)
v11 = CRand32::Random(&this->m_moveRndMan_Direction.m_randMove);
TSecType<long>::SetData(this->m_moveCtx.wc.lInputX, v11 % 3 - 1);
```
- **时间范围**: 1000-2000毫秒
- **时间单位**: 90毫秒 (0x5A)
- **实际单位**: 11-22个时间单位
- **方向算法**: `random % 3 - 1` → -1(左), 0(停), 1(右)
- **概率分布**: 三种状态等概率 (33.3%)

#### B. 有目标时的智能追击
**地址**: 0x99d153 (CtrlUpdateActiveMove)
```cpp
// 动态时间调整
v2 = CRand32::Random(&this->m_moveRndMan.m_randMove);
TSecType<long>::SetData(this->m_moveCtx.wc.nCount, (v2 + 400 * (2 - v2 / 0x190)) / 0x5A);

// 方向计算
double current_x = _ZtlSecureFuse<double>((int)&this->m_ap, this->m_ap._ZtlSecureTear_x_CS);
double target_x = GetTargetPos();

if (fabs(target_x - current_x) < 100.0) {
    // 距离太近，边界检测
    if (current_x == m_rcBound.left)
        direction = 1;  // 右移
    else if (current_x == m_rcBound.right)
        direction = -1; // 左移
} else {
    // 朝向目标移动
    direction = (target_x >= current_x) ? 1 : -1;
}
```
- **基础时间**: 400毫秒
- **计算公式**: `(random + 400 * (2 - random / 400)) / 90`
- **距离阈值**: 100像素
- **边界处理**: 自动规避边界碰撞

### 状态切换逻辑

#### 1. 移动状态检查
**地址**: 0x99d0fc (CtrlUpdateActiveMove)
```cpp
if (TSecType<long>::GetData(this->m_moveCtx.wc.nCount) > 0) {
    // 继续当前移动，减少计数
    TSecType<long>::SetData(nCount, count - 1);
    return;
}
```

#### 2. 目标存在性判断
**地址**: 0x99d109 (CtrlUpdateActiveMove)
```cpp
if (!this->m_pTarget) {
    // 进入随机移动模式
    this->m_moveRndMan_Direction.m_bGenerated = 1;
    generateRandomMovement();
} else {
    // 进入目标追击模式
    chaseTargetLogic();
}
```

#### 3. 目标类型验证
**地址**: 0x99d19c (CtrlUpdateActiveMove)
```cpp
if (this->m_pTarget->vfptr->IsKindOf(this->m_pTarget, &CUser::ms_RTTI_CUser) ||
    this->m_pTarget->vfptr->IsKindOf(this->m_pTarget, &CMob::ms_RTTI_CMob) ||
    this->m_pTarget->vfptr->IsKindOf(this->m_pTarget, &CSummoned::ms_RTTI_CSummoned)) {
    // 有效目标，执行追击逻辑
} else {
    // 无效目标处理
    ZXString<char>::Format(&errmsg, "R6025 %d %d %d %d", ...);
    save_error_log(&errmsg);
    CVecCtrlMob::ChaseTarget(this, 0, 0, 0);
}
```

## 飞行型怪物移动机制 (nMoveAbility = 4)

### 核心函数
- **CVecCtrlMob::CtrlUpdateActiveFly** (0x99beb0) - 飞行控制主函数
- **CVecCtrlMob::FlyCtrlGuardingBefore** (0x999ec0) - 飞行前准备

### 飞行数据结构
```cpp
// 飞行上下文
struct FlyContext {
    TSecType<int> bTarget;              // 是否有飞行目标
    SECPOINT *ptTarget;                 // 目标位置坐标
    TSecType<long> nRemain;            // 剩余飞行时间
    TSecType<long> nZMass;             // 当前Z区域索引
};

// 状态检查变量
bool m_bNeedForCheckFlyTargetInternal_A;  // 内部检查标志A
bool m_bNeedForCheckFlyTarget_A;          // 目标检查标志A  
bool m_bNeedForCheckFlyTargetInternal_B;  // 内部检查标志B
bool m_bNeedForCheckFlyTarget_B;          // 目标检查标志B
bool m_bABHackCheckNeed;                  // 反作弊检查标志
```

### 双模式飞行系统

#### A. 有敌对目标时的围绕飞行
**地址**: 0x99c17a (CtrlUpdateActiveFly)
```cpp
if (this->m_pTarget) {
    // 设置追击持续时间 (60-120帧)
    if (!nRemain) {
        v19 = CRand32::Random(&this->m_moveRndMan.m_randMove);
        nRemain = v19 % 0x3C + 60;  // 60-120帧
    }
    
    // 生成X轴偏移 (-120 到 +120)
    v23 = CRand32::Random(&this->m_moveRndManForPOS_X_A.m_randMove);
    x_offset = v23 % 0xF0;  // 0-240
    target_x = target_pos.x + x_offset - 120;  // -120 到 +120偏移
    
    // 生成Y轴偏移 (-40 到 +40)
    v27 = CRand32::Random(&this->m_moveRndManForPOS_Y_A.m_randMove);
    y_offset = v27 % 0x50;  // 0-80
    target_y = target_pos.y + y_offset - 40;   // -40 到 +40偏移
}
```

#### B. 无目标时的地图巡逻
**地址**: 0x99c375 (CtrlUpdateActiveFly)
```cpp
if (!this->m_pTarget) {
    // 1. 随机选择Z区域
    if (nRemain <= 0 || nZMass < 0) {
        mass_count = v38->m_aIndexZMass.count;
        v41 = CRand32::Random(&this->m_moveRndMan_Mass.m_randMove);
        selected_mass = v38->m_aIndexZMass.a[v41 % mass_count];
        
        // 设置巡逻持续时间
        mass_range = &v38->m_aMassRange.a[selected_mass];
        v43 = CRand32::Random(&this->m_moveRndMan.m_randMove);
        nRemain = (mass_range->high - 200 * (v43 / 0xC8) - mass_range->low + v43 + 40) >> 3;
    }
    
    // 2. 在选定区域随机选择立足点
    foothold_list = &v38->m_aaMassFootholdList.a[nZMass];
    v45 = CRand32::Random(&this->m_moveRndManForFH.m_randMove);
    selected_fh = foothold_list->a[v45 % foothold_list->count];
    
    // 3. 在立足点上随机选择位置
    foothold = CWvsPhysicalSpace2D::GetFoothold(v38, selected_fh);
    v47 = CRand32::Random(&this->m_moveRndManForPOS.m_randMove);
    pos_on_fh = v47 % foothold->length;
    
    // 4. 计算最终飞行目标位置
    target_x = pos_on_fh * foothold->uvx + foothold->x1;
    
    // 5. 添加Y轴随机偏移 (-50 到 +10)
    v49 = CRand32::Random(&this->m_moveRndManForTargetY.m_randMove);
    y_random = v49 % 0x3C;  // 0-60
    target_y = pos_on_fh * foothold->uvy + foothold->y1 + y_random - 50;
}
```

### 飞行目标控制

#### A. 目标到达检测
**地址**: 0x99bec5 (CtrlUpdateActiveFly)
```cpp
// 10像素精度控制
if (TSecType<int>::GetData(this->m_moveCtx.fc.bTarget)) {
    // X轴距离检测 (10像素容差)
    x_diff = fabs(target_x - current_x);
    if (x_diff >= 10.0) {
        input_x = (current_x <= target_x) ? 1 : -1;  // 朝向目标
    } else {
        input_x = 0;  // 停止X轴移动
    }
    
    // Y轴距离检测 (10像素容差)
    y_diff = fabs(target_y - current_y);
    if (y_diff >= 10.0) {
        if (current_y > target_y) {
            CVecCtrl::Jump(this);  // 向上飞行
        }
        CVecCtrl::SetInput(this, input_x, 0);
    } else {
        CVecCtrl::SetInput(this, input_x, -1);  // 向下飞行
    }
}
```

#### B. 区域边界控制
**地址**: 0x99c010 (CtrlUpdateActiveFly)
```cpp
// 定义内外两个控制区域
tagRECT rcAreaIn = {-11, -11, 11, 11};      // 内圈: 22x22像素
tagRECT rcAreaOut = {-150, -150, 150, 150}; // 外圈: 300x300像素

// 以目标位置为中心偏移区域
ZAPI.OffsetRect(&rcAreaIn, target_x, target_y);
ZAPI.OffsetRect(&rcAreaOut, target_x, target_y);

// 到达内圈或离开外圈时重新生成目标
if (ZAPI.PtInRect(&rcAreaIn, current_pos) ||
    (this->m_pTarget && !ZAPI.PtInRect(&rcAreaOut, enemy_pos))) {
    TSecType<int>::SetData(this->m_moveCtx.fc.bTarget, 0);  // 清除当前目标
}
```

### 随机数生成器管理
飞行型怪物使用**5个独立随机数生成器**：

```cpp
// 围绕敌对目标飞行时使用
CRand32 m_moveRndManForPOS_X_A;     // X轴偏移随机数
CRand32 m_moveRndManForPOS_Y_A;     // Y轴偏移随机数

// 地图巡逻时使用  
CRand32 m_moveRndMan_Mass;          // 区域选择随机数
CRand32 m_moveRndManForFH;          // 立足点选择随机数
CRand32 m_moveRndManForPOS;         // 位置选择随机数
CRand32 m_moveRndManForTargetY;     // Y轴偏移随机数

// 通用时间控制
CRand32 m_moveRndMan;               // 持续时间随机数
```

## 目标追击系统

### 核心函数
- **CMob::ChaseTarget** (0x642db0, 大小: 0x135) - 高层追击控制
- **CVecCtrlMob::ChaseTargetImp** (0x998ad0, 大小: 0x1aa) - 底层追击实现

### 追击逻辑
**地址**: 0x642db0 (CMob::ChaseTarget)
```cpp
void CMob::ChaseTarget(CMob *this, int bChase, IVecCtrlOwner *pTarget, int bForced) {
    if (CMob::IsActive(this)) {
        // 设置追击状态
        TSecType<int>::SetData(&this->m_bChasing, bChase);
        
        // 获取移动控制器
        m_pInterface = this->m_pvcActive.m_pInterface;
        if (m_pInterface)
            v10 = (CVecCtrlMob *)&m_pInterface[-3];
        
        // 同步追击状态到移动控制器
        TSecType<int>::SetData(&v10->m_bChasing, bChase);
        CVecCtrlMob::ChaseTargetImp(v10, bChase, pTarget, bForced);
        
        // 记录目标怪物ID
        if (bChase && pTarget && pTarget->GetType() == 1 && 
            pTarget->IsKindOf(&CMob::ms_RTTI_CMob)) {
            this->m_dwTargetMobID = CMob::GetMobID((CMob *)&pTarget[-1]);
        }
        
        CMob::SetShoeAttr(this);
    }
}
```

### 追击时间控制
**地址**: 0x99d726 (WorkUpdateActive)
```cpp
// 追击持续时间管理
if (!TSecType<int>::GetData(&this->m_bForcedChase)) {
    v12 = TSecType<long>::GetData(&this->m_tChaseDuration) - tElapse;
    TSecType<long>::SetData(&this->m_tChaseDuration, v12);
    if (v12 < 0) {
        // 追击时间结束，停止追击
        TSecType<int>::SetData(&this->m_bChasing, 0);
        CVecCtrlMob::ChaseTargetImp(this, 0, 0, 0);
    }
}
```

### 攻击范围检测
- **IsTargetInAttackRange** (0x645f50, 大小: 0xb72) - 检查目标是否在攻击范围内
- **IsChaseTargetEscort** (0x63b7b0, 大小: 0x7e) - 检查是否为护送追击目标
- **IsChaseTargetDazzle** (0x63b830, 大小: 0x6a) - 检查是否为眩晕追击目标

## 边界检测与碰撞处理

### 移动边界限制
**地址**: 0x99d2c3 (CtrlUpdateActiveMove)
```cpp
// 边界检测和方向调整
if ((double)this->m_rcBound.left == current_x) {
    TSecType<long>::SetData(this->m_moveCtx.wc.lInputX, 1);  // 强制右移
} else if ((double)this->m_rcBound.right == current_x) {
    TSecType<long>::SetData(this->m_moveCtx.wc.lInputX, -1); // 强制左移
}
```

### 梯子/绳索处理
- **IsAbleToClimbLadderOrRope** (0x996e10, 大小: 0x57) - 检查是否能爬梯子或绳子
- **CollisionDetectEscortDest** (0x99773c, 大小: 0x474) - 护送目标碰撞检测

## 移动动作解析系统

### 核心函数
**CMob::OnResolveMoveAction** (0x63caf0, 大小: 0x1ea)

### 功能概述
将输入状态(`nInputX`, `nInputY`)和当前移动动作(`nCurMoveAction`)转换为标准化的移动动作编码，该编码用于网络同步和动画控制。

### 动作编码规则

#### 编码格式分析
```cpp
// 基础编码结构 (从反编译代码推导)
// Bit 0: 方向位 (0=右, 1=左)
// Bit 1: 保留
// Bit 2: 状态位 (0=移动, 1=静止)
// Bit 3-5: 移动类型 (000=地面, 001=跳跃, 011=飞行, 100=梯子)

// 常见编码值
// 0x04 (100): 静止状态，面向右
// 0x05 (101): 静止状态，面向左  
// 0x02 (010): 地面移动，向右
// 0x03 (011): 地面移动，向左
// 0x06 (110): 飞行状态，向右
// 0x07 (111): 飞行状态，向左
// 0x10 (10000): 梯子/绳子状态
```

### 分移动能力的动作解析

#### 静止型 (nMoveAbility = 0)
```cpp
case 0:
    if (!nInputX) {
        // 无输入，保持当前朝向
        return nCurMoveAction & 1 | 4;  // 保留方向位 + 静止标志
    } else {
        // 有输入，只改变朝向不移动  
        return (nInputX < 0) | 4;       // 新方向 + 静止标志
    }
```

#### 行走型 (nMoveAbility = 1)
```cpp
case 1:
    if (!nInputX) {
        // 无输入，静止
        return nCurMoveAction & 1 | 4;  // 保留方向 + 静止标志
    } else {
        // 有输入，地面移动
        int move_type = (v8 != 0) ? 16 : 1;  // v8检查特殊状态
        return (nInputX < 0) | (2 * move_type);
    }
```

#### 跳跃型 (nMoveAbility = 3)
```cpp
case 3:
    if (!pvc->m_pfh) {
        // 无立足点，按飞行模式处理
        goto LABEL_34;  // 飞行编码逻辑
    }
    
    if (!nInputX) {
        // 无输入，静止
        return nCurMoveAction & 1 | 4;
    } else {
        // 有输入，跳跃移动
        int move_type = (v8 != 0) ? 16 : 1;
        return (nInputX < 0) | (2 * move_type);
    }
```

#### 飞行型 (nMoveAbility = 4)
```cpp
case 4:
    if (!nInputX) {
        // 无输入，飞行悬停
        int move_type = (v8 != 0) ? 16 : 6;  // 6=飞行类型
        return nCurMoveAction & 1 | (2 * move_type);
    } else {
        // 有输入，飞行移动
        int move_type = (v8 != 0) ? 16 : 6;
        return (nInputX < 0) | (2 * move_type);
    }
```

#### 护送型 (nMoveAbility = 6)
```cpp
case 6:
    if (pvc->m_pfh) {
        // 在立足点上
        if (nInputX) {
            return (nInputX < 0) | 2;      // 地面移动编码
        } else {
            return nCurMoveAction & 1 | 4; // 静止编码
        }
    } 
    else if (CVecCtrl::IsOnLadder(pvc) || CVecCtrl::IsOnRope(pvc)) {
        // 在梯子/绳子上
        if (nInputX) {
            return (nInputX < 0) | 0x10;   // 梯子移动编码
        } else {
            return nCurMoveAction & 1 | 0x10; // 梯子静止编码
        }
    } 
    else {
        // 空中状态，按飞行处理
        if (nInputX) {
            return (nInputX < 0) | 6;      // 飞行移动编码
        } else {
            return nCurMoveAction & 1 | 6; // 飞行悬停编码
        }
    }
```

### 特殊状态检查 (v8变量)
```cpp
// v8变量的计算逻辑
if (this->m_stat.tMGuardUp_ && this->m_pTemplate) {
    // 检查模板数据的特殊标志
    if (template_data[516]) {  // 偏移516处的标志位
        v8 = 1;  // 特殊状态激活
    }
}

// v8影响移动类型编码
// v8 = 0: 使用标准移动类型 (1, 6)
// v8 = 1: 使用特殊移动类型 (16) - 可能是反作弊或特殊效果
```

### 动作编码映射表

| 移动能力 | 输入状态 | 立足点状态 | 编码结果 | 含义 |
|----------|----------|------------|----------|------|
| 0 (静止) | 无输入 | - | `cur&1|4` | 保持朝向静止 |
| 0 (静止) | 有输入 | - | `(x<0)|4` | 改变朝向静止 |
| 1 (行走) | 无输入 | - | `cur&1|4` | 地面静止 |
| 1 (行走) | 有输入 | - | `(x<0)|2` | 地面移动 |
| 3 (跳跃) | 无立足点 | 无 | `(x<0)|6` | 空中飞行 |
| 3 (跳跃) | 有立足点 | 有 | `(x<0)|2` | 地面跳跃 |
| 4 (飞行) | 任意 | - | `(x<0)|12` | 飞行移动 |
| 6 (护送) | 立足点 | 有 | `(x<0)|2` | 地面移动 |
| 6 (护送) | 梯子/绳子 | 特殊 | `(x<0)|16` | 垂直移动 |
| 6 (护送) | 空中 | 无 | `(x<0)|6` | 飞行移动 |

### 网络同步意义

该编码系统为MapleStory的网络同步提供了统一的移动状态表示：

1. **压缩性**: 用单个整数表示复杂的移动状态
2. **兼容性**: 不同移动能力映射到统一的编码空间  
3. **扩展性**: 预留位支持未来的移动类型
4. **效率性**: 客户端可直接使用编码驱动动画和物理

## 其他移动模式

### 静止模式 (nMoveAbility = 0)
**函数**: `CVecCtrlMob::CtrlUpdateActiveStop` (0x99b6a0, 大小: 0x221)

特点：
- 怪物不主动移动
- 会转身面向目标
- 检查攻击范围
- 可能触发追击行为

### 跳跃模式 (nMoveAbility = 3)
**函数**: `CVecCtrlMob::CtrlUpdateActiveJump`

### 跳跃系统核心数据结构
```cpp
// 跳跃移动上下文
struct JumpContext {
    TSecType<int> bWantToJumpUp;        // 跳跃请求标志
    TSecType<long> nYBeforeJump;        // 跳跃前Y坐标
    TSecType<long> nCount;              // 移动持续时间计数
    TSecType<long> lInputX;             // X轴输入方向
};

// 跳跃相关成员
CStaticFoothold* m_pfhLandingNext;      // 下一个着陆立足点
int m_nCheckTarget;                     // 目标检查标志
```

### 跳跃模式五层控制逻辑

#### A. 着陆检查优先级控制
```cpp
if (this->m_pfhLandingNext.p) {
    // 已有预定着陆点，停止移动等待着陆
    CVecCtrl::SetInput(this, 0, 0);
    this->m_bPassedUpdateCtrl = 1;
    return;
}
```

#### B. 立足点状态验证
```cpp
if (!this->m_pfh) {
    // 无立足点，直接进入时间计数减少逻辑
    goto LABEL_49;  // 跳转到计数减少
}
```

#### C. 跳跃失败检测与修正
```cpp
if (TSecType<int>::GetData(this->m_moveCtx.jc.bWantToJumpUp)) {
    TSecType<int>::SetData(this->m_moveCtx.jc.bWantToJumpUp, 0);
    
    if (CVecCtrl::_ZtlSecureGet_m_nInputX(this)) {
        // 检查是否原地跳跃失败 (Y坐标未变化)
        int y_before_jump = TSecType<long>::GetData(this->m_moveCtx.jc.nYBeforeJump);
        double current_y = AbsPos::_ZtlSecureGet_y(&this->m_ap);
        
        if (tolong(current_y) == y_before_jump) {
            // 跳跃失败，反向移动防止卡死
            int current_input_x = CVecCtrl::_ZtlSecureGet_m_nInputX(this);
            CVecCtrl::SetInput(this, -current_input_x, 0);
        }
    }
}
```

#### D. 智能前方立足点分析
```cpp
// 获取前方立足点信息
double current_pos = GetCurrentRelativePos();
int input_x = CVecCtrl::_ZtlSecureGet_m_nInputX(this);
CStaticFoothold* forward_fh = CStaticFoothold::GetForwardLink(
    this->m_pfh, (double)input_x, current_pos, 40.0);

if (forward_fh && input_x) {
    // 检查前方立足点是否为下坡 (uvx <= 0)
    if (forward_fh->m_uvx <= 0.0) {
        double forward_slope_y = forward_fh->m_uvy;
        
        // 检查移动方向与坡度是否冲突 (input_x * slope_y < 0)
        if ((double)input_x * forward_slope_y < 0.0) {
            // 获取下下个立足点进行双重检查
            uint next_fh_id = (input_x >= 0) ? this->m_pfh->m_dwSNNext : this->m_pfh->m_dwSNPrev;
            
            if (next_fh_id) {
                CStaticFoothold* next_fh = GetFootholdById(next_fh_id);
                if (next_fh->m_uvx <= 0.0 && 
                    (double)input_x * next_fh->m_uvy < 0.0) {
                    
                    // 检查是否接近立足点边缘 (距离边缘1像素内)
                    bool near_edge = (input_x < 0 && current_pos < 1.0) ||
                                    (input_x > 0 && (this->m_pfh->m_len - 1.0) < current_pos);
                    
                    if (near_edge) {
                        // 边缘反向移动
                        CVecCtrl::SetInput(this, -input_x, 0);
                    } else if (TSecType<long>::GetData(this->m_moveCtx.jc.nCount) > 0) {
                        // 非边缘且有移动时间，执行跳跃
                        CVecCtrlMob::SetJumpNext(this, 1);
                        goto LABEL_49;
                    }
                }
            }
        }
    }
}
```

#### E. 目标导向移动控制

##### E1. 无目标时的随机移动
```cpp
if (!this->m_pTarget) {
    if (TSecType<long>::GetData(this->m_moveCtx.jc.nCount) <= 0) {
        uint random_action = CRand32::Random(&this->m_moveRndMan.m_randMove);
        ++this->m_moveRndMan.m_usRandCnt;
        
        if (random_action % 5) {  // 80%概率执行移动
            // 生成移动持续时间 (1000-2000ms，90ms单位)
            uint time_random = CRand32::Random(&this->m_moveRndMan.m_randMove);
            ++this->m_moveRndMan.m_usRandCnt;
            TSecType<long>::SetData(this->m_moveCtx.jc.nCount, 
                (time_random % 2000 + 1000) / 90);
            
            // 生成移动方向 (-1/0/1)
            uint dir_random = CRand32::Random(&this->m_moveRndMan.m_randMove);
            ++this->m_moveRndMan.m_usRandCnt;
            this->m_moveRndMan.m_nLastRnd = dir_random;
            this->m_moveRndMan.m_bPassed = 1;
            TSecType<long>::SetData(this->m_moveCtx.jc.lInputX, dir_random % 3 - 1);
            
            int input_direction = TSecType<long>::GetData(this->m_moveCtx.jc.lInputX);
            CVecCtrl::SetInput(this, input_direction, 0);
            this->m_nCheckTarget = input_direction;
            
            // 同步随机计数器
            this->m_path.m_usRandCnt = this->m_moveRndMan.m_usRandCnt;
        } else {  // 20%概率执行跳跃
            CVecCtrl::Jump(this);
        }
        this->m_path.m_usActualRandCnt = this->m_moveRndMan.m_usRandCnt;
    }
    goto LABEL_49;
}
```

##### E2. 有目标时的智能追击
```cpp
if (this->m_pTarget) {
    if (TSecType<long>::GetData(this->m_moveCtx.jc.nCount) <= 0) {
        // 跳跃条件检查
        int my_zmass = this->m_pfh->m_lZMass;
        int target_zmass = this->m_pTarget->GetZMass();
        SECPOINT target_pos = this->m_pTarget->GetPos();
        double my_y = AbsPos::_ZtlSecureGet_y(&this->m_ap);
        double my_x = GetCurrentPosX();
        
        // 检查是否需要跳跃到不同Z层
        bool need_jump = (my_zmass != target_zmass) &&     // 不同Z层
                        (my_y <= target_pos.y) &&           // 目标在上方
                        IsInZMassRange(target_zmass, my_x);  // 在目标Z层范围内
        
        if (need_jump) {
            uint jump_chance = CRand32::Random(&this->m_moveRndMan.m_randMove);
            ++this->m_moveRndMan.m_usRandCnt;
            
            if (jump_chance % 3 == 0) {  // 33%概率执行跳跃
                CVecCtrlMob::ClearMoveContext(this);
                CVecCtrlMob::SetJumpNext(this, 1);
                goto LABEL_49;
            }
        }
        
        // 标准追击移动逻辑
        uint time_random = CRand32::Random(&this->m_moveRndMan.m_randMove);
        ++this->m_moveRndMan.m_usRandCnt;
        
        // 动态时间调整公式: (random + 400 * (2 - random/400)) / 90
        TSecType<long>::SetData(this->m_moveCtx.jc.nCount, 
            (time_random + 400 * (2 - time_random / 400)) / 90);
        
        // 方向决策逻辑
        if (TSecType<long>::GetData(this->m_moveCtx.jc.lInputX)) {
            double distance = fabs(target_pos.x - my_x);
            
            if (distance < 100.0) {  // 距离目标过近时的边界处理
                if (my_x == this->m_rcBound.left) {
                    TSecType<long>::SetData(this->m_moveCtx.jc.lInputX, 1);   // 右移
                } else if (my_x == this->m_rcBound.right) {
                    TSecType<long>::SetData(this->m_moveCtx.jc.lInputX, -1);  // 左移
                }
            } else {
                // 标准追击方向计算
                int direction = (target_pos.x >= my_x) ? 1 : -1;
                TSecType<long>::SetData(this->m_moveCtx.jc.lInputX, direction);
            }
        }
        
        int final_input = TSecType<long>::GetData(this->m_moveCtx.jc.lInputX);
        CVecCtrl::SetInput(this, final_input, 0);
    }
}
```

### 跳跃模式特点总结

| 特性 | 实现方式 |
|------|----------|
| **智能跳跃** | 前方立足点分析，坡度冲突检测 |
| **防卡死机制** | 跳跃失败检测，反向移动，边缘处理 |
| **随机行为** | 80%移动 + 20%跳跃的随机分布 |
| **目标追击** | 33%跳跃概率的Z层切换机制 |
| **时间控制** | 动态时间调整公式，90ms时间单位 |
| **边界处理** | 立足点边缘检测，边界碰撞规避 |
| **状态同步** | 随机计数器同步，路径状态管理 |

### 跳跃与其他模式的区别

- **vs 行走模式**: 增加了立足点前瞻分析和跳跃能力
- **vs 飞行模式**: 受重力和立足点约束，但可越过障碍
- **vs 护送模式**: 保留了随机性，不依赖预定义路径
- **核心优势**: 在复杂地形中的自适应导航能力

### 护送模式 (nMoveAbility = 6)
**函数**: `CVecCtrlMob::CtrlUpdateActiveEscort` (0x99b8d0, 大小: 0x5dc)

### 护送系统核心数据结构
```cpp
// 护送目标点结构
struct EscortDest {
    SECPOINT dp;        // 目标坐标 (x, y)
    int nAttr;          // 属性标志 (1=梯子/绳子, 2=停留点)
    int ZMass;          // Z层索引
    int nStopDuration;  // 停留时间
};

// 护送上下文
ZArray<EscortDest> m_aEscorDest;        // 护送路径点数组
int m_nCurrentDestIndex;                // 当前目标索引
EscortDest m_Old_Dest;                  // 上一个目标点
bool m_bNeedReturnToStartTarget;        // 是否需要返回起始位置
SECPOINT m_ptStartTargetPosition;       // 起始目标位置
int m_nStartTargetPositionZmass;        // 起始位置Z层
bool m_bRightAngleJump;                 // 直角跳跃标志
bool m_bMoveMobOnLadderOrRopeX;         // 梯子/绳子X轴移动标志
```

### 护送模式三层控制逻辑

#### A. 有敌对目标时的简单追击 (地址: 0x99b8d7)
```cpp
if (this->m_pTarget) {
    // 检查Z层一致性
    if (m_pfh && m_pfh->m_lZMass != m_pTarget->GetZMass()) {
        // Z层不匹配，停止追击
        TSecType<int>::SetData(&this->m_bChasing, 0);
        CVecCtrlMob::ChaseTargetImp(this, 0, 0, 0);
    }
    
    // 简单的X轴方向追击
    double current_x = GetCurrentPosX();
    double target_x = GetTargetPosX();
    
    if (target_x > current_x)
        CVecCtrl::SetInput(this, 1, 0);     // 右移
    else if (target_x < current_x)
        CVecCtrl::SetInput(this, -1, 0);    // 左移
    else
        CVecCtrl::SetInput(this, 0, 0);     // 停止
    
    return;
}
```

#### B. 返回起始位置逻辑 (地址: 0x99ba27)
```cpp
if (this->m_bNeedReturnToStartTarget) {
    if (this->m_lZMass == this->m_nStartTargetPositionZmass) {
        // 在同一Z层，执行返回移动
        double current_x = GetCurrentPosX();
        double start_x = this->m_ptStartTargetPosition.x;
        
        if (start_x >= current_x) {
            // 目标在右侧或相同位置
            CVecCtrl::SetInput(this, 1, m_nInputY);
        }
        if (start_x < current_x) {
            // 目标在左侧
            CVecCtrl::SetInput(this, -1, m_nInputY);
        }
        return;
    } else {
        // Z层不匹配，取消返回
        this->m_bNeedReturnToStartTarget = 0;
    }
}
```

#### C. 路径点导航主逻辑 (地址: 0x99ba62)

##### 1. 前置条件检查
```cpp
if (!this->m_pfh || this->m_bMoveMobOnLadderOrRopeX) {
    return;  // 无立足点或在梯子/绳子上移动时跳过
}

// 处理跳跃状态
if (TSecType<int>::GetData(this->m_moveCtx.jc.bWantToJumpUp)) {
    TSecType<int>::SetData(this->m_moveCtx.jc.bWantToJumpUp, 0);
    
    if (CVecCtrl::_ZtlSecureGet_m_nInputX(this)) {
        // 检查是否在原地跳跃 (防止跳跃卡死)
        int y_before_jump = TSecType<long>::GetData(this->m_moveCtx.jc.nYBeforeJump);
        double current_y = AbsPos::_ZtlSecureGet_y(&this->m_ap);
        
        if (tolong(current_y) == y_before_jump) {
            // 跳跃失败，反向移动
            int current_input_x = CVecCtrl::_ZtlSecureGet_m_nInputX(this);
            CVecCtrl::SetInput(this, -current_input_x, m_nInputY);
        }
    }
}
```

##### 2. 路径点有效性检查
```cpp
if (ZArray<EscortDest>::IsEmpty(&this->m_aEscorDest) || 
    CVecCtrl::GetLadderOrRope(this) || 
    !this->m_pfh) {
    // 无路径点或在梯子/绳子上，停止移动
    CVecCtrl::SetInput(this, 0, 0);
    return;
}
```

##### 3. 当前目标点分析
```cpp
EscortDest* current_dest = &this->m_aEscorDest.a[this->m_nCurrentDestIndex];

// 特殊情况：直角跳跃处理
if (this->m_Old_Dest.dp.x == current_dest->dp.x && 
    this->m_Old_Dest.dp.y > current_dest->dp.y && 
    this->m_bRightAngleJump) {
    
    CVecCtrlMob::MoveMobOnRightAngleX(this);  // 执行直角移动
    CVecCtrl::SetInput(this, 0, m_nInputY);
    CVecCtrlMob::SetJumpNext(this, 0);
    this->m_bRightAngleJump = 0;
    return;
}
```

##### 4. 标准路径导航
```cpp
// X轴方向控制
if (current_dest->dp.x > current_x) {
    CVecCtrl::SetInput(this, 1, m_nInputY);   // 右移
}
if (current_dest->dp.x < current_x) {
    CVecCtrl::SetInput(this, -1, m_nInputY);  // 左移
}

// 跳跃需求检测
if (current_dest->nAttr != 1 &&                               // 非梯子/绳子点
    this->m_pfh->m_lZMass != current_dest->ZMass &&          // 不同Z层
    AbsPos::_ZtlSecureGet_y(&this->m_ap) <= current_dest->dp.y && // 目标在上方
    IsInZMassRange(current_dest->ZMass)) {                    // 在Z层范围内
    
    // 需要跳跃到不同Z层
    CVecCtrlMob::ClearMoveContext(this);
    
    // 调整移动方向 (避免跳跃冲突)
    if (current_dest->dp.x < current_x && CVecCtrl::_ZtlSecureGet_m_nInputX(this) > 0) {
        CVecCtrl::SetInput(this, -1, m_nInputY);
    }
    if (current_dest->dp.x > current_x && CVecCtrl::_ZtlSecureGet_m_nInputX(this) < 0) {
        CVecCtrl::SetInput(this, 1, m_nInputY);
    }
    
    CVecCtrlMob::SetJumpNext(this, 1);  // 标记下一帧跳跃
}
```

### 梯子/绳子控制系统 (地址: 0x99bd7d)

```cpp
// 梯子/绳子上的移动控制
if (current_dest->nAttr == 1) {  // 梯子/绳子目标点
    double current_y = GetCurrentPosY();
    double target_y = current_dest->dp.y;
    
    if (target_y <= current_y) {
        // 向下移动
        CVecCtrl::SetInput(this, m_nInputX, -1);
    } else {
        // 向上移动
        CVecCtrl::SetInput(this, m_nInputX, 1);
    }
} else if (this->m_Old_Dest.nAttr == 1) {
    // 刚离开梯子/绳子
    if (!CVecCtrl::GetLadderOrRope(this)) {
        // 完全离开，恢复正常移动
        CVecCtrl::SetInput(this, m_nInputX, 0);
    } else {
        // 仍在梯子/绳子上，继续垂直移动
        double current_y = GetCurrentPosY();
        double target_y = current_dest->dp.y;
        
        if (target_y <= current_y) {
            CVecCtrl::SetInput(this, m_nInputX, -1);
        } else {
            CVecCtrl::SetInput(this, m_nInputX, 1);
        }
    }
}
```

### 护送模式特点总结

| 特性 | 实现方式 |
|------|----------|
| **路径规划** | `ZArray<EscortDest>` 存储完整路径点序列 |
| **Z层切换** | 检测目标Z层差异，触发跳跃机制 |
| **梯子支持** | `nAttr=1` 标记，专门的垂直移动控制 |
| **停留机制** | `nStopDuration` 控制在特定点的等待时间 |
| **智能导航** | 多层级检查：敌对目标→返回起始→路径导航 |
| **跳跃优化** | 直角跳跃、反向移动防卡死机制 |
| **冲突处理** | 移动方向调整避免跳跃时的方向冲突 |

## 数据结构总结

### CMob 核心数据结构
- AFFECTEDSKILLENTRY - 受影响技能条目
- ATTACKENTRY - 攻击条目
- DAMAGEINFO - 伤害信息
- HITEFFECT - 击中效果
- TARGETINFO - 目标信息

### 相关类
- **CMobTemplate** (0xb520c8) - 怪物模板类
- **CMobPool** (0xb52088) - 怪物对象池
- **CVecCtrlMob** - 怪物向量控制器

## 移动能力对比总结

| 特性 | 静止型(0) | 行走型(1) | 跳跃型(3) | 飞行型(4) | 护送型(6) |
|------|-----------|-----------|-----------|-----------|-----------|
| **移动维度** | 无移动 | 1D地面移动 | 1D+智能跳跃 | 2D自由飞行 | 路径移动 |
| **重力影响** | - | 受重力影响 | 受重力影响 | 无重力约束 | 受重力影响 |
| **地形限制** | - | 受地形阻挡 | 智能越障 | 可越过障碍 | 沿路径移动 |
| **随机算法** | 无 | 三向随机 | 五向随机+跳跃 | 5个随机生成器 | 无随机 |
| **时间控制** | - | 90ms单位 | 90ms单位+动态调整 | 60-120帧 | 路径定义 |
| **距离容差** | - | 100像素 | 100像素+边缘检测 | 10像素精确 | 路径精确 |
| **追击方式** | 转身面向 | 直线追击 | 跳跃+追击 | 围绕偏移飞行 | 多层级逻辑 |
| **特殊机制** | - | 边界反弹 | 立足点前瞻分析 | 区域切换 | Z层切换 |
| **防卡死** | - | 基础 | 高级反向移动 | 区域重选 | 多重检查 |

## 完整函数地址映射

| 函数名 | 地址 | 大小 | 功能描述 |
|--------|------|------|----------|
| **CMob 核心函数** |
| CMob::Update | 0x654300 | 0x1abc | 怪物主更新函数 |
| CMob::ChaseTarget | 0x642db0 | 0x135 | 追击目标控制 |
| CMob::IsTargetInAttackRange | 0x645f50 | 0xb72 | 攻击范围检测 |
| CMob::OnResolveMoveAction | 0x63caf0 | 0x1ea | 处理移动动作 |
| CMob::IsChaseTargetEscort | 0x63b7b0 | 0x7e | 护送追击检查 |
| CMob::IsChaseTargetDazzle | 0x63b830 | 0x6a | 眩晕追击检查 |
| **CVecCtrlMob 移动控制** |
| CVecCtrlMob::WorkUpdateActive | 0x99d450 | 0x8d1 | 活跃状态主更新 |
| CVecCtrlMob::SetActive | 0x9987f0 | 0x2d8 | 设置活跃状态 |
| CVecCtrlMob::InspectUpdateActive | 0x9996f0 | 0x1e5 | 检查活跃更新 |
| CVecCtrlMob::ChaseTargetImp | 0x998ad0 | 0x1aa | 追击实现 |
| **行走型移动 (nMoveAbility=1)** |
| CVecCtrlMob::CtrlUpdateActiveMove | 0x99d090 | 0x3b9 | 行走型主控制 |
| **静止型移动 (nMoveAbility=0)** |
| CVecCtrlMob::CtrlUpdateActiveStop | 0x99b6a0 | 0x221 | 静止型控制 |
| **飞行型移动 (nMoveAbility=4)** |
| CVecCtrlMob::CtrlUpdateActiveFly | 0x99beb0 | 未知 | 飞行型主控制 |
| CVecCtrlMob::FlyCtrlGuardingBefore | 0x999ec0 | 未知 | 飞行前准备 |
| **护送型移动 (nMoveAbility=6)** |
| CVecCtrlMob::CtrlUpdateActiveEscort | 0x99b8d0 | 未知 | 护送型控制 |
| CVecCtrlMob::CollisionDetectEscortDest | 0x99773c | 0x474 | 护送碰撞检测 |
| **跳跃型移动 (nMoveAbility=3)** |
| CVecCtrlMob::CtrlUpdateActiveJump | 地址待确认 | 未知 | 跳跃型控制 |
| **辅助功能** |
| CVecCtrlMob::IsAbleToClimbLadderOrRope | 0x996e10 | 0x57 | 梯子/绳索检查 |
| CVecCtrlMob::IsCheatMobMoveRandImp | 0x99a860 | 0xb3b | 反作弊检查 |

## 需要进一步分析的方向

基于当前分析，以下领域需要更深入的研究：

### 1. **跳跃型移动机制 (nMoveAbility = 3)** ✅
- **函数**: `CVecCtrlMob::CtrlUpdateActiveJump` (详细分析已完成)
- **核心特性**: 五层智能控制逻辑，包含立足点前瞻分析和防卡死机制
- **算法亮点**: 80%移动+20%跳跃随机分布，33%跳跃概率的Z层切换

### 2. **护送型路径系统 (nMoveAbility = 6)** ✅
- **函数**: `CVecCtrlMob::CtrlUpdateActiveEscort` (0x99b8d0, 大小: 0x5dc)
- **核心特性**: 基于预定义路径点数组的智能导航系统
- **数据结构**: `EscortDest` 包含坐标、属性、Z层和停留时间信息

### 3. **移动动作解析系统** ✅
- **函数**: `CMob::OnResolveMoveAction` (0x63caf0, 大小: 0x1ea)
- **功能**: 根据移动能力和输入状态生成标准化的移动动作编码
- **机制**: 基于`nMoveAbility`的switch分发，处理不同移动模式的动作转换

### 4. **物理系统集成**
- **缺失**: 与 `CWvsPhysicalSpace2D` 的详细交互
- **需要**: 碰撞检测、立足点系统、重力模拟
- **重要性**: 移动系统的物理基础

### 5. **反作弊机制**
- **缺失**: `IsCheatMobMoveRandImp` 的具体检测逻辑
- **需要**: 作弊检测算法、合法性验证机制
- **重要性**: 游戏安全和公平性

### 6. **网络同步机制**
- **缺失**: 移动状态的网络同步协议
- **需要**: 数据包格式、同步频率、预测算法
- **重要性**: 多人游戏的一致性保证

### 7. **性能优化机制**
- **缺失**: 大量怪物时的性能优化策略
- **需要**: LOD系统、更新频率调整、内存管理
- **重要性**: 游戏性能和扩展性

### 8. **AI决策树**
- **缺失**: 目标选择和优先级算法
- **需要**: 威胁评估、目标切换条件、行为权重
- **重要性**: 怪物AI的智能化程度

这些方向的深入分析将进一步完善对MapleStory怪物移动系统的理解，为实现高质量的游戏服务器提供更全面的技术基础。