//! Test waker callbacks after scope cleanup

use sig_async::{spawn, poll_tasks};
use sig_async::task_runtime::with_task_runtime;
use sig_reactive::create_scope;
use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Poll, Waker};

#[test]
fn test_waker_called_after_scope_cleanup() {
    let saved_waker: Rc<RefCell<Option<Waker>>> = Rc::new(RefCell::new(None));
    let saved_waker_for_closure = saved_waker.clone();
    
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    create_scope(move || {
        spawn(async move {
            futures_util::future::poll_fn(|cx| {
                *saved_waker_for_closure.borrow_mut() = Some(cx.waker().clone());
                Poll::<()>::Pending
            }).await;
        });
        
        poll_tasks();
    });
    
    assert!(saved_waker.borrow().is_some(), "Waker should have been saved");
    
    let after_cleanup = with_task_runtime(|rt| rt.task_count());
    assert_eq!(after_cleanup, initial_count, "Task should be cleaned up");
    
    // Critical test: call waker on cleaned-up task
    if let Some(waker) = saved_waker.borrow().as_ref() {
        waker.wake_by_ref();  // Should NOT crash
        
        poll_tasks();
        
        let final_count = with_task_runtime(|rt| rt.task_count());
        assert_eq!(final_count, initial_count, 
                   "Task should NOT be resurrected by waker");
    }
}

#[test]
fn test_waker_spam_after_cleanup() {
    let saved_waker: Rc<RefCell<Option<Waker>>> = Rc::new(RefCell::new(None));
    let waker_for_closure = saved_waker.clone();
    
    let initial_count = with_task_runtime(|rt| rt.task_count());
    
    create_scope(move || {
        spawn(async move {
            futures_util::future::poll_fn(|cx| {
                *waker_for_closure.borrow_mut() = Some(cx.waker().clone());
                Poll::<()>::Pending
            }).await;
        });
        
        poll_tasks();
    });
    
    assert_eq!(with_task_runtime(|rt| rt.task_count()), initial_count);
    
    // Spam the waker 100 times
    if let Some(waker) = saved_waker.borrow().as_ref() {
        for i in 0..100 {
            waker.wake_by_ref();
            
            if i % 10 == 0 {
                poll_tasks();
            }
        }
    }
    
    poll_tasks();
    
    let final_count = with_task_runtime(|rt| rt.task_count());
    assert_eq!(final_count, initial_count, "Should handle waker spam");
}
