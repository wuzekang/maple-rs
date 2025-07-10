# 冒险岛输入系统详细分析

## 1. 输入系统架构

### 核心类
- **CInputSystem**: 输入系统单例类
  - 单例实例存储在 `?ms_pInstance@?$TSingleton@VCInputSystem@@@@1PAVCInputSystem@@A` (0xc68c20)
  - 使用 DirectInput8 作为底层输入接口

- **CSequencedKeyMan**: 按键序列管理器
  - 单例实例存储在 `?ms_pInstance@?$TSingleton@VCSequencedKeyMan@@@@1PAVCSequencedKeyMan@@A` (0xc68a68)
  - 负责管理连击检测和按键序列匹配

- **CFuncKeyMappedMan**: 功能键映射管理器
  - 单例实例存储在 `?ms_pInstance@?$TSingleton@VCFuncKeyMappedMan@@@@1PAVCFuncKeyMappedMan@@A` (0xc6aae4)
  - 管理快捷键映射

- **CQuickslotKeyMappedMan**: 快捷栏键映射管理器
  - 单例实例存储在 `?ms_pInstance@?$TSingleton@VCQuickslotKeyMappedMan@@@@1PAVCQuickslotKeyMappedMan@@A` (0xc6abe8)

## 2. 按键处理流程

### 2.1 主要入口点
```cpp
void CUserLocal::OnKey(CUserLocal *this, WPARAM wParam, LPARAM lParam)
// 地址: 0x936b19
// 功能: 处理键盘输入事件
```

### 2.2 处理流程
1. **输入预处理**
   - 提取扫描码: `v5 = BYTE2(lParam)`
   - 处理左右键反转（如果启用）
   - 特殊键处理（如方向键）

2. **按键按下处理** (lParam >= 0)
   - 调用 `CSequencedKeyMan::Process` 检测连击序列
   - 处理功能键映射 `CUserLocal::UseFuncKeyMapped`
   - 处理特殊模式（如移动模式）

3. **按键释放处理** (lParam < 0)
   - 通知序列管理器按键释放
   - 处理技能结束 `CUserLocal::OnKeyDownSkillEnd`
   - 停止宏系统（如果激活）

## 3. 连击系统

### 3.1 CSequencedKeyMan::Process
```cpp
int CSequencedKeyMan::Process(CSequencedKeyMan *this, int nScancode, int bDown)
// 地址: 0x6e3400
// 功能: 处理按键序列，检测连击
```

### 3.2 连击检测机制
1. **活动候选管理**
   - 维护活动的按键序列候选列表 `m_lActiveCandidate`
   - 每个候选包含：序列ID、当前索引、开始时间、累计超时

2. **超时处理**
   - 每个序列元素都有超时时间 `tExpire`
   - 超过时间窗口的序列会被移除

3. **序列匹配**
   - 使用虚函数检查按键是否匹配当前序列元素
   - 完成整个序列时触发对应的动作

### 3.3 连击技能
- **30连击**: 技能ID 21100004-21100005
- **100连击**: 技能ID 21110004
- **200连击**: 技能ID 21120006-21120007

连击计数存储在 `CUserLocal` 对象的偏移 1526 位置。

## 4. 输入缓冲机制

### 4.1 CDualKeyChecker
- 用于检测双键组合（如 Shift+技能键）
- 管理按键消息队列 `ZList<KeyMsg>`

### 4.2 输入队列
- 使用 `ZList` 模板类管理输入队列
- 支持按键优先级处理
- 具有防重复机制

## 5. 按键映射系统

### 5.1 功能键映射
- 支持自定义快捷键
- 配置保存在注册表或配置文件中
- 通过 `CUIKeyConfig` 界面进行设置

### 5.2 虚拟键码
- 使用 Windows 虚拟键码（VK_*）
- 支持扫描码到虚拟键码的转换 `MapVirtualKeyA`

## 6. 特殊输入处理

### 6.1 移动模式
- 按下 P 键切换移动模式
- 在移动模式下，方向键控制角色在地图格子间移动

### 6.2 冲刺取消
- 左右方向键可以取消冲刺状态
- 特殊处理技能 4321000（停止特效）

### 6.3 坐下状态
- 方向键输入会自动站起

## 7. 性能优化

### 7.1 输入过滤
- 过滤重复的按键事件
- 忽略无效的扫描码

### 7.2 对象池
- 使用 `ZRecyclableAvBuffer` 管理对象池
- 减少频繁的内存分配

## 8. 相关全局变量

- `?ms_dwKeyCounter@CWnd@@1KA` (0xc6f6f8): 按键计数器
- `?ms_KeyDownDelay@CCSWnd_Tab@@1KA` (0xc56788): Tab键延迟
- `?ms_KeyDownDelay@CCSWnd_List@@1KA` (0xc5678c): 列表键延迟

## 9. 注意事项

1. **线程安全**: 输入系统使用单例模式，需要注意多线程访问
2. **延迟处理**: 某些按键有内置延迟，防止过快触发
3. **状态依赖**: 输入处理依赖于角色当前状态（站立、移动、技能等）