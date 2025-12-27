use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_on_cleanup_basic() {
    let cleanup_called = Rc::new(RefCell::new(false));
    let cleanup_flag = cleanup_called.clone();
    let cleanup_check = cleanup_called.clone();
    
    create_scope(move || {
        on_cleanup(move || {
            *cleanup_flag.borrow_mut() = true;
        });
        
        // Cleanup should not be called yet
        assert_eq!(*cleanup_check.borrow(), false);
    });
    
    // After scope ends, cleanup should be called
    assert_eq!(*cleanup_called.borrow(), true);
}

#[test]
fn test_on_cleanup_lifo_order() {
    let execution_order = Rc::new(RefCell::new(Vec::new()));
    let order1 = execution_order.clone();
    let order2 = execution_order.clone();
    let order3 = execution_order.clone();
    
    create_scope(|| {
        on_cleanup(move || {
            order1.borrow_mut().push("first");
        });
        
        on_cleanup(move || {
            order2.borrow_mut().push("second");
        });
        
        on_cleanup(move || {
            order3.borrow_mut().push("third");
        });
    });
    
    // Cleanup should be called in LIFO order (reverse of registration)
    assert_eq!(*execution_order.borrow(), vec!["third", "second", "first"]);
}

#[test]
fn test_effect_cleanup_on_rerun() {
    create_scope(|| {
        let cleanup_count = Rc::new(RefCell::new(0));
        let cleanup_clone = cleanup_count.clone();
        
        let signal = Signal::new(1);
        
        create_effect(move || {
            let value = *signal.read();
            
            let cleanup_clone2 = cleanup_clone.clone();
            on_cleanup(move || {
                *cleanup_clone2.borrow_mut() += 1;
            });
            
            println!("Effect running with value: {}", value);
        });
        
        // No cleanup yet (effect ran once)
        assert_eq!(*cleanup_count.borrow(), 0);
        
        // Update signal - effect reruns, previous cleanup should run first
        *signal.write() = 2;
        assert_eq!(*cleanup_count.borrow(), 1);
        
        // Update again
        *signal.write() = 3;
        assert_eq!(*cleanup_count.borrow(), 2);
    });
    
    // Final cleanup when scope ends
    // Note: cleanup_count is already 2 from the effect reruns
}

#[test]
fn test_nested_scope_cleanup() {
    let outer_cleanup = Rc::new(RefCell::new(false));
    let inner_cleanup = Rc::new(RefCell::new(false));
    let outer_flag = outer_cleanup.clone();
    let inner_flag = inner_cleanup.clone();
    
    let outer_check = outer_cleanup.clone();
    let inner_check = inner_cleanup.clone();
    
    create_scope(move || {
        on_cleanup(move || {
            *outer_flag.borrow_mut() = true;
        });
        
        let inner_check2 = inner_check.clone();
        let outer_check2 = outer_check.clone();
        create_scope(move || {
            on_cleanup(move || {
                *inner_flag.borrow_mut() = true;
            });
            
            // Neither cleanup called yet
            assert_eq!(*inner_check2.borrow(), false);
            assert_eq!(*outer_check2.borrow(), false);
        });
        
        // Inner scope ended, inner cleanup called
        assert_eq!(*inner_check.borrow(), true);
        assert_eq!(*outer_check.borrow(), false);
    });
    
    // Outer scope ended, outer cleanup called
    assert_eq!(*outer_cleanup.borrow(), true);
}

#[test]
fn test_cleanup_with_captured_values() {
    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();
    
    create_scope(move || {
        let local_value = 42;
        
        let values_clone2 = values_clone.clone();
        on_cleanup(move || {
            values_clone2.borrow_mut().push(local_value);
        });
        
        on_cleanup(move || {
            values_clone.borrow_mut().push(100);
        });
    });
    
    // Cleanups should have captured and pushed values in LIFO order
    assert_eq!(*values.borrow(), vec![100, 42]);
}

#[test]
fn test_effect_cleanup_multiple_signals() {
    create_scope(|| {
        let cleanup_log = Rc::new(RefCell::new(Vec::new()));
        let log_clone = cleanup_log.clone();
        
        let signal1 = Signal::new(1);
        let signal2 = Signal::new(10);
        
        create_effect(move || {
            let v1 = *signal1.read();
            let v2 = *signal2.read();
            
            let log_clone2 = log_clone.clone();
            on_cleanup(move || {
                log_clone2.borrow_mut().push(format!("cleanup: v1={}, v2={}", v1, v2));
            });
        });
        
        // No cleanup yet
        assert_eq!(cleanup_log.borrow().len(), 0);
        
        // Update signal1 - cleanup should run before rerun
        *signal1.write() = 2;
        assert_eq!(cleanup_log.borrow().len(), 1);
        assert_eq!(cleanup_log.borrow()[0], "cleanup: v1=1, v2=10");
        
        // Update signal2
        *signal2.write() = 20;
        assert_eq!(cleanup_log.borrow().len(), 2);
        assert_eq!(cleanup_log.borrow()[1], "cleanup: v1=2, v2=10");
    });
}

#[test]
fn test_cleanup_in_nested_effects() {
    create_scope(|| {
        let outer_cleanups = Rc::new(RefCell::new(0));
        let inner_cleanups = Rc::new(RefCell::new(0));
        let outer_clone = outer_cleanups.clone();
        let inner_clone = inner_cleanups.clone();
        
        let outer_signal = Signal::new(1);
        let inner_signal = Signal::new(10);
        
        create_effect(move || {
            let _outer = *outer_signal.read();
            
            let outer_clone2 = outer_clone.clone();
            on_cleanup(move || {
                *outer_clone2.borrow_mut() += 1;
            });
            
            // Create inner effect
            let inner_clone2 = inner_clone.clone();
            create_effect(move || {
                let _inner = *inner_signal.read();
                
                let inner_clone3 = inner_clone2.clone();
                on_cleanup(move || {
                    *inner_clone3.borrow_mut() += 1;
                });
            });
        });
        
        // Initial state - no cleanups yet
        assert_eq!(*outer_cleanups.borrow(), 0);
        assert_eq!(*inner_cleanups.borrow(), 0);
        
        // Update inner signal - only inner effect reruns, its cleanup runs
        *inner_signal.write() = 20;
        assert_eq!(*outer_cleanups.borrow(), 0);
        assert_eq!(*inner_cleanups.borrow(), 1); // First inner effect cleanup
        
        // Update outer signal - outer reruns (cleanup), creates new inner effect
        *outer_signal.write() = 2;
        assert_eq!(*outer_cleanups.borrow(), 1); // Outer cleanup
        assert_eq!(*inner_cleanups.borrow(), 1); // Old inner cleanups
        
        // Update inner signal again - triggers ALL inner effects
        *inner_signal.write() = 30;
        assert_eq!(*outer_cleanups.borrow(), 1);
        assert_eq!(*inner_cleanups.borrow(), 3); // Both inner effects cleanup
    });
}
