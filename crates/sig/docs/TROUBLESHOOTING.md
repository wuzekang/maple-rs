# Troubleshooting Guide

Common issues and their solutions.

## Layout Issues

### Background/Border Not Filling Width

**Symptom**: Background color or border appears narrow, doesn't span full width.

**Cause**: All views are flex containers - they don't auto-fill like HTML `<div>`.

```rust
// ❌ Wrong
view().style(|s| s.padding(16.0).background(Color::WHITE))

// ✅ Correct
view().style(|s| s.w_full().padding(16.0).background(Color::WHITE))
```

**Solution**: Add `w_full()` or `h_full()` explicitly.

### Content Collapsed in flex_col

**Symptom**: Content area appears collapsed, doesn't fill vertical space.

**Cause**: Missing `min_height(0)` in nested flex layouts.

```rust
// ❌ Wrong
view().style(|s| s.flex_grow(1.0).overflow_y_scroll())

// ✅ Correct
view().style(|s| {
    s.flex_grow(1.0)
        .min_height(0.0)  // ← Critical!
        .overflow_y_scroll()
})
```

**Why?** Without `min_height(0)`, the container tries to fit its content, preventing scrolling.

### Sidebar Not Full Height in flex_row

**Symptom**: Sidebar doesn't extend to full height in horizontal layout.

**Cause**: Using `flex_grow` for height in `flex_row` (wrong axis).

```rust
// ❌ Wrong - flex_grow affects width in flex_row
view()
    .style(|s| s.flex().flex_row())
    .child(
        view().style(|s| s.width(300.0).flex_grow(1.0))
    )

// ✅ Correct - use h_full() for height
view()
    .style(|s| s.flex().flex_row())
    .child(
        view().style(|s| s.width(300.0).h_full())
    )
```

**Remember**: 
- In `flex_row`: `flex_grow` = width, `h_full()` = height
- In `flex_col`: `flex_grow` = height, `w_full()` = width

### Content Not Scrolling

**Symptom**: Overflow content is clipped or hidden.

**Checklist**:
1. ✅ Container has `overflow_y_scroll()`
2. ✅ Container has `min_height(0.0)` if in nested flex
3. ✅ Parent has `flex_grow(1.0)` to provide space
4. ✅ Container has defined height or flex layout

```rust
// Complete scrollable pattern
view()
    .style(|s| {
        s.flex_grow(1.0)       // Fill available space
            .min_height(0.0)    // Allow shrinking
            .overflow_y_scroll() // Enable scroll
    })
```

## Reactivity Issues

### UI Not Updating

**Symptom**: State changes but UI doesn't reflect them.

**Cause**: Not using `dynamic` to create reactive closures.

```rust
// ❌ Static - read once
let value = signal.read().clone();
text(value)

// ✅ Reactive - re-runs on signal change
dynamic(move || {
    text(signal.read())
})
```

### Icon/Visual Not Updating

**Symptom**: Expand/collapse icons don't update, selection highlight doesn't change.

**Cause**: Reading state outside `dynamic`.

```rust
// ❌ Static icon
let is_open = self.is_open(node);
text(if is_open { "▼" } else { "▶" })

// ✅ Reactive icon
dynamic(move || {
    let is_open = open_states.read().get(&key).copied().unwrap_or(false);
    text(if is_open { "▼" } else { "▶" })
})
```

### Effect Not Running

**Problem**: State changes but effect doesn't trigger.

**Checklist**:
1. ✅ Effect uses `signal.read()` (not `read_untracked()`)
2. ✅ Effect created before signal changes
3. ✅ Called `flush_pending_signals()` in tests
4. ✅ Effect closure captures signal, not its value

```rust
// ❌ Wrong - captures value
let value = signal.read().clone();
create_effect(move || {
    println!("{}", value);  // Won't update
});

// ✅ Correct - captures signal
create_effect(move || {
    println!("{}", signal.read());  // Updates on change
});
```

## Component State Issues

### Scroll Position Resets

**Symptom**: Scroll position jumps to top when data updates.

**Cause**: Recreating component instance in `dynamic`.

```rust
// ❌ Wrong - creates new VirtualList
dynamic(move || {
    let data = items.read();
    VirtualList::new(Signal::new(data.clone())).build(...)
})

// ✅ Correct - single instance
let items = Signal::new(vec![]);
VirtualList::new(items).build(...)
```

**Rule**: Create components once, update via Signals.

### Toggle/Expand Not Working

**Symptom**: Clicking expand button does nothing.

**Possible Causes**:

1. **Using pointer addresses as keys** (most common)
```rust
// ❌ Breaks after clone
fn node_key(node: &T) -> usize {
    node as *const T as usize
}

// ✅ Use hash
fn node_key(node: &T) -> u64 {
    use std::hash::{Hash, Hasher, DefaultHasher};
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}
```

2. **Missing `flush_pending_signals()` in tests**
```rust
tree.toggle_node(&root);
sig::runtime::flush_pending_signals();  // ← Add this
assert!(tree.is_open(&root));
```

