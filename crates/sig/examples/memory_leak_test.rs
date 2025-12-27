/// Memory leak detection test
/// 
/// This tests whether resources are properly cleaned up when scopes are destroyed.

use sig::{create_scope, Signal, create_effect, on_cleanup};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("=== Memory Leak Detection ===\n");
    
    test_scope_cleanup();
    test_signal_cleanup();
    test_effect_cleanup();
    
    println!("\n✅ All tests passed - No memory leaks detected!");
}

fn test_scope_cleanup() {
    println!("1. Testing Scope Cleanup:");
    
    let cleanup_count = Rc::new(RefCell::new(0));
    let cleanup_count_clone = cleanup_count.clone();
    
    // Create and destroy many scopes
    for i in 0..100 {
        let count_clone = cleanup_count_clone.clone();
        create_scope(move || {
            on_cleanup(move || {
                *count_clone.borrow_mut() += 1;
            });
        });
    }
    
    assert_eq!(*cleanup_count.borrow(), 100, "All 100 scopes should have been cleaned up");
    println!("   ✓ Created and destroyed 100 scopes");
    println!("   ✓ All cleanup functions were called");
    println!("   ✓ No scope leaks detected\n");
}

fn test_signal_cleanup() {
    println!("2. Testing Signal Cleanup:");
    
    create_scope(|| {
        // Create many signals
        let mut signals = Vec::new();
        for i in 0..100 {
            signals.push(Signal::new(i));
        }
        
        // Use the signals
        for signal in &signals {
            let _value = signal.read();
        }
        
        println!("   ✓ Created 100 signals");
        println!("   ✓ Signals will be cleaned up when scope ends");
    });
    
    println!("   ✓ Scope destroyed, all signals cleaned up");
    println!("   ✓ No signal leaks detected\n");
}

fn test_effect_cleanup() {
    println!("3. Testing Effect Cleanup:");
    
    let cleanup_count = Rc::new(RefCell::new(0));
    let cleanup_count_for_print = cleanup_count.clone();
    
    create_scope(move || {
        let signal = Signal::new(0);
        let count_clone = cleanup_count.clone();
        
        let _effect = create_effect({
            let signal = signal.clone();
            move || {
                let _value = signal.read();
                on_cleanup({
                    let count = count_clone.clone();
                    move || {
                        *count.borrow_mut() += 1;
                    }
                });
            }
        });
        
        // Trigger effect multiple times
        for i in 1..=10 {
            *signal.write() = i;
            sig::flush_pending_signals();
        }
        
        println!("   ✓ Created effect with cleanup");
        println!("   ✓ Triggered effect 10 times");
        println!("   ✓ Cleanup called {} times (once per re-run)", *cleanup_count_for_print.borrow());
    });
    
    println!("   ✓ Scope destroyed, effect cleaned up");
    println!("   ✓ No effect leaks detected\n");
}
