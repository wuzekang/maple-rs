# 冒险岛移动按键系统详细分析

## 1. 按键映射概览

### 基础移动按键
- **左箭头** (键码 37): 向左移动
- **右箭头** (键码 39): 向右移动  
- **上箭头** (键码 38): 爬梯子/绳子向上、进入传送门
- **下箭头** (键码 40): 爬梯子/绳子向下、趴下、下跳

### 特殊移动按键（功能键映射系统）
- **Alt**: 跳跃键（功能ID 53）
- **Alt + 下**: 向下跳跃（穿过平台）
- **Shift**: 快捷键/瞬移（通过快捷键映射系统）
- **Space**: NPC对话（功能ID 54）
- **X**: 坐下/站起（功能ID 51）
- **Ctrl**: 普通攻击（功能ID 52）
- **Z**: 拾取物品（功能ID 50）

## 2. 方向键处理机制

### 2.1 输入检测（CVecCtrlUser::WorkUpdateActive）
```cpp
// 地址: 0x9c45f0
// 左右方向键检测
if (CInputSystem::IsKeyPressed(pInputSystem, 37)) // 左
    lInputX = -1;
if (CInputSystem::IsKeyPressed(pInputSystem, 39)) // 右
    lInputX = 1;

// 上下方向键检测  
if (CInputSystem::IsKeyPressed(pInputSystem, 38)) // 上
    lInputY = -1;
if (CInputSystem::IsKeyPressed(pInputSystem, 40)) // 下
    lInputY = 1;
```

### 2.2 特殊处理
- **反向输入模式**: 当启用时，左右键功能互换
- **移动模式** (按P键切换): 方向键在地图格子间移动
- **冲刺中断**: 左右方向键可中断冲刺状态

## 3. 跳跃系统

### 3.1 普通跳跃（CVecCtrl::Jump）
```cpp
// 地址: 0x994430
void CVecCtrl::Jump(CVecCtrl *this) {
    this->m_bJumpNext = 0;
    this->m_bTryJumpedInFly = 1;
    if (CVecCtrl::JustJump(this))
        CVecCtrl::SetMovePathAttribute(this, 1);
}
```

### 3.2 跳跃物理参数
- **基础跳跃速度**: `dJumpSpeed`
- **跳跃高度公式**: `vy = -(dJumpSpeed * walkJump / g)`
- **飞行状态跳跃**: 高度减少到70%
- **梯子/绳子跳跃**: 高度减少到30%-50%

### 3.3 向下跳跃（CVecCtrl::FallDown）
```cpp
// 地址: 0x993d80
// 设置向下的速度
vy = dJumpSpeed * 0.35355339 / g
vx = 0
```

### 3.4 垂直跳跃（CUserLocal::VerticalJump）
```cpp
// 地址: 0x90f510
// Alt+方向键的特殊跳跃
// 根据按键方向设置水平冲力
int impulse = 0;
if (IsKeyPressed(39)) impulse += 100;  // 右
if (IsKeyPressed(37)) impulse -= 100;  // 左
impulse += (m_nMoveAction & 1) ? -200 : 200; // 朝向加成

SetImpactNext(impulse, -2300.0); // 设置冲击力
```

## 4. 爬梯子/绳子系统

### 4.1 检测机制
- 按上下方向键时自动搜索附近的梯子/绳子
- 使用 `CWvsPhysicalSpace2D::GetLadderOrRope` 查找
- 自动对齐角色到梯子/绳子位置

### 4.2 状态管理
- 成员变量 `_ZtlSecureTear_m_pLadderOrRope` 存储当前附着对象
- 通过虚函数表第5个函数处理脱离逻辑
- 从梯子跳跃时跳跃高度和水平速度都会减少

## 5. 移动状态（m_nMoveAction）

### 5.1 状态位定义
- **最低位（& 1）**: 角色朝向（0=右，1=左）
- **趴下状态**: `(m_nMoveAction & 0xFFFFFFFE) == 0xA`
- **冲刺状态**: `(m_nMoveAction & 0xFFFFFFFE) == 0x12`

### 5.2 趴下状态
- 身体碰撞盒: `left=-46, top=-31, right=0, bottom=0`
- 减少被击中的面积
- 某些地图可能限制趴下功能

