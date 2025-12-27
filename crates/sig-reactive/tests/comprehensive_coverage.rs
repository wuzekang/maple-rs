use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Signal Tests
// ============================================================================

#[test]
fn test_signal_read_untracked() {
    create_scope(|| {
        let signal = Signal::new(42);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            // read_untracked should NOT create a dependency
            let _value = *signal.read_untracked();
            *counter_clone.borrow_mut() += 1;
        });

        // Effect runs once during creation
        assert_eq!(*counter.borrow(), 1);

        // Update signal - effect should NOT run because we used read_untracked
        *signal.write() = 100;
        assert_eq!(*counter.borrow(), 1); // Still 1
    });
}

#[test]
fn test_signal_set_if_changed() {
    create_scope(|| {
        let signal = Signal::new(10);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let _value = *signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        // Clear initial run
        *counter.borrow_mut() = 0;

        // Set to same value - should NOT trigger effect
        signal.set_if_changed(10);
        assert_eq!(*counter.borrow(), 0);

        // Set to different value - should trigger effect
        signal.set_if_changed(20);
        assert_eq!(*counter.borrow(), 1);
    });
}

#[test]
fn test_signal_set_untracked() {
    create_scope(|| {
        let signal = Signal::new(5);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let _value = *signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        // Clear initial run
        *counter.borrow_mut() = 0;

        // set_untracked should NOT trigger effect
        signal.set_untracked(999);
        assert_eq!(*counter.borrow(), 0);

        // Verify the value was actually changed
        assert_eq!(*signal.read_untracked(), 999);
    });
}

#[test]
fn test_signal_id() {
    create_scope(|| {
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(2);

        let id1 = signal1.signal_id();
        let id2 = signal2.signal_id();

        // IDs should be different
        assert_ne!(id1, id2);

        // Same signal should have same ID
        assert_eq!(signal1.signal_id(), id1);
    });
}

#[test]
fn test_read_guard_as_ref() {
    create_scope(|| {
        let signal = Signal::new(String::from("hello"));
        let guard = signal.read();
        
        // Test AsRef trait
        let str_ref: &str = guard.as_ref();
        assert_eq!(str_ref, "hello");
    });
}

// ============================================================================
// Runtime/Scope Tests
// ============================================================================

#[test]
fn test_untrack() {
    create_scope(|| {
        let signal = Signal::new(1);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            // Read with untrack - should NOT create dependency
            let _value = untrack(|| *signal.read());
            *counter_clone.borrow_mut() += 1;
        });

        // Effect runs once
        assert_eq!(*counter.borrow(), 1);

        // Update signal - effect should NOT run
        *signal.write() = 100;
        assert_eq!(*counter.borrow(), 1);
    });
}

#[test]
fn test_on_cleanup() {
    let cleanup_called = Rc::new(RefCell::new(false));
    let cleanup_called_clone = cleanup_called.clone();

    create_scope(|| {
        on_cleanup(move || {
            *cleanup_called_clone.borrow_mut() = true;
        });
    });

    // Cleanup should be called when scope is destroyed
    assert!(*cleanup_called.borrow());
}

#[test]
fn test_on_cleanup_in_effect() {
    create_scope(|| {
        let signal = Signal::new(1);
        let cleanup_count = Rc::new(RefCell::new(0));
        let cleanup_count_clone = cleanup_count.clone();

        create_effect(move || {
            let _value = *signal.read();
            let cleanup_count_clone2 = cleanup_count_clone.clone();
            on_cleanup(move || {
                *cleanup_count_clone2.borrow_mut() += 1;
            });
        });

        // No cleanup called yet
        assert_eq!(*cleanup_count.borrow(), 0);

        // Update signal - should trigger cleanup from previous run
        *signal.write() = 2;
        assert_eq!(*cleanup_count.borrow(), 1);

        // Update again
        *signal.write() = 3;
        assert_eq!(*cleanup_count.borrow(), 2);
    });
}

#[test]
fn test_scope_hierarchy() {
    create_scope(|| {
        let parent_cleanup = Rc::new(RefCell::new(false));
        let child_cleanup = Rc::new(RefCell::new(false));
        
        let parent_cleanup_clone = parent_cleanup.clone();
        let child_cleanup_clone = child_cleanup.clone();

        on_cleanup(move || {
            *parent_cleanup_clone.borrow_mut() = true;
        });

        create_scope(|| {
            on_cleanup(move || {
                *child_cleanup_clone.borrow_mut() = true;
            });
        });

        // Child cleanup should be called when inner scope ends
        assert!(*child_cleanup.borrow());
        // Parent cleanup not called yet
        assert!(!*parent_cleanup.borrow());
    });
}

