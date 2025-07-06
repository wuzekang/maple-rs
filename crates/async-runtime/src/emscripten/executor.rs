use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::os::raw::{c_int, c_void};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

// Emscripten FFI 绑定
extern "C" {
    fn emscripten_set_main_loop_arg(
        func: extern "C" fn(*mut c_void),
        arg: *mut c_void,
        fps: c_int,
        simulate_infinite_loop: c_int,
    );
}

/// 任务结构
struct Task {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
    is_ready: Cell<bool>,
}

/// 执行器内部状态
struct ExecutorInner {
    // 任务存储（使用 Option 支持槽位重用）
    tasks: Vec<Option<Task>>,
    // 就绪任务队列（存储索引）
    ready_queue: VecDeque<usize>,
    // 空闲槽位列表
    free_slots: Vec<usize>,
}

/// 简单的 Emscripten Future 执行器
pub struct EmscriptenExecutor {
    inner: Rc<RefCell<ExecutorInner>>,
    // 待添加的新任务（放在 RefCell 外部，避免借用冲突）
    pending_tasks: Rc<RefCell<Vec<Pin<Box<dyn Future<Output = ()> + 'static>>>>>,
}

impl EmscriptenExecutor {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ExecutorInner {
                tasks: Vec::new(),
                ready_queue: VecDeque::new(),
                free_slots: Vec::new(),
            })),
            pending_tasks: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// 生成一个新的异步任务
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        // 尝试直接添加任务
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            // 找到一个槽位：重用空闲槽位或添加新槽位
            let index = if let Some(free_index) = inner.free_slots.pop() {
                inner.tasks[free_index] = Some(Task {
                    future: Box::pin(future),
                    is_ready: Cell::new(true),
                });
                free_index
            } else {
                let index = inner.tasks.len();
                inner.tasks.push(Some(Task {
                    future: Box::pin(future),
                    is_ready: Cell::new(true),
                }));
                index
            };
            
            // 新任务立即加入就绪队列
            inner.ready_queue.push_back(index);
        } else {
            // 如果无法立即添加（因为借用冲突），添加到待处理队列
            self.pending_tasks.borrow_mut().push(Box::pin(future));
        }
    }

    /// 运行执行器（接管控制流）
    pub fn run(self: &Rc<Self>) {
        let executor_ptr = Rc::into_raw(self.clone());
        unsafe {
            // 以 60 FPS 运行主循环，降低延迟
            emscripten_set_main_loop_arg(
                Self::main_loop,
                executor_ptr as *mut c_void,
                60,
                1, // 模拟无限循环
            );
        }
    }

    /// 主循环函数，由 Emscripten 调用
    extern "C" fn main_loop(arg: *mut c_void) {
        let executor = unsafe { &*(arg as *const EmscriptenExecutor) };
        
        // 设置当前执行器到线程局部存储
        CURRENT_EXECUTOR.with(|executor_cell| {
            *executor_cell.borrow_mut() = Some(executor as *const EmscriptenExecutor);
        });
        
        // 首先处理待添加的任务
        {
            let pending_tasks = std::mem::take(&mut *executor.pending_tasks.borrow_mut());
            if !pending_tasks.is_empty() {
                let mut inner = executor.inner.borrow_mut();
                for future in pending_tasks {
                    // 找到一个槽位：重用空闲槽位或添加新槽位
                    let index = if let Some(free_index) = inner.free_slots.pop() {
                        inner.tasks[free_index] = Some(Task {
                            future,
                            is_ready: Cell::new(true),
                        });
                        free_index
                    } else {
                        let index = inner.tasks.len();
                        inner.tasks.push(Some(Task {
                            future,
                            is_ready: Cell::new(true),
                        }));
                        index
                    };
                    
                    // 新任务立即加入就绪队列
                    inner.ready_queue.push_back(index);
                }
            }
        }
        
        // 取出所有就绪任务的索引，避免在轮询时持有借用
        let ready_tasks = {
            let mut inner = executor.inner.borrow_mut();
            if inner.ready_queue.is_empty() {
                return;
            }
            
            
            // 将所有就绪任务取出
            let mut tasks = Vec::new();
            while let Some(task_index) = inner.ready_queue.pop_front() {
                tasks.push(task_index);
            }
            tasks
        };

        // 处理每个就绪任务
        for task_index in ready_tasks {

            // 创建 Waker
            let waker = create_waker(task_index);
            let mut context = Context::from_waker(&waker);
            
            // 轮询任务（每次重新借用）
            let poll_result = {
                let mut inner = executor.inner.borrow_mut();
                if let Some(Some(task)) = inner.tasks.get_mut(task_index) {
                    // 重置就绪标记
                    task.is_ready.set(false);
                    Some(task.future.as_mut().poll(&mut context))
                } else {
                    None
                }
            };
            
            // 处理轮询结果
            if let Some(poll_result) = poll_result {
                match poll_result {
                    Poll::Ready(()) => {
                        // 任务完成，移除并标记槽位为空闲
                        let mut inner = executor.inner.borrow_mut();
                        inner.tasks[task_index] = None;
                        inner.free_slots.push(task_index);
                    }
                    Poll::Pending => {
                        // 任务未完成，等待下次唤醒
                    }
                }
            }
        }
    }
}

// 使用线程局部存储来避免 Send/Sync 要求
thread_local! {
    static CURRENT_EXECUTOR: RefCell<Option<*const EmscriptenExecutor>> = RefCell::new(None);
}

/// 任务 Waker 实现

struct TaskWaker {
    task_index: usize,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        // 从线程局部存储获取执行器
        CURRENT_EXECUTOR.with(|executor_cell| {
            if let Some(executor_ptr) = *executor_cell.borrow() {
                let executor = unsafe { &*executor_ptr };
                if let Ok(mut inner) = executor.inner.try_borrow_mut() {
                    // 检查任务是否存在且未就绪
                    if let Some(Some(task)) = inner.tasks.get(self.task_index) {
                        if !task.is_ready.get() {
                            // 标记任务为就绪并加入队列
                            task.is_ready.set(true);
                            inner.ready_queue.push_back(self.task_index);
                        }
                    }
                }
            }
        });
    }
}

/// 创建 Waker
fn create_waker(task_index: usize) -> Waker {
    Arc::new(TaskWaker { task_index }).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::ready;

    #[test]
    fn test_executor_creation() {
        let executor = EmscriptenExecutor::new();
        let inner = executor.inner.borrow();
        assert_eq!(inner.tasks.len(), 0);
        assert_eq!(inner.ready_queue.len(), 0);
        assert_eq!(inner.free_slots.len(), 0);
    }

    #[test]
    fn test_spawn() {
        let executor = EmscriptenExecutor::new();
        executor.spawn(ready(()));
        
        let inner = executor.inner.borrow();
        assert_eq!(inner.tasks.len(), 1);
        assert_eq!(inner.ready_queue.len(), 1);
        assert_eq!(inner.free_slots.len(), 0);
    }

    #[test]
    fn test_slot_reuse() {
        let executor = EmscriptenExecutor::new();
        
        // 生成两个任务
        executor.spawn(ready(()));
        executor.spawn(ready(()));
        
        {
            let mut inner = executor.inner.borrow_mut();
            // 模拟第一个任务完成
            inner.tasks[0] = None;
            inner.free_slots.push(0);
            inner.ready_queue.clear();
        }
        
        // 生成新任务，应该重用槽位 0
        executor.spawn(ready(()));
        
        let inner = executor.inner.borrow();
        assert!(inner.tasks[0].is_some());
        assert_eq!(inner.free_slots.len(), 0);
    }
}