use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Tests for uncovered code paths  
// ============================================================================

#[test]
fn test_default_runtime() {
    // Test Runtime::default() implementation
    let runtime = Runtime::default();
    drop(runtime);
}

#[test]
fn test_effect_scope_registration() {
    create_scope(|| {
        let signal = Signal::new(1);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        let _effect = create_effect(move || {
            let _val = *signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        assert_eq!(*counter.borrow(), 1);
    });
}

#[test]
fn test_signal_in_multiple_nested_effects() {
    create_scope(|| {
        let sig = Signal::new(1);
        let counter = Rc::new(RefCell::new(0));

        for level in 1..=3 {
            let c = counter.clone();
            create_effect(move || {
                let _v = *sig.read();
                *c.borrow_mut() += level;
            });
        }

        *counter.borrow_mut() = 0;
        *sig.write() = 2;

        assert_eq!(*counter.borrow(), 6); // 1 + 2 + 3
    });
}

#[test]
fn test_cleanup_with_signal_reads() {
    create_scope(|| {
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(10);
        let cleanup_ran = Rc::new(RefCell::new(false));

        let cleanup_ran_clone = cleanup_ran.clone();
        let sig2_clone = sig2;
        create_effect(move || {
            let _v1 = *sig1.read();
            
            let cleanup_ran_clone2 = cleanup_ran_clone.clone();
            on_cleanup(move || {
                let _v2 = *sig2_clone.read_untracked();
                *cleanup_ran_clone2.borrow_mut() = true;
            });
        });

        *sig1.write() = 2;
        assert!(*cleanup_ran.borrow());
    });
}

#[test]
fn test_effect_re_registration() {
    create_scope(|| {
        let signal = Signal::new(1);
        let run_count = Rc::new(RefCell::new(0));
        let run_count_clone = run_count.clone();

        create_effect(move || {
            let val = *signal.read();
            *run_count_clone.borrow_mut() += 1;
            
            if val > 5 {
                let _other = *signal.read();
            }
        });

        *run_count.borrow_mut() = 0;
        
        for i in 2..=10 {
            *signal.write() = i;
        }

        assert!(*run_count.borrow() > 0);
    });
}

#[test]
fn test_as_child_scope_cleanup() {
    create_scope(|| {
        let cleanup_count = Rc::new(RefCell::new(0));
        let cleanup_count_clone = cleanup_count.clone();

        let child_fn = as_child_scope(move |_: ()| {
            let cc = cleanup_count_clone.clone();
            on_cleanup(move || {
                *cc.borrow_mut() += 1;
            });
            42
        });

        let (_result1, _scope1) = child_fn(());
        let (_result2, _scope2) = child_fn(());
        let (_result3, _scope3) = child_fn(());
    });
}

#[test]
fn test_signal_with_mutable_guard_operations() {
    create_scope(|| {
        let signal = Signal::new(Vec::<i32>::new());
        
        {
            let mut guard = signal.write();
            guard.push(1);
            guard.push(2);
            guard.push(3);
            guard.sort();
            guard.reverse();
        }

        let result = signal.read();
        assert_eq!(*result, vec![3, 2, 1]);
    });
}

#[test]
fn test_effect_without_parent_scope() {
    create_scope(|| {
        let _effect = create_effect(|| {
            // Simple effect
        });
    });
}

#[test]
fn test_multiple_signals_with_conditional_updates() {
    create_scope(|| {
        let condition = Signal::new(true);
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(10);
        let sig_c = Signal::new(100);

        let result = Rc::new(RefCell::new(0));
        let result_clone = result.clone();

        create_effect(move || {
            if *condition.read() {
                if *sig_a.read() > 5 {
                    *result_clone.borrow_mut() = *sig_b.read();
                } else {
                    *result_clone.borrow_mut() = *sig_c.read();
                }
            }
        });

        assert_eq!(*result.borrow(), 100);

        *sig_a.write() = 10;
        assert_eq!(*result.borrow(), 10);

        *condition.write() = false;
        let prev = *result.borrow();
        *sig_a.write() = 20;
        // Effect doesn't read when condition is false
        assert_eq!(*result.borrow(), prev);
    });
}

#[test]
fn test_signal_equality_complex() {
    create_scope(|| {
        #[derive(PartialEq, Clone)]
        struct ComplexStruct {
            vec: Vec<i32>,
            opt: Option<String>,
        }

        let signal = Signal::new(ComplexStruct {
            vec: vec![1, 2, 3],
            opt: Some("test".to_string()),
        });

        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let _val = signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        *counter.borrow_mut() = 0;

        signal.set_if_changed(ComplexStruct {
            vec: vec![1, 2, 3],
            opt: Some("test".to_string()),
        });

        assert_eq!(*counter.borrow(), 0);

        signal.set_if_changed(ComplexStruct {
            vec: vec![1, 2, 3, 4],
            opt: Some("test".to_string()),
        });

        assert_eq!(*counter.borrow(), 1);
    });
}

#[test]
fn test_untrack_with_nested_effects() {
    create_scope(|| {
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(10);
        let counter = Rc::new(RefCell::new(0));

        let c1 = counter.clone();
        let sig2_clone = sig2;
        create_effect(move || {
            let v1 = *sig1.read();
            
            let c2 = c1.clone();
            create_effect(move || {
                let _v2 = untrack(|| *sig2_clone.read());
                *c2.borrow_mut() = v1;
            });
        });

        *counter.borrow_mut() = 0;
        *sig2.write() = 20;
        assert_eq!(*counter.borrow(), 0);

        *sig1.write() = 5;
        assert_eq!(*counter.borrow(), 5);
    });
}

#[test]
fn test_scope_hierarchy_with_cleanup() {
    let cleanup_log = Rc::new(RefCell::new(Vec::new()));
    let log1 = cleanup_log.clone();
    let log2 = cleanup_log.clone();
    let log3 = cleanup_log.clone();

    create_scope(move || {
        on_cleanup(move || {
            log1.borrow_mut().push("outer");
        });

        create_scope(move || {
            on_cleanup(move || {
                log2.borrow_mut().push("middle");
            });

            create_scope(move || {
                on_cleanup(move || {
                    log3.borrow_mut().push("inner");
                });
            });
        });
    });

    let log = cleanup_log.borrow();
    assert!(log.contains(&"inner"));
    assert!(log.contains(&"middle"));
    assert!(log.contains(&"outer"));
}

#[test]
fn test_effect_with_multiple_cleanup_functions() {
    create_scope(|| {
        let signal = Signal::new(1);
        let cleanup_count = Rc::new(RefCell::new(0));

        let cc = cleanup_count.clone();
        create_effect(move || {
            let _val = *signal.read();
            
            for i in 1..=3 {
                let cc_clone = cc.clone();
                on_cleanup(move || {
                    *cc_clone.borrow_mut() += i;
                });
            }
        });

        *cleanup_count.borrow_mut() = 0;
        *signal.write() = 2;

        assert_eq!(*cleanup_count.borrow(), 6);
    });
}

#[test]
fn test_chained_effects_with_cleanups() {
    create_scope(|| {
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(10);
        let cleanup_order = Rc::new(RefCell::new(Vec::new()));

        let co1 = cleanup_order.clone();
        let sig2_clone = sig2;
        create_effect(move || {
            let v1 = *sig1.read();
            *sig2_clone.write() = v1 * 10;
            
            let co1_clone = co1.clone();
            on_cleanup(move || {
                co1_clone.borrow_mut().push("effect1");
            });
        });

        let co2 = cleanup_order.clone();
        create_effect(move || {
            let _v2 = *sig2.read();
            
            let co2_clone = co2.clone();
            on_cleanup(move || {
                co2_clone.borrow_mut().push("effect2");
            });
        });

        cleanup_order.borrow_mut().clear();
        *sig1.write() = 5;

        let order = cleanup_order.borrow();
        assert!(order.contains(&"effect1"));
        assert!(order.contains(&"effect2"));
    });
}

#[test]
fn test_deeply_nested_with_different_signal_dependencies() {
    create_scope(|| {
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(10);
        let sig_c = Signal::new(100);
        
        let result = Rc::new(RefCell::new(0));
        let r1 = result.clone();
        let r2_outer = result.clone();

        let sig_b_clone = sig_b;
        let sig_c_clone = sig_c;
        
        create_effect(move || {
            let a = *sig_a.read();
            *r1.borrow_mut() = a;

            let r2 = r2_outer.clone();
            create_effect(move || {
                let b = *sig_b_clone.read();
                
                let r2_inner = r2.clone();
                create_effect(move || {
                    let c = *sig_c_clone.read();
                    *r2_inner.borrow_mut() = a + b + c;
                });
            });
        });

        *sig_c.write() = 200;
        // Nested effects should have updated
        assert!(*result.borrow() > 100);
    });
}

#[test]
fn test_signal_with_large_dependency_graph() {
    create_scope(|| {
        let signals: Vec<_> = (0..10).map(|i| Signal::new(i)).collect();
        let sum = Rc::new(RefCell::new(0));

        let sum_clone = sum.clone();
        let signals_clone = signals.clone();
        create_effect(move || {
            let total: i32 = signals_clone.iter().map(|s| *s.read()).sum();
            *sum_clone.borrow_mut() = total;
        });

        assert_eq!(*sum.borrow(), 45); // 0+1+2+...+9

        *signals[0].write() = 100;
        assert_eq!(*sum.borrow(), 145); // 100+1+2+...+9
    });
}

#[test]
fn test_effect_with_conditional_signal_access() {
    create_scope(|| {
        let toggle = Signal::new(true);
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(10);
        let count_a = Rc::new(RefCell::new(0));
        let count_b = Rc::new(RefCell::new(0));

        let ca = count_a.clone();
        let cb = count_b.clone();
        
        create_effect(move || {
            if *toggle.read() {
                let _a = *sig_a.read();
                *ca.borrow_mut() += 1;
            } else {
                let _b = *sig_b.read();
                *cb.borrow_mut() += 1;
            }
        });

        *count_a.borrow_mut() = 0;
        *count_b.borrow_mut() = 0;

        *sig_a.write() = 2;
        assert_eq!(*count_a.borrow(), 1);
        assert_eq!(*count_b.borrow(), 0);

        *toggle.write() = false;
        
        *count_a.borrow_mut() = 0;
        *count_b.borrow_mut() = 0;

        *sig_b.write() = 20;
        assert_eq!(*count_a.borrow(), 0);
        assert_eq!(*count_b.borrow(), 1);
    });
}
