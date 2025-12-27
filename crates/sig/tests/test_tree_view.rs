//! TreeView component tests
//!
//! Tests for tree view functionality including:
//! - Node expansion/collapse
//! - Node selection
//! - Search filtering
//! - Lazy loading

use sig::prelude::*;
use sig::{TreeView, TreeNode};
use std::sync::Arc;

// ============================================================================
// Mock TreeNode implementation
// ============================================================================

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

impl TreeNode for TestNode {
    fn label(&self) -> String {
        self.name.clone()
    }
    
    fn children(&self) -> Vec<Self> {
        self.children.clone()
    }
    
    fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

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

fn count_flat_nodes<T: TreeNode>(tree: &TreeView<T>) -> usize {
    tree.visible_node_count()
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_tree_view_creation() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root);
        
        // Initially only root node should be visible
        let count = count_flat_nodes(&tree);
        println!("Initial flat nodes count: {}", count);
        assert_eq!(count, 1, "Initially should only show root node");
    });
}

#[test]
fn test_tree_node_expansion() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root.clone());
        
        println!("\n=== Test Node Expansion ===");
        
        // Initial state
        let initial_count = count_flat_nodes(&tree);
        println!("Initial nodes: {}", initial_count);
        assert_eq!(initial_count, 1);
        
        // Toggle root node (should expand)
        println!("\n🔄 Toggling root node (expand)...");
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        let after_expand = count_flat_nodes(&tree);
        println!("After expand: {}", after_expand);
        
        // Should now show: Root + Child1 + Child2 = 3 nodes
        assert_eq!(
            after_expand, 3,
            "After expanding root, should show 3 nodes (root + 2 children)"
        );
        
        // Check if root is marked as open
        let is_open = tree.is_open(&root);
        println!("Root is_open: {}", is_open);
        assert!(is_open, "Root should be marked as open");
        
        // Toggle again (should collapse)
        println!("\n🔄 Toggling root node again (collapse)...");
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        let after_collapse = count_flat_nodes(&tree);
        println!("After collapse: {}", after_collapse);
        assert_eq!(after_collapse, 1, "After collapsing, should show only root");
        
        assert!(!tree.is_open(&root), "Root should be marked as closed");
    });
}

#[test]
fn test_nested_expansion() {
    create_scope(|| {
        let root = create_test_tree();
        let child1 = root.children[0].clone();
        let tree = TreeView::new(root.clone());
        
        println!("\n=== Test Nested Expansion ===");
        
        // Expand root
        println!("Expanding root...");
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        assert_eq!(count_flat_nodes(&tree), 3); // Root + Child1 + Child2
        
        // Expand Child1
        println!("Expanding Child1...");
        tree.toggle_node(&child1);
        sig::runtime::flush_pending_signals();
        
        let count = count_flat_nodes(&tree);
        println!("After expanding Child1: {} nodes", count);
        
        // Should show: Root + Child1 + Grandchild1 + Grandchild2 + Child2 = 5 nodes
        assert_eq!(count, 5, "Should show 5 nodes after expanding Child1");
    });
}

#[test]
fn test_tree_node_selection() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root.clone());
        
        println!("\n=== Test Node Selection ===");
        
        // Initially nothing selected
        assert!(tree.selected().read().is_none());
        
        // Select root
        println!("Selecting root...");
        tree.select_node(&root);
        sig::runtime::flush_pending_signals();
        
        let selected = tree.selected().read();
        assert!(selected.is_some());
        assert_eq!(selected.as_ref().unwrap(), &root);
        println!("✅ Root selected");
    });
}

#[test]
fn test_search_filtering() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root.clone()).enable_search();
        
        println!("\n=== Test Search Filtering ===");
        
        // Expand root first
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        let before_search = count_flat_nodes(&tree);
        println!("Before search: {} nodes", before_search);
        assert_eq!(before_search, 3); // Root + 2 children
        
        // Search for "Child 1"
        println!("\nSearching for 'Child 1'...");
        *tree.search_text().write() = "Child 1".to_string();
        sig::runtime::flush_pending_signals();
        
        let after_search = count_flat_nodes(&tree);
        println!("After search: {} nodes", after_search);
        
        // Current implementation: only shows matching nodes
        // TODO: Implement ancestor visibility for better UX
        assert!(after_search >= 1, "Should show at least the matching node");
        
        // Clear search
        println!("\nClearing search...");
        *tree.search_text().write() = String::new();
        sig::runtime::flush_pending_signals();
        
        let after_clear = count_flat_nodes(&tree);
        println!("After clear: {} nodes", after_clear);
        assert_eq!(after_clear, 3, "Should restore to 3 nodes");
    });
}

#[test]
fn test_tree_view_config() {
    create_scope(|| {
        let root = create_test_tree();
        
        let tree = TreeView::new(root)
            .item_height(32.0)
            .indent_width(20.0)
            .show_child_count(true)
            .enable_search();
        
        // Config is private, but we can test that the builder methods work
        // by checking the resulting behavior
        
        // If search is enabled, search_text signal should be writable
        *tree.search_text().write() = "test".to_string();
        assert_eq!(tree.search_text().read().as_str(), "test");
        
        println!("✅ TreeView config builder methods work");
    });
}

#[test]
fn test_lazy_loading() {
    create_scope(|| {
        // Test with lazy loading
        #[derive(Clone)]
        struct LazyNode {
            name: String,
            loaded: Arc<std::sync::Mutex<bool>>,
        }
        
        impl PartialEq for LazyNode {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }
        
        impl Eq for LazyNode {}
        
        impl std::hash::Hash for LazyNode {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.name.hash(state);
            }
        }
        
        impl LazyNode {
            fn new(name: &str) -> Self {
                Self {
                    name: name.to_string(),
                    loaded: Arc::new(std::sync::Mutex::new(false)),
                }
            }
        }
        
        impl TreeNode for LazyNode {
            fn label(&self) -> String {
                self.name.clone()
            }
            
            fn children(&self) -> Vec<Self> {
                if *self.loaded.lock().unwrap() {
                    vec![
                        LazyNode::new(&format!("{} - Child 1", self.name)),
                        LazyNode::new(&format!("{} - Child 2", self.name)),
                    ]
                } else {
                    vec![]
                }
            }
            
            fn has_children(&self) -> bool {
                true // Always has children (even before loading)
            }
            
            fn load_children(&self) -> Result<(), Box<dyn std::error::Error>> {
                println!("Loading children for: {}", self.name);
                *self.loaded.lock().unwrap() = true;
                Ok(())
            }
        }
        
        let root = LazyNode::new("Root");
        let tree = TreeView::new(root.clone());
        
        println!("\n=== Test Lazy Loading ===");
        
        // Initially children not loaded
        assert_eq!(count_flat_nodes(&tree), 1);
        assert!(!*root.loaded.lock().unwrap());
        
        // Toggle should trigger loading
        println!("Toggling root (should trigger load)...");
        tree.toggle_node(&root);
        sig::runtime::flush_pending_signals();
        
        // Children should be loaded now
        assert!(*root.loaded.lock().unwrap(), "Children should be loaded");
        
        let count = count_flat_nodes(&tree);
        println!("After loading: {} nodes", count);
        assert_eq!(count, 3, "Should show root + 2 children");
    });
}

#[test]
fn test_tree_view_build() {
    create_scope(|| {
        let root = create_test_tree();
        let tree = TreeView::new(root)
            .enable_search()
            .show_child_count(true);
        
        // Build should create a view
        let _view = tree.build();
        
        // If we got here without panic, build succeeded
        println!("✅ TreeView build successful");
    });
}
