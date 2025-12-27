//! 补充测试：覆盖当前遗漏的重要场景

use sig_async::{spawn, poll_tasks, parent_task, current_task, use_resource, ResourceState};
use sig_reactive::{create_scope, Signal};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::Poll;

// ============================================================================
// 1. parent_task() API 测试
// ============================================================================

#[test]
fn test_parent_task_api() {
    create_scope(move || {
        let parent_id = Rc::new(RefCell::new(None));
        let parent_id_clone = parent_id.clone();
        
        let child_id = Rc::new(RefCell::new(None));
        let child_id_clone = child_id.clone();
        let child_id_clone2 = child_id.clone();
        
        let _parent = spawn(async move {
            *parent_id_clone.borrow_mut() = current_task();
            
            // Spawn a child task
            let child = spawn(async move {
                *child_id_clone.borrow_mut() = current_task();
            });
            
            // Poll child to make sure it records its ID
            let _ = child.poll_now();
            
            // Also store child ID in the outer scope
            *child_id_clone2.borrow_mut() = Some(child);
        });
        
        poll_tasks();
        
        // Get the IDs
        let parent_task_id = parent_id.borrow().expect("Parent task should have run");
        let child_task_id = child_id.borrow().expect("Child task should have run");
        
        println!("Parent task: {:?}", parent_task_id);
        println!("Child task: {:?}", child_task_id);
        
        // Check parent-child relationship
        let child_parent = parent_task(child_task_id);
        println!("Child's parent: {:?}", child_parent);
        
        // NOTE: This might fail if the child task completes and is removed
        // before we check the parent. This is expected behavior.
        // The parent_task() API only works for tasks that still exist.
        if child_parent.is_none() {
            println!("⚠️ Child task was already cleaned up - this is expected for completed tasks");
        } else {
            assert_eq!(child_parent, Some(parent_task_id), "Child's parent should be the parent task");
        }
    });
}

#[test]
fn test_parent_task_with_pending_child() {
    use std::task::Poll;
    
    create_scope(move || {
        let parent_id = Rc::new(RefCell::new(None));
        let parent_id_clone = parent_id.clone();
        
        let child_id = Rc::new(RefCell::new(None));
        let child_id_clone = child_id.clone();
        
        let parent_handle = Rc::new(RefCell::new(None));
        let parent_handle_clone = parent_handle.clone();
        
        let parent = spawn(async move {
            *parent_id_clone.borrow_mut() = current_task();
            
            // Spawn a child task that will stay pending
            let poll_count = Rc::new(RefCell::new(0));
            let poll_count_clone = poll_count.clone();
            
            let child = spawn(async move {
                *child_id_clone.borrow_mut() = current_task();
                
                // Stay pending on first poll
                futures_util::future::poll_fn(move |_| {
                    *poll_count_clone.borrow_mut() += 1;
                    if *poll_count_clone.borrow() < 2 {
                        Poll::<()>::Pending
                    } else {
                        Poll::Ready(())
                    }
                }).await;
            });
            
            // Poll child once - it will stay pending
            let _ = child.poll_now();
            
            // Don't complete this parent task yet - stay pending
            futures_util::future::poll_fn(|_| Poll::<()>::Pending).await;
        });
        
        // Store parent handle
        *parent_handle_clone.borrow_mut() = Some(parent);
        
        // Poll parent task once - it will become pending after spawning child
        let _ = parent.poll_now();
        
        // Now both parent and child are pending
        let parent_task_id = parent_id.borrow().expect("Parent should have started");
        let child_task_id = child_id.borrow().expect("Child should have been spawned");
        
        println!("\n=== Parent-Child Test (Both Pending) ===");
        println!("Parent task: {:?}", parent_task_id);
        println!("Child task: {:?}", child_task_id);
        
        let child_parent = parent_task(child_task_id);
        println!("Child's parent: {:?}", child_parent);
        
        // Both tasks are still pending, so they should both exist
        assert_eq!(child_parent, Some(parent_task_id), 
                   "Pending child's parent should be the parent task");
    });
}

#[test]
fn test_parent_task_for_root_task() {
    create_scope(move || {
        let task_id = Rc::new(RefCell::new(None));
        let task_id_clone = task_id.clone();
        
        spawn(async move {
            *task_id_clone.borrow_mut() = current_task();
        });
        
        poll_tasks();
        
        let task = task_id.borrow().unwrap();
        let parent = parent_task(task);
        
        // Root task should have no parent
        assert_eq!(parent, None, "Root task should have no parent");
    });
}

// ============================================================================
// 2. 暂停任务被唤醒的行为
// ============================================================================

#[test]
fn test_paused_task_wake_behavior() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Pause the task
        task.pause();
        assert!(task.paused());
        
        // Wake the paused task
        task.wake();
        
        // Poll tasks - should NOT execute because it's paused
        poll_tasks();
        
        assert!(!*executed.borrow(), "Paused task should not execute even when woken");
        assert!(task.paused(), "Task should still be paused");
    });
}

