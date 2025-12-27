//! Task runtime for managing async tasks
//!
//! Similar to Dioxus, tasks are managed separately from the reactive system
//! in a dedicated runtime structure.
//!
//! This module provides a generic task runtime that can work with different
//! event loop implementations by requiring users to implement the EventLoopWaker trait.
//!
//! ## Architecture
//!
//! The task runtime uses a **channel-based** architecture to support cross-thread waking:
//!
//! 1. When a task is spawned, it gets a waker that includes a channel sender
//! 2. When the task's future completes (e.g., `tokio::time::sleep`), it wakes from any thread
//! 3. The waker sends a message through the channel (thread-safe)
//! 4. The main thread polls the channel and marks tasks as dirty
//! 5. The event loop is notified to poll tasks
//!
//! This allows `tokio::time::sleep`, `futures_timer::Delay`, and other cross-thread
//! async primitives to work correctly.

use crate::tasks::{LocalTask, Task, TaskId};
use sig_reactive::ScopeId;
use slotmap::DefaultKey;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;
use std::future::Future;

/// Messages that can be sent to the task runtime from any thread
#[derive(Debug, Clone, Copy)]
pub enum TaskMessage {
    /// A task has been woken and needs to be polled
    TaskNotified(TaskId),
}

/// Trait for waking the event loop when tasks are ready
/// 
/// Implement this trait to integrate with your event loop system.
/// 
/// # Example
/// 
/// ```
/// use sig_async::task_runtime::EventLoopWaker;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// // Example waker that tracks wake calls
/// struct MyEventLoopWaker {
///     wake_count: Rc<RefCell<usize>>,
/// }
/// 
/// impl EventLoopWaker for MyEventLoopWaker {
///     fn wake(&self) {
///         *self.wake_count.borrow_mut() += 1;
///     }
/// }
///
/// // Usage
/// let wake_count = Rc::new(RefCell::new(0));
/// let waker = MyEventLoopWaker { wake_count: wake_count.clone() };
/// waker.wake();
/// assert_eq!(*wake_count.borrow(), 1);
/// ```
pub trait EventLoopWaker: 'static {
    /// Wake the event loop to poll tasks
    fn wake(&self);
}

/// Task runtime - manages all async tasks
pub struct TaskRuntime {
    /// All spawned tasks
    pub(crate) tasks: RefCell<slotmap::SlotMap<DefaultKey, Rc<LocalTask>>>,
    
    /// Currently executing task
    pub(crate) current_task: Cell<Option<Task>>,
    
    /// Tasks waiting to be polled
    pub(crate) dirty_tasks: RefCell<VecDeque<TaskId>>,
    
    /// Event loop waker to send wake events
    pub(crate) event_waker: RefCell<Option<Box<dyn EventLoopWaker>>>,
    
    /// Scope to tasks mapping
    pub(crate) scope_tasks: RefCell<HashMap<ScopeId, HashSet<Task>>>,
    
    /// Channel for receiving task wake messages from any thread
    pub(crate) message_rx: RefCell<futures_channel::mpsc::UnboundedReceiver<TaskMessage>>,
    
    /// Channel sender for task wakers (cloned into each waker)
    pub(crate) message_tx: futures_channel::mpsc::UnboundedSender<TaskMessage>,
}

impl TaskRuntime {
    pub fn new() -> Self {
        let (tx, rx) = futures_channel::mpsc::unbounded();
        
        Self {
            tasks: RefCell::new(slotmap::SlotMap::new()),
            current_task: Cell::new(None),
            dirty_tasks: RefCell::new(VecDeque::new()),
            event_waker: RefCell::new(None),
            scope_tasks: RefCell::new(HashMap::new()),
            message_rx: RefCell::new(rx),
            message_tx: tx,
        }
    }
    
    /// Set the event loop waker
    /// This should be called once when the event loop is created
    pub fn set_event_waker(&self, waker: Box<dyn EventLoopWaker>) {
        *self.event_waker.borrow_mut() = Some(waker);
    }
    
    /// Get the currently running task
    pub fn current_task(&self) -> Option<Task> {
        self.current_task.get()
    }
    
    /// Get the parent task of a given task
    pub fn parent_task(&self, task: Task) -> Option<Task> {
        self.tasks.borrow().get(task.id)?.parent
    }
    
