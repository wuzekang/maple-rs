# Testing Guide

How to write effective tests for Sig components and applications.

## Test Setup

### Basic Test Structure

```rust
#[test]
fn test_component() {
    create_scope(|| {
        // All signals and effects cleaned up when scope ends
        let state = Signal::new(initial_value);
        let component = MyComponent::new(state);
        
        // Test assertions
        assert_eq!(component.count(), expected);
    });
}
```

**Key Rule**: Always wrap tests in `create_scope` for proper cleanup.

### Flushing Pending Signals

After updating signals, flush pending effects:

```rust
#[test]
fn test_reactive_update() {
    create_scope(|| {
        let state = Signal::new(false);
        let component = MyComponent::new(state.clone());
        
        // Initial state
        assert_eq!(component.visible_count(), 1);
        
        // Update state
        *state.write() = true;
        sig::runtime::flush_pending_signals();  // ← Critical!
        
        // Verify update
        assert_eq!(component.visible_count(), 3);
    });
}
```

**Why?** Effects are queued and run asynchronously. `flush_pending_signals()` runs them immediately.

### Using read_untracked in Assertions

```rust
#[test]
fn test_signal_value() {
    create_scope(|| {
        let count = Signal::new(0);
        
        *count.write() = 42;
        
        // ✅ read_untracked - doesn't create dependency
        assert_eq!(*count.read_untracked(), 42);
        
        // Also OK in tests
        assert_eq!(*count.read(), 42);
    });
}
```

## Test Patterns

### Pattern: Test Data Structures

Create simple, hashable test data:

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TestNode {
    id: usize,
    name: String,
    children: Vec<TestNode>,
}

impl TestNode {
    fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            children: vec![],
        }
    }
    
    fn with_children(mut self, children: Vec<TestNode>) -> Self {
        self.children = children;
        self
    }
}
```

### Pattern: Test Helpers

```rust
fn create_test_tree() -> TestNode {
    TestNode::new(0, "Root")
        .with_children(vec![
            TestNode::new(1, "Child 1")
                .with_children(vec![
                    TestNode::new(3, "Grandchild 1"),
                    TestNode::new(4, "Grandchild 2"),
                ]),
            TestNode::new(2, "Child 2"),
        ])
}

fn assert_tree_state(tree: &TreeView<TestNode>, expected_count: usize) {
    assert_eq!(
        tree.visible_node_count(), 
        expected_count,
        "Expected {} visible nodes",
        expected_count
    );
}
```

### Pattern: Testing TreeView

```rust
#[test]
fn test_tree_expansion() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root.clone());
        
        // Initially only root visible
        assert_eq!(tree.visible_node_count(), 1);
        
        // Expand root
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        // Now root + children visible
        assert_eq!(tree.visible_node_count(), 3);
        
        // Check if expanded
        assert!(tree.is_open(&root));
    });
}
```

### Pattern: Testing Search

```rust
#[test]
fn test_search_filtering() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root.clone())
            .enable_search();
        
        // Expand first
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        assert_eq!(tree.visible_node_count(), 3);
        
        // Apply search
        *tree.search_text().write() = "Child 1".to_string();
        sig::runtime::flush_pending_signals();
        
        // Filtered results
        let filtered_count = tree.visible_node_count();
        assert!(filtered_count < 3);
    });
}
```

### Pattern: Testing Lazy Loading

```rust
#[test]
fn test_lazy_loading() {
    create_scope(|| {
        #[derive(Clone)]
        struct LazyNode {
            name: String,
            loaded: Arc<Mutex<bool>>,
        }
        
        impl PartialEq for LazyNode {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }
        
        impl Eq for LazyNode {}
        
        impl Hash for LazyNode {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.name.hash(state);
            }
        }
        
        impl TreeNode for LazyNode {
            fn load_children(&self) -> Result<(), Box<dyn Error>> {
                *self.loaded.lock().unwrap() = true;
                Ok(())
            }
            // ... other methods
        }
        
        let root = LazyNode::new("Root");
        let tree = TreeView::new(root.clone());
        
        // Initially not loaded
        assert!(!*root.loaded.lock().unwrap());
        
        // Expand triggers loading
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        // Now loaded
        assert!(*root.loaded.lock().unwrap());
    });
}
```

## Testing Layout

### Pattern: Layout Pipeline Test

```rust
#[test]
fn test_layout() {
    create_scope(|| {
        let root = view()
            .style(|s| s.width(800.0).height(600.0))
            .child(/* ... */);
        
        // Critical: Flush signals to sync children
        sig::runtime::flush_pending_signals();
        
        // Compute layout
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(800.0),
            height: taffy::AvailableSpace::Definite(600.0),
        };
        let ctx = sig::RenderContext::new(1.0, (800.0, 600.0));
        root.id.compute_layout(available, &ctx);
        
        // Verify
        sig::runtime::with_layout(|runtime| {
            let layout = runtime.taffy.layout(root.id.node_id()).unwrap();
            assert_eq!(layout.size.width, 800.0);
        });
    });
}
```

## Common Test Utilities

### Helper: Count Visible Nodes

```rust
fn count_flat_nodes<T: TreeNode>(tree: &TreeView<T>) -> usize {
    tree.visible_node_count()
}
```

### Helper: Print Debug Info

```rust
fn print_tree_state<T: TreeNode>(tree: &TreeView<T>, label: &str) {
    println!("\n{}: {} nodes", label, tree.visible_node_count());
    if let Some(node) = tree.selected().read().as_ref() {
        println!("  Selected: {}", node.label());
    }
}
```

## Testing Checklist

- [ ] Wrap all tests in `create_scope`
- [ ] Call `flush_pending_signals()` after state updates
- [ ] Use `read_untracked()` in assertions to avoid side effects
- [ ] Test edge cases (empty, single item, large data)
- [ ] Test reactive updates (signals trigger UI updates)
- [ ] Test lazy loading (if applicable)
- [ ] Test search/filter (if applicable)
- [ ] Use `--nocapture` flag to see println output

## Example: Complete Test Suite

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test data
    fn create_test_data() -> Vec<Item> {
        vec![
            Item { id: 1, name: "First".into() },
            Item { id: 2, name: "Second".into() },
        ]
    }
    
    #[test]
    fn test_creation() {
        create_scope(|| {
            let items = Signal::new(create_test_data());
            let component = MyComponent::new(items);
            assert_eq!(component.count(), 2);
        });
    }
    
    #[test]
    fn test_update() {
        create_scope(|| {
            let items = Signal::new(create_test_data());
            let component = MyComponent::new(items.clone());
            
            *items.write() = vec![];
            sig::runtime::flush_pending_signals();
            
            assert_eq!(component.count(), 0);
        });
    }
    
    #[test]
    fn test_selection() {
        create_scope(|| {
            let component = MyComponent::new(/* ... */);
            let item = create_test_data()[0].clone();
            
            component.select(&item);
            sig::runtime::flush_pending_signals();
            
            assert_eq!(
                component.selected().read().as_ref(),
                Some(&item)
            );
        });
    }
}
```
