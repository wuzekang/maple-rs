# MapleStory v95 Arrow Rain (箭雨) 技能完整分析

## 概述
Arrow Rain（箭雨）是MapleStory弓箭手3转（游侠）的范围攻击技能，通过在指定区域内降下箭雨对多个敌人造成伤害。

**技能信息**
- 技能名称: Arrow Rain
- 技能ID: `3111004` (0x2F785C)
- 职业: 弓箭手3转（游侠/Ranger）
- 类型: 矩形范围攻击技能

## 技能实现流程图

```
玩家使用Arrow Rain (3111004)
    │
    ├─> CUserLocal::TryDoingShootAttack (0x927F07)
    │   ├─> 检测技能ID == 3111004
    │   ├─> 获取技能数据 SKILLENTRY::GetLevelData
    │   ├─> 计算攻击区域 (rcAffectedArea)
    │   ├─> 区域向上偏移 250 像素
    │   └─> 调用 RegisterFallingAnimation
    │
    ├─> CAnimationDisplayer::RegisterFallingAnimation (0x459B40)
    │   ├─> 加载WZ资源 (GetSpecialUOL)
    │   ├─> 解析动画参数 (x,y,a,fall,interval,count,start,duration)
    │   ├─> 创建 FALLINGINFO 结构
    │   └─> 添加到 m_Falling.m_lList 链表
    │
    └─> 游戏主循环更新
        ├─> CAnimationDisplayer::NonFieldUpdate (0x459EE0)
        ├─> TAnimation<FALLINGINFO>::Update
        └─> FALLINGINFO::Update
            ├─> 检查生命周期 (tCur > tEnd?)
            ├─> 检查更新时机 (tCur >= tUpdateNext?)
            └─> 生成箭矢批次 (nUpdateCount个)
                ├─> 随机位置
                ├─> 抛物线轨迹
                ├─> 下落动画
                └─> 注册为独立动画
```

## 核心数据结构

### FALLINGINFO 结构体
```cpp
struct FALLINGINFO {
    bool bLeft;                      // 朝向（左/右）
    RECT rcStart;                    // 起始区域（技能范围）
    int nX, nY;                      // 偏移量参数
    int nAlpha;                      // 透明度
    int tFall;                       // 基础下落时间
    int tUpdateInterval;             // 更新间隔（多久生成一批箭矢）
    int nUpdateCount;                // 每次更新生成的箭矢数量
    int tUpdateNext;                 // 下次更新时间
    int tEnd;                        // 动画结束时间
    ZArray<IWzProperty*> apProperty; // 箭矢动画帧数组
};
```

## 关键函数分析

### 1. 技能触发 - CUserLocal::TryDoingShootAttack
**地址**: `0x927F07`, `0x92805E`

```cpp
// Arrow Rain 特殊处理
if (nSkillID == 3111004) {
    // 1. 设置额外攻击延迟
    aAttackInfo[0].tDelay = ShootDelay + 400;  // 400ms额外延迟
    
    // 2. 获取技能攻击区域
    SKILLLEVELDATA* pLevelData = SKILLENTRY::GetLevelData(pSkill, nSLV);
    rcStart = pLevelData->rcAffectedArea;
    
    // 3. 根据角色位置调整区域
    adjust_rect(&rcStart, x, y, ...);
    
    // 4. Arrow Rain特有：区域向上偏移250像素
    rcStart.top -= 250;
    rcStart.bottom -= 250;
    
    // 5. 获取动画资源路径
    BSTR sUOL = SKILLENTRY::GetSpecialUOL(pSkill, ...);
    
    // 6. 注册下落动画
    CAnimationDisplayer::RegisterFallingAnimation(
        sUOL,                    // 动画资源路径
        bLeft,                   // 朝向
        &rcStart,                // 攻击区域
        aAttackInfo[0].tDelay    // 延迟时间
    );
}
```

### 2. 动画创建 - RegisterFallingAnimation
**地址**: `0x459B40`

```cpp
void CAnimationDisplayer::RegisterFallingAnimation(
    const wchar_t *sUOL,    // WZ资源路径
    int bLeft,              // 朝向
    RECT *rcStart,          // 攻击区域
    int tStart)             // 开始时间
{
    // 1. 加载WZ资源
    IWzProperty* pProp = IWzResMan::GetObjectA(g_rm, sUOL, ...);
    
    // 2. 创建FALLINGINFO并加入链表
    FALLINGINFO* pInfo = m_Falling.m_lList.AddTail();
    
    // 3. 复制基本参数
    pInfo->bLeft = bLeft;
    pInfo->rcStart = *rcStart;
    
    // 4. 从WZ资源读取动画参数
    pInfo->nX = pProp->GetItem("x")->GetInt32();           // X偏移
    pInfo->nY = pProp->GetItem("y")->GetInt32();           // Y偏移  
    pInfo->nAlpha = pProp->GetItem("a")->GetInt32();       // 透明度
    pInfo->tFall = pProp->GetItem("fall")->GetInt32();     // 下落时间
    pInfo->tUpdateInterval = pProp->GetItem("interval")->GetInt32();  // 更新间隔
    pInfo->nUpdateCount = pProp->GetItem("count")->GetInt32();        // 每次数量
    
    // 5. 计算时间参数
    int start = pProp->GetItem("start")->GetInt32();
    int duration = pProp->GetItem("duration")->GetInt32();
    pInfo->tUpdateNext = tStart + get_update_time() + start;
    pInfo->tEnd = tStart + get_update_time() + duration;
    
    // 6. 加载动画帧序列
    for (int i = 0; ; i++) {
        IWzProperty* pFrame = pProp->GetItem(itow(i));
        if (!pFrame) break;
        pInfo->apProperty.InsertBefore(-1, pFrame);
    }
}
```

