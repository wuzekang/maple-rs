# use_resource 快速参考

## 🚀 快速开始

```rust
use sig::prelude::*;

let user_id = Signal::new(1);

// ✅ 正确：在 async 前读取 signal
let user_data = use_resource(move || {
    let id = *user_id.read();  // 自动追踪！
    async move {
        fetch_user(id).await
    }
});

// Signal 变化自动触发重启
*user_id.write() = 2;  // 自动重新获取！
```

## ✅ 正确用法

### 单个依赖
```rust
let resource = use_resource(move || {
    let value = *signal.read();  // ← 在这里读取
    async move {
        fetch(value).await
    }
});
```

### 多个依赖
```rust
let resource = use_resource(move || {
    let id = *id_signal.read();
    let filter = filter_signal.read().clone();
    async move {
        fetch_filtered(id, &filter).await
    }
});
```

### 条件依赖
```rust
let resource = use_resource(move || {
    let enabled = *enabled_signal.read();
    let id = if enabled {
        *id_signal.read()
    } else {
        0
    };
    async move {
        if enabled {
            fetch(id).await
        } else {
            default_value()
        }
    }
});
```

## ❌ 常见错误

### 错误1：在 async block 内读取 ⚠️

```rust
// ❌ 错误 - 不会追踪
let resource = use_resource(move || async move {
    let id = *signal.read();  // ← 太晚了！
    fetch(id).await
});
```

**原因：** `async move {}` 创建 Future 但不立即执行，signal 读取发生在 Future 执行时，而不是 effect 运行时。

**注意：** 这与 Dioxus 不同！Dioxus 可以在 async block 内读取，因为它在每次 poll Future 时都会追踪依赖。我们的实现更简单但要求在 async block **之前**读取。详见 [Dioxus 对比文档](./dioxus_use_resource_comparison.md)。

### 错误2：忘记读取 signal
```rust
// ❌ 错误 - 从未读取 signal
let resource = use_resource(move || {
    // 没有读取任何 signal
    async move {
        fetch(1).await  // 硬编码的值
    }
});

*signal.write() = 2;  // 不会触发重启！
```

### 错误3：使用旧的手动模式
```rust
// ❌ 不再需要
create_effect(move || {
    let _ = *signal.read();
    resource.restart();  // 多余！
});
```

## 🎮 手动控制

即使有自动追踪，手动控制仍然可用：

```rust
// 强制刷新（忽略 signal 状态）
resource.restart();

// 暂停/恢复
resource.pause();
resource.resume();

// 取消
resource.cancel();

// 检查状态
match resource.state() {
    ResourceState::Pending => { /* 加载中 */ }
    ResourceState::Ready => { /* 就绪 */ }
    ResourceState::Paused => { /* 已暂停 */ }
    ResourceState::Stopped => { /* 已停止 */ }
}

// 获取值
if let Some(value) = resource.value() {
    println!("Data: {:?}", value);
}
```

## 📊 状态流转

```
Pending → Ready → (signal变化) → Pending → Ready
   ↓                   ↓              ↑
 Paused              Paused      (resume)
   ↓                   ↓
Stopped             Stopped
```

## 🔍 调试技巧

### 1. 打印追踪信息
```rust
let resource = use_resource(move || {
    let id = *signal.read();
    println!("Resource restarting with id={}", id);  // 调试输出
    async move {
        fetch(id).await
    }
});
```

### 2. 检查调用次数
```rust
let call_count = Arc::new(Mutex::new(0));

let resource = use_resource(move || {
    let id = *signal.read();
    let count = call_count.clone();
    async move {
        *count.lock().unwrap() += 1;
        fetch(id).await
    }
});
```

### 3. 使用状态信号
```rust
// 监控资源状态变化
create_effect(move || {
    println!("Resource state: {:?}", resource.state());
});
```

## 💡 最佳实践

1. **总是在 async block 前读取 signal**
2. **使用有意义的变量名保存读取的值**
3. **避免在 async block 内使用 signal**
4. **利用自动追踪，避免手动 restart**
5. **只在必要时使用手动控制**

## 📚 示例代码

```bash
# 运行综合示例（控制台）
cargo run --example resource_demo

# 运行交互示例（GUI）
cargo run --example resource_simple

# 运行测试
cargo test --package sig --test test_use_resource
```

## 🔗 相关文档

- [examples/README.md](../crates/sig/examples/README.md) - 示例说明
- [use_resource_fix_summary.md](./use_resource_fix_summary.md) - 修复详情
- [reactive/use_resource.rs](../crates/sig/src/reactive/use_resource.rs) - 实现代码
