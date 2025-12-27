//! Test cleanup of tasks that are actively running (returned Pending)
//! when their scope is destroyed

use sig_async::{spawn, poll_tasks};
use sig_async::task_runtime::with_task_runtime;
use sig_reactive::create_scope;
use std::cell::RefCell;
use std::rc::Rc;
use std::task::Poll;

#[test]
fn test_cleanup_of_pending_task_in_execution() {
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    let poll_count = Rc::new(RefCell::new(0));
    let poll_count_clone = poll_count.clone();
    
    create_scope(move || {
        let task_count_before = with_task_runtime(|rt| rt.task_count());
        
        // Spawn a task that will return Pending when polled
        spawn(async move {
            // This will return Pending on first poll
            futures_util::future::poll_fn(|_| {
                *poll_count_clone.borrow_mut() += 1;
                let count = *poll_count_clone.borrow();
                
                if count < 2 {
                    Poll::<()>::Pending
                } else {
                    Poll::Ready(())
                }
            }).await;
        });
        
        let task_count_after_spawn = with_task_runtime(|rt| rt.task_count());
        assert_eq!(task_count_after_spawn, task_count_before + 1);
        
        // Poll once - the task will return Pending
        poll_tasks();
        
        let task_count_after_poll = with_task_runtime(|rt| rt.task_count());
        
        // Verify task was polled but still exists (returned Pending)
        assert_eq!(*poll_count.borrow(), 1, "Task should have been polled once");
        assert_eq!(task_count_after_poll, task_count_before + 1, 
                   "Task should still exist after returning Pending");
        
        // Scope will be destroyed here with the task still in Pending state
    });
    
    // After scope is destroyed
    let final_count = with_task_runtime(|rt| rt.task_count());
    let final_dirty = with_task_runtime(|rt| rt.dirty_task_count());
    
    // Critical assertion: the Pending task should be cleaned up
    assert_eq!(final_count, initial_count, 
               "Pending task should be cleaned up when scope is destroyed");
    assert_eq!(final_dirty, 0, 
               "Dirty queue should be empty after scope cleanup");
}

#[test]
fn test_cleanup_multiple_pending_tasks() {
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    create_scope(move || {
        // Spawn 3 tasks that will all return Pending
        for _i in 0..3 {
            spawn(async move {
                futures_util::future::poll_fn(|_| {
                    Poll::<()>::Pending
                }).await;
            });
        }
        
        // Poll all - they will all return Pending
        poll_tasks();
        
        let after_poll = with_task_runtime(|rt| rt.task_count());
        
        // All 3 should still exist
        assert_eq!(after_poll, initial_count + 3);
    });
    
    let final_count = with_task_runtime(|rt| rt.task_count());
    
    assert_eq!(final_count, initial_count, 
               "All 3 Pending tasks should be cleaned up");
}

#[test]
fn test_cleanup_mixed_task_states() {
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    let completed = Rc::new(RefCell::new(false));
    let completed_clone = completed.clone();
    
    create_scope(move || {
        // Task 1: Will complete immediately
        spawn(async move {
            *completed_clone.borrow_mut() = true;
        });
        
        // Task 2: Will return Pending
        spawn(async move {
            futures_util::future::poll_fn(|_| Poll::<()>::Pending).await;
        });
        
        // Task 3: Not polled yet (in dirty queue) - but we'll cancel it before polling
        let task3 = spawn(async move {
            // This will be cancelled before poll
        });
        
        let after_spawn = with_task_runtime(|rt| rt.task_count());
        assert_eq!(after_spawn, initial_count + 3);
        
        // Cancel task3 before polling to keep it in the dirty queue but prevent execution
        task3.cancel();
        
        // Poll tasks - Task 1 completes, Task 2 returns Pending, Task 3 was cancelled
        poll_tasks();
        
        let after_poll = with_task_runtime(|rt| rt.task_count());
        
        // Task 1 should be removed (completed)
        // Task 2 should still exist (Pending)
        // Task 3 was cancelled
        assert!(*completed.borrow(), "Task 1 should have completed");
        assert_eq!(after_poll, initial_count + 1, 
                   "Should have 1 task: Pending task");
    });
    
    let final_count = with_task_runtime(|rt| rt.task_count());
    let final_dirty = with_task_runtime(|rt| rt.dirty_task_count());
    
    assert_eq!(final_count, initial_count, 
               "All remaining tasks should be cleaned up");
    assert_eq!(final_dirty, 0, 
               "Dirty queue should be empty");
}

#[test]
fn test_cleanup_nested_scopes_with_pending_tasks() {
    create_scope(|| {
        let outer_initial = with_task_runtime(|rt| rt.task_count());
        
        // Outer scope task - will be Pending
        spawn(async {
            futures_util::future::poll_fn(|_| Poll::<()>::Pending).await;
        });
        
        poll_tasks();
        
        create_scope(move || {
            // Inner scope task - also will be Pending
            spawn(async {
                futures_util::future::poll_fn(|_| Poll::<()>::Pending).await;
            });
            
            poll_tasks();
            
            let after_inner_spawn = with_task_runtime(|rt| rt.task_count());
            
            // Both outer and inner tasks should exist
            assert_eq!(after_inner_spawn, outer_initial + 2);
        });
        
        let after_inner_destroyed = with_task_runtime(|rt| rt.task_count());
        
        // Only outer task should remain
        assert_eq!(after_inner_destroyed, outer_initial + 1, 
                   "Inner Pending task should be cleaned up");
    });
}

#[test]
fn test_resource_with_pending_future_cleanup() {
    use sig_async::use_resource;
    
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    let poll_count = Rc::new(RefCell::new(0));
    let poll_count_clone = poll_count.clone();
    
    create_scope(move || {
        // Create a resource that will stay Pending
        let _resource = use_resource(move || {
            let count = poll_count_clone.clone();
            async move {
                futures_util::future::poll_fn(move |_| {
                    *count.borrow_mut() += 1;
                    
                    if *count.borrow() < 2 {
                        Poll::Pending
                    } else {
                        Poll::Ready(42)
                    }
                }).await
            }
        });
        
        // Poll once - resource will be Pending
        poll_tasks();
        
        let after_poll = with_task_runtime(|rt| rt.task_count());
        
        assert_eq!(*poll_count.borrow(), 1, "Resource should have been polled once");
        assert!(after_poll > initial_count, "Resource task should exist");
    });
    
    let final_count = with_task_runtime(|rt| rt.task_count());
    
    // Resource's Pending task should be cleaned up
    assert_eq!(final_count, initial_count, 
               "Resource Pending task should be cleaned up");
}
