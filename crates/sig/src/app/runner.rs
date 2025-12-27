use glam::Vec2;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::ElementState;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::{
  DragEvent, DragEventType, MouseEvent, MouseEventType, ViewId, WheelEvent,
  event::types::{DragData, MouseData, WheelData},
};

use super::{AppConfig, AppEvent};
use crate::event::dispatch::{dispatch_event, dispatch_event_to_view, update_cursor};
use crate::event::drag_state::DragState;
use crate::event::hit_test::hit_test;
use crate::render::VelloApp;

pub struct Runner {
  config: AppConfig,
  vello_app: VelloApp,
  root_view: Option<ViewId>,
  builder: Option<Box<dyn FnOnce() -> ViewId>>,
  window: Option<Arc<Window>>, // Keep a reference to the window
  cursor_pos: Option<Vec2>,
  hovered: HashSet<ViewId>,       // Track hovered views
  drag_state: DragState,          // Drag state
  hover_top_view: Option<ViewId>, // 🎯 Track top-most hovered view for AABB debug

  // Future for waiting on task messages
  wait_for_work: Option<Pin<Box<dyn Future<Output = ()> + 'static>>>,
  waker: std::task::Waker,
}

impl Runner {
  pub fn new<F>(
    config: AppConfig,
    proxy: winit::event_loop::EventLoopProxy<super::AppEvent>,
    builder: F,
  ) -> Self
  where
    F: FnOnce() -> ViewId + 'static,
  {
    // 🎯 Create waker in constructor
    let waker = {
      use futures_util::task::ArcWake;
      use std::sync::Arc;

      #[derive(Clone)]
      struct WakerImpl {
        proxy: winit::event_loop::EventLoopProxy<super::AppEvent>,
      }

      unsafe impl Send for WakerImpl {}
      unsafe impl Sync for WakerImpl {}

      impl ArcWake for WakerImpl {
        fn wake_by_ref(arc_self: &Arc<Self>) {
          let _ = arc_self.proxy.send_event(super::AppEvent::PollTasks);
        }
      }

      futures_util::task::waker(Arc::new(WakerImpl { proxy }))
    };

    Self {
      config,
      vello_app: VelloApp::new(),
      root_view: None,
      builder: Some(Box::new(builder)),
      window: None,
      cursor_pos: None,
      hovered: HashSet::new(),
      drag_state: DragState::default(),
      hover_top_view: None, // 🎯 Initialize hover_top_view
      wait_for_work: None,
      waker,
    }
  }

  fn build_view(&mut self) {
    if let Some(builder) = self.builder.take() {
      // Build view in the global scope
      // sig-reactive automatically creates a ROOT scope on initialization
      let view = builder();

      self.root_view = Some(view);
      // Note: We don't store root_scope_id anymore as we use the global ROOT scope
    }
  }

  /// Poll the wait_for_work future to check for async task messages
  /// Returns true if there's work to do
  fn poll_wait_for_work(&mut self) -> bool {
    let mut cx = Context::from_waker(&self.waker);

    // Create the future if it doesn't exist
    if self.wait_for_work.is_none() {
      self.wait_for_work = Some(Box::pin(async {
        loop {
          // 🎯 Wait for next message (registers waker with channel)
          if let Some(task_id) = crate::reactive::wait_for_task_message().await {
            // Task has been scheduled, break to process it
            break;
          }
        }
      }));
    }

    // Poll the future
    if let Some(fut) = self.wait_for_work.as_mut() {
      match fut.as_mut().poll(&mut cx) {
        Poll::Ready(_) => {
          // Reset the future for next time
          self.wait_for_work = None;
          true
        }
        Poll::Pending => false,
      }
    } else {
      false
    }
  }

