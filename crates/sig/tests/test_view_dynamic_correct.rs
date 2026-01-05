//! Correct test: Does View's effect properly track Dynamic's Signal changes?

use sig::prelude::*;
use sig::dynamic;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_view_tracks_dynamic_correctly() {
    create_scope(|| {
        println!("=== Test: View's effect should track Dynamic Signal changes ===\n");
        
        let trigger = Signal::new(0);
        let effect_runs = Rc::new(RefCell::new(0));
        let er_clone = effect_runs.clone();
        
        println!("Creating view with dynamic child...");
        let v = view()
            .name("root")
            .child(
                dynamic(move || {
                    let t = *trigger.read();
                    println!("  Dynamic effect running, trigger = {}", t);
                    view().name(format!("child_{}", t))
                })
            );
        
        // The View's effect should have run to flatten the children
        println!("Initial state\n");
        
        // Now update the trigger - this should:
        // 1. Trigger Dynamic's effect
        // 2. Dynamic creates a new child view
        // 3. Dynamic's Signal (which stores the children) is updated
        // 4. View's effect should detect this and re-flatten
        
        println!("Writing trigger = 1...");
        *trigger.write() = 1;
        println!();
        
        println!("Writing trigger = 2...");
        *trigger.write() = 2;
        println!();
        
        // The real question: Is View's effect being triggered?
        // We can't directly measure this without instrumenting the code,
        // but we can check if the view tree is being updated correctly
        
        println!("View ID: {:?}", v.id);
        println!("View children count: {}", v.id.get_children().len());
    });
}

#[test]
fn test_nested_dynamic_signal_lifecycle() {
    create_scope(|| {
        println!("=== Test: Nested Dynamic Signal Lifecycle ===\n");
        
        let outer_trigger = Signal::new(0);
        
        println!("Creating view().child(dynamic(|| dynamic(|| view())))...");
        let v = view().child(
            dynamic(move || {
                let t = *outer_trigger.read();
                println!("  Outer dynamic effect, trigger = {}", t);
                
                dynamic(move || {
                    println!("    Inner dynamic effect, trigger = {}", t);
                    view().name(format!("inner_{}", t))
                })
            })
        );
        
        println!("Initial state");
        println!("  View children count: {}\n", v.id.get_children().len());
        
        println!("Update outer_trigger to 1...");
        *outer_trigger.write() = 1;
        println!("  View children count: {}\n", v.id.get_children().len());
        
        println!("Update outer_trigger to 2...");
        *outer_trigger.write() = 2;
        println!("  View children count: {}\n", v.id.get_children().len());
        
        // Analysis:
        // - Outer dynamic's effect runs, creating a new inner dynamic
        // - Old inner dynamic's scope is destroyed
        // - New inner dynamic's Signal is created
        // - View's effect should track the NEW Signal
    });
}

#[test]
fn test_signal_in_same_vs_child_scope() {
    create_scope(|| {
        println!("=== Test: Signal subscription in same vs child scope ===\n");
        
        println!("--- Test A: Signal in SAME scope as Effect ---");
        let sig_a = Signal::new(100);
        let count_a = Rc::new(RefCell::new(0));
        let count_a_clone = count_a.clone();
        
        create_effect(move || {
            let val = *sig_a.read();
            *count_a_clone.borrow_mut() += 1;
            println!("  Effect A ran #{}, value: {}", *count_a_clone.borrow(), val);
        });
        
        *sig_a.write() = 200;
        println!("Effect A total runs: {}\n", *count_a.borrow());
        
        println!("--- Test B: Signal in CHILD scope, Effect in PARENT ---");
        let (sig_b, child_scope) = sig::reactive::as_child_scope(|_: ()| {
            Signal::new(100)
        })(());
        
        let count_b = Rc::new(RefCell::new(0));
        let count_b_clone = count_b.clone();
        
        create_effect(move || {
            let val = *sig_b.read();
            *count_b_clone.borrow_mut() += 1;
            println!("  Effect B ran #{}, value: {}", *count_b_clone.borrow(), val);
        });
        
        *sig_b.write() = 200;
        println!("Effect B total runs: {}", *count_b.borrow());
        
        if *count_a.borrow() == *count_b.borrow() {
            println!("\n✓ SAME behavior regardless of scope!");
        } else {
            println!("\n✗ DIFFERENT behavior! count_a={}, count_b={}", 
                     *count_a.borrow(), *count_b.borrow());
        }
    });
}

/// This is the real test: Does View's flatten read the correct Signal?
#[test]
fn test_view_flatten_reads_dynamic_signal() {
    create_scope(|| {
        println!("=== Test: What Signal does View's flatten read? ===\n");
        
        let trigger = Signal::new(0);
        
        // Create a Dynamic - it has a Signal internally
        let dyn_node = dynamic(move || {
            let t = *trigger.read();
            println!("  Dynamic effect running, trigger = {}", t);
            view().name(format!("child_{}", t))
        });
        
        // When we call .into_node(), it returns Node::Dynamic(signal)
        // That signal is what View's effect will read via flatten()
        
        println!("Creating view with dynamic...");
        let v = view().child(dyn_node);
        
        println!("View created");
        println!("View.children Signal: {:?}\n", v.children.signal_id());
        
        // Update trigger - Dynamic's signal should change
        println!("Updating trigger...");
        *trigger.write() = 1;
        
        println!("\nView children: {}", v.id.get_children().len());
        
        // The question: Did View's effect run and call flatten()?
        // flatten() should read Dynamic's signal via:
        // Node::Dynamic(signal) => signal.read()
    });
}
