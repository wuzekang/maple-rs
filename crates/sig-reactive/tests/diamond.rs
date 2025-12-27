use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_diamond_dependency() {
    create_scope(move || {
        let run_counts = Rc::new(RefCell::new((0, 0, 0)));
        let count_b = run_counts.clone();
        let count_c = run_counts.clone();
        let count_d = run_counts.clone();
        
        //     a
        //    / \
        //   b   c
        //    \ /
        //     d
        
        let a = Signal::new(1);
        let b = Signal::new(0);
        let c = Signal::new(0);
        let d = Signal::new(0);
        
        // Effect B: reads a, writes b
        create_effect(move || {
            let val = *a.read();
            count_b.borrow_mut().0 += 1;
            *b.write() = val * 2;
        });
        
        // Effect C: reads a, writes c
        create_effect(move || {
            let val = *a.read();
            count_c.borrow_mut().1 += 1;
            *c.write() = val * 3;
        });
        
        // Effect D: reads b and c, writes d
        create_effect(move || {
            let val_b = *b.read();
            let val_c = *c.read();
            count_d.borrow_mut().2 += 1;
            *d.write() = val_b + val_c;
        });
        
        // Initial state
        assert_eq!(*b.read(), 2); // 1 * 2
        assert_eq!(*c.read(), 3); // 1 * 3
        assert_eq!(*d.read(), 5); // 2 + 3
        
        let counts = run_counts.borrow();
        println!("Initial run counts: B={}, C={}, D={}", counts.0, counts.1, counts.2);
    });
}

#[test]
fn test_diamond_dependency_with_update() {
    create_scope(move || {
        let d_runs = Rc::new(RefCell::new(0));
        let d_runs_clone = d_runs.clone();
        
        let a = Signal::new(1);
        let b = Signal::new(0);
        let c = Signal::new(0);
        let d = Signal::new(0);
        
        // a → b
        create_effect(move || {
            *b.write() = *a.read() * 2;
        });
        
        // a → c
        create_effect(move || {
            *c.write() = *a.read() * 3;
        });
        
        // b, c → d
        create_effect(move || {
            *d.write() = *b.read() + *c.read();
            *d_runs_clone.borrow_mut() += 1;
        });
        
        // Clear count
        *d_runs.borrow_mut() = 0;
        
        // Update a
        *a.write() = 2;
        
        // b and c update, d should see final values
        assert_eq!(*b.read(), 4); // 2 * 2
        assert_eq!(*c.read(), 6); // 2 * 3
        assert_eq!(*d.read(), 10); // 4 + 6
        
        // D might run twice (once for b update, once for c update)
        println!("D ran {} times", d_runs.borrow());
        assert!(*d_runs.borrow() >= 1);
    });
}

#[test]
fn test_wide_fan_out() {
    create_scope(move || {
        let a = Signal::new(1);
        let signals: Vec<Signal<i32>> = (0..5).map(|_| Signal::new(0)).collect();
        
        // One signal updates many
        for sig in &signals {
            let sig_copy = *sig;
            create_effect(move || {
                *sig_copy.write() = *a.read() * 2;
            });
        }
        
        // All should be updated
        for sig in &signals {
            assert_eq!(*sig.read(), 2);
        }
        
        // Update a
        *a.write() = 5;
        
        // All should be updated
        for sig in &signals {
            assert_eq!(*sig.read(), 10);
        }
    });
}

#[test]
fn test_wide_fan_in() {
    create_scope(move || {
        let signals: Vec<Signal<i32>> = (0..5).map(|i| Signal::new(i)).collect();
        let result = Signal::new(0);
        let run_count = Rc::new(RefCell::new(0));
        let count_clone = run_count.clone();
        let signals_clone = signals.clone();
        
        // Many signals update one
        create_effect(move || {
            let sum: i32 = signals_clone.iter().map(|s| *s.read()).sum();
            *result.write() = sum;
            *count_clone.borrow_mut() += 1;
        });
        
        assert_eq!(*result.read(), 10); // 0+1+2+3+4
        *run_count.borrow_mut() = 0; // Reset
        
        // Update one signal
        *signals[0].write() = 10;
        assert_eq!(*result.read(), 20); // 10+1+2+3+4
        assert_eq!(*run_count.borrow(), 1);
        
        // Update another signal
        *signals[1].write() = 20;
        assert_eq!(*result.read(), 39); // 10+20+2+3+4
        assert_eq!(*run_count.borrow(), 2);
    });
}

#[test]
fn test_complex_graph() {
    create_scope(move || {
        //      a
        //    / | \
        //   b  c  d
        //    \ | /
        //      e
        
        let a = Signal::new(1);
        let b = Signal::new(0);
        let c = Signal::new(0);
        let d = Signal::new(0);
        let e = Signal::new(0);
        
        // a → b, c, d
        create_effect(move || { *b.write() = *a.read() + 1; });
        create_effect(move || { *c.write() = *a.read() + 2; });
        create_effect(move || { *d.write() = *a.read() + 3; });
        
        // b, c, d → e
        create_effect(move || {
            *e.write() = *b.read() + *c.read() + *d.read();
        });
        
        // Initial: a=1, b=2, c=3, d=4, e=9
        assert_eq!(*e.read(), 9);
        
        // Update a
        *a.write() = 2;
        
        // b=3, c=4, d=5, e=12
        assert_eq!(*b.read(), 3);
        assert_eq!(*c.read(), 4);
        assert_eq!(*d.read(), 5);
        assert_eq!(*e.read(), 12);
    });
}
