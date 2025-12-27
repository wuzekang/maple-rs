//! Tests for tasks module

use sig_async::tasks::{spawn, poll_tasks, current_task};
use sig_reactive::{create_scope, Signal};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::Poll;

#[test]
fn test_spawn_simple_task() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    create_scope(move || {
        spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Poll tasks to execute
        poll_tasks();
        
        assert!(*executed.borrow(), "Task should have been executed");
    });
}

#[test]
fn test_spawn_multiple_tasks() {
    let counter = Rc::new(RefCell::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();
    let c3 = counter.clone();
    
    create_scope(move || {
        spawn(async move {
            *c1.borrow_mut() += 1;
        });
        
        spawn(async move {
            *c2.borrow_mut() += 10;
        });
        
        spawn(async move {
            *c3.borrow_mut() += 100;
        });
        
        // Poll all tasks
        poll_tasks();
        
        assert_eq!(*counter.borrow(), 111, "All tasks should have executed");
    });
}

#[test]
fn test_task_cancel() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            // This should never execute because we cancel before polling
            *executed_clone.borrow_mut() = true;
        });
        
        // Cancel before polling
        task.cancel();
        
        // Poll tasks
        poll_tasks();
        
        assert!(!*executed.borrow(), "Cancelled task should not execute");
    });
}

#[test]
fn test_task_pause_resume() {
    let value = Rc::new(RefCell::new(0));
    let value_clone = value.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            *value_clone.borrow_mut() = 42;
        });
        
        // Pause the task
        task.pause();
        assert!(task.paused(), "Task should be paused");
        
        // Poll tasks - should not execute because paused
        poll_tasks();
        assert_eq!(*value.borrow(), 0, "Paused task should not execute");
        
        // Resume the task
        task.resume();
        assert!(!task.paused(), "Task should not be paused");
        
        // Poll again - should execute now
        poll_tasks();
        assert_eq!(*value.borrow(), 42, "Resumed task should execute");
    });
}

#[test]
fn test_task_wake() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Wake the task explicitly
        task.wake();
        
        // Poll tasks
        poll_tasks();
        
        assert!(*executed.borrow(), "Woken task should execute");
    });
}

#[test]
fn test_task_poll_now() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Poll the task directly
        let result = task.poll_now();
        
        assert_eq!(result, Poll::Ready(()), "Task should complete immediately");
        assert!(*executed.borrow(), "Task should have executed");
    });
}

#[test]
fn test_current_task_context() {
    let current_in_task = Rc::new(RefCell::new(None));
    let current_clone = current_in_task.clone();
    
    create_scope(move || {
        let task = spawn(async move {
            // Inside the task, current_task should return this task
            *current_clone.borrow_mut() = current_task();
        });
        
        // Poll the task
        let _ = task.poll_now();
        
        // The current task inside should match the spawned task
        assert_eq!(*current_in_task.borrow(), Some(task));
    });
}

#[test]
fn test_task_with_signal() {
    create_scope(move || {
        let signal = Signal::new(10);
        
        let result = Rc::new(RefCell::new(0));
        let result_clone = result.clone();
        
        spawn(async move {
            let value = *signal.read();
            *result_clone.borrow_mut() = value * 2;
        });
        
        poll_tasks();
        
        assert_eq!(*result.borrow(), 20);
    });
}

#[test]
fn test_nested_task_spawning() {
    let counter = Rc::new(RefCell::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();
    
    create_scope(move || {
        spawn(async move {
            *c1.borrow_mut() += 1;
            
            // Spawn another task from within
            spawn(async move {
                *c2.borrow_mut() += 10;
            });
        });
        
        // Poll all tasks
        poll_tasks();
        
        assert_eq!(*counter.borrow(), 11, "Both parent and child tasks should execute");
    });
}

// Note: We cannot test spawning outside of a scope easily because
// the global RUNTIME in sig-reactive always has a ROOT scope active.
// This is expected behavior - tasks can be spawned at the root level.
#[test]
fn test_spawn_at_root_scope() {
    // This should work - spawning at root scope level
    create_scope(move || {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        poll_tasks();
        
        assert!(*executed.borrow());
    });
}
