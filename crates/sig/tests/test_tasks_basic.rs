use sig::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_spawn_basic_task() {
    use sig::reactive::runtime::with_reactive_mut;
    
    // Initialize runtime
    with_reactive_mut(|_rt| {});
    
    create_scope(|| {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Poll tasks to execute the spawned task
        poll_tasks();
        
        assert!(*executed.borrow(), "Task should have executed");
    });
}

#[test]
fn test_task_pause_resume() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_rt| {});
    
    create_scope(|| {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let task = spawn(async move {
            *counter_clone.borrow_mut() += 1;
        });
        
        // Pause before polling
        task.pause();
        assert!(task.paused(), "Task should be paused");
        
        // Try to poll - should not execute
        poll_tasks();
        assert_eq!(*counter.borrow(), 0, "Paused task should not execute");
        
        // Resume and poll
        task.resume();
        assert!(!task.paused(), "Task should not be paused");
        
        poll_tasks();
        assert_eq!(*counter.borrow(), 1, "Resumed task should execute");
    });
}

#[test]
fn test_task_cancel() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_rt| {});
    
    create_scope(|| {
        let executed = Rc::new(RefCell::new(false));
        let executed_clone = executed.clone();
        
        let task = spawn(async move {
            *executed_clone.borrow_mut() = true;
        });
        
        // Cancel before polling
        task.cancel();
        
        // Poll - should not execute
        poll_tasks();
        
        assert!(!*executed.borrow(), "Cancelled task should not execute");
    });
}

#[test]
fn test_multiple_tasks() {
    use sig::reactive::runtime::with_reactive_mut;
    
    with_reactive_mut(|_rt| {});
    
    create_scope(|| {
        let results = Rc::new(RefCell::new(Vec::new()));
        
        for i in 0..5 {
            let results_clone = results.clone();
            spawn(async move {
                results_clone.borrow_mut().push(i);
            });
        }
        
        // Poll all tasks
        poll_tasks();
        
        assert_eq!(results.borrow().len(), 5, "All tasks should have executed");
    });
}
