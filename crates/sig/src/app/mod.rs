pub mod events;
mod runner;
pub mod runtime;

pub use events::*;
pub use runner::*;

// Re-export runtime functions
pub use runtime::{
  WindowState, clear_focus, get_focused, is_focused, set_focus, with_window, with_window_mut,
};

use crate::{Element, View};
use std::sync::Arc;
use winit::event_loop::EventLoop;

/// Event loop waker implementation for sig's EventLoop
struct SigEventLoopWaker {
  proxy: winit::event_loop::EventLoopProxy<AppEvent>,
}

impl crate::reactive::EventLoopWaker for SigEventLoopWaker {
  fn wake(&self) {
    let _ = self.proxy.send_event(AppEvent::PollTasks);
  }
}

/// Configuration for the application window
pub struct AppConfig {
  pub title: String,
  pub width: f64,
  pub height: f64,
}

impl Default for AppConfig {
  fn default() -> Self {
    Self {
      title: "Sig App".to_string(),
      width: 800.0,
      height: 600.0,
    }
  }
}

/// Run an application with the given root view builder
///
/// This function automatically manages a tokio runtime for async operations.
/// If a tokio runtime already exists, it will use that. Otherwise, it creates
/// a new multi-threaded runtime.
///
/// # Example
/// ```no_run
/// use sig::{AppConfig, run, view};
///
/// fn main() -> anyhow::Result<()> {
///     let config = AppConfig::default();
///     run(config, || view())
/// }
/// ```
pub fn run<F>(config: AppConfig, root_builder: F) -> anyhow::Result<()>
where
  F: FnOnce() -> View + 'static,
{
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();
  let _guard = rt.enter();

  run_blocking(config, root_builder)
}

/// Internal blocking run function (called within tokio runtime context)
fn run_blocking<F>(config: AppConfig, root_builder: F) -> anyhow::Result<()>
where
  F: FnOnce() -> View + 'static,
{
  let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;

  // 🎯 Create event loop proxy
  let proxy = event_loop.create_proxy();

  // Set the event waker for TaskRuntime (for local scheduling like spawn, resume)
  let event_waker = Box::new(SigEventLoopWaker {
    proxy: proxy.clone(),
  });
  crate::reactive::set_event_waker(event_waker);

  // Create Runner with proxy (waker will be created in Runner::new)
  let mut app = Runner::new(config, proxy, move || root_builder().id());

  event_loop.run_app(&mut app)?;
  Ok(())
}
