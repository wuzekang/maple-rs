//! Example of using sig-async with a custom EventLoopWaker
//!
//! This example demonstrates how to implement EventLoopWaker
//! for a custom event loop system.

use sig_async::{set_event_waker, spawn, poll_tasks, EventLoopWaker};
use sig_reactive::create_scope;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

/// Custom event for our simple event loop
#[derive(Debug)]
enum CustomEvent {
    PollTasks,
    Quit,
}

/// A simple channel-based event loop waker
struct ChannelWaker {
    sender: mpsc::Sender<CustomEvent>,
}

impl ChannelWaker {
    fn new(sender: mpsc::Sender<CustomEvent>) -> Self {
        Self { sender }
    }
}

impl EventLoopWaker for ChannelWaker {
    fn wake(&self) {
        // Send event to wake the event loop
        let _ = self.sender.send(CustomEvent::PollTasks);
        println!("🔔 Waker: Sent PollTasks event");
    }
}

fn main() {
    println!("🚀 Starting custom event loop example");
    
    // Create a channel for events
    let (tx, rx) = mpsc::channel();
    
    // Set up the waker
    let waker = Box::new(ChannelWaker::new(tx.clone()));
    set_event_waker(waker);
    
    // Counter to track task executions
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();
    
    // Create a scope and spawn the task
    create_scope(move || {
        let counter = counter_clone.clone();
        let tx = tx.clone();
        
        spawn(async move {
            println!("📝 Task: Starting async work...");
            
            // Simulate some async work
            for i in 1..=3 {
                println!("📝 Task: Step {}/3", i);
                *counter.borrow_mut() += 1;
                
                // In a real scenario, you'd await actual async operations here
                // For this example, we just yield to demonstrate the flow
                futures_util::future::ready(()).await;
            }
            
            println!("✅ Task: Complete!");
            
            // Signal to quit after task is done
            let _ = tx.send(CustomEvent::Quit);
        });
    });
    
    println!("🔄 Event loop: Starting");
    
    // Simple event loop
    for event in rx.iter() {
        match event {
            CustomEvent::PollTasks => {
                println!("⚙️  Event loop: Polling tasks...");
                
                // Poll all ready tasks using the exported function
                poll_tasks();
                
                println!("⚙️  Event loop: Tasks polled. Counter = {}", counter.borrow());
            }
            CustomEvent::Quit => {
                println!("🛑 Event loop: Received quit signal");
                break;
            }
        }
    }
    
    println!("👋 Example complete! Final counter value: {}", counter.borrow());
}
