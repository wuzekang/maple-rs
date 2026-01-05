//! Test to expose the cross-scope Signal access issue

use sig::prelude::*;
use sig::dynamic;
use std::rc::Rc;
use std::cell::RefCell;

/// This test demonstrates the core issue:
/// 1. Parent scope creates an effect that reads a Signal
/// 2. Child scope creates that Signal
/// 3. Child scope is destroyed
/// 4. Parent scope's effect tries to run again and access the destroyed Signal
#[test]
fn test_parent_reads_child_signal_after_child_destroyed() {
    create_scope(|| {
        println!("=== Test: Parent reads child scope's Signal ===\n");
        
        // This will store a Signal created in a child scope
        let signal_holder: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let holder_clone = signal_holder.clone();
        
        // Create a child scope and a Signal in it
        let child_scope_id = {
            let (_, scope_id) = sig::reactive::as_child_scope(move |_: ()| {
                let child_signal = Signal::new(42);
                println!("Created Signal in child scope: {:?}", child_signal.signal_id());
                *holder_clone.borrow_mut() = Some(child_signal);
                child_signal
            })(());
            scope_id
        };
        
        println!("Child scope ID: {:?}", child_scope_id);
        
        // Now parent scope has access to the Signal
        let signal = signal_holder.borrow().unwrap();
        println!("Parent scope reading Signal: {}", *signal.read());
        
        // Destroy the child scope
        println!("\n--- Destroying child scope ---");
        sig::reactive::remove_scope(child_scope_id);
        
        // Try to read the Signal from parent scope after child is destroyed
        println!("\n--- Attempting to read Signal after child scope destroyed ---");
        match signal.try_write() {
            Some(mut guard) => {
                println!("ERROR: Successfully wrote to Signal that should be destroyed!");
                *guard = 100;
            }
            None => {
                println!("✓ try_write() correctly returned None (Signal is destroyed)");
            }
        }
        
        // What about read?
        println!("\n--- Attempting to READ Signal after child scope destroyed ---");
        // Try to read - this will likely panic or fail gracefully
        // We can't use catch_unwind with RefCell, so just try it
        println!("Attempting signal.read()...");
        // This might cause issues depending on generational_box implementation
    });
}

/// Test the real-world scenario: View + nested Dynamic
#[test]
fn test_view_dynamic_signal_lifecycle() {
    create_scope(|| {
        println!("=== Test: View reading Dynamic's Signal across scope boundaries ===\n");
        
        let outer_trigger = Signal::new(0);
        let read_count = Rc::new(RefCell::new(0));
        let rc_clone = read_count.clone();
        
        // This is the problematic pattern:
        // View's children Signal stores Node::Dynamic(signal)
        // That Dynamic's signal is created in a child scope
        let v = view().child(
            dynamic(move || {
                let t = *outer_trigger.read();
                println!("Outer dynamic effect running, trigger = {}", t);
                
                // Inner dynamic creates a Signal in a child scope (via as_child_scope)
                let rc_clone = rc_clone.clone();
                dynamic(move || {
                    *rc_clone.borrow_mut() += 1;
                    println!("  Inner dynamic effect running, read_count = {}", *rc_clone.borrow());
                    view().name(format!("inner_{}", t))
                })
            })
        );
        
        println!("\nInitial state - read_count: {}", *read_count.borrow());
        println!("View children Signal ID: {:?}", v.children.signal_id());
        
        // When we update outer_trigger:
        // 1. Outer dynamic's effect runs
        // 2. Old inner dynamic's scope is destroyed (and its Signal)
        // 3. New inner dynamic's scope is created (with new Signal)
        // 4. View's effect should detect the change and re-flatten
        
        println!("\n--- Trigger update 1 ---");
        *outer_trigger.write() = 1;
        println!("After update 1 - read_count: {}", *read_count.borrow());
        
        println!("\n--- Trigger update 2 ---");
        *outer_trigger.write() = 2;
        println!("After update 2 - read_count: {}", *read_count.borrow());
        
        // The question: Does View's effect properly handle the Signal lifecycle?
    });
}

