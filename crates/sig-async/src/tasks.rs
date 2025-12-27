//! Task system for async operations in sig
//!
//! This module provides a task spawning system similar to Dioxus, allowing
//! components to spawn async tasks that are lifecycle-managed by scopes.

use crate::task_runtime::{with_task_runtime, TaskWakerHandle};
use sig_reactive::ScopeId;
use slotmap::DefaultKey;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Poll, Waker};

/// A task's unique identifier.
///
/// `Task` is a unique identifier for a task that has been spawned onto the runtime.
/// It can be used to control the task (pause, resume, cancel).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub(crate) id: TaskId,
    // We add a raw pointer to make this !Send + !Sync (tasks are single-threaded)
    unsend: PhantomData<*const ()>,
}

pub(crate) type TaskId = slotmap::DefaultKey;

impl Task {
    /// Create a task from a raw id
    pub(crate) const fn from_id(id: slotmap::DefaultKey) -> Self {
        Self {
            id,
            unsend: PhantomData,
        }
    }

    /// Start a new future on the same thread as the rest of the reactive system.
    ///
    /// This future will be managed by the reactive system and will be dropped when
    /// the scope that owns it is dropped.
    ///
    /// # Example
    /// ```
    /// use sig_async::tasks::Task;
    /// use sig_async::poll_tasks;
    /// use sig_reactive::create_scope;
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// let executed = Rc::new(RefCell::new(false));
    /// let executed_clone = executed.clone();
    ///
    /// create_scope(move || {
    ///     Task::new(async move {
    ///         *executed_clone.borrow_mut() = true;
    ///     });
    ///     
    ///     poll_tasks();
    ///     
    ///     assert!(*executed.borrow());
    /// });
    /// ```
    pub fn new(task: impl Future<Output = ()> + 'static) -> Self {
        spawn(task)
    }

    /// Drop the task immediately.
    pub fn cancel(self) {
        remove_task(self);
    }

    /// Pause the task.
    pub fn pause(&self) {
        self.set_active(false);
    }

    /// Resume the task.
    pub fn resume(&self) {
        self.set_active(true);
    }

    /// Check if the task is paused.
    pub fn paused(&self) -> bool {
        with_task_runtime(|rt| {
            rt.tasks.borrow().get(self.id).map(|task| !task.active.get()).unwrap_or(false)
        })
    }

    /// Wake the task to be polled.
    pub fn wake(&self) {
        with_task_runtime(|rt| {
            rt.schedule_task(self.id);
        });
    }

    /// Poll the task immediately.
    pub fn poll_now(&self) -> Poll<()> {
        handle_task_wakeup(*self)
    }

    /// Set the task as active or paused.
    fn set_active(&self, active: bool) {
        with_task_runtime(|rt| {
            if let Some(task) = rt.tasks.borrow().get(self.id) {
                let was_active = task.active.replace(active);
                if !was_active && active {
                    rt.schedule_task(self.id);
                }
            }
        });
    }
}

/// The internal task structure
pub(crate) struct LocalTask {
    pub(crate) scope: ScopeId,
    pub(crate) parent: Option<Task>,
    pub(crate) task: RefCell<Pin<Box<dyn Future<Output = ()> + 'static>>>,
    pub(crate) waker: Waker,
    pub(crate) active: Cell<bool>,
}

// Remove the old LocalTaskHandle - now it's TaskWakerHandle in task_runtime

/// Spawn a new task on the current scope
///
/// # Example
/// ```
/// use sig_async::{spawn, poll_tasks};
/// use sig_reactive::create_scope;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// let executed = Rc::new(RefCell::new(false));
/// let executed_clone = executed.clone();
///
/// create_scope(move || {
///     spawn(async move {
///         *executed_clone.borrow_mut() = true;
///     });
///     
///     poll_tasks();
///     
///     assert!(*executed.borrow());
/// });
/// ```
pub fn spawn(task: impl Future<Output = ()> + 'static) -> Task {
    spawn_task_inner(Box::pin(task))
}