  fn handle_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: winit::window::WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Focused(_focused) => {}
      WindowEvent::Resized(size) => {
        if size.width != 0 && size.height != 0 {
          self.vello_app.resize(size.width, size.height);
        }
      }
      // 🎯 Handle DPR changes (window moving between different displays)
      WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        self.vello_app.update_scale_factor(scale_factor);
        if let Some(window) = &self.window {
          window.request_redraw();
        }
      }
      WindowEvent::CursorMoved { position, .. } => {
        // 🎯 Convert physical pixels to logical pixels
        let scale_factor = self.vello_app.get_scale_factor().unwrap_or(1.0);
        let pos = Vec2::new(
          (position.x / scale_factor) as f32,
          (position.y / scale_factor) as f32,
        );
        self.cursor_pos = Some(pos);

        if let Some(view) = &self.root_view {
          // 🎯 Handle drag logic
          if let Some(drag_target_id) = self.drag_state.target {
            let distance = (pos - self.drag_state.start_position).length();

            if !self.drag_state.is_dragging && distance > self.drag_state.threshold {
              // Drag start: exceeded threshold
              self.drag_state.is_dragging = true;

              if !self.drag_state.is_scrollbar_drag {
                // Normal drag event
                let mut event = DragEvent::new(
                  drag_target_id,
                  DragData {
                    r#type: DragEventType::DragStart,
                    start_position: self.drag_state.start_position,
                    current_position: pos,
                    delta: Vec2::ZERO,
                    total_delta: pos - self.drag_state.start_position,
                  },
                );
                dispatch_event_to_view(drag_target_id, &mut event);
              }

              self.drag_state.last_position = pos;
            } else if self.drag_state.is_dragging {
              // Drag in progress
              let delta = pos - self.drag_state.last_position;
              let total_delta = pos - self.drag_state.start_position;

              // 🎯 Handle scrollbar dragging
              if self.drag_state.is_scrollbar_drag {
                let handled = crate::event::handler::scroll::handle_scrollbar_drag(
                  drag_target_id,
                  delta,
                  self.drag_state.scrollbar_direction_vertical,
                );
                if handled {
                  if let Some(window) = &self.window {
                    window.request_redraw();
                  }
                }
              } else {
                // 普通拖拽事件
                let mut event = DragEvent::new(
                  drag_target_id,
                  DragData {
                    r#type: DragEventType::Drag,
                    start_position: self.drag_state.start_position,
                    current_position: pos,
                    delta,
                    total_delta,
                  },
                );
                dispatch_event_to_view(drag_target_id, &mut event);
              }

              self.drag_state.last_position = pos;
            }
          }

          // Perform hit test to find target
          if let Some(path) = hit_test(&view, pos, Vec2::ZERO) {
            // Build new hovered set from path (target and all ancestors)
            let new_hovered: HashSet<ViewId> = path.iter().copied().collect();

            // Find entered and leaved views
            let entered = &new_hovered - &self.hovered;
            let leaved = &self.hovered - &new_hovered;

            // Trigger MouseEnter for newly hovered views
            for view_id in entered.iter() {
              let mut event = MouseEvent::new(
                *view_id,
                MouseData {
                  position: pos,
                  button: None,
                  state: ElementState::Pressed,
                  r#type: MouseEventType::MouseEnter,
                },
              );
              dispatch_event_to_view(*view_id, &mut event);
            }

            // Trigger MouseLeave for no longer hovered views
            for view_id in leaved.iter() {
              let mut event = MouseEvent::new(
                *view_id,
                MouseData {
                  position: pos,
                  button: None,
                  state: ElementState::Pressed,
                  r#type: MouseEventType::MouseLeave,
                },
              );
              dispatch_event_to_view(*view_id, &mut event);
            }

            // Update hovered set
            self.hovered = new_hovered;

            // Update cursor based on the topmost hovered view (first in path)
            if let Some(target_view) = path.first() {
              // 🎯 Update hover_top_view for AABB debug drawing
              self.hover_top_view = Some(*target_view);

              if let Some(window) = &self.window {
                update_cursor(window, *target_view);
              }

              let mut event = MouseEvent::new(
                *target_view,
                MouseData {
                  position: pos,
                  button: None,
                  state: ElementState::Pressed,
                  r#type: MouseEventType::MouseMove,
                },
              );
              dispatch_event(&view, &mut event);
            }
          }
        }
      }
      WindowEvent::MouseInput { state, button, .. } => {
        if let Some(pos) = self.cursor_pos {
          if let Some(view) = &self.root_view {
            let type_ = match state {
              ElementState::Pressed => MouseEventType::MouseDown,
              ElementState::Released => MouseEventType::MouseUp, // Todo: Handle Click
            };

            // 🎯 MouseDown: Record potential drag start point
            if type_ == MouseEventType::MouseDown {
              if let Some(path) = hit_test(&view, pos, Vec2::ZERO) {
                if let Some(target_view) = path.first() {
                  self.drag_state.start_position = pos;
                  self.drag_state.last_position = pos;
                  self.drag_state.target = Some(*target_view);
                  self.drag_state.is_dragging = false;

                  // 🎯 Handle focus change (find first focusable view in path)
                  let old_focus = crate::runtime::with_window(|state| state.focused_view);

                  // Find first focusable view in hit test path
                  let focusable_view = path
                    .iter()
                    .find(|view_id| {
                      crate::runtime::with_layout(|layout| {
                        layout
                          .view_states
                          .get(view_id)
                          .map(|state| state.focusable)
                          .unwrap_or(false)
                      })
                    })
                    .copied();

                  // Update focus state based on whether we found a focusable view
                  if old_focus != focusable_view {
                    // Trigger blur event on old focused view
                    if let Some(old_focused_id) = old_focus {
                      use crate::event::{FocusData, FocusEvent, FocusEventType};
                      let mut blur_event = FocusEvent::new(
                        old_focused_id,
                        FocusData {
                          r#type: FocusEventType::Blur,
                          related_target: focusable_view,
                        },
                      );
                      dispatch_event_to_view(old_focused_id, &mut blur_event);

                      // Request style update for old focused view
                      old_focused_id.request_style();
                    }

                    // Update focus state (could be Some or None)
                    crate::runtime::with_window_mut(|state| state.focused_view = focusable_view);

                    // If we have a new focus, trigger focus event
                    if let Some(new_focus) = focusable_view {
                      new_focus.request_style();

                      use crate::event::{FocusData, FocusEvent, FocusEventType};
                      let mut focus_event = FocusEvent::new(
                        new_focus,
                        FocusData {
                          r#type: FocusEventType::Focus,
                          related_target: old_focus,
                        },
                      );
                      dispatch_event_to_view(new_focus, &mut focus_event);
                    }

                    // 🎯 Request redraw after focus change
                    if let Some(window) = &self.window {
                      window.request_redraw();
                    }
                  }

                  // 🎯 Check if clicking on any scrollbar (traverse entire path)
                  let mut scrollbar_info = None;
                  let mut scrollbar_view = None;
                  for view_id in path.iter() {
                    if let Some(is_vertical) =
                      crate::event::handler::scroll::check_scrollbar(*view_id, pos)
                    {
                      scrollbar_info = Some(is_vertical);
                      scrollbar_view = Some(*view_id);
                      break;
                    }
                  }

                  if let Some(is_vertical) = scrollbar_info {
                    self.drag_state.is_scrollbar_drag = true;
                    self.drag_state.scrollbar_direction_vertical = is_vertical;
                    self.drag_state.target = scrollbar_view; // Use scroll container as target
                  } else {
                    self.drag_state.is_scrollbar_drag = false;
                  }
                }
              }
            }

            let mut event = MouseEvent::new(
              *view,
              MouseData {
                position: pos,
                button: Some(button),
                state,
                r#type: type_,
              },
            );
            dispatch_event(&view, &mut event);

            // 🎯 MouseUp: End drag (if dragging) or trigger click
            if type_ == MouseEventType::MouseUp {
              let was_dragging = self.drag_state.is_dragging;

              if was_dragging {
                // If dragging, send drag end event
                if let Some(drag_target_id) = self.drag_state.target {
                  let total_delta = pos - self.drag_state.start_position;
                  let delta = pos - self.drag_state.last_position;

                  let mut event = DragEvent::new(
                    drag_target_id,
                    DragData {
                      r#type: DragEventType::DragEnd,
                      start_position: self.drag_state.start_position,
                      current_position: pos,
                      delta,
                      total_delta,
                    },
                  );
                  dispatch_event_to_view(drag_target_id, &mut event);
                }
              } else {
                // If not dragging, trigger click event
                let mut click_event = MouseEvent::new(
                  *view,
                  MouseData {
                    position: pos,
                    button: Some(button),
                    state: ElementState::Pressed,
                    r#type: MouseEventType::Click,
                  },
                );
                dispatch_event(&view, &mut click_event);
              }

              // Reset drag state
              self.drag_state = DragState::default();
            }
          }
        }
      }
      WindowEvent::MouseWheel { delta, .. } => {
        if let Some(pos) = self.cursor_pos {
          if let Some(view) = &self.root_view {
            // Convert wheel delta to pixel values
            let (delta_x, delta_y) = match delta {
              winit::event::MouseScrollDelta::LineDelta(x, y) => {
                // One line is approximately 20 pixels
                (x * 20.0, y * 20.0)
              }
              winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
            };

            let mut event = WheelEvent::new(
              *view,
              WheelData {
                position: pos,
                delta_x,
                delta_y,
              },
            );
            dispatch_event(&view, &mut event);
          }
        }
      }
      WindowEvent::Ime(ime_event) => {
        // Dispatch IME event to focused view
        if let Some(focused_view) = crate::runtime::with_window(|state| state.focused_view) {
          use crate::event::{ImeData, ImeEvent};
          let mut event = ImeEvent::new(
            focused_view,
            ImeData {
              ime: ime_event.clone(),
            },
          );
          dispatch_event_to_view(focused_view, &mut event);

          if let Some(window) = &self.window {
            window.request_redraw();
          }
        }
      }
      WindowEvent::KeyboardInput { event, .. } => {
        // Dispatch keyboard event to focused view
        if let Some(focused_view) = crate::runtime::with_window(|state| state.focused_view) {
          use crate::event::{KeyboardData, KeyboardEvent};
          let mut kb_event = KeyboardEvent::new(
            focused_view,
            KeyboardData {
              logical_key: event.logical_key.clone(),
              state: event.state,
            },
          );
          dispatch_event_to_view(focused_view, &mut kb_event);

          if let Some(window) = &self.window {
            window.request_redraw();
          }
        }
      }
      WindowEvent::RedrawRequested => {
        // 🎯 延迟初始化：在第一次渲染前初始化 GPU
        // 这样可以避免在 resumed() 中阻塞主线程，确保窗口焦点正常获取
        if !self.vello_app.has_state() {
          if let Some(window) = &self.window {
            self.vello_app.resume(window.clone());
          }
        }

        if let Some(view) = &self.root_view {
          // ✅ sig-reactive automatically flushes effects when signals are written
          // Tasks are polled in user_event handler only

          self.vello_app.render(view, self.hover_top_view);
        }
      }
      _ => {}
    }
  }
}