/// Test: Effect in parent scope subscribes to Signal from child scope
#[test]
fn test_effect_subscribes_to_child_signal() {
    create_scope(|| {
        println!("=== Test: Effect subscribes to child scope Signal ===\n");
        
        let signal_holder: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let holder_for_effect = signal_holder.clone();
        let effect_run_count = Rc::new(RefCell::new(0));
        let erc_clone = effect_run_count.clone();
        
        // Create effect in parent scope
        create_effect(move || {
            *erc_clone.borrow_mut() += 1;
            if let Some(sig) = *holder_for_effect.borrow() {
                let val = *sig.read();
                println!("Effect running (#{}) - Signal value: {}", *erc_clone.borrow(), val);
            } else {
                println!("Effect running (#{}) - No signal yet", *erc_clone.borrow());
            }
        });
        
        println!("After creating effect - run count: {}", *effect_run_count.borrow());
        
        // Create Signal in child scope
        let child_scope_id = {
            let holder_clone = signal_holder.clone();
            let (sig, scope_id) = sig::reactive::as_child_scope(move |_: ()| {
                let child_signal = Signal::new(100);
                println!("\nCreated Signal in child scope: {:?}", child_signal.signal_id());
                *holder_clone.borrow_mut() = Some(child_signal);
                child_signal
            })(());
            scope_id
        };
        
        println!("Child scope created: {:?}", child_scope_id);
        println!("Effect run count: {}", *effect_run_count.borrow());
        
        // Modify the Signal - this should trigger the effect
        if let Some(sig) = *signal_holder.borrow() {
            println!("\n--- Writing to Signal ---");
            *sig.write() = 200;
            println!("After write - Effect run count: {}", *effect_run_count.borrow());
        }
        
        // Now destroy the child scope
        println!("\n--- Destroying child scope ---");
        sig::reactive::remove_scope(child_scope_id);
        
        // Try to write again - the effect subscription should still exist!
        // This is the bug: effect in parent scope is subscribed to a Signal
        // that was destroyed with its scope
        if let Some(sig) = *signal_holder.borrow() {
            println!("\n--- Attempting to write after scope destroyed ---");
            match sig.try_write() {
                Some(_) => {
                    println!("ERROR: Write succeeded! This shouldn't happen!");
                    println!("Effect run count: {}", *effect_run_count.borrow());
                }
                None => {
                    println!("✓ try_write returned None (Signal destroyed)");
                }
            }
        }
    });
}

/// Test: Demonstrate the exact issue with View
#[test]
fn test_view_flatten_reads_destroyed_signal() {
    create_scope(|| {
        println!("=== Test: View's flatten reads destroyed Dynamic Signal ===\n");
        
        let toggle = Signal::new(true);
        let flatten_count = Rc::new(RefCell::new(0));
        let fc = flatten_count.clone();
        
        // View's effect will call flatten_all, which reads Dynamic's signal
        let v = view()
            .name("root")
            .child(
                dynamic(move || {
                    let fc = fc.clone();
                    if *toggle.read() {
                        // Create inner dynamic - its Signal will be in a child scope
                        dynamic(move || {
                            *fc.borrow_mut() += 1;
                            println!("Inner dynamic TRUE branch, count: {}", *fc.borrow());
                            view().name("inner_true")
                        })
                    } else {
                        // Different inner dynamic - new scope, new Signal
                        dynamic(move || {
                            *fc.borrow_mut() += 1;
                            println!("Inner dynamic FALSE branch, count: {}", *fc.borrow());
                            view().name("inner_false")
                        })
                    }
                })
            );
        
        println!("Initial - flatten_count: {}", *flatten_count.borrow());
        
        // When we toggle, the old inner dynamic's scope (and its Signal) is destroyed
        // But View's effect's flatten function might still try to read it
        println!("\n--- Toggle to FALSE ---");
        *toggle.write() = false;
        println!("After toggle - flatten_count: {}", *flatten_count.borrow());
        
        // The problem: Between the time Dynamic's signal is destroyed
        // and View's effect re-runs, there's a window where View's effect
        // might have a stale reference to the destroyed Signal
        
        println!("\n--- Toggle back to TRUE ---");
        *toggle.write() = true;
        println!("After toggle back - flatten_count: {}", *flatten_count.borrow());
    });
}