fn spawn_task_inner(pinned_task: Pin<Box<dyn Future<Output = ()>>>) -> Task {
    let scope = sig_reactive::current_scope_id()
        .expect("Cannot spawn task outside of a scope");

    with_task_runtime(|rt| {
        // Create task ID
        let mut task_id = Task::from_id(DefaultKey::default());

        rt.tasks.borrow_mut().insert_with_key(|key| {
            task_id = Task::from_id(key);

            Rc::new(LocalTask {
                scope,
                active: Cell::new(true),
                parent: rt.current_task.get(),
                task: RefCell::new(pinned_task),
                waker: futures_util::task::waker(Arc::new(TaskWakerHandle { 
                    id: task_id.id,
                    tx: rt.message_sender(),
                })),
            })
        });

        // Register the task with its scope in the task runtime
        rt.scope_tasks.borrow_mut()
            .entry(scope)
            .or_insert_with(HashSet::new)
            .insert(task_id);

        // 🔥 Register cleanup to remove all tasks when scope is destroyed
        // Only register this cleanup once per scope (when the first task is spawned)
        if rt.scope_tasks.borrow().get(&scope).map_or(false, |tasks| tasks.len() == 1) {
            sig_reactive::on_cleanup(move || {
                cleanup_scope_tasks(scope);
            });
        }

        // Schedule the task to be polled
        rt.schedule_task(task_id.id);

        task_id
    })
}

/// Remove a task from the runtime
pub(crate) fn remove_task(id: Task) {
    with_task_runtime(|rt| {
        let task = rt.tasks.borrow_mut().remove(id.id);

        if let Some(task) = task {
            // Remove the task from the scope in task runtime
            if let Some(scope_tasks) = rt.scope_tasks.borrow_mut().get_mut(&task.scope) {
                scope_tasks.remove(&id);
            }

            // Remove from dirty tasks queue
            rt.dirty_tasks.borrow_mut().retain(|&tid| tid != id.id);
        }
    });
}

/// Handle a task wakeup by polling it
fn handle_task_wakeup(id: Task) -> Poll<()> {
    let task = with_task_runtime(|rt| rt.tasks.borrow().get(id.id).cloned());

    // The task was removed from the scheduler, so we can just ignore it
    let Some(task) = task else {
        return Poll::Ready(());
    };

    // If a task woke up but is paused, we can just ignore it
    if !task.active.get() {
        return Poll::Pending;
    }

    let mut cx = std::task::Context::from_waker(&task.waker);

    // Save the current task and set this as current
    let prev_task = with_task_runtime(|rt| {
        let prev = rt.current_task.get();
        rt.current_task.set(Some(id));
        prev
    });

    // Poll the future
    let poll_result = task.task.borrow_mut().as_mut().poll(&mut cx);

    // Restore previous task
    with_task_runtime(|rt| {
        rt.current_task.set(prev_task);
    });

    if poll_result.is_ready() {
        // Task is complete - remove it
        remove_task(id);
    }

    poll_result
}

/// Get the currently running task
pub fn current_task() -> Option<Task> {
    with_task_runtime(|rt| rt.current_task())
}

/// Get the parent task of the given task
pub fn parent_task(task: Task) -> Option<Task> {
    with_task_runtime(|rt| rt.parent_task(task))
}

/// Poll all pending tasks
/// 
/// This function first processes all messages from the channel (cross-thread wake notifications),
/// then polls all dirty tasks.
pub fn poll_tasks() {
    // First, process all pending messages from the channel
    // This handles wake notifications from background threads
    with_task_runtime(|rt| {
        rt.process_messages();
    });
    
    // Now poll all dirty tasks
    loop {
        let task_id = with_task_runtime(|rt| rt.dirty_tasks.borrow_mut().pop_front());
        
        let Some(task_id) = task_id else {
            break;
        };
        
        let task = Task::from_id(task_id);
        let _ = handle_task_wakeup(task);
    }
}

/// Cleanup all tasks belonging to a specific scope
/// 
/// This should be called when a scope is destroyed to prevent memory leaks.
/// It removes all tasks that belong to the scope and its descendants.
pub fn cleanup_scope_tasks(scope_id: ScopeId) {
    with_task_runtime(|rt| {
        // Get all tasks for this scope
        let tasks_to_remove: Vec<Task> = rt.scope_tasks.borrow()
            .get(&scope_id)
            .map(|tasks| tasks.iter().copied().collect())
            .unwrap_or_default();
        
        // Remove each task
        for task in tasks_to_remove {
            remove_task(task);
        }
        
        // Remove the scope entry from scope_tasks map
        rt.scope_tasks.borrow_mut().remove(&scope_id);
    });
}
