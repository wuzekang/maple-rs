//! Deep dive: Why doesn't the effect get triggered?

use sig::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_subscription_mechanism_across_scopes() {
    create_scope(|| {
        println!("=== Investigating subscription mechanism ===\n");
        
        let signal_holder: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let holder_for_effect = signal_holder.clone();
        let reads = Rc::new(RefCell::new(Vec::<i32>::new()));
        let reads_clone = reads.clone();
        
        println!("Step 1: Create Signal in child scope FIRST");
        let child_scope_id = {
            let holder_clone = signal_holder.clone();
            let (child_signal, scope_id) = sig::reactive::as_child_scope(move |_: ()| {
                let child_signal = Signal::new(100);
                *holder_clone.borrow_mut() = Some(child_signal);
                child_signal
            })(());
            println!("  Created Signal ID: {:?} in Scope {:?}", child_signal.signal_id(), scope_id);
            scope_id
        };
        
        println!("\nStep 2: Create effect AFTER Signal exists");
        create_effect(move || {
            if let Some(sig) = *holder_for_effect.borrow() {
                let val = *sig.read();
                reads_clone.borrow_mut().push(val);
                println!("  Effect read: {}", val);
            }
        });
        
        println!("  Reads after effect creation: {:?}", *reads.borrow());
        
        println!("\nStep 3: Write to Signal");
        if let Some(sig) = *signal_holder.borrow() {
            println!("  Writing 200...");
            *sig.write() = 200;
        }
        println!("  Reads after write: {:?}", *reads.borrow());
        
        if reads.borrow().len() >= 2 {
            println!("\n✓ Effect WAS triggered!");
        } else {
            println!("\n✗ Effect was NOT triggered (BUG)");
        }
        
        println!("\nStep 4: Destroy child scope");
        sig::reactive::remove_scope(child_scope_id);
        
        println!("\nStep 5: Try to write again");
        if let Some(sig) = *signal_holder.borrow() {
            match sig.try_write() {
                Some(_) => println!("  ERROR: Write succeeded after scope destroyed!"),
                None => println!("  ✓ Write failed (correct)"),
            }
        }
    });
}

#[test]
fn test_effect_creation_order_matters() {
    create_scope(|| {
        println!("=== Does effect creation order matter? ===\n");
        
        println!("--- Test A: Effect BEFORE Signal ---");
        let holder_a: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let holder_a_clone = holder_a.clone();
        let reads_a = Rc::new(RefCell::new(0));
        let reads_a_clone = reads_a.clone();
        
        create_effect(move || {
            if let Some(sig) = *holder_a_clone.borrow() {
                let _ = *sig.read();
                *reads_a_clone.borrow_mut() += 1;
                println!("  Effect A ran, count: {}", *reads_a_clone.borrow());
            }
        });
        
        let sig_a = Signal::new(100);
        *holder_a.borrow_mut() = Some(sig_a);
        
        *sig_a.write() = 200;
        println!("Effect A run count: {}\n", *reads_a.borrow());
        
        println!("--- Test B: Signal BEFORE Effect ---");
        let sig_b = Signal::new(100);
        let reads_b = Rc::new(RefCell::new(0));
        let reads_b_clone = reads_b.clone();
        
        create_effect(move || {
            let _ = *sig_b.read();
            *reads_b_clone.borrow_mut() += 1;
            println!("  Effect B ran, count: {}", *reads_b_clone.borrow());
        });
        
        *sig_b.write() = 200;
        println!("Effect B run count: {}", *reads_b.borrow());
    });
}

#[test]
fn test_minimal_repro() {
    create_scope(|| {
        println!("=== Minimal reproduction case ===\n");
        
        // This is the simplest case that shows the problem
        let holder: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let holder_clone = holder.clone();
        let count = Rc::new(RefCell::new(0));
        let count_clone = count.clone();
        
        // Effect created BEFORE Signal exists
        create_effect(move || {
            if let Some(sig) = *holder_clone.borrow() {
                let val = *sig.read();  // Subscribe to Signal
                *count_clone.borrow_mut() += 1;
                println!("Effect ran #{}, value: {}", *count_clone.borrow(), val);
            } else {
                println!("Effect ran but no signal yet");
            }
        });
        
        println!("After effect creation, count: {}\n", *count.borrow());
        
        // Create Signal in child scope
        let sig = {
            let holder_clone2 = holder.clone();
            let (sig, _scope_id) = sig::reactive::as_child_scope(move |_: ()| {
                let s = Signal::new(42);
                println!("Created Signal: {:?}", s.signal_id());
                *holder_clone2.borrow_mut() = Some(s);
                s
            })(());
            sig
        };
        
        println!("After signal creation, count: {}\n", *count.borrow());
        
        // Write to Signal - will effect run?
        println!("Writing 100...");
        *sig.write() = 100;
        println!("After write, count: {}\n", *count.borrow());
        
        // Analysis
        if *count.borrow() == 1 {
            println!("Result: Effect ran initially but NOT on write (BUG)");
        } else if *count.borrow() == 2 {
            println!("Result: Effect ran on write (CORRECT)");
        } else {
            println!("Result: Unexpected count: {}", *count.borrow());
        }
    });
}
