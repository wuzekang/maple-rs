# use_resource 限制解除 - 实现总结

## 背景

原有的 `use_resource` 有一个限制：必须在 async block **之前**读取 Signal，不能在 async block **内部**读取。

```rust
// ❌ 不工作（旧版本）
let resource = use_resource(move || async move {
    let id = *user_id.read();  // 在这里读取不会被追踪
    fetch_user(id).await
});

// ✅ 必须这样（旧版本）
let resource = use_resource(move || {
    let id = *user_id.read();  // 必须在这里读取
    async move {
        fetch_user(id).await
    }
});
```

## 解决方案

### 新增 API：`use_resource_with_tracking`

现在你可以直接在 async block 内读取 Signal！

```rust
use sig::reactive::use_resource_with_tracking;

// ✅ 现在可以了！
let resource = use_resource_with_tracking(move || async move {
    let id = *user_id.read();  // 完全支持！
    fetch_user(id).await
});
```

## 实现原理

### 问题根源

通过全链路追踪 sig 的依赖收集机制，发现：

1. **依赖收集依赖 effect stack**
   ```rust
   // Signal::read() 的实现
   pub fn read(&self) -> ReadGuard<T> {
       let signal_id = self.inner.read().id;
       with_reactive(|state| {
           if let Some(effect_rc) = state.effects.last() {  // ← 从 stack 获取
               state.subscribe(signal_id, effect_rc.clone());
           }
       });
       // ...
   }
   ```

2. **effect 运行时机制**
   ```rust
   // create_effect 时
   with_reactive_mut(|state| {
       state.effects.push(effect_rc.clone());  // ← 推入 stack
   });
   
   (callback)();  // 执行，此时 signal.read() 能追踪
   
   with_reactive_mut(|state| {
       state.effects.pop();  // ← 弹出 stack
   });
   ```

3. **async block 内的问题**
   - async block 创建 Future
   - Future 在 spawn 的 task 中 poll
   - **poll 时 effect stack 是空的！** ← 这是核心问题

### 解决方案：ReactiveContextFuture

创建一个 Future wrapper，在每次 poll 时维持 effect context：

```rust
pub struct ReactiveContextFuture<F> {
    inner: Pin<Box<F>>,
    effect_rc: Rc<RefCell<Effect>>,
}

impl<F: Future> Future for ReactiveContextFuture<F> {
    type Output = F::Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 1. Poll 前：推入 effect
        super::runtime::with_reactive_mut(|state| {
            state.effects.push(effect_rc.clone());
        });
        
        // 2. 执行 poll - 此时 signal.read() 能追踪到！
        let result = this.inner.as_mut().poll(cx);
        
        // 3. Poll 后：弹出 effect
        super::runtime::with_reactive_mut(|state| {
            state.effects.pop();
        });
        
        result
    }
}
```

### 完整流程

```rust
pub fn use_resource_with_tracking<T, F>(mut future_fn: impl FnMut() -> F + 'static) -> Resource<T> {
    // 存储 effect_rc
    let effect_rc_storage = Rc::new(RefCell::new(None));
    
    create_effect(move || {
        // 1. 获取当前 effect（create_effect 正在运行的 effect）
        let current_effect = with_reactive(|state| {
            state.effects.last().cloned()
        });
        
        // 2. 存储供后续使用
        *effect_rc_storage.borrow_mut() = current_effect.clone();
        
        // 3. 创建 future 并包装
        let fut = future_fn();
        let wrapped = ReactiveContextFuture::new(fut, current_effect.unwrap());
        
        // 4. Spawn - 每次 poll 都会维持 effect context
        spawn(wrapped);
    });
}
```

## 文件变更

### 新增文件

1. **`crates/sig/src/reactive/reactive_context.rs`**
   - `ReactiveContextFuture` - Future wrapper 实现

2. **`crates/sig/tests/test_use_resource_with_tracking.rs`**
   - 测试新 API 的功能
   - 验证多个 signal 追踪
   - 对比新旧 API

3. **`crates/sig/examples/resource_tracking_demo.rs`**
   - 演示新旧 API 的使用
   - 展示实际效果

### 修改文件

1. **`crates/sig/src/reactive/mod.rs`**
   - 新增 `pub mod reactive_context;`
   - 导出 `ReactiveContextFuture`

2. **`crates/sig/src/reactive/use_resource.rs`**
   - 新增 `use_resource_with_tracking` 函数
   - 更新文档说明两种 API 的区别

3. **`docs/dioxus_use_resource_comparison.md`**
   - 更新为"限制已解除"
   - 详细说明实现原理
   - 提供使用指南

## 测试结果

### 新测试
```bash
$ cargo test --test test_use_resource_with_tracking
running 3 tests
test test_use_resource_with_tracking_basic ... ok
test test_use_resource_with_tracking_multiple_signals ... ok
test test_comparison_old_vs_new ... ok
```

### 旧测试（确保兼容性）
```bash
$ cargo test --test test_use_resource
running 5 tests
test test_use_resource_basic ... ok
test test_use_resource_reactive_deps ... ok
test test_use_resource_manual_restart ... ok
test test_use_resource_pause_resume ... ok
test test_use_resource_cancel ... ok
```

### 示例运行
```bash
$ cargo run --example resource_tracking_demo
=== Resource Tracking Demo ===

1. Old Style (use_resource):
   Fetching data for value: 0
   Initial result: Some(0)
   Updating counter to 5...
   Fetching data for value: 5
   New result: Some(50)

2. New Style (use_resource_with_tracking):
   Fetching data for value: 0
   Initial result: Some(0)
   Updating counter to 5...
   Fetching data for value: 5
   New result: Some(50)
```

## API 选择指南

### 使用 `use_resource`（原有）

✅ 适合：
- 依赖简单，在外部读取更清晰
- 性能敏感场景
- 不需要在 async 内读取 signal

```rust
let count = Signal::new(0);
let resource = use_resource(move || {
    let n = *count.read();  // 清晰的依赖
    async move {
        expensive_operation(n).await
    }
});
```

### 使用 `use_resource_with_tracking`（新增）

✅ 适合：
- 需要在 async block 内读取 signal
- 复杂异步逻辑
- 更自然的编程风格

```rust
let a = Signal::new(1);
let b = Signal::new(2);

let resource = use_resource_with_tracking(move || async move {
    let x = *a.read();
    let result1 = call1(x).await;
    let y = *b.read();  // 可以穿插读取
    call2(result1, y).await
});
```

## 性能影响

### ReactiveContextFuture 开销

每次 poll：
- 2 次 `with_reactive_mut` 调用
- Effect stack 的 push/pop

**实际影响**：
- IO 密集型：可忽略（相比网络/磁盘延迟）
- 计算密集型：微小（主要时间在计算）
- 高频 poll：可考虑使用原有 `use_resource`

## 总结

✅ **成功解除了 async block 内无法读取 Signal 的限制**

✅ **提供了两种 API，灵活选择**

✅ **实现原理清晰，基于 effect stack 机制**

✅ **所有测试通过，向后兼容**

现在 sig 的 `use_resource` 功能已经和 Dioxus 一样强大了！
