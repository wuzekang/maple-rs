//! Comprehensive use_resource demonstration
//!
//! This example shows all aspects of use_resource:
//! 1. Automatic dependency tracking - signals are tracked automatically
//! 2. Manual controls - pause, resume, cancel, restart
//! 3. State management - pending, ready, paused, stopped
//! 4. Best practices - how to properly structure async code with signals

use sig::prelude::*;

// ============================================================================
// Helper functions to simulate async operations
// ============================================================================

async fn fetch_user(id: i32) -> String {
    // Simulate async API call
    format!("User #{}", id)
}

async fn fetch_data(id: i32, category: &str) -> String {
    // Simulate async API call with category
    format!("{} data for ID {}", category, id)
}

// ============================================================================
// Example 1: Basic Auto-Tracking
// ============================================================================

fn example_1_basic() {
    use sig::reactive::runtime::{with_reactive_mut, flush_pending_signals};
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Example 1: Basic Auto-Tracking                          ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let user_id = Signal::new(1);
        
        // Create resource with automatic dependency tracking
        // IMPORTANT: Read signals BEFORE the async block to establish dependencies
        let user_data = use_resource(move || {
            let id = *user_id.read(); // ✅ This is automatically tracked!
            async move {
                println!("  📡 Fetching user {}...", id);
                fetch_user(id).await
            }
        });
        
        // Initial fetch
        poll_tasks();
        println!("  ✓ Initial: {:?}\n", user_data.value());
        
        // Change user_id - resource automatically restarts!
        println!("  🔄 Changing user_id to 2...");
        *user_id.write() = 2;
        flush_pending_signals();
        poll_tasks();
        println!("  ✓ After change: {:?}\n", user_data.value());
        
        // Change again
        println!("  🔄 Changing user_id to 3...");
        *user_id.write() = 3;
        flush_pending_signals();
        poll_tasks();
        println!("  ✓ After change: {:?}\n", user_data.value());
        
        println!("  ✅ No manual .restart() calls needed!");
    });
}

// ========================================================================
// Example 2: Multiple Dependencies
// ============================================================================

fn example_2_multiple_deps() {
    use sig::reactive::runtime::{with_reactive_mut, flush_pending_signals};
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Example 2: Multiple Dependencies                        ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let data_id = Signal::new(1);
        let category = Signal::new("Profile".to_string());
        
        // Resource that depends on BOTH signals
        let data = use_resource(move || {
            let id = *data_id.read();           // Tracks data_id
            let cat = category.read().clone();  // Tracks category
            async move {
                println!("  📡 Fetching {} data for ID {}...", cat, id);
                fetch_data(id, &cat).await
            }
        });
        
        // Initial fetch
        poll_tasks();
        println!("  ✓ Initial: {:?}\n", data.value());
        
        // Change data_id - triggers restart
        println!("  🔄 Changing data_id to 2...");
        *data_id.write() = 2;
        flush_pending_signals();
        poll_tasks();
        println!("  ✓ Result: {:?}\n", data.value());
        
        // Change category - also triggers restart!
        println!("  🔄 Changing category to 'Settings'...");
        *category.write() = "Settings".to_string();
        flush_pending_signals();
        poll_tasks();
        println!("  ✓ Result: {:?}\n", data.value());
        
        println!("  ✅ Both signals are tracked automatically!");
    });
}

// ============================================================================
// Example 3: Manual Controls
// ============================================================================

fn example_3_manual_controls() {
    use sig::reactive::runtime::{with_reactive_mut, flush_pending_signals};
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Example 3: Manual Controls (pause/resume/cancel)        ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let user_id = Signal::new(1);
        
        let user_data = use_resource(move || {
            let id = *user_id.read();
            async move {
                println!("  📡 Fetching user {}...", id);
                fetch_user(id).await
            }
        });
        
        // Initial fetch
        poll_tasks();
        println!("  ✓ State: {:?}, Value: {:?}\n", user_data.state(), user_data.value());
        
        // Manual restart (useful for refresh)
        println!("  🔄 Manually restarting...");
        user_data.restart();
        poll_tasks();
        println!("  ✓ After restart: {:?}\n", user_data.value());
        
        // Pause
        println!("  ⏸️  Pausing resource...");
        user_data.pause();
        println!("  ✓ State: {:?}\n", user_data.state());
        
        // Resume
        println!("  ▶️  Resuming resource...");
        user_data.resume();
        poll_tasks();
        println!("  ✓ State: {:?}\n", user_data.state());
        
        // Cancel
        println!("  🛑 Canceling resource...");
        user_data.cancel();
        println!("  ✓ State: {:?}\n", user_data.state());
        
        println!("  ✅ Manual controls available when needed!");
    });
}

