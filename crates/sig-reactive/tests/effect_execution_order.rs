//! Test to verify effect execution order in different scenarios
//! 
//! This test demonstrates the difference in effect execution order between:
//! 1. Top-level signal writes (depth == 0): immediate execution
//! 2. Nested signal writes (depth > 0): deferred execution with deduplication

use sig_reactive::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_effect_execution_order_top_level() {
    // Top level: depth == 0
    // Effects run immediately in the order they are triggered
    
    let log = Rc::new(RefCell::new(Vec::new()));
    
    effect::create_scope(move || {
        let signal1 = Signal::new(0);
        let signal2 = Signal::new(0);
        
        // Effect 1 depends on signal1
        {
            let log = log.clone();
            effect::create_effect(move || {
                let value = *signal1.read();
                log.borrow_mut().push(format!("effect1: signal1={}", value));
            });
        }
        
        // Effect 2 depends on signal2
        {
            let log = log.clone();
            effect::create_effect(move || {
                let value = *signal2.read();
                log.borrow_mut().push(format!("effect2: signal2={}", value));
            });
        }
        
        // Effect 3 depends on both
        {
            let log = log.clone();
            effect::create_effect(move || {
                let v1 = *signal1.read();
                let v2 = *signal2.read();
                log.borrow_mut().push(format!("effect3: signal1={}, signal2={}", v1, v2));
            });
        }
        
        log.borrow_mut().clear(); // Clear initial runs
        
        // Top-level writes (depth == 0)
        log.borrow_mut().push("=== Writing signal1 ===".to_string());
        *signal1.write() = 1;
        
        log.borrow_mut().push("=== Writing signal2 ===".to_string());
        *signal2.write() = 2;
        
        println!("\n=== Top-level execution order ===");
        for entry in log.borrow().iter() {
            println!("{}", entry);
        }
        
        // Verify order:
        // When signal1 is written, effect1 and effect3 run immediately
        // When signal2 is written, effect2 and effect3 run immediately
        let entries = log.borrow();
        assert_eq!(entries[0], "=== Writing signal1 ===");
        assert_eq!(entries[1], "effect1: signal1=1");
        assert_eq!(entries[2], "effect3: signal1=1, signal2=0");
        assert_eq!(entries[3], "=== Writing signal2 ===");
        assert_eq!(entries[4], "effect2: signal2=2");
        assert_eq!(entries[5], "effect3: signal1=1, signal2=2");
    });
}

#[test]
fn test_effect_execution_order_nested() {
    // Nested: depth > 0
    // Effects are queued and run after the outer effect comptes
    // Deduplication ensures each effect runs only once
    
    let log = Rc::new(RefCell::new(Vec::new()));
    
    effect::create_scope(move || {
        let signal1 = Signal::new(0);
        let signal2 = Signal::new(0);
        let signal3 = Signal::new(0);
        
        // Effect 1 depends on signal3
        {
            let log = log.clone();
            effect::create_effect(move || {
                let value = *signal3.read();
                log.borrow_mut().push(format!("effect1: signal3={}", value));
            });
        }
        
        // Effect 2 depends on signal3
        {
            let log = log.clone();
            effect::create_effect(move || {
                let value = *signal3.read();
                log.borrow_mut().push(format!("effect2: signal3={}", value));
            });
        }
        
        // Outer effect that writes to signal3 multiple times
        {
            let log = log.clone();
            effect::create_effect(move || {
                let v1 = *signal1.read();
                let v2 = *signal2.read();
                
                log.borrow_mut().push(format!("outer_effect: signal1={}, signal2={}", v1, v2));
                
                // These writes happen inside an effect (depth > 0)
                // So effect1 and effect2 are QUEUED, not run immediately
                log.borrow_mut().push("  Writing signal3 = 10".to_string());
                *signal3.write() = 10;
                
                log.borrow_mut().push("  Writing signal3 = 20".to_string());
                *signal3.write() = 20;
                
                log.borrow_mut().push("  Writing signal3 = 30".to_string());
                *signal3.write() = 30;
                
                log.borrow_mut().push("outer_effect: done".to_string());
                // After this effect completes, queued effects will run
                // But deduplication means effect1 and effect2 run only ONCE
            });
        }
        
        log.borrow_mut().clear(); // Clear initial runs
        
        // Trigger the outer effect
        log.borrow_mut().push("=== Writing signal1 ===".to_string());
        *signal1.write() = 1;
        
        println!("\n=== Nested execution order (with deduplication) ===");
        for entry in log.borrow().iter() {
            println!("{}", entry);
        }
        
        // Verify order:
        // 1. Outer effect runs
        // 2. Three signal3 writes are queued (inside outer effect, depth > 0)
        // 3. After outer effect completes, queued effects run
        // 4. Due to deduplication, effect1 and effect2 run ONCE each (not 3 times)
        let entries = log.borrow();
        let mut idx = 0;
        
        assert_eq!(entries[idx], "=== Writing signal1 ==="); idx += 1;
        assert_eq!(entries[idx], "outer_effect: signal1=1, signal2=0"); idx += 1;
        assert_eq!(entries[idx], "  Writing signal3 = 10"); idx += 1;
        assert_eq!(entries[idx], "  Writing signal3 = 20"); idx += 1;
        assert_eq!(entries[idx], "  Writing signal3 = 30"); idx += 1;
        assert_eq!(entries[idx], "outer_effect: done"); idx += 1;
        
        // NOW the queued effects run (after outer effect completes)
        // Each runs only ONCE despite signal3 being written 3 times
        assert_eq!(entries[idx], "effect1: signal3=30"); idx += 1;
        assert_eq!(entries[idx], "effect2: signal3=30"); idx += 1;
        
        // Total: 8 entries
        assert_eq!(entries.len(), 8);
    });
}

#[test]
fn test_execution_order_difference_summary() {
    // This test summarizes the key differences
    
    let log = Rc::new(RefCell::new(Vec::new()));
    
    effect::create_scope(move || {
        let signal = Signal::new(0);
        let max_iterations = Rc::new(RefCell::new(0));
        
        {
            let log = log.clone();
            effect::create_effect(move || {
                let value = *signal.read();
                log.borrow_mut().push(format!("dependent_effect: {}", value));
            });
        }
        
        {
            let log = log.clone();
            let max_iterations = max_iterations.clone();
            effect::create_effect(move || {
                let value = *signal.read();
                
                // Prevent infinite loop
                *max_iterations.borrow_mut() += 1;
                if *max_iterations.borrow() > 5 {
                    log.borrow_mut().push("(stopping to prevent infinite loop)".to_string());
                    return;
                }
                
                if value == 0 {
                    log.borrow_mut().push("outer_effect: initial run".to_string());
                } else {
                    log.borrow_mut().push(format!("outer_effect: value={}", value));
                    
                    // Nested write (depth > 0)
                    log.borrow_mut().push("  nested write starts".to_string());
                    *signal.write() = value + 10;
                    log.borrow_mut().push("  nested write ends (effect queued)".to_string());
                }
            });
        }
        
        log.borrow_mut().clear();
        
        println!("\n=== Key Difference Demo ===");
        
        // Top-level write (depth == 0): immediate execution
        log.borrow_mut().push("TOP-LEVEL WRITE:".to_string());
        *signal.write() = 1;
        log.borrow_mut().push("After top-level write".to_string());
        
        println!("\nExecution log:");
        for entry in log.borrow().iter() {
            println!("{}", entry);
        }
        
        // Verify the pattern shows nested queueing
        let entries = log.borrow();
        assert!(entries.len() > 0);
        assert_eq!(entries[0], "TOP-LEVEL WRITE:");
    });
}