// Test removed - RUNTIME and internal APIs are private
// We test context through public APIs in other tests

// Test removed - internal runtime API

// Test removed - internal runtime API

#[test]
fn test_as_child_scope() {
    create_scope(|| {
        let cleanup_count = Rc::new(RefCell::new(0));
        let cleanup_count_clone = cleanup_count.clone();

        let child_fn = as_child_scope(move |x: i32| {
            let cleanup_count_clone2 = cleanup_count_clone.clone();
            on_cleanup(move || {
                *cleanup_count_clone2.borrow_mut() += 1;
            });
            x * 2
        });

        // Call child function multiple times
        let (result1, _scope1) = child_fn(10);
        assert_eq!(result1, 20);

        let (result2, _scope2) = child_fn(20);
        assert_eq!(result2, 40);

        // Note: scopes are managed internally and cleaned up when parent scope ends
    });
}

// ============================================================================
// Effect Tests
// ============================================================================

#[test]
fn test_effect_cleanup_on_rerun() {
    create_scope(|| {
        let signal = Signal::new(1);
        let cleanup_log = Rc::new(RefCell::new(Vec::new()));
        let cleanup_log_clone = cleanup_log.clone();

        create_effect(move || {
            let value = *signal.read();
            let cleanup_log_clone2 = cleanup_log_clone.clone();
            
            on_cleanup(move || {
                cleanup_log_clone2.borrow_mut().push(format!("cleanup: {}", value));
            });
        });

        // No cleanup yet
        assert_eq!(cleanup_log.borrow().len(), 0);

        // Update signal - previous cleanup should run
        *signal.write() = 2;
        assert_eq!(cleanup_log.borrow().len(), 1);
        assert_eq!(cleanup_log.borrow()[0], "cleanup: 1");

        // Update again
        *signal.write() = 3;
        assert_eq!(cleanup_log.borrow().len(), 2);
        assert_eq!(cleanup_log.borrow()[1], "cleanup: 2");
    });
}

#[test]
fn test_multiple_signals_in_effect() {
    create_scope(|| {
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(10);
        let signal3 = Signal::new(100);
        
        let sum = Rc::new(RefCell::new(0));
        let sum_clone = sum.clone();

        create_effect(move || {
            let v1 = *signal1.read();
            let v2 = *signal2.read();
            let v3 = *signal3.read();
            *sum_clone.borrow_mut() = v1 + v2 + v3;
        });

        assert_eq!(*sum.borrow(), 111);

        // Update any signal should trigger effect
        *signal1.write() = 5;
        assert_eq!(*sum.borrow(), 115);

        *signal2.write() = 50;
        assert_eq!(*sum.borrow(), 155);

        *signal3.write() = 500;
        assert_eq!(*sum.borrow(), 555);
    });
}

#[test]
fn test_conditional_signal_reading() {
    create_scope(|| {
        let condition = Signal::new(true);
        let signal_a = Signal::new(1);
        let signal_b = Signal::new(100);
        
        let result = Rc::new(RefCell::new(0));
        let result_clone = result.clone();

        create_effect(move || {
            if *condition.read() {
                *result_clone.borrow_mut() = *signal_a.read();
            } else {
                *result_clone.borrow_mut() = *signal_b.read();
            }
        });

        assert_eq!(*result.borrow(), 1);

        // Update signal_a - should trigger
        *signal_a.write() = 10;
        assert_eq!(*result.borrow(), 10);

        // Update signal_b - should NOT trigger (not being read)
        *signal_b.write() = 200;
        assert_eq!(*result.borrow(), 10);

        // Switch condition
        *condition.write() = false;
        assert_eq!(*result.borrow(), 200);

        // Now signal_b updates should trigger
        *signal_b.write() = 300;
        assert_eq!(*result.borrow(), 300);

        // signal_a updates should NOT trigger anymore
        *signal_a.write() = 99;
        assert_eq!(*result.borrow(), 300);
    });
}

#[test]
fn test_signal_write_guard_multiple_modifications() {
    create_scope(|| {
        let signal = Signal::new(vec![1, 2, 3]);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let _value = signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        *counter.borrow_mut() = 0;

        // Multiple modifications in same write guard should only trigger once
        {
            let mut guard = signal.write();
            guard.push(4);
            guard.push(5);
            guard.push(6);
        } // Effect triggers on drop

        assert_eq!(*counter.borrow(), 1);
        assert_eq!(signal.read().len(), 6);
    });
}

