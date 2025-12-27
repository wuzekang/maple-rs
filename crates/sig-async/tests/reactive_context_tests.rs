//! Tests for reactive_context module

use sig_async::reactive_context::ReactiveContextFuture;
use sig_reactive::{create_scope, Signal, create_effect};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::Poll;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;

// Helper to create a simple future
struct SimpleFuture {
    value: i32,
    polled: Rc<RefCell<bool>>,
}

impl Future for SimpleFuture {
    type Output = i32;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        *self.polled.borrow_mut() = true;
        Poll::Ready(self.value)
    }
}

#[test]
fn test_reactive_context_future_basic() {
    create_scope(move || {
        let signal = Signal::new(10);
        let result = Rc::new(RefCell::new(0));
        let result_clone = result.clone();
        
        create_effect(move || {
            let effect_rc = sig_reactive::current_effect()
                .expect("Should be inside effect");
            
            let polled = Rc::new(RefCell::new(false));
            let polled_clone = polled.clone();
            
            let fut = SimpleFuture {
                value: *signal.read(),
                polled: polled_clone,
            };
            
            let mut ctx_fut = ReactiveContextFuture::new(fut, effect_rc);
            
            // Create a waker
            let waker = futures_util::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            
            // Poll the future
            if let Poll::Ready(value) = Pin::new(&mut ctx_fut).poll(&mut cx) {
                *result_clone.borrow_mut() = value;
            }
            
            assert!(*polled.borrow(), "Future should have been polled");
        });
        
        assert_eq!(*result.borrow(), 10);
    });
}

#[test]
fn test_reactive_context_with_signal_reads() {
    create_scope(move || {
        let signal = Signal::new(100);
        let reads = Rc::new(RefCell::new(Vec::new()));
        let reads_clone = reads.clone();
        
        create_effect(move || {
            let effect_rc = sig_reactive::current_effect()
                .expect("Should be inside effect");
            
            struct ReadingFuture {
                signal: Signal<i32>,
                reads: Rc<RefCell<Vec<i32>>>,
            }
            
            impl Future for ReadingFuture {
                type Output = ();
                
                fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                    // Read signal multiple times during polling
                    let value1 = *self.signal.read();
                    let value2 = *self.signal.read();
                    
                    self.reads.borrow_mut().push(value1);
                    self.reads.borrow_mut().push(value2);
                    
                    Poll::Ready(())
                }
            }
            
            let fut = ReadingFuture {
                signal,
                reads: reads_clone.clone(),
            };
            
            let mut ctx_fut = ReactiveContextFuture::new(fut, effect_rc);
            
            let waker = futures_util::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            
            let _ = Pin::new(&mut ctx_fut).poll(&mut cx);
        });
        
        // Should have read the signal twice
        assert_eq!(*reads.borrow(), vec![100, 100]);
    });
}

#[test]
fn test_reactive_context_pending_future() {
    create_scope(move || {
        let poll_count = Rc::new(RefCell::new(0));
        let poll_count_clone = poll_count.clone();
        
        create_effect(move || {
            let effect_rc = sig_reactive::current_effect()
                .expect("Should be inside effect");
            
            struct PendingFuture {
                count: Rc<RefCell<i32>>,
            }
            
            impl Future for PendingFuture {
                type Output = ();
                
                fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                    *self.count.borrow_mut() += 1;
                    
                    // Return pending on first poll, ready on second
                    if *self.count.borrow() >= 2 {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                }
            }
            
            let fut = PendingFuture {
                count: poll_count_clone.clone(),
            };
            
            let mut ctx_fut = ReactiveContextFuture::new(fut, effect_rc);
            
            let waker = futures_util::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            
            // First poll - should be pending
            assert_eq!(Pin::new(&mut ctx_fut).poll(&mut cx), Poll::Pending);
            assert_eq!(*poll_count.borrow(), 1);
            
            // Second poll - should be ready
            assert_eq!(Pin::new(&mut ctx_fut).poll(&mut cx), Poll::Ready(()));
            assert_eq!(*poll_count.borrow(), 2);
        });
    });
}
