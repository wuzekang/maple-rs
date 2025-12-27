use sig::prelude::*;
use sig::reactive::runtime::flush_pending_signals;
use sig::reactive::use_resource_with_tracking;
use std::sync::{Arc, Mutex};

#[test]
fn test_use_resource_with_tracking_basic() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let counter = Signal::new(0);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_for_resource = call_count.clone();
        
        // ✅ NEW: Can read signal INSIDE async block!
        let resource = use_resource_with_tracking(move || {
            let call_count = call_count_for_resource.clone();
            async move {
                let val = *counter.read();  // ✅ Read inside async - will be tracked!
                *call_count.lock().unwrap() += 1;
                println!("Resource executing with counter = {}", val);
                val * 10
            }
        });
        
        poll_tasks();
        println!("First run: value = {:?}, call_count = {}", 
                 resource.value(), *call_count.lock().unwrap());
        assert_eq!(resource.value(), Some(0));
        assert_eq!(*call_count.lock().unwrap(), 1);
        
        // Change counter - should trigger re-execution
        println!("\nChanging counter to 5...");
        *counter.write() = 5;
        flush_pending_signals();
        poll_tasks();
        
        println!("After change: value = {:?}, call_count = {}", 
                 resource.value(), *call_count.lock().unwrap());
        
        assert_eq!(resource.value(), Some(50));
        assert_eq!(*call_count.lock().unwrap(), 2, "Resource should re-execute when counter changes!");
    });
}

#[test]
fn test_use_resource_with_tracking_multiple_signals() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let a = Signal::new(1);
        let b = Signal::new(2);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();
        
        // Read multiple signals inside async block
        let resource = use_resource_with_tracking(move || {
            let call_count = call_count_clone.clone();
            async move {
                let val_a = *a.read();
                let val_b = *b.read();
                *call_count.lock().unwrap() += 1;
                val_a + val_b
            }
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(3));
        assert_eq!(*call_count.lock().unwrap(), 1);
        
        // Change first signal
        *a.write() = 10;
        flush_pending_signals();
        poll_tasks();
        assert_eq!(resource.value(), Some(12));
        assert_eq!(*call_count.lock().unwrap(), 2);
        
        // Change second signal
        *b.write() = 20;
        flush_pending_signals();
        poll_tasks();
        assert_eq!(resource.value(), Some(30));
        assert_eq!(*call_count.lock().unwrap(), 3);
    });
}

#[test]
fn test_comparison_old_vs_new() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let counter1 = Signal::new(0);
        let counter2 = Signal::new(0);
        
        // Old style - must read before async
        let old_resource = use_resource(move || {
            let val = *counter1.read();  // Must read here
            async move {
                val * 10
            }
        });
        
        // New style - can read inside async
        let new_resource = use_resource_with_tracking(move || async move {
            let val = *counter2.read();  // Can read here!
            val * 10
        });
        
        poll_tasks();
        assert_eq!(old_resource.value(), Some(0));
        assert_eq!(new_resource.value(), Some(0));
        
        // Both should react to changes
        *counter1.write() = 5;
        *counter2.write() = 5;
        flush_pending_signals();
        poll_tasks();
        
        assert_eq!(old_resource.value(), Some(50));
        assert_eq!(new_resource.value(), Some(50));
    });
}
