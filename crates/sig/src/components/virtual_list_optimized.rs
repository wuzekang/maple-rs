//! VirtualList Optimized - Performance optimized virtual scrolling list
//!
//! This is an optimized version that addresses scroll stuttering issues
//! by reducing style dirty bubbling overhead.
//!
//! Key optimizations:
//! 1. Throttle scroll updates to ~60fps
//! 2. Reuse spacer views instead of recreating them
//! 3. Direct taffy style updates for spacers

use crate::{
  Element, Interactive, IntoElement, Signal, Styleable, View, ViewId, ViewTuple, create_effect,
  each, view,
};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

/// Optimized Virtual list component
#[derive(Clone)]
pub struct VirtualListOptimized<T>
where
  T: Clone + PartialEq + 'static,
{
  view: View,
  data: Signal<Vec<T>>,
  height: Option<f32>,
  item_height: f32,
  buffer_size: usize,
  disabled: bool,
  throttle_ms: u64,
}

impl<T> VirtualListOptimized<T>
where
  T: Clone + PartialEq + 'static,
{
  /// Create optimized virtual list
  pub fn new(data: Signal<Vec<T>>) -> Self {
    let view = view().name("VirtualListOptimized");

    Self {
      view,
      data,
      height: None,
      item_height: 32.0,
      buffer_size: 10,
      disabled: false,
      throttle_ms: 16, // ~60fps
    }
  }

  /// Set list height
  pub fn height(mut self, height: f32) -> Self {
    self.height = Some(height);
    self
  }

  /// Set item height
  pub fn item_height(mut self, height: f32) -> Self {
    self.item_height = height;
    self
  }

  /// Set buffer size
  pub fn buffer_size(mut self, size: usize) -> Self {
    self.buffer_size = size;
    self
  }

  /// Set scroll throttle interval in milliseconds
  pub fn throttle_ms(mut self, ms: u64) -> Self {
    self.throttle_ms = ms;
    self
  }

  /// Build virtual list view
  pub fn build<VT, F>(self, view_fn: F) -> Self
  where
    VT: ViewTuple,
    F: Fn(&T, usize) -> VT + Clone + 'static,
  {
    let data = self.data.clone();
    let item_height = self.item_height;
    let buffer_size = self.buffer_size;
    let disabled = self.disabled;
    let height = self.height;
    let throttle_ms = self.throttle_ms;

    // Scroll state
    let scroll_offset = Signal::new(0.0);
    let viewport_height = Signal::new(height.unwrap_or(600.0));

    // Container style
    self.view.style(move |s| {
      let mut style = s.w_full().overflow_y_scroll();
      if let Some(h) = height {
        style = style.height(h);
      } else {
        style = style.flex_grow(1.0);
      }
      style
    });

    // Indexed data item for each
    #[derive(Clone, PartialEq)]
    struct IndexedItem<T: Clone + PartialEq> {
      item: T,
      index: usize,
    }

    // 🔥 Optimization 1: Create reusable spacer views (not wrapped in dynamic)
    let top_spacer = view().name("TopSpacer");
    let bottom_spacer = view().name("BottomSpacer");

    // 🔥 Optimization 2: Update spacer heights via effects (direct taffy updates)
    // This avoids recreating views and reduces style dirty bubbling
    {
      let top_spacer_id = top_spacer.id();
      create_effect(move || {
        if disabled {
          return;
        }

        let _items = data.read();
        let offset = *scroll_offset.read();

        let start = (offset / item_height).floor() as usize;
        let start_with_buffer = start.saturating_sub(buffer_size);
        let h = start_with_buffer as f32 * item_height;

        // Direct taffy style update (avoids style dirty bubbling)
        crate::runtime::with_layout_mut(|runtime| {
          if let Ok(mut style) = runtime.taffy.style(top_spacer_id.node_id()).cloned() {
            style.size.height = if h > 0.0 {
              taffy::Dimension::Length(h)
            } else {
              taffy::Dimension::Length(0.0)
            };
            let _ = runtime.taffy.set_style(top_spacer_id.node_id(), style);
            let _ = runtime.taffy.mark_dirty(top_spacer_id.node_id());
          }
        });
      });
    }

    {
      let bottom_spacer_id = bottom_spacer.id();
      create_effect(move || {
        if disabled {
          return;
        }

        let items = data.read();
        let total = items.len();
        let offset = *scroll_offset.read();
        let height = *viewport_height.read();

        let end = ((offset + height) / item_height).ceil() as usize + 1;
        let end_with_buffer = (end + buffer_size).min(total);
        let h = (total.saturating_sub(end_with_buffer)) as f32 * item_height;

        // Direct taffy style update (avoids style dirty bubbling)
        crate::runtime::with_layout_mut(|runtime| {
          if let Ok(mut style) = runtime.taffy.style(bottom_spacer_id.node_id()).cloned() {
            style.size.height = if h > 0.0 {
              taffy::Dimension::Length(h)
            } else {
              taffy::Dimension::Length(0.0)
            };
            let _ = runtime.taffy.set_style(bottom_spacer_id.node_id(), style);
            let _ = runtime.taffy.mark_dirty(bottom_spacer_id.node_id());
          }
        });
      });
    }

    // Content container with fixed spacers
    let content = view().style(|s| s.flex_col().w_full()).child((
      top_spacer.style(|s| s.w_full()),
      // Visible items list - uses each for smart diffing
      if disabled {
        // Render all items when disabled
        let view_fn_clone = view_fn.clone();
        each(
          move || {
            data
              .read()
              .iter()
              .enumerate()
              .map(|(idx, item)| IndexedItem {
                item: item.clone(),
                index: idx,
              })
              .collect::<Vec<_>>()
          },
          move |indexed| view_fn_clone(&indexed.item, indexed.index),
        )
      } else {
        // Virtual scrolling enabled, render only visible items
        let view_fn_clone = view_fn.clone();
        each(
          move || {
            let items = data.read();
            let total = items.len();
            let offset = *scroll_offset.read();
            let height = *viewport_height.read();

            // Calculate visible range
            let start = (offset / item_height).floor() as usize;
            let end = ((offset + height) / item_height).ceil() as usize + 1;
            let start_with_buffer = start.saturating_sub(buffer_size);
            let end_with_buffer = (end + buffer_size).min(total);

            // Return visible items (with actual index)
            items[start_with_buffer..end_with_buffer]
              .iter()
              .enumerate()
              .map(|(relative_idx, item)| {
                let actual_idx = start_with_buffer + relative_idx;
                IndexedItem {
                  item: item.clone(),
                  index: actual_idx,
                }
              })
              .collect::<Vec<_>>()
          },
          move |indexed| view_fn_clone(&indexed.item, indexed.index),
        )
      },
      bottom_spacer.style(|s| s.w_full()),
    ));

    // 🔥 Optimization 3: Throttle scroll events
    let last_update = Rc::new(Cell::new(Instant::now()));

    self.view.child(content).on_scroll(move |event| {
      if !disabled {
        let now = Instant::now();
        let elapsed = now.duration_since(last_update.get()).as_millis() as u64;

        // Only update if enough time has passed
        if elapsed >= throttle_ms {
          *scroll_offset.write() = event.scroll_top;
          *viewport_height.write() = event.client_height;
          last_update.set(now);
        }
      }
    });

    self
  }
}

