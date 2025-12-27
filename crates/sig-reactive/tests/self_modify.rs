use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_effect_self_modify_with_exit_condition() {
    create_scope(|| {
        let signal = Signal::new(0);
        let run_count = Rc::new(RefCell::new(0));
        let run_count_clone = run_count.clone();
        
        // Effect modifies the signal it reads, but has exit condition
        create_effect(move || {
            let value = *signal.read();  // ReadGuard drops immediately
            *run_count_clone.borrow_mut() += 1;
            
            if value < 10 {
                *signal.write() = value + 1;  // Should work!
            }
        });
        
        // Effect should have run 11 times (0..10)
        assert_eq!(*run_count.borrow(), 11);
        assert_eq!(*signal.read(), 10);
    });
}

#[test]
fn test_circular_dependency_a_b_c_a() {
    create_scope(|| {
        let signal_a = Signal::new(0);
        let signal_b = Signal::new(0);
        let signal_c = Signal::new(0);
        
        let run_counts = Rc::new(RefCell::new((0, 0, 0)));
        let counts1 = run_counts.clone();
        let counts2 = run_counts.clone();
        let counts3 = run_counts.clone();
        
        // Effect A: reads signal_a, writes signal_b
        create_effect(move || {
            let val_a = *signal_a.read();
            counts1.borrow_mut().0 += 1;
            
            if val_a < 3 {
                *signal_b.write() = val_a + 1;
            }
        });
        
        // Effect B: reads signal_b, writes signal_c
        create_effect(move || {
            let val_b = *signal_b.read();
            counts2.borrow_mut().1 += 1;
            
            if val_b < 3 {
                *signal_c.write() = val_b + 1;
            }
        });
        
        // Effect C: reads signal_c, writes signal_a (completes the circle!)
        create_effect(move || {
            let val_c = *signal_c.read();
            counts3.borrow_mut().2 += 1;
            
            if val_c < 3 {
                *signal_a.write() = val_c + 1;
            }
        });
        
        // The cycle runs: a=0 -> b=1 -> c=2 -> a=3 (stop at a=3)
        let counts = run_counts.borrow();
        println!("Run counts: A={}, B={}, C={}", counts.0, counts.1, counts.2);
        
        // Effect A runs twice: initial (a=0) and after c writes a=3
        assert_eq!(counts.0, 2, "Effect A should run 2 times");
        // Effect B runs once: after a writes b=1
        assert_eq!(counts.1, 1, "Effect B should run 1 time");
        // Effect C runs once: after b writes c=2
        assert_eq!(counts.2, 1, "Effect C should run 1 time");
        
        // Final values
        assert_eq!(*signal_a.read(), 3);
        assert_eq!(*signal_b.read(), 1);
        assert_eq!(*signal_c.read(), 2);
    });
}

#[test]
fn test_circular_dependency_with_external_trigger() {
    create_scope(|| {
        let a = Signal::new(0);
        let b = Signal::new(0);
        let c = Signal::new(0);
        
        // a -> b -> c -> a cycle
        create_effect(move || {
            let val = *a.read();
            if val > 0 && val < 5 {
                *b.write() = val;
            }
        });
        
        create_effect(move || {
            let val = *b.read();
            if val > 0 && val < 5 {
                *c.write() = val;
            }
        });
        
        create_effect(move || {
            let val = *c.read();
            if val > 0 && val < 5 {
                *a.write() = val + 1;
            }
        });
        
        // All start at 0, no cycle yet
        assert_eq!(*a.read(), 0);
        
        // Trigger the cycle by setting a=1
        *a.write() = 1;
        
        // Should cycle: a=1 -> b=1 -> c=1 -> a=2 -> b=2 -> c=2 -> ... -> a=5 (stop)
        assert_eq!(*a.read(), 5);
        assert_eq!(*b.read(), 4);
        assert_eq!(*c.read(), 4);
    });
}

#[test]
fn test_circular_dependency_immediate_stop() {
    create_scope(|| {
        let a = Signal::new(10);
        let b = Signal::new(10);
        let c = Signal::new(10);
        
        // All conditions are false from start, so cycle neverarts
        create_effect(move || {
            let val = *a.read();
            if val < 5 {
                *b.write() = val + 1;
            }
        });
        
        create_effect(move || {
            let val = *b.read();
            if val < 5 {
                *c.write() = val + 1;
            }
        });
        
        create_effect(move || {
            let val = *c.read();
            if val < 5 {
                *a.write() = val + 1;
            }
        });
        
        // No cycle should occur
        assert_eq!(*a.read(), 10);
        assert_eq!(*b.read(), 10);
        assert_eq!(*c.read(), 10);
    });
}