// ============================================================================
// Example 4: Common Patterns & Best Practices
// ============================================================================

fn example_4_best_practices() {
    use sig::reactive::runtime::{with_reactive_mut, flush_pending_signals};
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Example 4: Best Practices                               ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let user_id = Signal::new(1);
        
        println!("  ✅ CORRECT: Read signals before async block");
        println!("  ```rust");
        println!("  let resource = use_resource(move || {{");
        println!("      let id = *user_id.read();  // ✅ Tracked!");
        println!("      async move {{");
        println!("          fetch_user(id).await");
        println!("      }}");
        println!("  }});");
        println!("  ```\n");
        
        println!("  ❌ WRONG: Read signals inside async block");
        println!("  ```rust");
        println!("  let resource = use_resource(move || async move {{");
        println!("      let id = *user_id.read();  // ❌ NOT tracked!");
        println!("      fetch_user(id).await");
        println!("  }});");
        println!("  ```\n");
        
        println!("  📝 Explanation:");
        println!("  - Signals must be read during effect execution");
        println!("  - async block creates a Future but doesn't execute immediately");
        println!("  - Read signals BEFORE async move to establish dependencies\n");
        
        // Demonstrate the correct pattern
        let user_data = use_resource(move || {
            let id = *user_id.read(); // ✅ Read here!
            async move {
                fetch_user(id).await
            }
        });
        
        poll_tasks();
        println!("  ✓ Initial: {:?}", user_data.value());
        
        *user_id.write() = 2;
        flush_pending_signals();
        poll_tasks();
        println!("  ✓ After change: {:?} (auto-tracked!)\n", user_data.value());
    });
}

// ============================================================================
// Example 5: Before vs After Comparison
// ============================================================================

fn example_5_comparison() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Example 5: Before vs After                              ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    println!("  ❌ BEFORE (Manual restart required):");
    println!("  ```rust");
    println!("  let data_id = Signal::new(1);");
    println!("  let data = use_resource(move || async move {{");
    println!("      let id = *data_id.read();");
    println!("      fetch_data(id).await");
    println!("  }});");
    println!();
    println!("  // Manual tracking required!");
    println!("  create_effect(move || {{");
    println!("      let _ = *data_id.read();  // Track dependency");
    println!("      data.restart();            // Manual restart!");
    println!("  }});");
    println!("  ```\n");
    
    println!("  ✅ AFTER (Automatic tracking):");
    println!("  ```rust");
    println!("  let data_id = Signal::new(1);");
    println!("  let data = use_resource(move || {{");
    println!("      let id = *data_id.read();  // Automatically tracked!");
    println!("      async move {{");
    println!("          fetch_data(id).await");
    println!("      }}");
    println!("  }});");
    println!("  // That's it! No manual effect needed.");
    println!("  ```\n");
    
    println!("  Benefits:");
    println!("  ✓ Less boilerplate code");
    println!("  ✓ Harder to forget to track dependencies");
    println!("  ✓ More aligned with SolidJS/Leptos patterns");
    println!("  ✓ Cleaner, more maintainable code\n");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("\n");
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║         🚀 use_resource Comprehensive Demo 🚀            ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    example_1_basic();
    example_2_multiple_deps();
    example_3_manual_controls();
    example_4_best_practices();
    example_5_comparison();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Summary                                                  ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    println!("  🎯 Key Takeaways:");
    println!("  1. use_resource automatically tracks signal dependencies");
    println!("  2. Read signals BEFORE async block for proper tracking");
    println!("  3. Manual controls (pause/resume/cancel) available when needed");
    println!("  4. Multiple signals can be tracked simultaneously");
    println!("  5. No need for manual .restart() calls\n");
    
    println!("  📚 For UI examples, run:");
    println!("     cargo run --example resource_simple\n");
}
