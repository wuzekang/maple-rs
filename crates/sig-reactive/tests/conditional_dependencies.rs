use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// 条件依赖收集测试 - 验证多次执行时依赖是否正确更新
// ============================================================================

#[test]
fn test_dependency_changes_with_condition() {
    create_scope(|| {
        let condition = Signal::new(true);
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(100);
        let run_count = Rc::new(RefCell::new(0));
        let run_count_clone = run_count.clone();

        create_effect(move || {
            *run_count_clone.borrow_mut() += 1;
            
            if *condition.read() {
                let _ = *sig_a.read();  // 依赖 sig_a
            } else {
                let _ = *sig_b.read();  // 依赖 sig_b
            }
        });

        // 初始运行一次
        assert_eq!(*run_count.borrow(), 1);
        *run_count.borrow_mut() = 0;

        // 当 condition = true 时，只依赖 sig_a
        *sig_a.write() = 2;
        assert_eq!(*run_count.borrow(), 1, "sig_a 应该触发");
        
        *run_count.borrow_mut() = 0;
        *sig_b.write() = 200;
        assert_eq!(*run_count.borrow(), 0, "sig_b 不应该触发");

        // 切换条件
        *run_count.borrow_mut() = 0;
        *condition.write() = false;
        assert_eq!(*run_count.borrow(), 1, "condition 变化应该触发");

        // 现在应该只依赖 sig_b，不依赖 sig_a
        *run_count.borrow_mut() = 0;
        *sig_b.write() = 300;
        assert_eq!(*run_count.borrow(), 1, "sig_b 现在应该触发");

        *run_count.borrow_mut() = 0;
        *sig_a.write() = 3;
        assert_eq!(*run_count.borrow(), 0, "sig_a 现在不应该触发");
    });
}

#[test]
fn test_dependency_cleanup_on_rerun() {
    create_scope(|| {
        let show = Signal::new(true);
        let sig1 = Signal::new(1);
        let sig2 = Signal::new(2);
        let sig3 = Signal::new(3);
        let run_count = Rc::new(RefCell::new(0));
        let rc = run_count.clone();

        create_effect(move || {
            *rc.borrow_mut() += 1;
            
            if *show.read() {
                // 读取 sig1, sig2, sig3
                let _ = *sig1.read();
                let _ = *sig2.read();
                let _ = *sig3.read();
            }
            // 当 show=false 时，不读取任何 signal
        });

        *run_count.borrow_mut() = 0;

        // show=true 时，所有 signal 都应该触发
        *sig1.write() = 10;
        assert_eq!(*run_count.borrow(), 1);
        
        *run_count.borrow_mut() = 0;
        *sig2.write() = 20;
        assert_eq!(*run_count.borrow(), 1);

        *run_count.borrow_mut() = 0;
        *sig3.write() = 30;
        assert_eq!(*run_count.borrow(), 1);

        // 切换到 show=false
        *run_count.borrow_mut() = 0;
        *show.write() = false;
        assert_eq!(*run_count.borrow(), 1);

        // 现在所有之前的依赖都应该被清除
        *run_count.borrow_mut() = 0;
        *sig1.write() = 100;
        assert_eq!(*run_count.borrow(), 0, "sig1 不应该再触发");

        *sig2.write() = 200;
        assert_eq!(*run_count.borrow(), 0, "sig2 不应该再触发");

        *sig3.write() = 300;
        assert_eq!(*run_count.borrow(), 0, "sig3 不应该再触发");
    });
}

