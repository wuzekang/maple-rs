//! Test for on_cleanup functionality

use sig::{create_scope, on_cleanup};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_on_cleanup_in_scope() {
    let cleanup_called = Rc::new(RefCell::new(false));
    let cleanup_called_clone = cleanup_called.clone();

    create_scope(|| {
        on_cleanup(move || {
            *cleanup_called_clone.borrow_mut() = true;
        });
    });

    assert_eq!(
        *cleanup_called.borrow(),
        true,
        "Cleanup should be called when scope is destroyed"
    );
}

#[test]
fn test_multiple_cleanups() {
    let cleanup1_called = Rc::new(RefCell::new(false));
    let cleanup2_called = Rc::new(RefCell::new(false));
    let cleanup1_clone = cleanup1_called.clone();
    let cleanup2_clone = cleanup2_called.clone();

    create_scope(|| {
        on_cleanup(move || {
            *cleanup1_clone.borrow_mut() = true;
        });
        on_cleanup(move || {
            *cleanup2_clone.borrow_mut() = true;
        });
    });

    assert_eq!(*cleanup1_called.borrow(), true, "First cleanup should be called");
    assert_eq!(*cleanup2_called.borrow(), true, "Second cleanup should be called");
}

#[test]
fn test_cleanup_execution_order() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let order1 = order.clone();
    let order2 = order.clone();
    let order3 = order.clone();

    create_scope(|| {
        on_cleanup(move || {
            order1.borrow_mut().push(1);
        });
        on_cleanup(move || {
            order2.borrow_mut().push(2);
        });
        on_cleanup(move || {
            order3.borrow_mut().push(3);
        });
    });

    // Cleanups should be called in reverse order (LIFO)
    let execution_order = order.borrow().clone();
    assert_eq!(
        execution_order,
        vec![3, 2, 1],
        "Cleanups should be called in reverse order (LIFO)"
    );
}

#[test]
fn test_nested_scope_cleanup() {
    let outer_cleanup = Rc::new(RefCell::new(false));
    let inner_cleanup = Rc::new(RefCell::new(false));
    let outer_clone = outer_cleanup.clone();
    let inner_clone = inner_cleanup.clone();
    let inner_check = inner_cleanup.clone();

    create_scope(move || {
        on_cleanup(move || {
            *outer_clone.borrow_mut() = true;
        });

        create_scope(move || {
            on_cleanup(move || {
                *inner_clone.borrow_mut() = true;
            });
        });

        // Inner cleanup should be called when inner scope ends
        assert_eq!(
            *inner_check.borrow(),
            true,
            "Inner cleanup should be called first"
        );
    });

    // Outer cleanup should be called when outer scope ends
    assert_eq!(
        *outer_cleanup.borrow(),
        true,
        "Outer cleanup should be called after scope ends"
    );
}

#[test]
fn test_on_cleanup_with_resource() {
    // Simulate resource management with on_cleanup
    let resource_dropped = Rc::new(RefCell::new(false));
    let resource_dropped_clone = resource_dropped.clone();

    create_scope(|| {
        // Simulate resource acquisition
        let _resource_id = 42;
        
        on_cleanup(move || {
            // Simulate resource cleanup
            *resource_dropped_clone.borrow_mut() = true;
        });
    });

    assert_eq!(
        *resource_dropped.borrow(),
        true,
        "Resource should be cleaned up when scope is destroyed"
    );
}
