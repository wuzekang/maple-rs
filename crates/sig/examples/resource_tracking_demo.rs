/// Demo: Comparing use_resource vs use_resource_with_tracking
///
/// This example demonstrates the difference between the two resource APIs:
/// - use_resource: Must read signals BEFORE the async block
/// - use_resource_with_tracking: Can read signals INSIDE the async block
///
/// Run with: cargo run --example resource_tracking_demo

use sig::prelude::*;
use sig::reactive::{use_resource_with_tracking, runtime::flush_pending_signals};

fn main() {
    println!("=== Resource Tracking Demo ===\n");
    
    // Initialize reactive runtime
    sig::reactive::runtime::with_reactive_mut(|_| {});
    
    create_scope(|| {
        demo_old_style();
        println!("\n---\n");
        demo_new_style();
        println!("\n---\n");
        demo_multiple_signals();
    });
}

fn demo_old_style() {
    println!("1. Old Style (use_resource):");
    println!("   Must read signals BEFORE async block\n");
    
    let counter = Signal::new(0);
    
    // ❌ Old style: Must read before async
    let resource = use_resource(move || {
        let val = *counter.read();  // ← Must read here!
        async move {
            // Simulate async work
            println!("   Fetching data for value: {}", val);
            val * 10
        }
    });
    
    poll_tasks();
    println!("   Initial result: {:?}", resource.value());
    
    // Update the signal
    println!("   Updating counter to 5...");
    *counter.write() = 5;
    flush_pending_signals();
    poll_tasks();
    
    println!("   New result: {:?}", resource.value());
}

fn demo_new_style() {
    println!("2. New Style (use_resource_with_tracking):");
    println!("   Can read signals INSIDE async block\n");
    
    let counter = Signal::new(0);
    
    // ✅ New style: Can read inside async!
    let resource = use_resource_with_tracking(move || async move {
        let val = *counter.read();  // ← Can read here too!
        // Simulate async work
        println!("   Fetching data for value: {}", val);
        val * 10
    });
    
    poll_tasks();
    println!("   Initial result: {:?}", resource.value());
    
    // Update the signal
    println!("   Updating counter to 5...");
    *counter.write() = 5;
    flush_pending_signals();
    poll_tasks();
    
    println!("   New result: {:?}", resource.value());
}

fn demo_multiple_signals() {
    println!("3. Multiple Signals (with_tracking):");
    println!("   Reading multiple signals inside async block\n");
    
    let a = Signal::new(10);
    let b = Signal::new(20);
    
    let resource = use_resource_with_tracking(move || async move {
        let val_a = *a.read();
        let val_b = *b.read();
        println!("   Computing {} + {}", val_a, val_b);
        val_a + val_b
    });
    
    poll_tasks();
    println!("   Initial result: {:?}", resource.value());
    
    println!("   Updating a to 100...");
    *a.write() = 100;
    flush_pending_signals();
    poll_tasks();
    println!("   After updating a: {:?}", resource.value());
    
    println!("   Updating b to 200...");
    *b.write() = 200;
    flush_pending_signals();
    poll_tasks();
    println!("   After updating b: {:?}", resource.value());
}
