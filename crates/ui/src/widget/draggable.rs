use crate::event::Interactive;
use crate::reactive::{create_rw_signal, SignalGet, SignalUpdate};
use crate::style::Styleable;
use crate::{view, Element, ViewId};
use glam::Vec2;
use peniko::Color;
use taffy::Point;

pub struct DraggableBox {
  initial_position: Vec2,
  size: Vec2,
  color: Color,
}

impl DraggableBox {
  pub fn new(position: Vec2, size: Vec2, color: Color) -> Self {
    Self {
      initial_position: position,
      size,
      color,
    }
  }
}

impl Element for DraggableBox {
  fn id(&self) -> ViewId {
    ViewId::new()
  }

  fn name(&self) -> String {
    "DraggableBox".to_string()
  }
}

impl Interactive for DraggableBox {}

impl DraggableBox {
    pub fn into_view(self) -> impl Element {
        let accumulated_offset = create_rw_signal(Vec2::ZERO);  // 累积的总偏移
        let current_drag_offset = create_rw_signal(Vec2::ZERO); // 当前拖拽的偏移
        let is_dragging = create_rw_signal(false);

        view()
            .style(move |s| {
                let total_offset = accumulated_offset.get() + current_drag_offset.get();
                s.absolute()
                    .left(self.initial_position.x)
                    .top(self.initial_position.y)
                    .width(self.size.x)
                    .height(self.size.y)
                    .translate_x(total_offset.x)
                    .translate_y(total_offset.y)
                    .background(if is_dragging.get() {
                        self.color.with_alpha(0.7)
                    } else {
                        self.color
                    })
                    .cursor(crate::style::Cursor::POINTER)
            })
            .on_drag_start({
                let is_dragging = is_dragging.clone();
                let current_drag_offset = current_drag_offset.clone();
                move |_event| {
                    is_dragging.set(true);
                    current_drag_offset.set(Vec2::ZERO); // 重置当前拖拽偏移
                    println!("Drag started");
                }
            })
            .on_drag({
                let current_drag_offset = current_drag_offset.clone();
                move |event| {
                    current_drag_offset.set(event.total_delta);
                    println!("Dragging, current delta: {:?}", event.total_delta);
                }
            })
            .on_drag_end({
                let is_dragging = is_dragging.clone();
                let accumulated_offset = accumulated_offset.clone();
                let current_drag_offset = current_drag_offset.clone();
                move |event| {
                    is_dragging.set(false);
                    // 将当前拖拽的偏移累积到总偏移中
                    let new_accumulated = accumulated_offset.get() + event.total_delta;
                    accumulated_offset.set(new_accumulated);
                    current_drag_offset.set(Vec2::ZERO); // 清空当前拖拽偏移
                    println!("Drag ended, accumulated offset: {:?}", new_accumulated);
                }
            })
            .children("Drag me!")
    }
}

// 便捷函数，用于创建可拖动的元素
pub fn use_draggable<F>(
    element: impl Element + Interactive,
    on_position_change: F,
) -> impl Element + Interactive
where
    F: Fn(Vec2) + 'static,
{
    let accumulated_offset = create_rw_signal(Vec2::ZERO);
    let current_drag_offset = create_rw_signal(Vec2::ZERO);

    element
        .on_drag_start({
            let current_drag_offset = current_drag_offset.clone();
            move |_event| {
                current_drag_offset.set(Vec2::ZERO); // 重置当前拖拽偏移
            }
        })
        .on_drag({
            let current_drag_offset = current_drag_offset.clone();
            let accumulated_offset = accumulated_offset.clone();
            move |event| {
                current_drag_offset.set(event.total_delta);
                let total_offset = accumulated_offset.get() + event.total_delta;
                on_position_change(total_offset);
            }
        })
        .on_drag_end({
            let accumulated_offset = accumulated_offset.clone();
            let current_drag_offset = current_drag_offset.clone();
            move |event| {
                // 将当前拖拽的偏移累积到总偏移中
                let new_accumulated = accumulated_offset.get() + event.total_delta;
                accumulated_offset.set(new_accumulated);
                current_drag_offset.set(Vec2::ZERO);
            }
        })
}
