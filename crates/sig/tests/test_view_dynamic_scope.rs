//! Test for scope lifetime issue in View with nested Dynamic

use sig::prelude::*;
use sig::dynamic;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_view_child_nested_dynamic_scope_hierarchy() {
    create_scope(|| {
        let toggle = Signal::new(true);
        
        let flatten_count = Rc::new(RefCell::new(0));
        let fc1 = flatten_count.clone();
        let fc2 = flatten_count.clone();
        
        // This creates a problematic scope hierarchy:
        // - View's signal in Scope A
        // - View's effect reads Dynamic's signal
        // - Outer Dynamic's signal in Scope B (child of A)  
        // - Inner Dynamic's signal in Scope C (child of effect B, grandchild of A)
        
        let _v = view().child(
            dynamic(move || {
                let fc1 = fc1.clone();
                let fc2 = fc2.clone();
                if *toggle.read() {
                    dynamic(move || {
                        *fc1.borrow_mut() += 1;
                        view().name("inner_true")
                    })
                } else {
                    dynamic(move || {
                        *fc2.borrow_mut() += 1;
                        view().name("inner_false")
                    })
                }
            })
        );
        
        println!("Initial flatten_count: {}", *flatten_count.borrow());
        
        // This should trigger the outer dynamic's effect
        // which will destroy the old inner dynamic's scope
        *toggle.write() = false;
        
        println!("After toggle, flatten_count: {}", *flatten_count.borrow());
    });
}

#[test]
fn test_simple_view_dynamic_scope() {
    create_scope(|| {
        let counter = Signal::new(0);
        
        // Simpler case: view().child(dynamic(...))
        let _v = view().child(
            dynamic(move || {
                let count = *counter.read();
                view().name(format!("child_{}", count))
            })
        );
        
        // Update counter - this recreates the inner view
        *counter.write() = 1;
        *counter.write() = 2;
        
        println!("Simple view+dynamic completed");
    });
}

#[test]
fn test_deeply_nested_dynamic() {
    create_scope(|| {
        let level1 = Signal::new(0);
        let level2 = Signal::new(0);
        
        let _v = view().child(
            dynamic(move || {
                let l1 = *level1.read();
                dynamic(move || {
                    let l2 = *level2.read();
                    view().name(format!("nested_{}_{}", l1, l2))
                })
            })
        );
        
        println!("Initial state");
        
        // Update level2 - inner dynamic updates
        *level2.write() = 1;
        println!("After level2 update");
        
        // Update level1 - outer dynamic updates, inner dynamic recreated
        *level1.write() = 1;
        println!("After level1 update - inner dynamic should be recreated");
        
        // Update level2 again
        *level2.write() = 2;
        println!("After second level2 update");
    });
}