#[test]
fn test_nested_conditional_dependencies() {
    create_scope(|| {
        let outer = Signal::new(true);
        let inner = Signal::new(true);
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(2);
        let sig_c = Signal::new(3);
        let sig_d = Signal::new(4);
        let run_count = Rc::new(RefCell::new(0));
        let rc = run_count.clone();

        create_effect(move || {
            *rc.borrow_mut() += 1;
            
            if *outer.read() {
                if *inner.read() {
                    let _ = *sig_a.read();  // outer=T, inner=T
                } else {
                    let _ = *sig_b.read();  // outer=T, inner=F
                }
            } else {
                if *inner.read() {
                    let _ = *sig_c.read();  // outer=F, inner=T
                } else {
                    let _ = *sig_d.read();  // outer=F, inner=F
                }
            }
        });

        *run_count.borrow_mut() = 0;

        // outer=T, inner=T -> 只依赖 sig_a
        *sig_a.write() = 10;
        assert_eq!(*run_count.borrow(), 1);
        *run_count.borrow_mut() = 0;
        *sig_b.write() = 20;
        *sig_c.write() = 30;
        *sig_d.write() = 40;
        assert_eq!(*run_count.borrow(), 0);

        // 切换 inner
        *run_count.borrow_mut() = 0;
        *inner.write() = false;  // outer=T, inner=F -> 应该依赖 sig_b
        assert_eq!(*run_count.borrow(), 1);

        *run_count.borrow_mut() = 0;
        *sig_b.write() = 200;
        assert_eq!(*run_count.borrow(), 1);
        *run_count.borrow_mut() = 0;
        *sig_a.write() = 100;
        *sig_c.write() = 300;
        *sig_d.write() = 400;
        assert_eq!(*run_count.borrow(), 0);

        // 切换 outer
        *run_count.borrow_mut() = 0;
        *outer.write() = false;  // outer=F, inner=F -> 应该依赖 sig_d
        assert_eq!(*run_count.borrow(), 1);

        *run_count.borrow_mut() = 0;
        *sig_d.write() = 4000;
        assert_eq!(*run_count.borrow(), 1);
        *run_count.borrow_mut() = 0;
        *sig_a.write() = 1000;
        *sig_b.write() = 2000;
        *sig_c.write() = 3000;
        assert_eq!(*run_count.borrow(), 0);
    });
}

#[test]
fn test_loop_conditional_dependencies() {
    create_scope(|| {
        let count = Signal::new(3);
        let signals = vec![
            Signal::new(1),
            Signal::new(2),
            Signal::new(3),
            Signal::new(4),
            Signal::new(5),
        ];
        let sum = Rc::new(RefCell::new(0));
        let sum_clone = sum.clone();

        let signals_clone = signals.clone();
        create_effect(move || {
            let n = *count.read();
            let mut total = 0;
            
            // 只读取前 n 个 signal
            for i in 0..n {
                total += *signals_clone[i].read();
            }
            
            *sum_clone.borrow_mut() = total;
        });

        // 初始：读取前3个 (1+2+3=6)
        assert_eq!(*sum.borrow(), 6);

        // 更新前3个应该触发
        *signals[0].write() = 10;
        assert_eq!(*sum.borrow(), 15);  // 10+2+3

        *signals[1].write() = 20;
        assert_eq!(*sum.borrow(), 33);  // 10+20+3

        *signals[2].write() = 30;
        assert_eq!(*sum.borrow(), 60);  // 10+20+30

        // 更新第4、5个不应该触发（未被读取）
        *signals[3].write() = 40;
        assert_eq!(*sum.borrow(), 60);  // 不变

        *signals[4].write() = 50;
        assert_eq!(*sum.borrow(), 60);  // 不变

        // 增加 count，现在应该读取前5个
        *count.write() = 5;
        assert_eq!(*sum.borrow(), 150);  // 10+20+30+40+50

        // 现在第4、5个应该能触发了
        *signals[3].write() = 400;
        assert_eq!(*sum.borrow(), 510);  // 10+20+30+400+50

        *signals[4].write() = 500;
        assert_eq!(*sum.borrow(), 960);  // 10+20+30+400+500

        // 减少 count，第4、5个又不应该触发了
        *count.write() = 2;
        assert_eq!(*sum.borrow(), 30);  // 10+20

        *signals[3].write() = 4000;
        *signals[4].write() = 5000;
        assert_eq!(*sum.borrow(), 30);  // 不变
    });
}

#[test]
fn test_early_return_dependency() {
    create_scope(|| {
        let condition = Signal::new(true);
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(2);
        let result = Rc::new(RefCell::new(0));
        let result_clone = result.clone();

        create_effect(move || {
            if !*condition.read() {
                return;  // 提前返回
            }
            
            // 只有 condition=true 时才会读取这些
            *result_clone.borrow_mut() = *sig_a.read() + *sig_b.read();
        });

        assert_eq!(*result.borrow(), 3);  // 1+2

        // condition=true 时，sig_a 和 sig_b 都应该触发
        *sig_a.write() = 10;
        assert_eq!(*result.borrow(), 12);  // 10+2

        *sig_b.write() = 20;
        assert_eq!(*result.borrow(), 30);  // 10+20

        // 切换 condition=false
        *condition.write() = false;
        // result 不应该被更新（提前返回）
        let old_result = *result.borrow();

        // sig_a 和 sig_b 更新不应该触发 effect
        *sig_a.write() = 100;
        *sig_b.write() = 200;
        assert_eq!(*result.borrow(), old_result);
    });
}

