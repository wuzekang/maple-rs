//! Test: Portal child capturing signals from dying scope

use sig::prelude::*;
use sig::{portal, dynamic};
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_portal_child_captures_dying_signal() {
    create_scope(|| {
        println!("=== Test: Portal captures signal from dying scope ===\n");

        let _container = portal::provide();

        let read_count = Rc::new(RefCell::new(0));
        let rc = read_count.clone();

        create_scope(move || {
            println!("Child scope: creating short_lived Signal");

            // Signal created in child scope
            let short_lived = Signal::new(42);

            // Portal child captures it directly
            portal::child({
                let rc = rc.clone();
                dynamic(move || {
                    println!("  Dynamic running, attempting to read short_lived...");

                    // Try to read the signal
                    match short_lived.try_read() {
                        Some(val) => {
                            println!("    ✓ Read succeeded: {}", *val);
                            *rc.borrow_mut() += 1;
                        }
                        None => {
                            println!("    ✗ Read failed: Signal was dropped");
                        }
                    }

                    view()
                })
            });

            println!("Child scope: ending");
        });

        println!("\nChild scope destroyed");
        println!("Total successful reads: {}\n", *read_count.borrow());

        // The dynamic should have run at least once
        // But after scope is destroyed, subsequent runs should fail
    });
}

#[test]
fn test_portal_nested_scope_issue() {
    create_scope(|| {
        println!("=== Test: Nested scope Signal capture ===\n");

        let _container = portal::provide();

        // Outer scope Signal (long-lived)
        let outer_signal = Signal::new(1);

        create_scope(move || {
            println!("Inner scope: start");

            // Inner scope Signal (short-lived)
            let inner_signal = Signal::new(2);

            // Portal captures both directly
            portal::child(dynamic(move || {
                println!("  Dynamic running:");

                let outer_val = outer_signal.try_read()
                    .map(|v| *v)
                    .unwrap_or_else(|| {
                        println!("    ✗ outer_signal dropped");
                        -1
                    });

                let inner_val = inner_signal.try_read()
                    .map(|v| *v)
                    .unwrap_or_else(|| {
                        println!("    ✗ inner_signal dropped");
                        -1
                    });

                println!("    outer={}, inner={}", outer_val, inner_val);

                view()
            }));

            println!("Inner scope: ending");
        });

        println!("\nInner scope destroyed (inner_signal dropped)");
        println!("Outer scope still alive (outer_signal alive)\n");

        // Update outer signal to trigger dynamic
        println!("Updating outer_signal...");
        *outer_signal.write() = 10;

        println!("\n✓ Test completed");
    });
}

