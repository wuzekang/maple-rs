//! Tests for dirty nodes cleanup and memory management
//!
//! These tests verify that:
//! 1. Dirty nodes don't accumulate when repeatedly updating signals with same values
//! 2. Nested dynamic components are properly cleaned up
//! 3. VirtualList items are cleaned up correctly
//! 4. set_if_changed provides expected performance benefits

use sig::prelude::*;
use sig::{dynamic, runtime, VirtualList, Node};

/// Helper function to get current dirty nodes and view states count
fn get_stats() -> (usize, usize) {
    runtime::with_layout(|rt| {
        (rt.style_dirty_nodes.len(), rt.view_states.len())
    })
}

#[test]
fn test_basic_dynamic_no_accumulation() {
    create_scope(move || {
        let data = Signal::new(vec![1, 2, 3]);

        let _view = dynamic(move || {
            let items = data.read().clone();
            
            view()
                .style(|s| s.flex().flex_col())
                .child(
                    items.iter().map(|num| {
                        view()
                            .style(|s| s.width(100.0).height(20.0))
                            .child(text(format!("Item {}", num)))
                    }).collect::<Vec<_>>()
                )
        });

        let (initial_dirty, initial_views) = get_stats();

        // Write the SAME value 10 times
        for _ in 0..10 {
            *data.write() = vec![1, 2, 3];
            runtime::flush_pending_signals();
        }

        let (final_dirty, final_views) = get_stats();

        // With proper cleanup, counts should stay stable
        assert_eq!(
            initial_dirty, final_dirty,
            "Dirty nodes should not accumulate when writing same value"
        );
        assert_eq!(
            initial_views, final_views,
            "View states should not accumulate when writing same value"
        );
    });
}

#[test]
fn test_nested_dynamic_cleanup() {
    create_scope(move || {
        let counter = Signal::new(0);

        // Create a view with nested dynamics (simulating node_row structure)
        let _view = dynamic(move || {
            let count = *counter.read();
            
            view()
                .style(|s| s.flex().flex_col())
                .child((
                    // First nested dynamic
                    dynamic(move || {
                        text(format!("Icon {}", count))
                            .style(|s| s.font_size(10.0))
                    }),
                    // Second nested dynamic
                    dynamic(move || {
                        text(format!("Name {}", count))
                            .style(|s| s.font_size(14.0))
                    }),
                ))
        });

        let (initial_dirty, initial_views) = get_stats();

        // Increment counter 20 times (forces Dynamic rebuild each time)
        for i in 1..=20 {
            *counter.write() = i;
            runtime::flush_pending_signals();
        }

        let (final_dirty, final_views) = get_stats();

        // Counts should stay stable (nested scopes properly cleaned up)
        assert_eq!(
            initial_dirty, final_dirty,
            "Dirty nodes should not accumulate with nested dynamics"
        );
        assert_eq!(
            initial_views, final_views,
            "View states should not accumulate with nested dynamics"
        );
    });
}

#[test]
fn test_virtuallist_with_set_if_changed() {
    #[derive(Clone, Debug, PartialEq)]
    struct TreeNode {
        id: usize,
        depth: usize,
        name: String,
    }

    create_scope(move || {
        let open_states = Signal::new(std::collections::HashMap::new());
        
        let flat_nodes = Signal::new(vec![
            TreeNode { id: 0, depth: 0, name: "Root".to_string() },
            TreeNode { id: 1, depth: 1, name: "Child 1".to_string() },
            TreeNode { id: 2, depth: 1, name: "Child 2".to_string() },
            TreeNode { id: 3, depth: 2, name: "Grandchild 1".to_string() },
        ]);

        // Effect rebuilds flat_nodes when open_states changes
        let flat_nodes_for_effect = flat_nodes.clone();
        let open_states_for_effect = open_states.clone();
        create_effect(move || {
            let _states = open_states_for_effect.read();
            
            let new_list = vec![
                TreeNode { id: 0, depth: 0, name: "Root".to_string() },
                TreeNode { id: 1, depth: 1, name: "Child 1".to_string() },
                TreeNode { id: 2, depth: 1, name: "Child 2".to_string() },
                TreeNode { id: 3, depth: 2, name: "Grandchild 1".to_string() },
            ];
            
            // Use set_if_changed to avoid unnecessary updates
            flat_nodes_for_effect.set_if_changed(new_list);
        });

        // Create VirtualList
        let _list_view = dynamic(move || {
            let nodes = flat_nodes.read().clone();
            
            VirtualList::new(Signal::new(nodes))
                .item_height(28.0)
                .buffer_size(10)
                .build(move |node, _index| {
                    view()
                        .style(|s| s.width(200.0).height(28.0).padding(4.0))
                        .child(text(format!("{}{}", "  ".repeat(node.depth), node.name)))
                })
        });

        let (initial_dirty, initial_views) = get_stats();

        // Toggle node 0 multiple times
        for _ in 0..20 {
            let mut states = open_states.write();
            let current = states.get(&0).copied().unwrap_or(false);
            states.insert(0, !current);
            drop(states);

            runtime::flush_pending_signals();
        }

        let (final_dirty, final_views) = get_stats();

        // With set_if_changed, updates should be skipped and counts stay stable
        assert_eq!(
            initial_dirty, final_dirty,
            "VirtualList dirty nodes should not accumulate with set_if_changed"
        );
        assert_eq!(
            initial_views, final_views,
            "VirtualList view states should not accumulate with set_if_changed"
        );
    });
}

#[test]
fn test_set_if_changed_skips_updates() {
    create_scope(move || {
        let data = Signal::new(vec![1, 2, 3]);
        let update_count = std::rc::Rc::new(std::cell::Cell::new(0));
        let update_count_clone = update_count.clone();

        create_effect(move || {
            let _ = data.read();
            update_count_clone.set(update_count_clone.get() + 1);
        });

        runtime::flush_pending_signals();
        let initial_count = update_count.get();

        // Use set_if_changed with same value - should NOT trigger effect
        for _ in 0..10 {
            data.set_if_changed(vec![1, 2, 3]);
            runtime::flush_pending_signals();
        }

        assert_eq!(
            update_count.get(),
            initial_count,
            "set_if_changed should skip updates when value is unchanged"
        );

        // Use set_if_changed with different value - should trigger effect
        data.set_if_changed(vec![4, 5, 6]);
        runtime::flush_pending_signals();

        assert_eq!(
            update_count.get(),
            initial_count + 1,
            "set_if_changed should trigger update when value changes"
        );
    });
}

#[test]
fn test_scope_children_cleanup() {
    create_scope(move || {
        let trigger = Signal::new(0);

        // Create nested scopes via dynamic
        let _view = dynamic(move || {
            let _ = *trigger.read();
            
            // Outer dynamic creates a scope with inner dynamics
            let items: Vec<Node> = (0..3).map(|i| {
                dynamic(move || {
                    text(format!("Item {}", i))
                }).into_node()
            }).collect();
            
            view().child(items)
        });

        runtime::flush_pending_signals();
        let (initial_dirty, initial_views) = get_stats();

        // Trigger rebuild multiple times
        for i in 1..=10 {
            *trigger.write() = i;
            runtime::flush_pending_signals();
        }

        let (final_dirty, final_views) = get_stats();

        // All child scopes should be recursively cleaned up
        assert_eq!(
            initial_dirty, final_dirty,
            "Recursive scope cleanup should prevent dirty node accumulation"
        );
        assert_eq!(
            initial_views, final_views,
            "Recursive scope cleanup should prevent view state accumulation"
        );
    });
}
