# WZ 文件格式规范

## 概述

WZ 文件是 MapleStory（冒险岛）游戏使用的专有数据包格式，用于存储游戏资源，包括图像、音频、脚本和其他游戏数据。WZ 文件采用分层树状结构，支持压缩和加密。

## 文件结构

### 整体布局

```
[文件头 (Header)]
[目录数据 (Directory Data)]
[图像/属性数据 (Image/Property Data)]
```

### 文件头（Header）

| 偏移 | 大小 | 字段 | 描述 |
|------|------|------|------|
| 0x00 | 4 | ident | 文件标识，固定为 "PKG1" |
| 0x04 | 8 | fsize | 文件大小（64位） |
| 0x0C | 4 | fstart | 数据开始位置 |
| 0x10 | fstart-17 | copyright | 版权信息字符串 |

`fstart` 指向实际数据开始的位置，通常在文件头之后。

## 目录结构（Directory）

目录包含文件的层次结构信息。

### 目录格式

```
[条目数量] - WzInt
[目录条目 1]
[目录条目 2]
...
[目录条目 n]
```

### 目录条目类型

每个目录条目以一个字节的类型标识开始：

| 值 | 类型 | 描述 |
|----|------|------|
| 1 | UnknownType | 未知类型，跳过处理 |
| 2 | RetrieveStringFromOffset | 从偏移位置获取字符串 |
| 3 | WzDirectory | 子目录 |
| 4 | WzImage | 图像文件 |

### 目录条目结构

根据类型不同，条目结构有所差异：

**类型 1（UnknownType）**：
- 跳过 10 字节（4 + 4 + 2）

**类型 2（RetrieveStringFromOffset）**：
- 字符串偏移（4字节）
- 文件大小（WzInt）
- 校验和（WzInt）
- 数据偏移（WzOffset）

**类型 3 和 4（WzDirectory/WzImage）**：
- 名称（WzString）
- 文件大小（WzInt）
- 校验和（WzInt）
- 数据偏移（WzOffset）

## 数据类型

### 基本类型

#### WzInt
压缩整数格式，第一个字节决定后续读取方式：
- 如果 `byte == 0x80`：读取后续 4 字节作为 int32
- 否则：`byte` 本身就是值（有符号 int8）

#### WzInt64
类似 WzInt，但用于 64 位整数

#### WzOffset
使用哈希值计算的加密偏移：
```
加密值 = 读取的 uint32
偏移 = (加密值 ^ 0xFFFFFFFF) * hash ^ (wz_mutable_key[偏移位置] * 2) + fstart
```

#### WzString
字符串存储格式：
1. 长度标识（int8）
2. 根据长度标识：
   - 如果 > 0：Unicode 字符串（长度 * 2 字节）
   - 如果 < 0：ASCII 字符串（-长度 字节）
   - 如果 = 0：后续 4 字节为实际长度
3. 字符串内容（可能加密）

### 属性类型（Property Types）

| ID | 类型 | 描述 |
|----|------|------|
| 0 | Null | 空值 |
| 2, 11 | Short | 16位整数 |
| 3, 19 | Int | 32位整数（WzInt） |
| 20 | Long | 64位整数 |
| 4 | Float | 32位浮点数 |
| 5 | Double | 64位浮点数 |
| 8 | String | 字符串 |
| 9 | Extended | 扩展属性 |

### 扩展属性类型

扩展属性以字符串标识类型：

| 类型字符串 | 描述 |
|-----------|------|
| "Property" | 属性容器，包含子属性 |
| "Canvas" | 画布，包含图像数据 |
| "Shape2D#Vector2D" | 2D向量 |
| "Shape2D#Convex2D" | 凸多边形 |
| "Sound_DX8" | 音频数据 |
| "UOL" | 引用链接 |

### 值类型（Value Types）

- **Short**: int16
- **Int**: int32  
- **Long**: int64
- **Float**: float32
- **Double**: float64
- **String**: 加密字符串
- **Vector**: 2D向量 (x, y)
- **UOL**: 统一对象链接（引用路径）
- **RawData**: 原始二进制数据
- **Lua**: Lua脚本

### 子属性类型（SubProperty Types）

- **Property**: 通用属性容器
- **Convex**: 凸多边形数据
- **Sound**: 音频数据（包含时长、格式等元数据）
- **PNG**: 图像数据（包含宽度、高度、格式等）

## 图像结构（WzImage）

WzImage 是包含实际游戏资源的容器。

### 图像头

| 字节值 | 含义 |
|--------|------|
| 0x73 | 标准图像头（WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET） |
| 0x1B | 带偏移的图像头（WZ_IMAGE_HEADER_BYTE_WITH_OFFSET） |
| 0x01 | Lua 脚本文件 |

### 图像内容

标准图像以 "Property" 字符串和值 0 开始，后续是属性列表。

## 字符串编码

字符串根据 IV 和位置进行加密/解密：

1. **ASCII 字符串**：每个字符与密钥异或
2. **Unicode 字符串**：每个 16 位字符与密钥异或

加密密钥根据字符串在文件中的位置动态生成。

## 加密和版本

### IV（初始化向量）

不同地区版本使用不同的 IV：
- GMS（全球）: 特定 4 字节值
- EMS（欧洲）: 特定 4 字节值
- BMS: 特定 4 字节值

### 版本检测

1. **文件版本头**：存储在文件开始位置的 16 位值
2. **补丁版本**：用于计算哈希值，范围通常 1-2000
3. **64位客户端检测**：版本号 >= 770 表示 64 位客户端

### 版本哈希计算

```
version_hash = 0
for char in patch_version.to_string():
    version_hash = version_hash * 32 + char_code + 1

if encver != patch_version:
    enc = 0xff ^ (version_hash >> 24) & 0xff 
          ^ (version_hash >> 16) & 0xff 
          ^ (version_hash >> 8) & 0xff 
          ^ version_hash & 0xff
    
    if enc == encver:
        使用 version_hash
```

### 密钥生成

WZ 使用动态密钥系统（WzMutableKey）进行数据加密，密钥根据：
- 初始 IV
- 文件中的位置
- AES 加密算法

生成并应用于字符串和偏移值的解密。

## 解析流程

1. **读取文件头**：验证文件标识，获取文件大小和数据起始位置
2. **检测版本**：通过尝试不同的 IV 和版本号组合来确定正确的解密参数
3. **解析目录**：递归读取目录结构，构建文件树
4. **按需解析图像**：当需要访问具体资源时，解析对应的 WzImage
5. **解析属性**：根据属性类型读取具体数据

## 特殊处理

### UOL（统一对象链接）
UOL 是指向其他节点的引用，解析时需要：
1. 获取引用路径
2. 从当前节点或根节点查找目标
3. 替换 UOL 节点为实际目标节点

### 图像格式
WZ 支持多种图像压缩格式：
- 未压缩（ARGB8888、RGB565 等）
- DXT 压缩（DXT3、DXT5）
- zlib 压缩

### 音频格式
支持的音频格式包括：
- PCM
- MP3
- WMA

音频数据可能经过额外的加密处理。