### 3. 动画更新 - FALLINGINFO::Update
**地址**: 未直接找到（模板实例化）

```cpp
int FALLINGINFO::Update(int tCur) {
    // 1. 生命周期检查
    if (tCur > this->tEnd) {
        return 1;  // 动画结束，移除
    }
    
    // 2. 更新时机检查
    if (tCur < this->tUpdateNext) {
        return 0;  // 未到更新时间
    }
    
    // 3. 设置下次更新时间
    this->tUpdateNext += this->tUpdateInterval;
    
    // 4. 生成这一批箭矢
    for (int i = 0; i < this->nUpdateCount; i++) {
        // 随机位置
        int x = rcStart.left + rand() % (rcStart.right - rcStart.left);
        int y = rcStart.top + rand() % (rcStart.bottom - rcStart.top);
        
        // 随机动画帧
        int frameIdx = rand() % apProperty.size();
        
        // 创建动画层
        IWzGr2DLayer* pLayer = LoadLayer(apProperty[frameIdx], x, y);
        
        // 计算抛物线轨迹
        int xOffset = this->nX;
        if (xOffset != 0) {
            int r = rand() % 41 - 20;  // [-20, 20]
            yOffset = nY + (xOffset * (20 - r)) / nY;
        }
        
        // 下落时间随机化
        int fallTime = tFall - 150 + rand() % 300;  // ±150ms
        
        // 设置动画
        pLayer->RelOffset(xOffset, yOffset, fallTime);
        pLayer->Getalpha()->RelMove(alpha, 0, 150);
        pLayer->Animate(GA_REPEAT);
        
        // 注册为独立动画
        RegisterRepeatAnimation(pLayer, fallTime);
    }
    
    return 0;  // 继续更新
}
```

### 4. 主更新循环 - NonFieldUpdate
**地址**: `0x459EE0`

```cpp
void CAnimationDisplayer::NonFieldUpdate(int tCur) {
    // 更新各种动画系统
    TAnimation<ONETIMEINFO>::Update(&m_OneTime, tCur);
    TAnimation<REPEATINFO>::Update(&m_Repeat, tCur);
    TAnimation<FALLINGINFO>::Update(&m_Falling, tCur);  // Arrow Rain在这里更新
    TAnimation<EXPLOSIONINFO>::Update(&m_Explosion, tCur);
    // ... 其他动画类型
}
```

## 技能特征判断函数

| 函数名 | 地址 | 返回值 | 说明 |
|--------|------|--------|------|
| is_attack_area_set_by_data | 0x6ED907 | true | 攻击区域由数据定义 |
| is_rect_attack_shoot_skill | 0x6ED9AF | true | 矩形范围射击技能 |
| is_shoot_skill_not_showing_bullet | 0x6EDC7D | true | 不显示弹射物 |
| is_shoot_skill_not_switched_to_melee_attack | 0x6EDD55 | true | 不切换近战 |

## 动画时序示例

假设 WZ 资源中的参数：
- `interval` = 100ms（每100ms生成一波）
- `count` = 5（每波5个箭矢）
- `duration` = 2000ms（持续2秒）
- `fall` = 800ms（下落时间）

```
时间线：
t=0ms    : 技能触发，创建FALLINGINFO
t=100ms  : 第1波箭矢（5个），每个下落650-950ms
t=200ms  : 第2波箭矢（5个）
t=300ms  : 第3波箭矢（5个）
...
t=1900ms : 最后一波
t=2000ms : FALLINGINFO结束，停止生成
t=2950ms : 最后的箭矢落地，动画完全结束
```

## 关键算法

### 抛物线轨迹算法
```cpp
// 创建自然的抛物线下落效果
if (nX != 0) {
    int r = rand() % 41 - 20;  // 随机偏差 [-20, 20]
    yOffset = nY + (nX * (20 - r)) / nY;
}
// 效果：横向移动越远，纵向偏移越大
```

### 随机性设计
- **位置**: 在攻击区域内均匀分布
- **时间**: 下落时间 ±150ms 随机
- **透明度**: 128-191 之间随机
- **动画帧**: 从资源数组随机选择

## 相关全局变量和常量

| 名称 | 地址 | 类型 | 说明 |
|------|------|------|------|
| TSingleton<CAnimationDisplayer>::ms_pInstance | - | CAnimationDisplayer* | 动画显示器单例 |
| g_rm | 0xC63EA8 | IWzResMan* | WZ资源管理器 |
| g_rand | - | CRand32 | 随机数生成器 |
| RANGER_ARROW_RAIN | 0x2F785C | enum | 技能ID枚举值 |

## 技术亮点

1. **生成器模式**: FALLINGINFO 作为箭矢生成器，定期产生新动画
2. **物理模拟**: 抛物线公式创造真实的下落效果
3. **性能优化**: 链表管理，自动清理过期动画
4. **数据驱动**: 所有参数从 WZ 资源加载，易于调整

## 总结

Arrow Rain 通过巧妙的动画系统设计，实现了持续的箭雨效果：
- 使用 FALLINGINFO 作为生成器，定期创建箭矢
- 每个箭矢是独立的动画对象，有自己的轨迹和时间
- 通过随机性和物理模拟创造自然的视觉效果
- 技能的持续时间、频率、数量等完全由数据配置