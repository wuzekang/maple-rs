use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_immediate_effect_execution() {
    create_scope(|| {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let signal = Signal::new(1);
        
        // Create effect that increments counter when signal changes
        create_effect(move || {
            let _value = *signal.read();
            *counter_clone.borrow_mut() += 1;
        });
        
        // Effect should have run once during creation
        assert_eq!(*counter.borrow(), 1);
        
        // Modify signal - effect should run immediately
        *signal.write() = 2;
        
        // Counter should be incremented immediately after write
        assert_eq!(*counter.borrow(), 2);
        
        // Modify signal again
        *signal.write() = 3;
        
        // Counter should be incremented again immediately
        assert_eq!(*counter.borrow(), 3);
    });
}

#[test]
fn test_multiple_effects_immediate_execution() {
    create_scope(|| {
        let counter1 = Rc::new(RefCell::new(0));
        let counter2 = Rc::new(RefCell::new(0));
        let counter1_clone = counter1.clone();
        let counter2_clone = counter2.clone();
        
        let signal = Signal::new(10);
        
        // Create two effects subscribed to the same signal
        create_effect(move || {
            let _value = *signal.read();
            *counter1_clone.borrow_mut() += 1;
        });
        
        create_effect(move || {
            let _value = *signal.read();
            *counter2_clone.borrow_mut() += 1;
        });
        
        // Both effects should have run once
        assert_eq!(*counter1.borrow(), 1);
        assert_eq!(*counter2.borrow(), 1);
        
        // Modify signal - both effects should run immediately
        *signal.write() = 20;
        
        // Both counters should be incremented
        assert_eq!(*counter1.borrow(), 2);
        assert_eq!(*counter2.borrow(), 2);
    });
}

#[test]
fn test_nested_signal_updates() {
    create_scope(|| {
        let execution_order = Rc::new(RefCell::new(Vec::new()));
        let execution_order_clone = execution_order.clone();
        
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(10);
        
        // Effect that reads signal1 and updates signal2
        create_effect(move || {
            let value = *signal1.read();
            execution_order_clone.borrow_mut().push(format!("effect1: {}", value));
            
            // Update signal2 inside effect
            *signal2.write() = value * 10;
        });
        
        let execution_order_clone2 = execution_order.clone();
        
        // Effect that reads signal2
        create_effect(move || {
            let value = *signal2.read();
            execution_order_clone2.borrow_mut().push(format!("effect2: {}", value));
        });
        
        // Clear initial execution
        execution_order.borrow_mut().clear();
        
        // Update signal1
        *signal1.write() = 5;
        
        // Both effects should have executed immediately
        let order = execution_order.borrow();
        assert!(order.contains(&"effect1: 5".to_string()));
        assert!(order.contains(&"effect2: 50".to_string()));
        
        // Effect2 should run after effect1 updates signal2
        let effect1_index = order.iter().position(|s| s == "effect1: 5").unwrap();
        let effect2_index = order.iter().position(|s| s == "effect2: 50").unwrap();
        assert!(effect2_index > effect1_index);
    });
}

#[test]
fn test_nested_effects() {
    create_scope(|| {
        let outer_counter = Rc::new(RefCell::new(0));
        let inner_counter = Rc::new(RefCell::new(0));
        let outer_counter_clone = outer_counter.clone();
        let inner_counter_clone = inner_counter.clone();
        
        let outer_signal = Signal::new(1);
        let inner_signal = Signal::new(10);
        
        // Outer effect that creates an inner effect
        create_effect(move || {
            let _outer_value = *outer_signal.read();
            *outer_counter_clone.borrow_mut() += 1;
            
            // Create nested effect
            let inner_counter_clone2 = inner_counter_clone.clone();
            create_effect(move || {
                let _inner_value = *inner_signal.read();
                *inner_counter_clone2.borrow_mut() += 1;
            });
        });
        
        // Outer effect runs once, inner effect runs once
        assert_eq!(*outer_counter.borrow(), 1);
        assert_eq!(*inner_counter.borrow(), 1);
        
        // Update inner signal - should trigger inner effect
        *inner_signal.write() = 20;
        assert_eq!(*inner_counter.borrow(), 2);
        assert_eq!(*outer_counter.borrow(), 1); // outer unchanged
        
        // Update outer signal - creates a NEW inner effect
        *outer_signal.write() = 2;
        assert_eq!(*outer_counter.borrow(), 2);
        // New inner effect runs once
        assert_eq!(*inner_counter.borrow(), 3);
        
        // Update inner signal again - triggers BOTH inner effects (old and new)
        *inner_signal.write() = 30;
        assert_eq!(*inner_counter.borrow(), 5); // Both inner effects run
    });
}

