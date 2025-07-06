//! 跨平台异步运行时抽象
//! 
//! 这个 crate 提供了统一的异步运行时 API，在不同平台上使用不同的实现：
//! - Emscripten: 使用自定义的基于事件循环的执行器
//! - 其他平台: 使用 tokio

// Emscripten 平台
#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
mod emscripten;

#[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
pub use emscripten::*;

// 非 Emscripten 平台
#[cfg(not(all(target_arch = "wasm32", target_os = "emscripten")))]
mod tokio_runtime;

#[cfg(not(all(target_arch = "wasm32", target_os = "emscripten")))]
pub use tokio_runtime::*;

// 通用 trait 和类型
pub use std::future::Future;
pub use std::time::Duration;