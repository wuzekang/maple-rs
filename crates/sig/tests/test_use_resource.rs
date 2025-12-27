use sig::prelude::*;
use sig::reactive::runtime::flush_pending_signals;
use std::sync::{Arc, Mutex};

#[test]
fn test_use_resource_reactive_deps() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let counter = Signal::new(0);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_for_resource = call_count.clone();
        
        // IMPORTANT: Read signal BEFORE async block to establish dependency!
        let resource = use_resource(move || {
            let val = *counter.read();  // ✅ Read here - will be tracked!
            let call_count = call_count_for_resource.clone();
            async move {
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
        
        // 改变 counter - 应该触发重新执行
        println!("\nChanging counter to 5...");
        *counter.write() = 5;
        flush_pending_signals();  // 触发 effect
        poll_tasks();
        
        println!("After change: value = {:?}, call_count = {}", 
                 resource.value(), *call_count.lock().unwrap());
        
        assert_eq!(resource.value(), Some(50));
        assert_eq!(*call_count.lock().unwrap(), 2, "Resource should re-execute when counter changes!");
    });
}

#[test]
fn test_use_resource_basic() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        
        let resource = use_resource(move || {
            let called = called_clone.clone();
            async move {
                *called.lock().unwrap() = true;
                42
            }
        });
        
        assert_eq!(resource.state(), ResourceState::Pending);
        
        poll_tasks();
        
        assert_eq!(resource.state(), ResourceState::Ready);
        assert_eq!(resource.value(), Some(42));
        assert!(*called.lock().unwrap());
    });
}

#[test]
fn test_use_resource_manual_restart() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        let resource = use_resource(move || {
            let counter = counter_clone.clone();
            async move {
                let mut val = counter.lock().unwrap();
                *val += 1;
                *val
            }
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(1));
        
        // 手动 restart
        resource.restart();
        poll_tasks();
        assert_eq!(resource.value(), Some(2));
    });
}

#[test]
fn test_use_resource_pause_resume() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let resource = use_resource(|| async {
            100
        });
        
        // Pause before polling
        resource.pause();
        assert_eq!(resource.state(), ResourceState::Paused);
        
        poll_tasks();
        // Paused task won't complete
        
        // Resume and poll again
        resource.resume();
        poll_tasks();
        
        assert_eq!(resource.state(), ResourceState::Ready);
        assert_eq!(resource.value(), Some(100));
    });
}

#[test]
fn test_use_resource_cancel() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_| {});
    
    create_scope(|| {
        let resource = use_resource(|| async {
            999
        });
        
        resource.cancel();
        assert_eq!(resource.state(), ResourceState::Stopped);
        
        poll_tasks();
        assert_eq!(resource.state(), ResourceState::Stopped);
        assert_eq!(resource.value(), None);
    });
}
