//! Tests for task_runtime module

use sig_async::task_runtime::{EventLoopWaker, TaskRuntime, set_event_waker, with_task_runtime};
use std::cell::RefCell;
use std::rc::Rc;

// Test event loop waker
#[derive(Clone)]
struct TestWaker {
    wake_count: Rc<RefCell<usize>>,
}

impl TestWaker {
    fn new() -> Self {
        Self {
            wake_count: Rc::new(RefCell::new(0)),
        }
    }
    
    fn wake_count(&self) -> usize {
        *self.wake_count.borrow()
    }
}

impl EventLoopWaker for TestWaker {
    fn wake(&self) {
        *self.wake_count.borrow_mut() += 1;
    }
}

#[test]
fn test_task_runtime_creation() {
    let runtime = TaskRuntime::new();
    assert!(runtime.current_task().is_none());
}

#[test]
fn test_task_runtime_default() {
    let runtime = TaskRuntime::default();
    assert!(runtime.current_task().is_none());
}

#[test]
fn test_set_event_waker() {
    let waker = TestWaker::new();
    let waker_clone = waker.clone();
    
    set_event_waker(Box::new(waker));
    
    // Schedule a dummy task (will fail since no actual task exists, but should wake)
    with_task_runtime(|rt| {
        rt.schedule_task(slotmap::DefaultKey::default());
    });
    
    // Waker should have been called
    assert_eq!(waker_clone.wake_count(), 1);
}

#[test]
fn test_task_runtime_schedule_without_waker() {
    let runtime = TaskRuntime::new();
    
    // Should not panic when scheduling without a waker
    runtime.schedule_task(slotmap::DefaultKey::default());
    
    // Task should be scheduled (we can't check dirty queue directly, but it shouldn't panic)
}

#[test]
fn test_task_runtime_schedule_with_waker() {
    let runtime = TaskRuntime::new();
    let waker = TestWaker::new();
    let waker_clone = waker.clone();
    
    runtime.set_event_waker(Box::new(waker));
    
    // Schedule a task
    runtime.schedule_task(slotmap::DefaultKey::default());
    
    // Waker should be called
    assert_eq!(waker_clone.wake_count(), 1);
}

#[test]
fn test_multiple_schedules() {
    let runtime = TaskRuntime::new();
    let waker = TestWaker::new();
    let waker_clone = waker.clone();
    
    runtime.set_event_waker(Box::new(waker));
    
    // Schedule multiple tasks
    runtime.schedule_task(slotmap::DefaultKey::default());
    runtime.schedule_task(slotmap::DefaultKey::default());
    runtime.schedule_task(slotmap::DefaultKey::default());
    
    // Waker should be called 3 times
    assert_eq!(waker_clone.wake_count(), 3);
}
