//! Example demonstrating how signal location tracking helps debug memory leaks
//!
//! Run with:
//! cargo run --package sig-reactive --example debug_signal_leak

use sig_reactive::*;
use std::cell::RefCell;

// Simulated global storage that might leak signals
thread_local! {
    static LEAKED_SIGNALS: RefCell<Vec<Signal<i32>>> = RefCell::new(Vec::new());
}

fn main() {
    println!("=== Signal Leak Debugging Example ===\n");
    
    // Scenario: Creating signals in a loop but accidentally storing them
    effect::create_scope(|| {
        println!("Creating signals that might leak...\n");
        
        for i in 0..5 {
            let signal = Signal::new(i);
            
            // Oops! We're storing signals in a global, which might leak
            LEAKED_SIGNALS.with(|leaked| {
                leaked.borrow_mut().push(signal);
            });
        }
        
        #[cfg(debug_assertions)]
        {
            // Now let's debug where these signals came from
            println!("Leaked signals inspection:");
            LEAKED_SIGNALS.with(|leaked| {
                for (idx, signal) in leaked.borrow().iter().enumerate() {
                    println!(
                        "  Signal {}: created at {}",
                        idx,
                        signal.created_at()
                    );
                }
            });
            
            println!("\n✅ Location tracking helps identify where leaked signals originated!");
            println!("   You can now refactor these lines to prevent the leak.\n");
        }
        
        #[cfg(not(debug_assertions))]
        {
            println!("⚠️  Run in debug mode to see signal creation locations.");
        }
    });
    
    // Demonstrate with a more complex scenario
    demonstrate_nested_leak();
}

fn demonstrate_nested_leak() {
    println!("=== Nested Scope Leak Example ===\n");
    
    effect::create_scope(|| {
        println!("Creating signals in nested helper functions...\n");
        
        let signal1 = create_user_signal();
        let signal2 = create_config_signal();
        let signal3 = create_state_signal();
        
        #[cfg(debug_assertions)]
        {
            println!("Signal origins:");
            println!("  User signal:   {}", signal1.created_at());
            println!("  Config signal: {}", signal2.created_at());
            println!("  State signal:  {}", signal3.created_at());
            
            println!("\n✅ Even with helper functions, we know exactly where signals originated!");
            println!("   This is thanks to #[track_caller] attribute.\n");
        }
    });
}

#[track_caller]
fn create_user_signal() -> Signal<i32> {
    Signal::new(100)
}

#[track_caller]
fn create_config_signal() -> Signal<i32> {
    Signal::new(200)
}

#[track_caller]
fn create_state_signal() -> Signal<i32> {
    Signal::new(300)
}
