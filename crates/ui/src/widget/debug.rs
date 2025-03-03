use crate::animation::use_raf;
use crate::runtime::RUNTIME;
use crate::style::Styleable;
use crate::{text, view, Text, View};
use peniko::Color;
use reactive::{create_rw_signal, SignalGet, SignalUpdate};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

fn fps_text() -> Text {
    let signal = create_rw_signal(0.0);
    let data = Rc::new(RefCell::new(VecDeque::new()));
    use_raf({
        let data = data.clone();
        move |delta| {
            data.borrow_mut().push_back(delta);
            if data.borrow().len() > 100 {
                data.borrow_mut().pop_front();
            }
            signal.set(data.borrow().iter().sum::<f32>() / data.borrow().len() as f32);
        }
    });
    text(move || format!("{:.0}", (1000.0 / signal.get()).round()))
}

fn node_count_text() -> Text {
    let content = create_rw_signal("".to_string());
    use_raf(move |_| {
        let node_count = RUNTIME
            .with_borrow(|r| r.taffy.borrow().total_node_count())
            .to_string();
        let value = format!("{}", node_count);
        if content != value {
            content.set(value);
        }
    });
    text(move || content.get())
}

pub fn debug() -> View {
    view((
        view(("Node: ", "FPS: ")).style(|s| s.flex_col().items_end()),
        view((node_count_text(), fps_text())).style(|s| s.flex_col().items_end()),
    ))
    .style(|s| {
        s.absolute()
            .right(0)
            .top(0)
            .padding_top(8)
            .padding_right(16)
            .padding_bottom(8)
            .padding_left(16)
            .background(Color::BLACK.multiply_alpha(0.5))
            .color(Color::WHITE)
            .font_size(14.0)
            .line_height(16.0)
            .flex_row()
            .justify_center()
            .items_end()
            .pointer_events_none()
    })
}