#[test]
fn test_nested_effects_with_different_signals() {
    create_scope(|| {
        let execution_log = Rc::new(RefCell::new(Vec::new()));
        let log_clone1 = execution_log.clone();
        let log_clone2 = execution_log.clone();
        
        let signal_a = Signal::new(1);
        let signal_b = Signal::new(100);
        
        // Outer effect reads signal_a
        create_effect(move || {
            let value_a = *signal_a.read();
            log_clone1.borrow_mut().push(format!("outer: a={}", value_a));
            
            // Inner effect reads signal_b
            let log_clone3 = log_clone2.clone();
            create_effect(move || {
                let value_b = *signal_b.read();
                log_clone3.borrow_mut().push(format!("inner: b={}", value_b));
            });
        });
        
        // Clear initial execution log
        execution_log.borrow_mut().clear();
        
        // Update signal_a - should trigger outer effect and create new inner effect
        *signal_a.write() = 2;
        {
            let log = execution_log.borrow();
            assert_eq!(log.len(), 2);
            assert_eq!(log[0], "outer: a=2");
            assert_eq!(log[1], "inner: b=100");
        }
        
        execution_log.borrow_mut().clear();
        
        // Update signal_b - should trigger all existing inner effects
        *signal_b.write() = 200;
        {
            let log = execution_log.borrow();
            // Two inner effects should run (one from initial, one from update)
            assert_eq!(log.len(), 2);
            assert!(log.iter().all(|s| s.starts_with("inner: b=200")));
        }
    });
}

#[test]
fn test_effect_reads_multiple_signals() {
    create_scope(|| {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(10);
        
        // Effect that reads both signals
        create_effect(move || {
            let _v1 = *signal1.read();
            let _v2 = *signal2.read();
            *counter_clone.borrow_mut() += 1;
        });
        
        // Effect runs once during creation
        assert_eq!(*counter.borrow(), 1);
        
        // Update signal1 - effect should run
        *signal1.write() = 2;
        assert_eq!(*counter.borrow(), 2);
        
        // Update signal2 - effect should run
        *signal2.write() = 20;
        assert_eq!(*counter.borrow(), 3);
        
        // Update both signals - effect should run twice
        *signal1.write() = 3;
        *signal2.write() = 30;
        assert_eq!(*counter.borrow(), 5);
    });
}

#[test]
fn test_nested_effect_with_outer_reads_after_inner() {
    create_scope(|| {
        let execution_log = Rc::new(RefCell::new(Vec::new()));
        let log_clone1 = execution_log.clone();
        let log_clone2 = execution_log.clone();
        
        let signal_a = Signal::new(1);
        let signal_b = Signal::new(100);
        let signal_c = Signal::new(1000);
        
        // Outer effect reads signal_a, creates inner effect, then reads signal_c
        create_effect(move || {
            let value_a = *signal_a.read();
            log_clone1.borrow_mut().push(format!("outer-before: a={}", value_a));
            
            // Create inner effect that reads signal_b
            let log_clone3 = log_clone2.clone();
            let signal_a_clone = signal_a;
            create_effect(move || {
                let value_b = *signal_b.read();
                log_clone3.borrow_mut().push(format!("inner: b={}", value_b));
                
                // Inner effect also reads signal_a
                let value_a_inner = *signal_a_clone.read();
                log_clone3.borrow_mut().push(format!("inner-reads-a: a={}", value_a_inner));
            });
            
            // Outer effect continues to read signal_c AFTER creating inner effect
            let value_c = *signal_c.read();
            log_clone1.borrow_mut().push(format!("outer-after: c={}", value_c));
        });
        
        // Clear initial execution log
        execution_log.borrow_mut().clear();
        
        // Update signal_a - should trigger outer effect and all inner effects
        *signal_a.write() = 2;
        {
            let log = execution_log.borrow();
            println!("After signal_a update: {:?}", log);
            
            // Outer effect runs first
            assert!(log.contains(&"outer-before: a=2".to_string()));
            assert!(log.contains(&"outer-after: c=1000".to_string()));
            
            // New inner effect runs
            assert!(log.iter().any(|s| s == "inner: b=100"));
            assert!(log.iter().any(|s| s == "inner-reads-a: a=2"));
            
            // Old inner effect also runs (because it reads signal_a)
            assert_eq!(log.iter().filter(|s| s.starts_with("inner-reads-a:")).count(), 2);
        }
        
        execution_log.borrow_mut().clear();
        
        // Update signal_c - should only trigger outer effect
        *signal_c.write() = 2000;
        {
            let log = execution_log.borrow();
            println!("After signal_c update: {:?}", log);
            
            // Outer effect runs
            assert!(log.contains(&"outer-before: a=2".to_string()));
            assert!(log.contains(&"outer-after: c=2000".to_string()));
            
            // A new inner effect is created
            assert!(log.iter().any(|s| s == "inner: b=100"));
        }
        
        execution_log.borrow_mut().clear();
        
        // Update signal_b - should only trigger all inner effects
        *signal_b.write() = 200;
        {
            let log = execution_log.borrow();
            println!("After signal_b update: {:?}", log);
            
            // Outer effect should NOT run
            assert!(!log.iter().any(|s| s.starts_with("outer-")));
            
            // All inner effects run (3 of them: initial + from signal_a update + from signal_c update)
            assert_eq!(log.iter().filter(|s| *s == "inner: b=200").count(), 3);
        }
    });
}
