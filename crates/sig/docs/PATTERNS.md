# Advanced Patterns

Design patterns and best practices for building Sig applications.

## Reactivity Patterns

### Pattern: Responsive UI Updates

**Problem**: UI elements don't update when state changes.

**Cause**: Reading signal value once instead of subscribing to changes.

```rust
// ❌ Static - read once, never updates
let is_open = state.read().clone();
view().child(
    text(if is_open { "Open" } else { "Closed" })
)

// ✅ Reactive - subscribes to changes
view().child(
    dynamic(move || {
        let is_open = state.read();
        text(if *is_open { "Open" } else { "Closed" })
    })
)
```

**Real Example: TreeView Expand Icon**

```rust
// Icon should update when node expands/collapses
view().child(
    dynamic(move || {
        let is_open = open_states
            .read()
            .get(&node_key)
            .copied()
            .unwrap_or(false);
        
        text(if is_open { "▼" } else { "▶" })
    })
)
```

### Pattern: Computed Signals

Create derived state from multiple signals:

```rust
let first_name = Signal::new("John".to_string());
let last_name = Signal::new("Doe".to_string());

// Computed full name
dynamic(move || {
    let full = format!("{} {}", 
        first_name.read(), 
        last_name.read()
    );
    text(full)
})
```

### Pattern: Effect Setup Timing

**Problem**: Effects set up in `build()` miss early signal changes.

**Solution**: Set up effects in constructor.

```rust
// ❌ Effect runs too late
impl MyComponent {
    pub fn build(self) -> View {
        create_effect(move || {
            // Won't catch changes before build()
        });
    }
}

// ✅ Effect active immediately
impl MyComponent {
    pub fn new(data: Signal<Vec<T>>) -> Self {
        let component = Self { data: data.clone() };
        
        let comp = component.clone();
        create_effect(move || {
            let _ = data.read();
            comp.on_data_change();
        });
        
        component
    }
}
```

## Component Lifecycle Patterns

### Pattern: Preserving Component State

**Problem**: Component recreated on every update, losing scroll position.

```rust
// ❌ Wrong - new component instance each time
dynamic(move || {
    let data = items.read().clone();
    VirtualList::new(Signal::new(data))  // New instance!
        .build(|item| render(item))
})
```

**Solution**: Create once, update via Signal.

```rust
// ✅ Correct - single instance, updates reactively
let items = Signal::new(vec![]);

VirtualList::new(items.clone())
    .build(|item| render(item))

// Later: update data
*items.write() = new_data;  // Preserves scroll position
```

**Benefits:**
- Scroll position preserved
- Selection state maintained
- Better performance
- Smooth UX

### Pattern: Conditional Component Creation

When you DO want to recreate:

```rust
// Different components based on mode
dynamic(move || {
    match *mode.read() {
        Mode::List => list_view(data.clone()),
        Mode::Grid => grid_view(data.clone()),
        Mode::Table => table_view(data.clone()),
    }
})
```

## State Management Patterns

### Pattern: Stable Node Identity with Hash

**Problem**: Pointer addresses change when objects are cloned.

```rust
// ❌ Wrong - address changes after clone
fn node_key(node: &T) -> usize {
    node as *const T as usize
}
```

**Solution**: Use Hash for stable identity.

```rust
// ✅ Correct - hash is content-based
fn node_key(node: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    node.hash(&mut hasher);
    hasher.finish()
}
```

### Pattern: Newtype Wrapper for Orphan Rules

**Problem**: Can't implement external trait for external type.

```rust
// ❌ Error: orphan rules
impl TreeNode for Arc<RwLock<Data>> {}
```

**Solution**: Create wrapper.

```rust
// ✅ Wrapper is local
#[derive(Clone)]
struct DataNode(Arc<RwLock<Data>>);

impl PartialEq for DataNode {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DataNode {}

impl Hash for DataNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl TreeNode for DataNode {
    // ... implementation
}
```

## Layout Patterns

### Pattern: Fixed Header + Scrollable Content

```rust
view()
    .style(|s| s.w_full().h_full().flex().flex_col())
    .child((
        // Header (fixed)
        view()
            .style(|s| {
                s.w_full()
                    .padding(16.0)
                    .flex_shrink(0.0)
            }),
        
        // Content (scrollable)
        view()
            .style(|s| {
                s.w_full()
                    .flex_grow(1.0)
                    .min_height(0.0)     // ← Critical!
                    .overflow_y_scroll()
            }),
    ))
```

### Pattern: Sidebar + Main Content

```rust
view()
    .style(|s| s.w_full().h_full().flex().flex_row())
    .child((
        // Sidebar
        view()
            .style(|s| {
                s.width(300.0)
                    .h_full()          // ← Not flex_grow!
                    .flex_shrink(0.0)
                    .overflow_y_scroll()
            }),
        
        // Main
        view()
            .style(|s| {
                s.flex_grow(1.0)
                    .h_full()
                    .overflow_y_scroll()
            }),
    ))
```

### Pattern: Resizable Panels

```rust
let width = Signal::new(280.0);

view()
    .style(|s| s.flex().flex_row().h_full())
    .child((
        Resizable::horizontal(width)
            .min_size(200.0)
            .max_size(600.0)
            .child(/* sidebar */),
        
        view()
            .style(|s| s.flex_grow(1.0).h_full())
            .child(/* main */),
    ))
```

## Anti-Patterns to Avoid

### ❌ Nested dynamic

```rust
// ❌ Don't nest dynamic
dynamic(move || {
    dynamic(move || text(signal.read()))
})

// ✅ One level is enough
dynamic(move || text(signal.read()))
```

### ❌ Reading signals outside dynamic

```rust
// ❌ Won't update
let value = signal.read().clone();
view().child(text(value))

// ✅ Read inside dynamic
view().child(
    dynamic(move || text(signal.read()))
)
```

### ❌ Creating components in loops

```rust
// ❌ Poor performance
for item in items {
    let list = VirtualList::new(Signal::new(vec![item]));
}

// ✅ One VirtualList for all items
VirtualList::new(Signal::new(items))
```
