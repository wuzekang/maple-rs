pub mod executor;

use std::future::Future;
use std::time::Duration;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use executor::EmscriptenExecutor;

/// 在 Emscripten 中运行异步任务直到完成
/// 注意：这会接管控制流，永不返回
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + 'static,
    F::Output: 'static,
{
    let executor = EmscriptenExecutor::new();
    
    // 将 future 转换为一个永不返回的任务
    executor.spawn(async move {
        let _output = future.await;
        // 在 Emscripten 中，我们不能返回结果
        // 因为 executor.run() 会接管控制流
        // 所有后续逻辑都应该在异步任务内完成
    });
    
    // 运行执行器，这会接管控制流
    executor.run();
    
    // 这行代码实际上永远不会执行
    unreachable!("EmscriptenExecutor::run() should never return")
}

/// 生成一个新的异步任务
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    // 这里我们假设有一个全局的执行器实例
    // 在实际使用中，应该通过某种方式管理执行器的生命周期
    log::warn!("spawn called - this requires a running EmscriptenExecutor");
}

/// 异步延时
pub async fn sleep(duration: Duration) {
    SleepFuture::new(duration).await
}

/// 让出当前任务的执行权
pub async fn yield_now() {
    YieldNow::new().await
}

// Sleep 实现
struct SleepFuture {
    start_time: Option<f64>,
    duration_ms: f64,
}

impl SleepFuture {
    fn new(duration: Duration) -> Self {
        Self {
            start_time: None,
            duration_ms: duration.as_millis() as f64,
        }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.start_time.is_none() {
            self.start_time = Some(current_time_ms());
        }

        let elapsed = current_time_ms() - self.start_time.unwrap();
        if elapsed >= self.duration_ms {
            Poll::Ready(())
        } else {
            // 设置一个定时器来唤醒任务
            let waker = cx.waker().clone();
            let remaining = (self.duration_ms - elapsed) as i32;
            set_timeout(move || {
                waker.wake();
            }, remaining.max(1));
            
            Poll::Pending
        }
    }
}

// YieldNow 实现
struct YieldNow {
    yielded: bool,
}

impl YieldNow {
    fn new() -> Self {
        Self { yielded: false }
    }
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            // 立即唤醒，但让其他任务先执行
            let waker = cx.waker().clone();
            set_timeout(move || {
                waker.wake();
            }, 0);
            Poll::Pending
        }
    }
}

// Emscripten JavaScript 辅助函数
#[cfg(target_os = "emscripten")]
fn current_time_ms() -> f64 {
    extern "C" {
        fn emscripten_get_now() -> f64;
    }
    unsafe { emscripten_get_now() }
}

#[cfg(target_os = "emscripten")]
fn set_timeout<F: FnOnce() + 'static>(callback: F, delay_ms: i32) {
    use std::os::raw::{c_void, c_int};
    
    extern "C" {
        fn emscripten_set_timeout(
            callback: extern "C" fn(*mut c_void),
            timeout: f64,
            user_data: *mut c_void,
        ) -> c_int;
    }
    
    extern "C" fn timeout_callback(user_data: *mut c_void) {
        unsafe {
            let callback: Box<Box<dyn FnOnce()>> = Box::from_raw(user_data as *mut Box<dyn FnOnce()>);
            callback();
        }
    }
    
    let boxed_callback = Box::new(Box::new(callback) as Box<dyn FnOnce()>);
    unsafe {
        emscripten_set_timeout(
            timeout_callback,
            delay_ms as f64,
            Box::into_raw(boxed_callback) as *mut c_void,
        );
    }
}

// 非 Emscripten 平台的模拟实现（用于测试）
#[cfg(not(target_os = "emscripten"))]
fn current_time_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64
}

#[cfg(not(target_os = "emscripten"))]
fn set_timeout<F: FnOnce() + 'static>(_callback: F, _delay_ms: i32) {
    // 在非 Emscripten 平台上，这个函数不应该被调用
    panic!("set_timeout is only available on Emscripten platform");
}