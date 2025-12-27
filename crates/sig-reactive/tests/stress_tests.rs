use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// 关键边界情况测试
// ============================================================================

// 1. 防止无限循环 - 使用 untrack 避免借用冲突
// ============================================================================

#[test]
fn test_effect_modifying_signal_with_untrack() {
    create_scope(|| {
        let signal = Signal::new(1);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            // ✅ 使用 untrack 读取，避免递归追踪
            let val = untrack(|| *signal.read());
            *counter_clone.borrow_mut() += 1;

            if val < 5 {
                *signal.write() = val + 1;
            }
        });

        // Effect 只运行一次（因为用了 untrack）
        assert_eq!(*counter.borrow(), 1);
        
        // 手动触发才会运行
        *signal.write() = 10;
        assert_eq!(*counter.borrow(), 1); // 还是1，因为没有追踪
    });
}

#[test]
fn test_separate_signals_no_recursion() {
    create_scope(|| {
        let input = Signal::new(1);
        let output = Signal::new(0);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        // ✅ 读input，写output - 安全
        create_effect(move || {
            let val = *input.read();
            *counter_clone.borrow_mut() += 1;
            *output.write() = val * 2;
        });

        assert_eq!(*counter.borrow(), 1);
        assert_eq!(*output.read(), 2);

        *input.write() = 5;
        assert_eq!(*counter.borrow(), 2);
        assert_eq!(*output.read(), 10);
    });
}

// 2. Signal 离开 Scope 后的行为
// ============================================================================

#[test]
fn test_signal_survives_scope() {
    let signal = {
        create_scope(|| {
            let _sig = Signal::new(1);
        });
        // 在外部创建signal
        Signal::new(42)
    };

    // Signal 应该仍然可用
    assert_eq!(*signal.read(), 42);
    *signal.write() = 100;
    assert_eq!(*signal.read(), 100);
}

// 3. Context 作用域边界
// ============================================================================

#[test]
fn test_context_not_available_after_scope() {
    #[derive(Clone)]
    struct Config { value: i32 }

    create_scope(|| {
        create_scope(|| {
            provide_context(Config { value: 42 });
            assert!(has_context::<Config>());
        }); // Inner scope 结束

        // Context 应该不可见
        assert!(!has_context::<Config>());
        assert!(consume_context::<Config>().is_none());
    });
}

#[test]
fn test_context_shadowing_multiple_levels() {
    #[derive(Clone, PartialEq, Debug)]
    struct Value { num: i32 }

    create_scope(|| {
        provide_context(Value { num: 1 });

        create_scope(|| {
            provide_context(Value { num: 2 });
            assert_eq!(consume_context::<Value>().unwrap().num, 2);

            create_scope(|| {
                provide_context(Value { num: 3 });
                assert_eq!(consume_context::<Value>().unwrap().num, 3);
            });

            // 回到 level 2
            assert_eq!(consume_context::<Value>().unwrap().num, 2);
        });

        // 回到 level 1
        assert_eq!(consume_context::<Value>().unwrap().num, 1);
    });
}

// 4. Cleanup 执行顺序
// ============================================================================

#[test]
fn test_cleanup_lifo_order() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let o1 = order.clone();
    let o2 = order.clone();
    let o3 = order.clone();

    create_scope(move || {
        on_cleanup(move || o1.borrow_mut().push(1));
        on_cleanup(move || o2.borrow_mut().push(2));
        on_cleanup(move || o3.borrow_mut().push(3));
    });

    // 应该按 LIFO 顺序：3, 2, 1
    assert_eq!(*order.borrow(), vec![3, 2, 1]);
}

#[test]
fn test_nested_cleanup_order() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let o1 = order.clone();
    let o2 = order.clone();
    let o3 = order.clone();
    let o4 = order.clone();

    create_scope(move || {
        on_cleanup(move || o1.borrow_mut().push("outer-1"));

        create_scope(move || {
            on_cleanup(move || o2.borrow_mut().push("inner-1"));
            on_cleanup(move || o3.borrow_mut().push("inner-2"));
        });

        on_cleanup(move || o4.borrow_mut().push("outer-2"));
    });

    let result = order.borrow();
    // 顺序：inner-2, inner-1, outer-2, outer-1
    assert_eq!(result[0], "inner-2");
    assert_eq!(result[1], "inner-1");
    assert_eq!(result[2], "outer-2");
    assert_eq!(result[3], "outer-1");
}

// 5. 大规模测试
// ============================================================================

#[test]
fn test_many_signals_in_scope() {
    create_scope(|| {
        let signals: Vec<_> = (0..1000).map(|i| Signal::new(i)).collect();
        
        for (i, sig) in signals.iter().enumerate() {
            assert_eq!(*sig.read(), i);
            *sig.write() = i * 2;
            assert_eq!(*sig.read(), i * 2);
        }
    });
}

