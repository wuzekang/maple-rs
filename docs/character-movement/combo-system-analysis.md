# 冒险岛连击系统分析

## CSequencedKeyMan::Process 函数分析

### 函数地址
- 地址：`0x6e3400`
- 大小：`0x47d`
- 函数签名：`int __thiscall CSequencedKeyMan::Process(CSequencedKeyMan *this, int nScancode, int bDown)`

### 功能概述
这个函数是处理按键序列（连击）的核心函数。它负责：
1. 接收按键输入（扫描码和按下/抬起状态）
2. 管理活动的按键序列候选
3. 检测并触发连击技能

### 主要流程

1. **输入验证**
   - 检查当前是否为冲刺状态（`CUserLocal::IsDashing`）
   - 过滤重复的按键输入
   - 特殊处理：扫描码 42 会被转换为 54

2. **活动序列处理**
   - 遍历所有活动的候选序列（`m_lActiveCandidate`）
   - 检查每个序列的超时状态（`dwAccumulatedExpire`）
   - 匹配当前按键是否符合序列要求

3. **序列匹配**
   - 使用虚函数调用检查按键匹配：`v13->vfptr[1].__vecDelDtor(v13, nScancode, bDown)`
   - 如果匹配成功，增加当前索引
   - 完成序列时触发动作：`v25->vfptr[2].__vecDelDtor(v25, 0)`

4. **新序列候选**
   - 从候选池（`m_aCandidatePool`）中查找可能的新序列
   - 创建新的活动候选并添加到列表

### 关键数据结构

#### ActiveCandidateEntity
- `nSequenceID`：序列ID
- `nCurrentIndex`：当前匹配到的按键索引
- `dwStartTick`：序列开始时间
- `dwAccumulatedExpire`：累计超时时间

#### KeySequenceElement
- 基类，包含虚函数表
- `tExpire`：元素的超时时间
- 虚函数用于：
  - 检查按键匹配
  - 检查重复按键
  - 执行动作

## 连击技能系统

### CComboSmash 类
- 继承自 `KeySequenceElement`
- 虚函数表地址：`0xb54e68`
- `DoAction` 函数地址：`0x6e08d0`

### 连击计数要求
通过 `get_required_combo_count` 函数（地址：`0x6ee020`）获取不同技能所需的连击数：

- 技能 21100004-21100005：需要 30 连击
- 技能 21110004：需要 100 连击
- 技能 21120006-21120007：需要 200 连击

### 连击计数器
- 存储在 `CUserLocal` 的偏移 1526 位置（`m_Data[1526].m_RefCount`）
- 全局变量 `CTS_ComboCounter` 可能用于临时存储

### 技能执行流程
1. 检查当前连击数是否满足技能要求
2. 检查玩家是否学习了对应技能
3. 根据方向设置技能朝向
4. 调用 `CUserLocal::DoActiveSkill` 执行技能

## 调用关系
`CSequencedKeyMan::Process` 被以下函数调用：
- `CUserLocal::OnJoystickButton` (地址：0x936657, 0x936880)
- `CUserLocal::OnKey` (地址：0x936b22, 0x937119)

## 其他相关类
- `KeySequenceElementIgnoreUp`：忽略按键抬起的序列元素
- `CComboDrain`：连击吸血技能
- `CComboBarrier`：连击屏障技能
- `CComboAbilityBuff`：连击能力增益

## 序列配置
序列的具体配置在 `CSequencedKeyMan::Restore` 函数中进行（地址：0x6e11f0），该函数负责初始化所有的按键序列定义。