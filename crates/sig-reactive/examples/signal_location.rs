//! Example demonstrating signal creation location tracking
//!
//! Run with:
//! cargo run --package sig-reactive --example signal_location

use sig_reactive::*;

fn main() {
    effect::create_scope(|| {
        // Create signals at different locations
        let signal1 = Signal::new(42);
        let signal2 = Signal::new("hello");
        let signal3 = Signal::new(vec![1, 2, 3]);

        // Print debug info (includes creation location in debug builds)
        println!("Signal 1: {:?}", signal1);
        println!("Signal 2: {:?}", signal2);
        println!("Signal 3: {:?}", signal3);

        #[cfg(debug_assertions)]
        {
            println!("\nCreation locations:");
            println!("Signal 1 created at: {}", signal1.created_at());
            println!("Signal 2 created at: {}", signal2.created_at());
            println!("Signal 3 created at: {}", signal3.created_at());
        }

        #[cfg(not(debug_assertions))]
        {
            println!("\nNote: Creation location tracking is only available in debug builds.");
        }

        // Create a signal from a helper function to show different location
        let signal4 = create_signal_in_helper();
        println!("\nSignal 4: {:?}", signal4);
        
        #[cfg(debug_assertions)]
        println!("Signal 4 created at: {}", signal4.created_at());
    });
}

#[track_caller]
fn create_signal_in_helper() -> Signal<i32> {
    Signal::new(100)
}
