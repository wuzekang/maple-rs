//! Example demonstrating that Resource is now Copy
//!
//! This shows that Resource can be freely copied without explicit cloning.

use sig_async::{use_resource, poll_tasks};
use sig_reactive::{create_scope, Signal};

fn main() {
    create_scope(move || {
        let count = Signal::new(1);
        
        // Create a resource
        let resource = use_resource(move || {
            let n = *count.read();
            async move {
                format!("Count: {}", n)
            }
        });
        
        // ✨ Resource is Copy! We can use it multiple times without .clone()
        let resource_copy_1 = resource;  // No .clone() needed!
        let resource_copy_2 = resource;  // Still no .clone() needed!
        
        poll_tasks();
        
        // All copies work and refer to the same resource
        println!("Original: {:?}", resource.value());
        println!("Copy 1:   {:?}", resource_copy_1.value());
        println!("Copy 2:   {:?}", resource_copy_2.value());
        
        // Verify they all show the same value
        assert_eq!(resource.value(), Some("Count: 1".to_string()));
        assert_eq!(resource_copy_1.value(), Some("Count: 1".to_string()));
        assert_eq!(resource_copy_2.value(), Some("Count: 1".to_string()));
        
        // Change the signal
        *count.write() = 2;
        poll_tasks();
        
        // All copies automatically see the update
        println!("\nAfter update:");
        println!("Original: {:?}", resource.value());
        println!("Copy 1:   {:?}", resource_copy_1.value());
        println!("Copy 2:   {:?}", resource_copy_2.value());
        
        assert_eq!(resource.value(), Some("Count: 2".to_string()));
        assert_eq!(resource_copy_1.value(), Some("Count: 2".to_string()));
        assert_eq!(resource_copy_2.value(), Some("Count: 2".to_string()));
        
        println!("\n✅ Resource is Copy! All copies work seamlessly.");
    });
}