#[test]
fn test_match_expression_dependencies() {
    create_scope(|| {
        #[derive(Clone, Copy, PartialEq)]
        enum State {
            A,
            B,
            C,
        }

        let state = Signal::new(State::A);
        let sig_a = Signal::new(1);
        let sig_b = Signal::new(2);
        let sig_c = Signal::new(3);
        let result = Rc::new(RefCell::new(0));
        let rc = result.clone();

        create_effect(move || {
            *rc.borrow_mut() = match *state.read() {
                State::A => *sig_a.read(),
                State::B => *sig_b.read(),
                State::C => *sig_c.read(),
            };
        });

        // State::A -> 依赖 sig_a
        assert_eq!(*result.borrow(), 1);
        *sig_a.write() = 10;
        assert_eq!(*result.borrow(), 10);
        *sig_b.write() = 20;
        *sig_c.write() = 30;
        assert_eq!(*result.borrow(), 10);  // 不变

        // 切换到 State::B -> 依赖 sig_b
        *state.write() = State::B;
        assert_eq!(*result.borrow(), 20);
        *sig_b.write() = 200;
        assert_eq!(*result.borrow(), 200);
        *sig_a.write() = 100;
        *sig_c.write() = 300;
        assert_eq!(*result.borrow(), 200);  // 不变

        // 切换到 State::C -> 依赖 sig_c
        *state.write() = State::C;
        assert_eq!(*result.borrow(), 300);
        *sig_c.write() = 3000;
        assert_eq!(*result.borrow(), 3000);
        *sig_a.write() = 1000;
        *sig_b.write() = 2000;
        assert_eq!(*result.borrow(), 3000);  // 不变
    });
}

#[test]
fn test_dynamic_dependency_count() {
    create_scope(|| {
        let enabled = vec![
            Signal::new(true),
            Signal::new(true),
            Signal::new(false),
            Signal::new(false),
        ];
        let values = vec![
            Signal::new(1),
            Signal::new(2),
            Signal::new(3),
            Signal::new(4),
        ];
        let sum = Rc::new(RefCell::new(0));
        let sum_clone = sum.clone();

        let enabled_clone = enabled.clone();
        let values_clone = values.clone();
        create_effect(move || {
            let mut total = 0;
            for i in 0..4 {
                if *enabled_clone[i].read() {
                    total += *values_clone[i].read();
                }
            }
            *sum_clone.borrow_mut() = total;
        });

        // 初始：0和1启用 (1+2=3)
        assert_eq!(*sum.borrow(), 3);

        // 更新启用的值应该触发
        *values[0].write() = 10;
        assert_eq!(*sum.borrow(), 12);  // 10+2

        *values[1].write() = 20;
        assert_eq!(*sum.borrow(), 30);  // 10+20

        // 更新未启用的值不应该触发
        *values[2].write() = 30;
        *values[3].write() = 40;
        assert_eq!(*sum.borrow(), 30);  // 不变

        // 启用第2个
        *enabled[2].write() = true;
        assert_eq!(*sum.borrow(), 60);  // 10+20+30

        // 现在第2个的值更新应该触发
        *values[2].write() = 300;
        assert_eq!(*sum.borrow(), 330);  // 10+20+300

        // 禁用第0个
        *enabled[0].write() = false;
        assert_eq!(*sum.borrow(), 320);  // 20+300

        // 第0个的值更新不应该触发
        *values[0].write() = 1000;
        assert_eq!(*sum.borrow(), 320);  // 不变
    });
}

#[test]
fn test_optional_dependency() {
    create_scope(|| {
        let has_value = Signal::new(true);
        let value = Signal::new(Some(42));
        let fallback = Signal::new(0);
        let result = Rc::new(RefCell::new(0));
        let rc = result.clone();

        create_effect(move || {
            *rc.borrow_mut() = if *has_value.read() {
                value.read().unwrap_or(0)
            } else {
                *fallback.read()
            };
        });

        assert_eq!(*result.borrow(), 42);

        // has_value=true 时，依赖 value
        *value.write() = Some(100);
        assert_eq!(*result.borrow(), 100);

        *fallback.write() = 999;
        assert_eq!(*result.borrow(), 100);  // fallback 不应该触发

        // 切换到 fallback
        *has_value.write() = false;
        assert_eq!(*result.borrow(), 999);

        *fallback.write() = 111;
        assert_eq!(*result.borrow(), 111);

        *value.write() = Some(200);
        assert_eq!(*result.borrow(), 111);  // value 不应该触发
    });
}
