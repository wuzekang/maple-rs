use std::future::Future;

/// 使用 tokio 运行时运行异步任务直到完成
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(future)
}

/// 生成一个新的异步任务
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}

/// 重导出 tokio 的异步原语
pub use tokio::time::sleep;
pub use tokio::task::yield_now;