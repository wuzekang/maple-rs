//! Tests for integration with tokio::time::sleep

use sig_async::tasks::{spawn, poll_tasks};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::sleep;

/// Helper function to poll tasks in a loop until completion or timeout
async fn poll_until_complete(timeout: Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        poll_tasks();
        // Yield to tokio to allow sleep timers to fire
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
    // Final poll
    poll_tasks();
}

#[tokio::test]
async fn test_spawn_with_tokio_sleep() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    // Spawn at root scope level (tasks can be spawned at root)
    spawn(async move {
        // Sleep for a short duration
        sleep(Duration::from_millis(10)).await;
        *executed_clone.borrow_mut() = true;
    });
    
    // Poll tasks to execute
    poll_tasks();
    
    // Poll until tasks complete
    poll_until_complete(Duration::from_millis(50)).await;
    
    assert!(*executed.borrow(), "Task with sleep should have been executed");
}

#[tokio::test]
async fn test_multiple_tasks_with_different_sleep_durations() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let o1 = order.clone();
    let o2 = order.clone();
    let o3 = order.clone();
    
    // Task with 30ms sleep
    spawn(async move {
        sleep(Duration::from_millis(30)).await;
        o1.borrow_mut().push(3);
    });
    
    // Task with 10ms sleep
    spawn(async move {
        sleep(Duration::from_millis(10)).await;
        o2.borrow_mut().push(1);
    });
    
    // Task with 20ms sleep
    spawn(async move {
        sleep(Duration::from_millis(20)).await;
        o3.borrow_mut().push(2);
    });
    
    // Poll all tasks
    poll_tasks();
    
    // Poll until all tasks complete
    poll_until_complete(Duration::from_millis(100)).await;
    
    // Tasks should complete in order based on sleep duration
    assert_eq!(*order.borrow(), vec![1, 2, 3], "Tasks should complete in order of sleep duration");
}

#[tokio::test]
async fn test_task_cancel_with_sleep() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    let task = spawn(async move {
        sleep(Duration::from_millis(10)).await;
        *executed_clone.borrow_mut() = true;
    });
    
    // Cancel before the sleep completes
    task.cancel();
    
    // Poll tasks
    poll_tasks();
    
    // Poll until timeout
    poll_until_complete(Duration::from_millis(50)).await;
    
    assert!(!*executed.borrow(), "Cancelled task should not execute even after sleep duration");
}

#[tokio::test]
async fn test_task_pause_resume_with_sleep() {
    let value = Rc::new(RefCell::new(0));
    let value_clone = value.clone();
    
    let task = spawn(async move {
        sleep(Duration::from_millis(10)).await;
        *value_clone.borrow_mut() = 42;
    });
    
    // Pause the task
    task.pause();
    
    // Poll tasks - should not execute because paused
    poll_tasks();
    
    // Poll for a while
    poll_until_complete(Duration::from_millis(30)).await;
    assert_eq!(*value.borrow(), 0, "Paused task should not execute");
}

#[tokio::test]
async fn test_sequential_sleeps_in_task() {
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();
    
    spawn(async move {
        *counter_clone.borrow_mut() += 1;
        sleep(Duration::from_millis(10)).await;
        
        *counter_clone.borrow_mut() += 10;
        sleep(Duration::from_millis(10)).await;
        
        *counter_clone.borrow_mut() += 100;
    });
    
    poll_tasks();
    
    // Poll until all sleeps complete
    poll_until_complete(Duration::from_millis(50)).await;
    
    assert_eq!(*counter.borrow(), 111, "All sequential operations should complete");
}

#[tokio::test]
async fn test_nested_task_spawning_with_sleep() {
    let counter = Rc::new(RefCell::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();
    
    spawn(async move {
        sleep(Duration::from_millis(10)).await;
        *c1.borrow_mut() += 1;
        
        // Spawn another task from within
        spawn(async move {
            sleep(Duration::from_millis(10)).await;
            *c2.borrow_mut() += 10;
        });
        
        poll_tasks();
    });
    
    // Poll all tasks
    poll_tasks();
    
    // Poll until both tasks complete
    poll_until_complete(Duration::from_millis(50)).await;
    
    assert_eq!(*counter.borrow(), 11, "Both parent and child tasks should execute");
}

#[tokio::test]
async fn test_task_with_zero_duration_sleep() {
    let executed = Rc::new(RefCell::new(false));
    let executed_clone = executed.clone();
    
    spawn(async move {
        // Zero duration sleep - should yield to scheduler
        sleep(Duration::from_millis(0)).await;
        *executed_clone.borrow_mut() = true;
    });
    
    poll_tasks();
    
    // Poll until complete
    poll_until_complete(Duration::from_millis(10)).await;
    
    assert!(*executed.borrow(), "Task with zero duration sleep should execute");
}

#[tokio::test]
async fn test_interleaved_sleep_and_computation() {
    let results = Rc::new(RefCell::new(Vec::new()));
    let r1 = results.clone();
    
    spawn(async move {
        r1.borrow_mut().push("start");
        
        sleep(Duration::from_millis(10)).await;
        r1.borrow_mut().push("after_first_sleep");
        
        // Some computation
        let sum: u32 = (1..100).sum();
        r1.borrow_mut().push(if sum > 0 { "computed" } else { "error" });
        
        sleep(Duration::from_millis(10)).await;
        r1.borrow_mut().push("after_second_sleep");
        
        r1.borrow_mut().push("end");
    });
    
    poll_tasks();
    
    // Poll until task completes
    poll_until_complete(Duration::from_millis(50)).await;
    
    let expected = vec!["start", "after_first_sleep", "computed", "after_second_sleep", "end"];
    assert_eq!(*results.borrow(), expected, "Task should execute in correct order");
}

#[tokio::test]
async fn test_sleep_with_signal_update() {
    use sig_reactive::Signal;
    
    let final_value = Rc::new(RefCell::new(0));
    let final_value_clone = final_value.clone();
    
    let signal = Signal::new(10);
    
    spawn(async move {
        // Read initial value
        let initial = *signal.read();
        
        // Wait a bit
        sleep(Duration::from_millis(10)).await;
        
        // Update signal
        *signal.write() = initial * 2;
        
        sleep(Duration::from_millis(10)).await;
        
        // Read final value
        *final_value_clone.borrow_mut() = *signal.read();
    });
    
    poll_tasks();
    
    // Poll until all operations complete
    poll_until_complete(Duration::from_millis(50)).await;
    
    // The task should eventually read the updated value (20)
    assert_eq!(*final_value.borrow(), 20, "Signal should have been updated to 20");
}

#[tokio::test]
async fn test_long_running_task_with_periodic_sleeps() {
    let iterations = Rc::new(RefCell::new(0));
    let iterations_clone = iterations.clone();
    
    spawn(async move {
        for _ in 0..5 {
            *iterations_clone.borrow_mut() += 1;
            sleep(Duration::from_millis(5)).await;
        }
    });
    
    poll_tasks();
    
    // Poll until all iterations complete
    poll_until_complete(Duration::from_millis(50)).await;
    
    assert_eq!(*iterations.borrow(), 5, "All iterations should complete");
}
