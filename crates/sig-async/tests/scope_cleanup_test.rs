//! Test to verify tasks are cleaned up when scopes are destroyed

use sig_async::{spawn, poll_tasks};
use sig_async::task_runtime::with_task_runtime;
use sig_reactive::create_scope;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_completed_task_cleanup() {
    println!("\n=== Test: Completed task cleanup ===");
    
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    create_scope(move || {
        let task_count_before = with_task_runtime(|rt| rt.task_count());
        println!("Tasks before spawn: {}", task_count_before);
        
        spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        let task_count_after_spawn = with_task_runtime(|rt| rt.task_count());
        println!("Tasks after spawn: {}", task_count_after_spawn);
        
        poll_tasks();
        
        let task_count_after_poll = with_task_runtime(|rt| rt.task_count());
        println!("Tasks after poll (completed): {}", task_count_after_poll);
        println!("Task executed: {}", *executed.borrow());
        
        // Task should be removed after completion
        assert_eq!(task_count_after_poll, task_count_before);
    });
    
    // After scope is destroyed
    let task_count_after_scope = with_task_runtime(|rt| rt.task_count());
    println!("Tasks after scope destroyed: {}", task_count_after_scope);
}

#[test]
fn test_pending_task_cleanup_on_scope_destroy() {
    println!("\n=== Test: Pending task cleanup on scope destroy ===");
    
    let initial_task_count = with_task_runtime(|rt| rt.task_count());
    println!("Initial task count: {}", initial_task_count);
    
    create_scope(move || {
        let task_count_before = with_task_runtime(|rt| rt.task_count());
        println!("Tasks before spawn: {}", task_count_before);
        
        // Spawn a task but DON'T poll it - it remains pending
        spawn(async move {
            panic!("This task should never execute because scope is destroyed!");
        });
        
        let task_count_after_spawn = with_task_runtime(|rt| rt.task_count());
        println!("Tasks after spawn (not polled): {}", task_count_after_spawn);
        
        // Verify task was spawned
        assert_eq!(task_count_after_spawn, task_count_before + 1);
        
        // Don't poll - let the task remain pending when scope is destroyed
    });
    
    // After scope is destroyed - check if task was cleaned up
    let task_count_after_scope = with_task_runtime(|rt| rt.task_count());
    let dirty_count = with_task_runtime(|rt| rt.dirty_task_count());
    
    println!("Tasks after scope destroyed: {}", task_count_after_scope);
    println!("Dirty tasks after scope destroyed: {}", dirty_count);
    
    let scope_tasks_count = with_task_runtime(|rt| rt.scope_task_count());
    println!("Total tasks in scope_tasks map: {}", scope_tasks_count);
    
    // This is the critical assertion - tasks should be cleaned up
    // EXPECTED: task_count_after_scope == initial_task_count (task was removed)
    // ACTUAL: Will tell us if cleanup happens or not
    println!("\n⚠️  CRITICAL CHECK:");
    if task_count_after_scope == initial_task_count {
        println!("✅ Tasks ARE cleaned up when scope is destroyed");
    } else {
        println!("❌ Tasks are NOT cleaned up when scope is destroyed!");
        println!("   This is a MEMORY LEAK - tasks remain after scope destruction");
    }
}

#[test]
fn test_nested_scopes_task_cleanup() {
    println!("\n=== Test: Nested scopes task cleanup ===");
    
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    create_scope(move || {
        println!("Outer scope created");
        
        spawn(async move {
            // Don't execute
        });
        
        let outer_task_count = with_task_runtime(|rt| rt.task_count());
        println!("Tasks in outer scope: {}", outer_task_count);
        
        create_scope(move || {
            println!("Inner scope created");
            
            spawn(async move {
                // Don't execute
            });
            
            let inner_task_count = with_task_runtime(|rt| rt.task_count());
            println!("Tasks in inner scope: {}", inner_task_count);
            
            assert_eq!(inner_task_count, outer_task_count + 1);
        });
        
        let after_inner_scope = with_task_runtime(|rt| rt.task_count());
        println!("Tasks after inner scope destroyed: {}", after_inner_scope);
        
        if after_inner_scope == outer_task_count {
            println!("✅ Inner scope task cleaned up");
        } else {
            println!("❌ Inner scope task NOT cleaned up");
        }
    });
    
    let final_task_count = with_task_runtime(|rt| rt.task_count());
    println!("Tasks after all scopes destroyed: {}", final_task_count);
    
    if final_task_count == initial_count {
        println!("✅ All tasks cleaned up");
    } else {
        println!("❌ Tasks leaked: {} remaining", final_task_count - initial_count);
    }
}

#[test]
fn test_multiple_tasks_in_scope_cleanup() {
    println!("\n=== Test: Multiple tasks in single scope cleanup ===");
    
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    create_scope(move || {
        // Spawn multiple tasks without polling
        for i in 0..5 {
            spawn(async move {
                println!("Task {} should not execute", i);
            });
        }
        
        let task_count = with_task_runtime(|rt| rt.task_count());
        println!("Spawned 5 tasks, total count: {}", task_count);
        
        assert_eq!(task_count, initial_count + 5);
    });
    
    let final_count = with_task_runtime(|rt| rt.task_count());
    println!("Tasks after scope destroyed: {}", final_count);
    
    if final_count == initial_count {
        println!("✅ All 5 tasks cleaned up");
    } else {
        println!("❌ Task leak: {} tasks remaining", final_count - initial_count);
    }
}