3. **Not propagating state to component**
```rust
// Make sure toggle updates the Signal
self.open_states.write().insert(key, !is_open);
self.rebuild_flat_list();  // ← Triggers update
```

### Selection State Lost

**Symptom**: Selected item not highlighted or loses selection.

**Cause**: Not using Signal for selection state.

```rust
// ❌ Wrong - local state
let mut selected = None;

// ✅ Correct - Signal
let selected = Signal::new(None);

// Selection updates UI automatically
dynamic(move || {
    let sel = selected.read();
    // UI responds to changes
})
```

## Trait Implementation Issues

### Orphan Rules Error

**Symptom**: `error[E0117]: only traits defined in the current crate can be implemented`

**Cause**: Trying to implement external trait for external type.

```rust
// ❌ Error
impl TreeNode for Arc<RwLock<Data>> {
    // Neither TreeNode nor Arc is local
}
```

**Solution**: Create newtype wrapper.

```rust
// ✅ Wrapper is local
#[derive(Clone)]
struct DataNode(Arc<RwLock<Data>>);

impl TreeNode for DataNode {
    // This works!
}
```

### Missing Hash/Eq Implementation

**Symptom**: `the trait Hash is not implemented for ...`

**Solution**: Add required derives or manual implementations.

```rust
// For simple types
#[derive(Clone, PartialEq, Eq, Hash)]
struct MyNode { /* ... */ }

// For types with Arc/Rc
impl Hash for MyNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}
```

## Performance Issues

### Slow Rendering

**Symptom**: UI lags or stutters.

**Possible Causes**:

1. **Not using VirtualList for large lists**
```rust
// ❌ Slow for 1000+ items
items.iter().map(|item| render_item(item))

// ✅ Fast
VirtualList::new(items).build(|item| render_item(item))
```

2. **Too many dynamic blocks**
```rust
// ❌ Excessive reactivity
view().child(
    items.iter().map(|item| {
        dynamic(move || text(item))  // Each item has dynamic!
    })
)

// ✅ One dynamic for all
dynamic(move || {
    items.read().iter().map(|item| text(item))
})
```

3. **Cloning large data structures**
```rust
// ❌ Clones on every read
dynamic(move || {
    let big_vec = signal.read().clone();  // Expensive!
    // ...
})

// ✅ Clone only what you need
dynamic(move || {
    let count = signal.read().len();  // Cheap!
    text(format!("Count: {}", count))
})
```

### Memory Leaks

**Symptom**: Memory usage grows over time.

**Cause**: Not cleaning up scopes/effects.

**Solution**: Use `create_scope` and `on_cleanup`.

```rust
create_scope(|| {
    let signal = Signal::new(0);
    
    on_cleanup(|| {
        println!("Scope cleaned up");
    });
    
    // All signals/effects cleaned when scope ends
});
```

## Testing Issues

### Tests Failing Intermittently

**Cause**: Not flushing pending signals.

```rust
// ❌ May fail
*state.write() = true;
assert!(component.is_active());

// ✅ Reliable
*state.write() = true;
sig::runtime::flush_pending_signals();
assert!(component.is_active());
```

### Tests Pass but App Doesn't Work

**Cause**: Tests don't exercise the full render pipeline.

**Solution**: Run the actual app to test visual/interaction behavior.

```bash
cargo run -p your_app
```

Tests verify logic, but manual testing verifies UX.

## Diagnostic Checklist

When debugging issues, check:

```
Layout Issues:
  □ Added w_full() / h_full()?
  □ Used min_height(0) with flex_grow in nested layouts?
  □ In flex_row, using h_full() (not flex_grow) for height?

Reactivity Issues:
  □ Wrapped in dynamic()?
  □ Reading signal inside closure?
  □ Not reading signal.clone() outside?

Component Issues:
  □ Creating component once (not in dynamic)?
  □ Using Hash for node identity (not pointers)?
  □ Set up effects in constructor (not build)?

Test Issues:
  □ Wrapped in create_scope()?
  □ Called flush_pending_signals()?
  □ Using read_untracked() in assertions?
```

## Quick Fixes

| Problem | Quick Fix |
|---------|-----------|
| Background not filling | Add `w_full()` or `h_full()` |
| Content collapsed | Add `min_height(0.0)` + `flex_grow(1.0)` |
| Sidebar not full height | Use `h_full()` in flex_row |
| UI not updating | Wrap in `dynamic(move \|\| ...)` |
| Scroll position resets | Create component once, not in dynamic |
| Toggle not working | Use Hash, not pointer addresses |
| Test failing | Add `flush_pending_signals()` |
| Icon not updating | Read state inside `dynamic` |

## Getting Help

1. Check this guide first
2. Review [PATTERNS.md](PATTERNS.md) for best practices
3. Look at examples in `examples/` directory
4. Check existing tests in `tests/` directory
5. Search for similar patterns in the codebase