impl ApplicationHandler<AppEvent> for Runner {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    // Create window if not exists
    if self.window.is_none() {
      let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
        .with_resizable(true)
        .with_title(&self.config.title);
      let window = Arc::new(event_loop.create_window(attr).unwrap());

      // 🎯 默认禁用 IME，只在需要时启用（避免影响焦点）
      window.set_ime_allowed(false);

      self.window = Some(window.clone());

      // 设置全局重绘请求器
      crate::runtime::with_window_mut(|state| {
        state.redraw_requester = Some(window.clone());
      });

      // 🎯 延迟 GPU 初始化到第一次渲染，避免阻塞焦点请求
      // GPU 初始化（pollster::block_on）可能耗时 50-200ms，会干扰窗口焦点授予
      // 移动到 RedrawRequested 事件中处理
      
      // ✅ 在窗口完全初始化后请求焦点
      window.focus_window();
    } else {
      // Re-initialize vello if needed (app resume from suspended state)
      if let Some(window) = &self.window {
        if !self.vello_app.has_state() {
          self.vello_app.resume(window.clone());
        }
      }
    }

    // Build view if not ready
    if self.root_view.is_none() {
      self.build_view();

      if let Some(window) = &self.window {
        window.request_redraw();
      }
    }
  }

  fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
    self.vello_app.suspend();
  }

  /// Handle custom user events
  /// This is where we poll tasks independently of rendering
  fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
    match event {
      AppEvent::PollTasks => {
        // 🎯 Dioxus-style: Continuously poll until pending
        loop {
          // Poll the wait_for_work future
          let has_async_work = self.poll_wait_for_work();

          // Also poll tasks directly (for immediate/synchronous scheduling)
          crate::reactive::poll_tasks();

          // Check if there's more work
          let has_more_work = crate::reactive::with_task_runtime(|rt| rt.has_dirty_tasks());

          if !has_async_work && !has_more_work {
            break; // No more work, return to event loop
          }
        }

        // Request a redraw if tasks modified the UI
        if let Some(window) = &self.window {
          window.request_redraw();
        }
      }
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: winit::window::WindowId,
    event: WindowEvent,
  ) {
    if let Some(window) = &self.window {
      if window.id() != window_id {
        return;
      }

      self.handle_event(event_loop, window_id, event);
    }
  }
}