#[test]
fn test_many_effects_on_one_signal() {
    create_scope(|| {
        let signal = Signal::new(0);
        let counter = Rc::new(RefCell::new(0));

        // 创建 100 个 effect
        for _ in 0..100 {
            let c = counter.clone();
            create_effect(move || {
                let _val = *signal.read();
                *c.borrow_mut() += 1;
            });
        }

        assert_eq!(*counter.borrow(), 100);

        *counter.borrow_mut() = 0;
        *signal.write() = 1;
        assert_eq!(*counter.borrow(), 100);
    });
}

#[test]
fn test_deep_scope_nesting() {
    fn nest(depth: usize, log: Rc<RefCell<Vec<usize>>>) {
        if depth == 0 {
            return;
        }
        let log_clone = log.clone();
        create_scope(move || {
            let log_clone2 = log_clone.clone();
            on_cleanup(move || {
                log_clone2.borrow_mut().push(depth);
            });
            nest(depth - 1, log_clone.clone());
        });
    }

    let log = Rc::new(RefCell::new(Vec::new()));
    let log_clone = log.clone();
    create_scope(move || {
        nest(50, log_clone);
    });

    // 应该有 50 个 cleanup，按深度递增顺序
    assert_eq!(log.borrow().len(), 50);
    assert_eq!(log.borrow()[0], 1);
    assert_eq!(log.borrow()[49], 50);
}

// 6. set_if_changed 性能优化
// ============================================================================

#[test]
fn test_set_if_changed_no_unnecessary_updates() {
    create_scope(|| {
        let signal = Signal::new(vec![1, 2, 3]);
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            let _val = signal.read();
            *counter_clone.borrow_mut() += 1;
        });

        *counter.borrow_mut() = 0;

        // 设置相同值，不应触发
        signal.set_if_changed(vec![1, 2, 3]);
        assert_eq!(*counter.borrow(), 0);

        // 设置不同值，应触发
        signal.set_if_changed(vec![1, 2, 3, 4]);
        assert_eq!(*counter.borrow(), 1);
    });
}

// 7. untrack 嵌套
// ============================================================================

#[test]
fn test_untrack_nested_complex() {
    create_scope(|| {
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(10);
        let sig3 = Signal::new(100);
        let result = Rc::new(RefCell::new(0));
        let result_clone = result.clone();

        create_effect(move || {
            let v1 = *sig1.read(); // 追踪
            let v2 = untrack(|| {
                let v2_val = *sig2.read(); // 不追踪
                let v3 = *sig3.read(); // 不追踪
                v2_val + v3
            });
            *result_clone.borrow_mut() = v1 + v2;
        });

        assert_eq!(*result.borrow(), 111);

        *result.borrow_mut() = 0;

        // sig2 和 sig3 的更新不应触发
        *sig2.write() = 20;
        *sig3.write() = 200;
        assert_eq!(*result.borrow(), 0);

        // 只有 sig1 触发
        *sig1.write() = 5;
        assert_eq!(*result.borrow(), 225); // 5 + 20 + 200
    });
}

// 8. 特殊类型
// ============================================================================

#[test]
fn test_signal_with_option() {
    create_scope(|| {
        let signal = Signal::new(Option::<i32>::None);

        assert!(signal.read().is_none());

        *signal.write() = Some(42);
        assert_eq!(*signal.read(), Some(42));

        signal.set_if_changed(Some(42)); // 相同，不触发
        *signal.write() = None;
        assert!(signal.read().is_none());
    });
}

#[test]
fn test_signal_with_shared_data() {
    create_scope(|| {
        #[derive(Clone)]
        struct Shared {
            data: Rc<RefCell<Vec<i32>>>,
        }

        let shared = Shared {
            data: Rc::new(RefCell::new(vec![1, 2, 3])),
        };

        let signal = Signal::new(shared.clone());

        // 修改共享数据
        shared.data.borrow_mut().push(4);

        // Signal 中的数据也会变（因为是 Rc）
        assert_eq!(signal.read().data.borrow().len(), 4);
    });
}

// 9. 空操作
// ============================================================================

#[test]
fn test_empty_scopes() {
    for _ in 0..100 {
        create_scope(|| {});
    }
}

#[test]
fn test_empty_effects() {
    create_scope(|| {
        for _ in 0..100 {
            create_effect(|| {});
        }
    });
}

// 10. as_child_scope
// ============================================================================

#[test]
fn test_as_child_scope_many_calls() {
    create_scope(|| {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        let child_fn = as_child_scope(move |x: i32| {
            *counter_clone.borrow_mut() += x;
            x * 2
        });

        let mut sum = 0;
        for i in 1..=10 {
            let (result, _) = child_fn(i);
            sum += result;
        }

        assert_eq!(sum, 110); // 2+4+6+...+20
        assert_eq!(*counter.borrow(), 55); // 1+2+3+...+10
    });
}