/// Implement Element trait
impl<T> Element for VirtualListOptimized<T>
where
  T: Clone + PartialEq + 'static,
{
  fn id(&self) -> ViewId {
    self.view.id
  }

  fn name(&self) -> String {
    "VirtualListOptimized".to_string()
  }
}

/// Convenience function
pub fn virtual_list_optimized<T>(data: Signal<Vec<T>>) -> VirtualListOptimized<T>
where
  T: Clone + PartialEq + 'static,
{
  VirtualListOptimized::new(data)
}

impl<T> IntoElement for VirtualListOptimized<T>
where
  T: Clone + PartialEq + 'static,
{
  fn into_element(self) -> crate::Node {
    crate::Node::View(self.id())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Signal;

  #[test]
  fn test_optimized_virtual_list_properties() {
    let data = Signal::new(vec![1, 2, 3]);
    let list = VirtualListOptimized::new(data)
      .height(500.0)
      .item_height(40.0)
      .buffer_size(5)
      .throttle_ms(20);

    assert_eq!(list.height, Some(500.0));
    assert_eq!(list.item_height, 40.0);
    assert_eq!(list.buffer_size, 5);
    assert_eq!(list.throttle_ms, 20);
    assert!(!list.disabled);
  }

  #[test]
  fn test_optimized_virtual_list_defaults() {
    let data = Signal::new(vec!["a", "b", "c"]);
    let list = VirtualListOptimized::new(data);

    assert_eq!(list.height, None);
    assert_eq!(list.item_height, 32.0);
    assert_eq!(list.buffer_size, 10);
    assert_eq!(list.throttle_ms, 16);
    assert!(!list.disabled);
  }
}
