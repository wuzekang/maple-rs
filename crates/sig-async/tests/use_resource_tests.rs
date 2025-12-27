//! Tests for use_resource module

use sig_async::{use_resource, use_resource_with_tracking, poll_tasks, ResourceState};
use sig_reactive::{create_scope, Signal};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_use_resource_basic() {
    create_scope(move || {
        let resource = use_resource(|| async {
            42
        });
        
        // Initially pending
        assert_eq!(resource.state(), ResourceState::Pending);
        assert!(resource.pending());
        assert!(!resource.ready());
        
        // Poll tasks to execute
        poll_tasks();
        
        // Now should be ready
        assert_eq!(resource.state(), ResourceState::Ready);
        assert!(resource.ready());
        assert_eq!(resource.value(), Some(42));
    });
}

#[test]
fn test_use_resource_with_signal() {
    create_scope(move || {
        let count = Signal::new(10);
        
        let resource = use_resource(move || {
            let value = *count.read();
            async move {
                value * 2
            }
        });
        
        poll_tasks();
        
        assert_eq!(resource.value(), Some(20));
    });
}

#[test]
fn test_use_resource_auto_restart_on_signal_change() {
    create_scope(move || {
        let count = Signal::new(10);
        
        let resource = use_resource(move || {
            let value = *count.read();
            async move {
                value * 2
            }
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(20));
        
        // Change the signal
        *count.write() = 20;
        
        // Resource should restart automatically
        assert_eq!(resource.state(), ResourceState::Pending);
        
        poll_tasks();
        assert_eq!(resource.value(), Some(40));
    });
}

#[test]
fn test_resource_cancel() {
    create_scope(move || {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        let resource = use_resource(move || {
            let exec = executed_clone.clone();
            async move {
                *exec.borrow_mut() = true;
                42
            }
        });
        
        // Cancel before polling
        resource.cancel();
        
        poll_tasks();
        
        // Task should not have executed
        assert!(!*executed.borrow());
        assert_eq!(resource.state(), ResourceState::Stopped);
    });
}

#[test]
fn test_resource_pause_resume() {
    create_scope(move || {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        let resource = use_resource(move || {
            let exec = executed_clone.clone();
            async move {
                *exec.borrow_mut() = true;
                42
            }
        });
        
        // Pause immediately
        resource.pause();
        assert_eq!(resource.state(), ResourceState::Paused);
        
        poll_tasks();
        
        // Should not execute while paused
        assert!(!*executed.borrow());
        
        // Resume and poll
        resource.resume();
        assert_eq!(resource.state(), ResourceState::Pending);
        
        poll_tasks();
        
        // Now should execute
        assert!(*executed.borrow());
        assert_eq!(resource.value(), Some(42));
    });
}

#[test]
fn test_resource_restart() {
    create_scope(move || {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let resource = use_resource(move || {
            let c = counter_clone.clone();
            async move {
                *c.borrow_mut() += 1;
                *c.borrow()
            }
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(1));
        
        // Manually restart
        resource.restart();
        poll_tasks();
        
        assert_eq!(resource.value(), Some(2));
    });
}

#[test]
fn test_resource_clear() {
    create_scope(move || {
        let resource = use_resource(|| async {
            42
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(42));
        
        // Clear the value
        resource.clear();
      assert_eq!(resource.value(), None);
    });
}

#[test]
fn test_use_resource_with_tracking() {
    create_scope(move || {
        let count = Signal::new(10);
        
        // With tracking, we can read signals inside the async block
        let resource = use_resource_with_tracking(move || async move {
            let value = *count.read();  // Reading inside async block
            value * 2
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(20));
        
        // Change signal
        *count.write() = 20;
        
        poll_tasks();
        assert_eq!(resource.value(), Some(40));
    });
}

#[test]
fn test_resource_clone() {
    create_scope(move || {
        let resource = use_resource(|| async {
            42
        });
        
        poll_tasks();
        
        // Resource is Copy - can be used multiple times directly
        assert_eq!(resource.value(), Some(42));
        assert_eq!(resource.value(), Some(42));
        
        // All copies refer to the same underlying resource
        resource.clear();
        assert_eq!(resource.value(), None);
    });
}

#[test]
fn test_resource_multiple_signals() {
    create_scope(move || {
        let a = Signal::new(10);
        let b = Signal::new(20);
        
        let resource = use_resource(move || {
            let val_a = *a.read();
            let val_b = *b.read();
            async move {
                val_a + val_b
            }
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(30));
        
        // Change first signal
        *a.write() = 15;
        poll_tasks();
        assert_eq!(resource.value(), Some(35));
        
        // Change second signal
        *b.write() = 25;
        poll_tasks();
        assert_eq!(resource.value(), Some(40));
    });
}

#[test]
fn test_resource_state_transitions() {
    create_scope(move || {
        let resource = use_resource(|| async {
            42
        });
        
        // Initial state
        assert_eq!(resource.state(), ResourceState::Pending);
        
        poll_tasks();
        
        // After completion
        assert_eq!(resource.state(), ResourceState::Ready);
        
        // Pause
        resource.pause();
        assert_eq!(resource.state(), ResourceState::Paused);
        
        // Resume
        resource.resume();
        assert_eq!(resource.state(), ResourceState::Pending);
        
        // Cancel
        resource.cancel();
        assert_eq!(resource.state(), ResourceState::Stopped);
    });
}

// ============================================================================
// Tests for use_resource with create_effect
// ============================================================================

#[test]
fn test_use_resource_with_effect_basic() {
    use sig_reactive::create_effect;
    
    create_scope(move || {
        let signal = Signal::new(10);
        let effect_run_count = Rc::new(RefCell::new(0));
        let effect_run_count_clone = effect_run_count.clone();
        
        // Create resource
        let resource = use_resource(move || {
            let value = *signal.read();
            async move {
                value * 2
            }
        });
        
        // Create effect that reads resource value
        create_effect(move || {
            *effect_run_count_clone.borrow_mut() += 1;
            let _value = resource.value();
        });
        
        // Effect runs once initially
        assert_eq!(*effect_run_count.borrow(), 1);
        
        // Poll tasks to complete resource
        poll_tasks();
        
        // Effect should run again when resource value changes
        assert_eq!(*effect_run_count.borrow(), 2);
        assert_eq!(resource.value(), Some(20));
    });
}

#[test]
fn test_use_resource_with_effect_signal_change() {
    use sig_reactive::create_effect;
    
    create_scope(move || {
        let input = Signal::new(5);
        let effect_values = Rc::new(RefCell::new(Vec::new()));
        let effect_values_clone = effect_values.clone();
        
        // Create resource that depends on signal
        let resource = use_resource(move || {
            let value = *input.read();
            async move {
                value * 3
            }
        });
        
        // Create effect that collects resource values
        create_effect(move || {
            if let Some(value) = resource.value() {
                effect_values_clone.borrow_mut().push(value);
            }
        });
        
        // Poll initial task
        poll_tasks();
        assert_eq!(*effect_values.borrow(), vec![15]);
        
        // Change signal - resource should restart
        *input.write() = 10;
        poll_tasks();
        
        // Effect should see new value
        assert_eq!(*effect_values.borrow(), vec![15, 30]);
        
        // Change signal again
        *input.write() = 7;
        poll_tasks();
        
        assert_eq!(*effect_values.borrow(), vec![15, 30, 21]);
    });
}

#[test]
fn test_use_resource_with_effect_state_tracking() {
    use sig_reactive::create_effect;
    
    create_scope(move || {
        let trigger = Signal::new(0);
        let state_changes = Rc::new(RefCell::new(Vec::new()));
        let state_changes_clone = state_changes.clone();
        
        let resource = use_resource(move || {
            let _t = *trigger.read();
            async move {
                100
            }
        });
        
        // Effect tracks resource state changes
        create_effect(move || {
            let state = resource.state();
            state_changes_clone.borrow_mut().push(state);
        });
        
        // Initial state should be Pending
        assert!(state_changes.borrow().contains(&ResourceState::Pending));
        
        poll_tasks();
        
        // Should transition to Ready
        assert!(state_changes.borrow().contains(&ResourceState::Ready));
        
        // Trigger resource restart
        *trigger.write() = 1;
        
        // Should go back to Pending
        let states = state_changes.borrow();
        assert_eq!(states.last(), Some(&ResourceState::Pending));
        drop(states);
        
        poll_tasks();
        
        // And back to Ready
        let states = state_changes.borrow();
        assert_eq!(states.last(), Some(&ResourceState::Ready));
    });
}

#[test]
fn test_use_resource_with_effect_derived_state() {
    use sig_reactive::create_effect;
    
    create_scope(move || {
        let user_id = Signal::new(1);
        let is_loading = Rc::new(RefCell::new(false));
        let is_loading_clone = is_loading.clone();
        let has_data = Rc::new(RefCell::new(false));
        let has_data_clone = has_data.clone();
        
        let resource = use_resource(move || {
            let id = *user_id.read();
            async move {
                format!("User #{}", id)
            }
        });
        
        
        // Effect that derives loading and data state
        create_effect(move || {
            *is_loading_clone.borrow_mut() = resource.pending();
            *has_data_clone.borrow_mut() = resource.ready();
        });
        
        // Initially loading
        assert_eq!(*is_loading.borrow(), true);
        assert_eq!(*has_data.borrow(), false);
        
        poll_tasks();
        
        // After loading
        assert_eq!(*is_loading.borrow(), false);
        assert_eq!(*has_data.borrow(), true);
        assert_eq!(resource.value(), Some("User #1".to_string()));
        
        // Change user - should start loading again
        *user_id.write() = 2;
        
        assert_eq!(*is_loading.borrow(), true);
        assert_eq!(*has_data.borrow(), false);
        
        poll_tasks();
        
        assert_eq!(*is_loading.borrow(), false);
        assert_eq!(*has_data.borrow(), true);
        assert_eq!(resource.value(), Some("User #2".to_string()));
    });
}

#[test]
fn test_use_resource_with_effect_multiple_resources() {
    use sig_reactive::create_effect;
    
    create_scope(move || {
        let id1 = Signal::new(1);
        let id2 = Signal::new(2);
        let combined = Rc::new(RefCell::new(None));
        let combined_clone = combined.clone();
        
        let resource1 = use_resource(move || {
            let id = *id1.read();
            async move {
                id * 10
            }
        });
        
        let resource2 = use_resource(move || {
            let id = *id2.read();
            async move {
                id * 100
            }
        });
        
        // Effect that combines two resources
        create_effect(move || {
            if let (Some(v1), Some(v2)) = (resource1.value(), resource2.value()) {
                *combined_clone.borrow_mut() = Some(v1 + v2);
            }
        });
        
        poll_tasks();
        assert_eq!(*combined.borrow(), Some(210)); // 10 + 200
        
        // Change first resource
        *id1.write() = 3;
        poll_tasks();
        assert_eq!(*combined.borrow(), Some(230)); // 30 + 200
        
        // Change second resource
        *id2.write() = 5;
        poll_tasks();
        assert_eq!(*combined.borrow(), Some(530)); // 30 + 500
    });
}