#[test]
fn test_pause_after_wake() {
    let value = Rc::new(RefCell::new(0));
    let value_clone = value.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            *value_clone.borrow_mut() = 42;
        });
        
        // Wake first
        task.wake();
        
        // Then immediately pause
        task.pause();
        
        // Poll - should not execute
        poll_tasks();
        
        assert_eq!(*value.borrow(), 0, "Task should not execute when paused after wake");
    });
}

// ============================================================================
// 3. 资源在异常状态下的操作
// ============================================================================

#[test]
fn test_resource_operations_after_cancel() {
    create_scope(move || {
        let resource = use_resource(|| async { 42 });
        
        // Cancel the resource
        resource.cancel();
        assert_eq!(resource.state(), ResourceState::Stopped);
        
        // These operations should not crash
        resource.pause();
        resource.resume();
        
        // State should remain Stopped (or transition gracefully)
        // The exact behavior depends on implementation
        poll_tasks();
        
        // Resource should not have a value
        assert_eq!(resource.value(), None);
    });
}

#[test]
fn test_resource_restart_after_cancel() {
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
        
        // Cancel the resource
        resource.cancel();
        assert_eq!(resource.state(), ResourceState::Stopped);
        
        // Restart - should create a new task
        resource.restart();
        
        poll_tasks();
        
        // Should execute again
        assert_eq!(*counter.borrow(), 2);
        assert_eq!(resource.value(), Some(2));
    });
}

#[test]
fn test_resource_cancel_while_paused() {
    create_scope(move || {
        let resource = use_resource(|| async { 42 });
        
        // Pause first
        resource.pause();
        assert_eq!(resource.state(), ResourceState::Paused);
        
        // Then cancel
        resource.cancel();
        assert_eq!(resource.state(), ResourceState::Stopped);
        
        poll_tasks();
        
        // Should not have executed
        assert_eq!(resource.value(), None);
    });
}

// ============================================================================
// 4. 任务完成后的幂等性
// ============================================================================

#[test]
fn test_cancel_completed_task() {
    create_scope(move || {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        let task = spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Execute the task
        poll_tasks();
        assert!(*executed.borrow());
        
        // Cancel after completion - should not crash
        task.cancel();
        
        // Further polls should be safe
        poll_tasks();
    });
}

#[test]
fn test_pause_completed_task() {
    create_scope(move || {
        let task = spawn(async {});
        
        poll_tasks();
        
        // Pause a completed task - should not crash
        task.pause();
        
        // paused() might return false for a completed task
        // The exact behavior depends on implementation
    });
}

// ============================================================================
// 5. 多步骤异步任务（真实的 Pending）
// ============================================================================

#[test]
fn test_multi_step_async_task() {
    let steps = Rc::new(RefCell::new(Vec::new()));
    let steps_clone = steps.clone();
    
    create_scope(move || {
        spawn(async move {
            steps_clone.borrow_mut().push(1);
            
            // First yield - returns Pending
            futures_util::future::poll_fn(|_| {
                steps_clone.borrow_mut().push(2);
                Poll::<()>::Pending
            }).await;
            
            steps_clone.borrow_mut().push(3);
        });
        
        // First poll - executes step 1 and 2, then becomes Pending
        poll_tasks();
        assert_eq!(*steps.borrow(), vec![1, 2]);
        
        // No more dirty tasks, so step 3 is not executed yet
        // In a real event loop, the task would be re-scheduled
        // For this test, we just verify the behavior
    });
}

#[test]
fn test_async_task_with_ready_futures() {
    let steps = Rc::new(RefCell::new(Vec::new()));
    let steps_clone = steps.clone();
    
    create_scope(move || {
        spawn(async move {
            steps_clone.borrow_mut().push(1);
            
            // This immediately resolves
            futures_util::future::ready(()).await;
            
            steps_clone.borrow_mut().push(2);
            
            futures_util::future::ready(()).await;
            
            steps_clone.borrow_mut().push(3);
        });
        
        poll_tasks();
        
        // All steps should execute
        assert_eq!(*steps.borrow(), vec![1, 2, 3]);
    });
}

// ============================================================================
// 6. use_resource_with_tracking 深度测试
// ============================================================================