    /// Schedule a task to be polled
    pub fn schedule_task(&self, task_id: TaskId) {
        self.dirty_tasks.borrow_mut().push_back(task_id);
        
        // Wake the event loop if a waker is set
        if let Some(waker) = self.event_waker.borrow().as_ref() {
            waker.wake();
        }
    }
    
    
    /// Process all pending task messages from the channel
    /// 
    /// This should be called before polling tasks to ensure all
    /// cross-thread wake notifications are processed.
    /// 
    /// Returns the number of messages processed.
    pub fn process_messages(&self) -> usize {
        use futures_util::stream::StreamExt;
        
        let mut count = 0;
        let mut rx = self.message_rx.borrow_mut();
        
        // Process all available messages without blocking
        loop {
            match rx.try_next() {
                Ok(Some(msg)) => {
                    match msg {
                        TaskMessage::TaskNotified(task_id) => {
                            drop(rx); // Release borrow
                            self.schedule_task(task_id);
                            rx = self.message_rx.borrow_mut(); // Re-borrow
                            count += 1;
                        }
                    }
                }
                Ok(None) => {
                    // Channel closed
                    break;
                }
                Err(_) => {
                    // No more messages
                    break;
                }
            }
        }
        
        count
    }
    
    /// Get a clone of the message sender for creating wakers
    pub(crate) fn message_sender(&self) -> futures_channel::mpsc::UnboundedSender<TaskMessage> {
        self.message_tx.clone()
    }
    
    /// Get the number of active tasks (for testing/debugging)
    pub fn task_count(&self) -> usize {
        self.tasks.borrow().len()
    }
    
    /// Get the number of dirty tasks (for testing/debugging)
    pub fn dirty_task_count(&self) -> usize {
        self.dirty_tasks.borrow().len()
    }
    
    /// Check if there are dirty tasks
    pub fn has_dirty_tasks(&self) -> bool {
        !self.dirty_tasks.borrow().is_empty()
    }
    
    /// Get access to message receiver for async waiting
    /// 
    /// # Safety
    /// This should only be used for creating async wait futures.
    /// Do not hold the borrow across await points.
    pub fn poll_message_receiver(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<TaskMessage>> {
        use futures_util::stream::StreamExt;
        self.message_rx.borrow_mut().poll_next_unpin(cx)
    }
    
    /// Get the total number of tasks across all scopes (for testing/debugging)
    pub fn scope_task_count(&self) -> usize {
        self.scope_tasks.borrow().values().map(|s| s.len()).sum()
    }
}

impl Default for TaskRuntime {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Thread-local task runtime
// ============================================================================

thread_local! {
    static TASK_RUNTIME: RefCell<TaskRuntime> = RefCell::new(TaskRuntime::new());
}

/// Access task runtime (immutable)
pub fn with_task_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&TaskRuntime) -> R,
{
    TASK_RUNTIME.with(|rt| f(&rt.borrow()))
}

/// Access task runtime (mutable)
pub fn with_task_runtime_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut TaskRuntime) -> R,
{
    TASK_RUNTIME.with(|rt| f(&mut rt.borrow_mut()))
}

/// Set a generic event loop waker for task waking
pub fn set_event_waker(waker: Box<dyn EventLoopWaker>) {
    with_task_runtime(|rt| {
        rt.set_event_waker(waker);
    });
}

/// Create a future that waits for the next task message
/// 
/// This future, when polled, will register its waker with the channel.
/// When a message arrives, the channel will automatically call the waker.
/// 
/// This is similar to Dioxus's approach.
pub async fn wait_for_task_message() -> Option<TaskId> {
    use std::future::poll_fn;
    use futures_util::stream::StreamExt;
    
    // Poll the channel and let it register our waker
    let msg = poll_fn(|cx| {
        with_task_runtime(|rt| {
            rt.message_rx.borrow_mut().poll_next_unpin(cx)
        })
    }).await?;
    
    match msg {
        TaskMessage::TaskNotified(id) => {
            // Schedule the task
            with_task_runtime(|rt| {
                rt.schedule_task(id);
            });
            Some(id)
        }
    }
}

/// Waker handle that marks tasks as dirty when awakened
/// 
/// This waker is **thread-safe** and can be called from any thread.
/// It only sends a message through the channel. The channel's Stream
/// implementation will automatically call the registered waker (from wait_for_message).
pub(crate) struct TaskWakerHandle {
    pub(crate) id: TaskId,
    pub(crate) tx: futures_channel::mpsc::UnboundedSender<TaskMessage>,
}

// Make TaskWakerHandle explicitly Send + Sync
unsafe impl Send for TaskWakerHandle {}
unsafe impl Sync for TaskWakerHandle {}

impl futures_util::task::ArcWake for TaskWakerHandle {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Simply send a message through the channel
        // The channel will automatically wake any task waiting on rx.next().await
        let _ = arc_self.tx.unbounded_send(TaskMessage::TaskNotified(arc_self.id));
    }
}