## 6. 特殊移动功能

### 6.1 冲刺系统
- 相关常量:
  - `CTS_Dash_Speed`: 冲刺速度
  - `CTS_Dash_Jump`: 冲刺跳跃
- 使用 `CDashTrigger` 类管理
- 技能ID 4321000: 特殊冲刺取消效果

### 6.2 飞行/游泳
- 飞行加速度: `flyAcc`
- 游泳速度: `swimSpeedV`
- 空中可多次跳跃（飞行状态）

### 6.3 护送怪物模式
- 特殊的移动速度限制
- 固定水平速度: 125.0 * 1.3

## 7. 功能键映射（UseFuncKeyMapped）

### 7.1 映射管理系统
- **CFuncKeyMappedMan**: 功能键映射管理器（地址: 0x56872f）
- **s_aDefaultFKM**: 默认功能键映射表（地址: 0xc56910）
- **UseFuncKeyMapped**: 主要的功能键处理函数（地址: 0x932f13）

### 7.2 功能ID对应（类型5 - 系统功能）
- **50**: 拾取物品 - 调用 `CDropPool::TryPickUpDrop`
- **51**: X键功能 - 调用 `CUserLocal::HandleXKeyDown`
- **52**: Ctrl键功能 - 调用 `CUserLocal::HandleCtrlKeyDown`
- **53**: 跳跃功能 - 调用 `CUserLocal::Jump` 或 `CUserLocal::FallDown`
- **54**: NPC对话功能 - 调用 `CUserLocal::TalkToNpc`

### 7.3 跳跃的特殊处理
```cpp
case 53: // 跳跃功能
    if (CUserLocal::IsSit(this)) {
        CWvsContext::SendGetUpFromChairRequest(lParam, 500);
        return 1;
    }
    IsKeyPressed = CInputSystem::IsKeyPressed(TSingleton<CInputSystem>::ms_pInstance, 40);
    if (IsKeyPressed) {
        CUserLocal::FallDown(this); // 向下跳跃
    } else {
        CUserLocal::Jump(this, 0); // 普通跳跃
    }
```

### 7.4 功能键类型系统
- **类型1**: 技能 - 调用 `CUserLocal::DoActiveSkill`
- **类型2**: 物品 - 各种物品使用逻辑
- **类型3**: 表情 - 调用 `CWvsContext::SendEmotionChange`
- **类型4**: 其他功能
- **类型5**: 系统功能（移动相关按键）
- **类型6**: 表情相关
- **类型7**: 特效物品
- **类型8**: 宏 - 调用 `CMacroSysMan::DoActiveMacro`

### 7.5 R键的特殊处理
```cpp
// 在UseFuncKeyMapped函数开头的特殊处理
if (v4 == 82 && !v5) { // 键码82 = R键
    v8 = TSingleton<CDropPool>::ms_pInstance;
    v9 = GetPos(&this->IVecCtrlOwner, &result);
    CDropPool::TryPickUpDrop(v8, v9);
    return 1;
}
```
- R键（键码82）绕过功能键映射系统，直接调用拾取功能
- 这是为了确保拾取功能的响应速度和可靠性

## 8. 输入优化

### 8.1 输入缓冲
- 使用 `CDualKeyChecker` 处理组合键
- 支持按键序列检测（连击系统）

### 8.2 状态检查
- `IsImmovable`: 是否不能移动
- `IsStun`: 是否眩晕
- `IsAttract`: 是否被吸引

### 8.3 防作弊机制
- 速度值使用安全包装（`_ZtlSecureTear`）
- 服务器验证移动合法性

## 9. 音效处理
- 跳跃音效: `play_game_sound("Jump", 100)`
- 仅限玩家角色播放，怪物/NPC不播放

## 10. 注意事项

1. **窗口焦点**: 只有游戏窗口获得焦点时才处理输入
2. **状态优先级**: 某些状态（如眩晕）会覆盖正常输入
3. **地图限制**: 某些地图可能禁用特定移动功能
4. **延迟处理**: Tab键和列表键有特殊的按键延迟
5. **安全检查**: 所有移动都需要通过物理系统验证