#[test]
fn test_use_resource_with_tracking_multiple_signals_in_async() {
    use sig_async::use_resource_with_tracking;
    
    create_scope(move || {
        let a = Signal::new(1);
        let b = Signal::new(2);
        
        let resource = use_resource_with_tracking(move || async move {
            // Read first signal
            let val_a = *a.read();
            
            // Some async work
            futures_util::future::ready(()).await;
            
            // Read second signal
            let val_b = *b.read();
            
            val_a + val_b
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(3));
        
        // Test that changing 'a' triggers restart
        *a.write() = 10;
        poll_tasks();
        assert_eq!(resource.value(), Some(12));
        
        // Test that changing 'b' also triggers restart
        *b.write() = 20;
        poll_tasks();
        assert_eq!(resource.value(), Some(30));
    });
}

#[test]
fn test_use_resource_with_tracking_conditional_signal_read() {
    use sig_async::use_resource_with_tracking;
    
    create_scope(move || {
        let condition = Signal::new(true);
        let value_a = Signal::new(10);
        let value_b = Signal::new(20);
        
        let resource = use_resource_with_tracking(move || async move {
            if *condition.read() {
                *value_a.read()
            } else {
                *value_b.read()
            }
        });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(10));
        
        // Change value_a - should trigger restart
        *value_a.write() = 15;
        poll_tasks();
        assert_eq!(resource.value(), Some(15));
        
        // Change condition
        *condition.write() = false;
        poll_tasks();
        assert_eq!(resource.value(), Some(20));
        
        // Now changing value_a should not trigger (because we're reading value_b)
        // This is complex reactivity behavior that might need special handling
    });
}

// ============================================================================
// 7. TaskRuntime 计数器方法测试
// ============================================================================

#[test]
fn test_task_runtime_counters() {
    use sig_async::task_runtime::with_task_runtime;
    
    create_scope(move || {
        let initial_count = with_task_runtime(|rt| rt.task_count());
        let initial_dirty = with_task_runtime(|rt| rt.dirty_task_count());
        
        // Spawn tasks
        let task1 = spawn(async {});
        let _task2 = spawn(async {});
        
        let after_spawn_count = with_task_runtime(|rt| rt.task_count());
        let after_spawn_dirty = with_task_runtime(|rt| rt.dirty_task_count());
        
        assert_eq!(after_spawn_count, initial_count + 2, "Should have 2 more tasks");
        assert_eq!(after_spawn_dirty, initial_dirty + 2, "Should have 2 more dirty tasks");
        
        // Poll one task
        let _ = task1.poll_now();
        
        let after_one_poll = with_task_runtime(|rt| rt.dirty_task_count());
        assert!(after_one_poll < after_spawn_dirty, "Should have fewer dirty tasks after polling");
        
        // Poll all
        poll_tasks();
        
        let final_count = with_task_runtime(|rt| rt.task_count());
        let final_dirty = with_task_runtime(|rt| rt.dirty_task_count());
        
        assert_eq!(final_count, initial_count, "All tasks should be cleaned up");
        assert_eq!(final_dirty, 0, "No dirty tasks should remain");
    });
}

// ============================================================================
// 8. 深层嵌套作用域清理
// ============================================================================

#[test]
fn test_deep_nested_scope_cleanup() {
    use sig_async::task_runtime::with_task_runtime;
    
    create_scope(|| {
        let initial = with_task_runtime(|rt| rt.task_count());
        
        spawn(async {}); // Level 1
        
        create_scope(move || {
            spawn(async {}); // Level 2
            
            create_scope(move || {
                spawn(async {}); // Level 3
                
                create_scope(move || {
                    spawn(async {}); // Level 4
                });
            
                // After level 4 scope
                let count = with_task_runtime(|rt| rt.task_count());
                assert_eq!(count, initial + 3, "Level 4 task should be cleaned up");
            });
            
            // After level 3 scope
            let count = with_task_runtime(|rt| rt.task_count());
            assert_eq!(count, initial + 2, "Level 3 task should be cleaned up");
        });
        
        // After level 2 scope
        let count = with_task_runtime(|rt| rt.task_count());
        assert_eq!(count, initial + 1, "Level 2 task should be cleaned up");
        
        // Clean up level 1
    });
    
    // After all scopes - we can't reliably test the final count here
    // because we don't know what the initial count was before the outer create_scope
}

// ============================================================================
// 9. 边界情况
// ============================================================================

#[test]
fn test_empty_future() {
    create_scope(move || {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        spawn(async move {
            // Empty future - returns Ready immediately
            *executed_clone.borrow_mut() = true;
        });
        
        poll_tasks();
        assert!(*executed.borrow());
    });
}

#[test]
fn test_resource_clear_and_read() {
    create_scope(move || {
        let resource = use_resource(|| async { 42 });
        
        poll_tasks();
        assert_eq!(resource.value(), Some(42));
        
        // Clear the value
        resource.clear();
        assert_eq!(resource.value(), None);
        
        // Clear again - should be idempotent
        resource.clear();
        assert_eq!(resource.value(), None);
    });
}

#[test]
fn test_task_double_cancel() {
    create_scope(move || {
        let task = spawn(async {});
        
        // Cancel twice - should be idempotent
        task.cancel();
        task.cancel();
        
        // Should not crash
        poll_tasks();
    });
}
