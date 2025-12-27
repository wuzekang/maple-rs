use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_untrack_prevents_subscription() {
    create_scope(move || {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let signal = Signal::new(1);
        
        create_effect(move || {
            // Read with untrack - should NOT subscribe
            let value = untrack(|| *signal.read());
            *counter_clone.borrow_mut() = value;
        });
        
        assert_eq!(*counter.borrow(), 1);
        
        // Update signal - effect should NOT run
        *signal.write() = 2;
        assert_eq!(*counter.borrow(), 1); // Still 1, effect didn't run
    });
}

#[test]
fn test_untrack_mixed_with_normal_read() {
    create_scope(move || {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(10);
        
        create_effect(move || {
            // Normal read - subscribes
            let v1 = *signal1.read();
            
            // Untracked read - doesn't subscribe
            let v2 = untrack(|| *signal2.read());
            
            *counter_clone.borrow_mut() = v1 + v2;
        });
        
        assert_eq!(*counter.borrow(), 11);
        
        // Update signal1 - effect runs
        *signal1.write() = 2;
        assert_eq!(*counter.borrow(), 12);
        
        // Update signal2 - effect should NOT run
        *signal2.write() = 20;
        assert_eq!(*counter.borrow(), 12); // Still 12
        
        // Update signal1 again - effect runs and sees new signal2 value
        *signal1.write() = 3;
        assert_eq!(*counter.borrow(), 23); // 3 + 20
    });
}

#[test]
fn test_nested_untrack() {
    create_scope(move || {
        let runs = Rc::new(RefCell::new(0));
        let runs_clone = runs.clone();
        
        let signal = Signal::new(1);
        
        create_effect(move || {
            // Nested untrack
            let value = untrack(|| {
                untrack(|| *signal.read())
            });
            
            *runs_clone.borrow_mut() = value;
        });
        
        assert_eq!(*runs.borrow(), 1);
        
        // Update signal - effect should NOT run
        *signal.write() = 2;
        assert_eq!(*runs.borrow(), 1);
    });
}

#[test]
fn test_read_untracked_api() {
    create_scope(move || {
        let runs = Rc::new(RefCell::new(0));
        let runs_clone = runs.clone();
        
        let signal = Signal::new(1);
        
        create_effect(move || {
            // Use read_untracked() instead of untrack(|| read())
            let _value = *signal.read_untracked();
            *runs_clone.borrow_mut() += 1;
        });
        
        assert_eq!(*runs.borrow(), 1);
        
        // Update signal - effect should NOT run
        *signal.write() = 2;
        assert_eq!(*runs.borrow(), 1);
    });
}

#[test]
fn test_untrack_in_effect_that_modifies() {
    create_scope(move || {
        let trigger = Signal::new(0);
        let counter = Signal::new(0);
        
        create_effect(move || {
            let _ = *trigger.read(); // Subscribe to trigger
            
            // Read counter without subscribing
            let current = untrack(|| *counter.read());
            *counter.write() = current + 1;
        });
        
        assert_eq!(*counter.read(), 1);
        
        // Update trigger - effect runs, counter increments
        *trigger.write() = 1;
        assert_eq!(*counter.read(), 2);
        
        // Update counter directly - effect doesn't run
        *counter.write() = 100;
        assert_eq!(*counter.read(), 100);
        
        // Update trigger again - effect runs with new counter value
        *trigger.write() = 2;
        assert_eq!(*counter.read(), 101);
    });
}

#[test]
fn test_untrack_create_effect_inside() {
    create_scope(move || {
        let outer_runs = Rc::new(RefCell::new(0));
        let inner_runs = Rc::new(RefCell::new(0));
        let outer_clone = outer_runs.clone();
        let inner_clone = inner_runs.clone();
        
        let outer_signal = Signal::new(1);
        let inner_signal = Signal::new(10);
        
        create_effect(move || {
            let _v = *outer_signal.read();
            *outer_clone.borrow_mut() += 1;
            
            // Create effect inside untrack - should still work
            untrack(|| {
                let inner_clone2 = inner_clone.clone();
                create_effect(move || {
                    let _v = *inner_signal.read();
                    *inner_clone2.borrow_mut() += 1;
                });
            });
        });
        
        assert_eq!(*outer_runs.borrow(), 1);
        assert_eq!(*inner_runs.borrow(), 1);
        
        // Update outer - creates new inner effect
        *outer_signal.write() = 2;
        assert_eq!(*outer_runs.borrow(), 2);
        assert_eq!(*inner_runs.borrow(), 2); // New inner effect ran
        
        // Update inner signal - all inner effects run
        *inner_signal.write() = 20;
        assert_eq!(*outer_runs.borrow(), 2); // Outer unchanged
        assert_eq!(*inner_runs.borrow(), 4); // Both inner effects ran
    });
}