#[test]
fn test_deeply_nested_scopes() {
    let cleanup_order = Rc::new(RefCell::new(Vec::new()));
    let c1 = cleanup_order.clone();
    let c2 = cleanup_order.clone();
    let c3 = cleanup_order.clone();

    create_scope(move || {
        let c1_clone = c1.clone();
        on_cleanup(move || c1_clone.borrow_mut().push("level1"));

        create_scope(move || {
            let c2_clone = c2.clone();
            on_cleanup(move || c2_clone.borrow_mut().push("level2"));

            create_scope(move || {
                let c3_clone = c3.clone();
                on_cleanup(move || c3_clone.borrow_mut().push("level3"));
            });
        });
    });

    // Cleanups should be called in LIFO order
    let order = cleanup_order.borrow();
    assert_eq!(order.len(), 3);
    assert_eq!(order[0], "level3");
    assert_eq!(order[1], "level2");
    assert_eq!(order[2], "level1");
}

#[test]
fn test_effect_with_no_dependencies() {
    create_scope(|| {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        // Effect with no signal reads
        create_effect(move || {
            *counter_clone.borrow_mut() += 1;
        });

        // Should run once during creation
        assert_eq!(*counter.borrow(), 1);

        // Create a signal and update it - should NOT trigger effect
        let signal = Signal::new(10);
        *signal.write() = 20;
        
        assert_eq!(*counter.borrow(), 1); // Still 1
    });
}

#[test]
fn test_signal_data_creation() {
    create_scope(|| {
        let signal1 = Signal::new("hello");
        let signal2 = Signal::new(42);
        let signal3 = Signal::new(vec![1, 2, 3]);

        assert_eq!(*signal1.read(), "hello");
        assert_eq!(*signal2.read(), 42);
        assert_eq!(*signal3.read(), vec![1, 2, 3]);
    });
}

#[test]
fn test_write_guard_deref_mut() {
    create_scope(|| {
        let signal = Signal::new(String::from("hello"));
        
        {
            let mut guard = signal.write();
            guard.push_str(" world");
            assert_eq!(*guard, "hello world");
        }

        assert_eq!(*signal.read(), "hello world");
    });
}

#[test]
fn test_signal_clone_and_copy() {
    create_scope(|| {
        let signal1 = Signal::new(100);
        let signal2 = signal1; // Copy
        let signal3 = signal1.clone(); // Clone

        // All should point to the same signal
        assert_eq!(signal1.signal_id(), signal2.signal_id());
        assert_eq!(signal1.signal_id(), signal3.signal_id());

        *signal1.write() = 200;
        assert_eq!(*signal2.read(), 200);
        assert_eq!(*signal3.read(), 200);
    });
}

// ============================================================================
// Edge Cases and Error Scenarios
// ============================================================================

#[test]
fn test_empty_scope() {
    // Create and immediately destroy scope
    create_scope(|| {
        // Do nothing
    });
}

#[test]
fn test_effect_updates_unrelated_signal() {
    create_scope(|| {
        let signal_a = Signal::new(1);
        let signal_b = Signal::new(100);
        
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        // Effect reads A, writes B
        create_effect(move || {
            let value_a = *signal_a.read();
            *counter_clone.borrow_mut() += 1;
            *signal_b.write() = value_a * 10;
        });

        *counter.borrow_mut() = 0;

        // Update A - should trigger effect and update B
        *signal_a.write() = 5;
        assert_eq!(*counter.borrow(), 1);
        assert_eq!(*signal_b.read(), 50);
    });
}

// Test removed - internal runtime API

// Test removed - internal runtime API

// Test removed - internal runtime API

// Test removed - Effect::scope_id is private

#[test]
fn test_signalid_ordering() {
    let id1 = SignalId::new();
    let id2 = SignalId::new();
    let id3 = SignalId::new();

    // SignalIds should be ordered by creation
    assert!(id1 < id2);
    assert!(id2 < id3);
    assert!(id1 < id3);

    // Test equality
    assert_eq!(id1, id1);
    assert_ne!(id1, id2);
}

#[test]
fn test_scopeid_root_constant() {
    let root = ScopeId::ROOT;
    assert_eq!(root.0, 1);
}
