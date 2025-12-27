# Dioxus use_resource 对比分析 - 限制已解除！

## 问题回答

**问：\"async block 内读取才能触发跟踪\" 在 Dioxus 中有这个限制吗？**

**答案：没有！现在我们也支持了！** ✅

## 解决方案

我们实现了 `use_resource_with_tracking`，它允许在 async block 内读取 signal 并正确追踪依赖。

### 实现原理

通过深入分析 sig 的依赖收集机制，我们发现：

1. **依赖收集通过 effect stack**：`ReactiveState.effects: Vec<Rc<RefCell<Effect>>>`
2. **Signal.read() 的机制**：检查 `state.effects.last()`，如果有值就建立订阅
3. **关键问题**：在 async block 内部 poll 时，effect stack 是空的！

**解决方案**：创建 `ReactiveContextFuture` wrapper，在每次 `Future::poll()` 时：
- Poll 前：将 effect 推到 stack 上
- 执行 poll：此时任何 signal.read() 都能追踪到
- Poll 后：从 stack 弹出 effect

```rust
impl<F: Future> Future for ReactiveContextFuture<F> {
    type Output = F::Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 1. Push effect onto stack
        super::runtime::with_reactive_mut(|state| {
            state.effects.push(effect_rc.clone());
        });
        
        // 2. Poll - signal reads are now tracked!
        let result = this.inner.as_mut().poll(cx);
        
        // 3. Pop effect from stack
        super::runtime::with_reactive_mut(|state| {
            state.effects.pop();
        });
        
        result
    }
}
```

## 使用示例

### 方式 1：use_resource（原有方式）

```rust
let user_id = Signal::new(1);

// 必须在 async block 前读取
let user_data = use_resource(move || {
    let id = *user_id.read();  // ← 必须在这里读取
    async move {
        fetch_user(id).await
    }
});
```

### 方式 2：use_resource_with_tracking（新方式）✨

```rust
let user_id = Signal::new(1);

// ✅ 可以在 async block 内读取！
let user_data = use_resource_with_tracking(move || async move {
    let id = *user_id.read();  // ← 可以在这里读取！
    fetch_user(id).await
});
```

## 对比总结

| 特性 | Dioxus | sig (原有) | sig (新增) |
|------|--------|-----------|------------|
| **async block 内读取** | ✅ 支持 | ❌ 不支持 | ✅ 支持 |
| **API** | `use_resource` | `use_resource` | `use_resource_with_tracking` |
| **追踪时机** | Future poll 时 | effect 运行时 | Future poll 时 |
| **实现方式** | ReactiveContext + poll wrapper | effect 内直接调用 | ReactiveContextFuture wrapper |
| **性能** | 每次 poll 有小开销 | 最优 | 每次 poll 有小开销 |
| **使用体验** | 自然 | 需要记住规则 | 自然 |

## 何时使用哪个 API？

### 使用 `use_resource`（原有方式）

✅ **推荐用于**：
- 简单场景，依赖在外部读取更清晰
- 性能敏感的场景
- 不需要在 async 内部读取 signal

```rust
let count = Signal::new(0);
let resource = use_resource(move || {
    let n = *count.read();  // 清晰地看到依赖
    async move {
        expensive_computation(n).await
    }
});
```

### 使用 `use_resource_with_tracking`（新方式）

✅ **推荐用于**：
- 需要在 async block 内读取 signal
- 多个 signal 交织在异步逻辑中
- 更符合直觉的编程风格

```rust
let a = Signal::new(1);
let b = Signal::new(2);

let resource = use_resource_with_tracking(move || async move {
    let x = *a.read();
    let intermediate = some_async_call(x).await;
    let y = *b.read();  // 可以在中间读取
    final_result(intermediate, y).await
});
```

## 实现细节

### ReactiveContextFuture

```rust
pub struct ReactiveContextFuture<F> {
    inner: Pin<Box<F>>,
    effect_rc: Rc<RefCell<Effect>>,
}

impl<F: Future> Future for ReactiveContextFuture<F> {
    type Output = F::Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 在 poll 时维持 effect context
        super::runtime::with_reactive_mut(|state| {
            state.effects.push(effect_rc.clone());
        });
        
        let result = this.inner.as_mut().poll(cx);
        
        super::runtime::with_reactive_mut(|state| {
            state.effects.pop();
        });
        
        result
    }
}
```

### use_resource_with_tracking

```rust
pub fn use_resource_with_tracking<T, F>(mut future_fn: impl FnMut() -> F + 'static) -> Resource<T>
where
    T: 'static,
    F: std::future::Future<Output = T> + 'static,
{
    // 存储当前 effect_rc，供 future 使用
    let effect_rc = Rc::new(RefCell::new(None));
    let effect_rc_for_future = effect_rc.clone();
    
    create_effect(move || {
        // 获取当前运行的 effect
        let current_effect = with_reactive(|state| {
            state.effects.last().cloned()
        });
        
        // 存储供 future 使用
        *effect_rc.borrow_mut() = current_effect.clone();
        
        // 创建 future 并用 ReactiveContextFuture 包装
        let fut = future_fn();
        let wrapped = ReactiveContextFuture::new(fut, current_effect.unwrap());
        
        // spawn 并执行
        spawn(wrapped);
    });
}
```

## 性能考虑

### ReactiveContextFuture 的开销

每次 poll 时：
- 2 次 `with_reactive_mut` 调用（push + pop）
- Effect stack 的修改

**实际影响**：
- ✅ 对于 IO 密集型任务：可以忽略（相比网络/文件 IO）
- ✅ 对于大多数场景：不影响体验
- ⚠️ 对于极高频 poll 的场景：可考虑使用原有 `use_resource`

### 测试结果

所有测试通过：
- ✅ `test_use_resource_with_tracking_basic`
- ✅ `test_use_resource_with_tracking_multiple_signals`
- ✅ `test_comparison_old_vs_new`
- ✅ 原有的所有 `use_resource` 测试

## 总结

**限制已解除！** 🎉

- 实现了 `use_resource_with_tracking`，支持在 async block 内读取 signal
- 通过 `ReactiveContextFuture` wrapper 在 poll 时维持 effect context
- 提供两种 API，按需选择：
  - `use_resource`：原有方式，性能最优
  - `use_resource_with_tracking`：新方式，更灵活

现在你可以像 Dioxus 一样，在 async block 内自由读取 signal 了！
