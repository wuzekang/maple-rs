//! Test to verify scope ownership and Signal lifecycle issues

use sig::prelude::*;
use sig::dynamic;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_signal_ownership_in_view() {
    create_scope(|| {
        // Track which scopes are created and destroyed
        let scope_log = Rc::new(RefCell::new(Vec::<String>::new()));
        let log1 = scope_log.clone();
        let log2 = scope_log.clone();
        
        let toggle = Signal::new(0);
        
        let _v = view().child(
            dynamic(move || {
                let count = *toggle.read();
                let log1 = log1.clone();
                let log2 = log2.clone();
                
                log1.borrow_mut().push(format!("outer_dynamic_effect_{}", count));
                
                // Inner dynamic - creates Signal in a child scope of the outer dynamic's effect
                dynamic(move || {
                    log2.borrow_mut().push(format!("inner_dynamic_effect_{}", count));
                    view().name(format!("view_{}", count))
                })
            })
        );
        
        println!("=== Initial state ===");
        for log in scope_log.borrow().iter() {
            println!("{}", log);
        }
        
        // Trigger outer dynamic update - this should destroy the old inner dynamic's scope
        *toggle.write() = 1;
        
        println!("\n=== After first toggle ===");
        for log in scope_log.borrow().iter() {
            println!("{}", log);
        }
        
        *toggle.write() = 2;
        
        println!("\n=== After second toggle ===");
        for log in scope_log.borrow().iter() {
            println!("{}", log);
        }
        
        println!("\n=== Scope log summary ===");
        println!("Total events: {}", scope_log.borrow().len());
    });
}

/// Test demonstrating the issue: View's effect tries to flatten
/// a Dynamic's Signal that was created in a child scope
#[test]  
fn test_cross_scope_signal_access() {
    create_scope(|| {
        let trigger = Signal::new(0);
        
        // Create a view that contains a dynamic
        let v = view().child(
            dynamic(move || {
                let t = *trigger.read();
                view().name(format!("child_{}", t))
            })
        );
        
        println!("View ID: {:?}", v.id);
        
        // The View has an effect that reads v.children Signal
        // The Dynamic creates a Signal in a child scope
        // When we update trigger, the Dynamic's effect recreates its scope
        
        *trigger.write() = 1;
        println!("After update 1");
        
        *trigger.write() = 2;
        println!("After update 2");
        
        // At this point, if there's a lifecycle mismatch, we might see issues
        // when the View's effect tries to flatten the Dynamic's Signal
    });
}

/// Test the exact scenario: view().child(dynamic(|| dynamic(|| view())))
#[test]
fn test_triple_nesting_scenario() {
    create_scope(|| {
        println!("=== Testing view().child(dynamic(|| dynamic(|| view()))) ===\n");
        
        let counter = Signal::new(0);
        
        let v = view()
            .name("root_view")
            .child(
                dynamic(move || {
                    let c = *counter.read();
                    println!("Outer dynamic effect running, counter = {}", c);
                    
                    dynamic(move || {
                        println!("  Inner dynamic effect running, counter = {}", c);
                        view().name(format!("inner_view_{}", c))
                    })
                })
            );
        
        println!("\nRoot view ID: {:?}", v.id);
        println!("Root view children signal ID: {:?}", v.children.signal_id());
        
        println!("\n--- Update counter to 1 ---");
        *counter.write() = 1;
        
        println!("\n--- Update counter to 2 ---");
        *counter.write() = 2;
        
        println!("\n=== Test completed ===");
    });
}
