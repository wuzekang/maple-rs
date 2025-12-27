use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Additional Edge Cases for Maximum Coverage
// ============================================================================

#[test]
fn test_effect_drops_properly() {
    create_scope(|| {
        let signal = Signal::new(10);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        {
            let _effect = create_effect(move || {
                let _value = *signal.read();
                *counter_clone.borrow_mut() += 1;
            });
            // effect goes out of scope here
        }

        // Effect has run once
        assert!(*counter.borrow() >= 1);
    });
}

#[test]
fn test_signal_with_complex_types() {
    create_scope(|| {
        #[derive(Clone, PartialEq, Debug)]
        struct ComplexData {
            values: Vec<i32>,
            name: String,
        }

        let signal = Signal::new(ComplexData {
            values: vec![1, 2, 3],
            name: "test".to_string(),
        });

        {
            let guard = signal.read();
            assert_eq!(guard.values, vec![1, 2, 3]);
            assert_eq!(guard.name, "test");
        }

        signal.set_if_changed(ComplexData {
            values: vec![1, 2, 3],
            name: "test".to_string(),
        });

        // Should not trigger (same value)
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        create_effect(move || {
            let _val = signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        *counter.borrow_mut() = 0;

        signal.set_if_changed(ComplexData {
            values: vec![1, 2, 3],
            name: "test".to_string(),
        });

        assert_eq!(*counter.borrow(), 0);
    });
}

#[test]
fn test_multiple_effects_on_same_signal_different_scopes() {
    create_scope(|| {
        let signal = Signal::new(1);
        let counter1 = Rc::new(RefCell::new(0));
        let counter2 = Rc::new(RefCell::new(0));

        let c1 = counter1.clone();
        let signal_clone = signal;
        create_effect(move || {
            let _v = *signal_clone.read();
            *c1.borrow_mut() += 1;
        });

        let c2_outer = counter2.clone();
        create_scope(move || {
            let c2 = c2_outer.clone();
            create_effect(move || {
                let _v = *signal.read();
                *c2.borrow_mut() += 1;
            });
        });

        *counter1.borrow_mut() = 0;
        *counter2.borrow_mut() = 0;

        *signal.write() = 2;

        // First effect should run
        assert_eq!(*counter1.borrow(), 1);
        // Second effect's scope was destroyed, so it shouldn't run
        assert_eq!(*counter2.borrow(), 0);
    });
}

#[test]
fn test_signal_updates_during_cleanup() {
    create_scope(|| {
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(100);
        let log = Rc::new(RefCell::new(Vec::new()));

        let log_clone1 = log.clone();
        let log_clone2 = log.clone();
        let signal2_clone = signal2;

        create_effect(move || {
            let val = *signal1.read();
            log_clone1.borrow_mut().push(format!("effect: {}", val));

            let log_clone3 = log_clone2.clone();
            on_cleanup(move || {
                log_clone3.borrow_mut().push("cleanup".to_string());
                // Update another signal during cleanup
                *signal2_clone.write() = 999;
            });
        });

        log.borrow_mut().clear();

        *signal1.write() = 2;

        let log_vec = log.borrow();
        assert!(log_vec.contains(&"cleanup".to_string()));
        assert!(log_vec.contains(&"effect: 2".to_string()));
    });
}

#[test]
fn test_chained_signal_updates() {
    create_scope(|| {
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(10);
        let sig3 = Signal::new(100);

        let sig2_clone = sig2;
        create_effect(move || {
            let v1 = *sig1.read();
            *sig2_clone.write() = v1 * 10;
        });

        let sig3_clone = sig3;
        create_effect(move || {
            let v2 = *sig2.read();
            *sig3_clone.write() = v2 * 10;
        });

        // Update sig1 should cascade to sig2 and sig3
        *sig1.write() = 5;

        assert_eq!(*sig2.read(), 50);
        assert_eq!(*sig3.read(), 500);
    });
}

#[test]
fn test_untrack_nested() {
    create_scope(|| {
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(10);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let v1 = *sig1.read();
            let v2 = untrack(|| *sig2.read());
            *counter_clone.borrow_mut() = v1 + v2;
        });

        assert_eq!(*counter.borrow(), 11);

        // Update sig2 - should NOT trigger effect
        *sig2.write() = 20;
        assert_eq!(*counter.borrow(), 11);

        // Update sig1 - should trigger effect
        *sig1.write() = 5;
        assert_eq!(*counter.borrow(), 25); // 5 + 20
    });
}

#[test]
fn test_scope_with_many_effects() {
    create_scope(|| {
        let signal = Signal::new(0);
        let sum = Rc::new(RefCell::new(0));

        // Create many effects
        for i in 1..=10 {
            let sum_clone = sum.clone();
            create_effect(move || {
                let _val = *signal.read();
                *sum_clone.borrow_mut() += i;
            });
        }

        // All effects run once
        assert_eq!(*sum.borrow(), 55); // 1+2+...+10

        *sum.borrow_mut() = 0;

        // Update signal - all effects run again
        *signal.write() = 1;
        assert_eq!(*sum.borrow(), 55);
    });
}

#[test]
fn test_write_guard_drop_behavior() {
    create_scope(|| {
        let signal = Signal::new(vec![1, 2, 3]);
        let run_count = Rc::new(RefCell::new(0));
        let run_count_clone = run_count.clone();

        create_effect(move || {
            let _v = signal.read();
            *run_count_clone.borrow_mut() += 1;
        });

        *run_count.borrow_mut() = 0;

        // Test that effect runs exactly once after guard drops
        {
            let mut guard = signal.write();
            guard.push(4);
            guard.push(5);
            // Effect hasn't run yet
            assert_eq!(*run_count.borrow(), 0);
        } // Guard drops here, effect should run

        assert_eq!(*run_count.borrow(), 1);
    });
}

#[test]
fn test_as_child_scope_with_signal() {
    create_scope(|| {
        let signal = Signal::new(1);
        
        let child_fn = as_child_scope(move |multiplier: i32| {
            let value = *signal.read();
            value * multiplier
        });

        let (result1, _scope1) = child_fn(10);
        assert_eq!(result1, 10);

        *signal.write() = 5;

        let (result2, _scope2) = child_fn(10);
        assert_eq!(result2, 50);
    });
}

#[test]
fn test_effect_with_early_return() {
    create_scope(|| {
        let signal = Signal::new(5);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let val = *signal.read();
            if val < 10 {
                return; // Early return
            }
            *counter_clone.borrow_mut() += 1;
        });

        // Counter should not increment (val = 5 < 10)
        assert_eq!(*counter.borrow(), 0);

        *signal.write() = 15;

        // Now counter should increment
        assert_eq!(*counter.borrow(), 1);
    });
}

#[test]
fn test_signal_with_option_type() {
    create_scope(|| {
        let signal = Signal::new(Some(42));

        {
            let guard = signal.read();
            assert_eq!(*guard, Some(42));
        }

        signal.set_if_changed(Some(42)); // Same value
        signal.set_if_changed(Some(100)); // Different value

        assert_eq!(*signal.read(), Some(100));

        signal.set_if_changed(None);
        assert_eq!(*signal.read(), None);
    });
}

#[test]
fn test_deeply_nested_effects() {
    create_scope(|| {
        let sig = Signal::new(1);
        let depth = Rc::new(RefCell::new(0));

        let d1 = depth.clone();
        let d_for_nested = depth.clone();
        create_effect(move || {
            let _v = *sig.read();
            *d1.borrow_mut() = 1;

            let d2 = d_for_nested.clone();
            let d_for_nested2 = d_for_nested.clone();
            create_effect(move || {
                let _v = *sig.read();
                *d2.borrow_mut() = 2;

                let d3 = d_for_nested2.clone();
                create_effect(move || {
                    let _v = *sig.read();
                    *d3.borrow_mut() = 3;
                });
            });
        });

        assert_eq!(*depth.borrow(), 3);
    });
}

#[test]
fn test_cleanup_order_multiple_effects() {
    create_scope(|| {
        let signal = Signal::new(1);
        let order = Rc::new(RefCell::new(Vec::new()));

        let o1 = order.clone();
        create_effect(move || {
            let _v = *signal.read();
            let o1_clone = o1.clone();
            on_cleanup(move || {
                o1_clone.borrow_mut().push("cleanup1");
            });
        });

        let o2 = order.clone();
        create_effect(move || {
            let _v = *signal.read();
            let o2_clone = o2.clone();
            on_cleanup(move || {
                o2_clone.borrow_mut().push("cleanup2");
            });
        });

        order.borrow_mut().clear();

        *signal.write() = 2;

        // Both cleanups should have run
        let ord = order.borrow();
        assert!(ord.contains(&"cleanup1"));
        assert!(ord.contains(&"cleanup2"));
    });
}

#[test]
fn test_signal_read_and_write_in_same_effect() {
    create_scope(|| {
        let sig1 = Signal::new(0);
        let sig2 = Signal::new(100);
        let iterations = Rc::new(RefCell::new(0));

        let iter_clone = iterations.clone();
        create_effect(move || {
            let val = *sig1.read();
            *iter_clone.borrow_mut() += 1;
            
            // Prevent infinite loop
            if *iter_clone.borrow() < 3 {
                *sig2.write() = val + 1;
            }
        });

        // Effect should run at least once
        assert!(*iterations.borrow() >= 1);
    });
}

#[test]
fn test_signal_id_uniqueness() {
    create_scope(|| {
        let mut ids = std::collections::HashSet::new();
        
        for _ in 0..100 {
            let sig = Signal::new(0);
            ids.insert(sig.signal_id());
        }

        // All IDs should be unique
        assert_eq!(ids.len(), 100);
    });
}

#[test]
fn test_scope_cleanup_with_no_effects() {
    let ran = Rc::new(RefCell::new(false));
    let ran_clone = ran.clone();

    create_scope(move || {
        on_cleanup(move || {
            *ran_clone.borrow_mut() = true;
        });
        // No effects, just cleanup
    });
    
    assert!(*ran.borrow());
}

#[test]
fn test_effect_reading_signal_multiple_times() {
    create_scope(|| {
        let signal = Signal::new(5);
        let sum = Rc::new(RefCell::new(0));
        let sum_clone = sum.clone();

        create_effect(move || {
            let v1 = *signal.read();
            let v2 = *signal.read();
            let v3 = *signal.read();
            *sum_clone.borrow_mut() = v1 + v2 + v3;
        });

        assert_eq!(*sum.borrow(), 15);

        *signal.write() = 10;
        assert_eq!(*sum.borrow(), 30);
    });
}
