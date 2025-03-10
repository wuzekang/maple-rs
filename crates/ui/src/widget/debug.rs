use crate::animation::use_raf;
use crate::mutation_observer::MutationObserver;
use crate::runtime::RUNTIME;
use crate::style::Styleable;
use crate::widget::scroll_view::scroll_view;
use crate::{dynamic, fragment, text, view, Element, Fragment, Interactive, Text, View, ViewId};
use peniko::Color;
use reactive::{
    create_rw_signal, on_cleanup, provide_context, use_context, SignalGet, SignalTrack,
    SignalUpdate,
};
use std::any::TypeId;
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

pub fn monitor() -> View {
    view()
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
        .children((
            view()
                .style(|s| s.flex_col().items_end())
                .children(("Node: ", "FPS: ")),
            view()
                .style(|s| s.flex_col().items_end())
                .children((node_count_text(), fps_text())),
        ))
}

pub fn node_view(id: ViewId) -> Fragment {
    let Exclude(exclude) = use_context().unwrap();
    if id == exclude {
        return fragment(());
    }
    fragment(
        view().style(|s| s.flex_col()).children((
            view()
                .children((text(move || format!("{}", id.element().borrow().name()))))
                .style(|s| s.height(20).items_center()),
            view().style(|s| s.margin_left(8).flex_col()).children(
                id.children()
                    .iter()
                    .map(|id| node_view(*id))
                    .collect::<Vec<_>>(),
            ),
        )),
    )
}

#[derive(Clone, Copy)]
struct Exclude(ViewId);
pub fn debug() -> View {
    let root: ViewId = use_context().unwrap();
    let mutation = create_rw_signal(false);
    let observer = MutationObserver::new(move || {
        mutation.set(true);
    });
    observer.observe(root);
    on_cleanup(move || {
        observer.disconnect();
    });
    let element = view();

    provide_context(Exclude(element.id()));

    element.children((
        scroll_view(view().composite().children(dynamic(move || {
            if mutation.get() {
                node_view(root)
            } else {
                fragment(())
            }
        })))
        .style(|s| {
            s.background(Color::WHITE)
                .padding_left(8)
                .w_full()
                .h_full()
                .overflow_y_hidden()
        }),
        monitor(),
    ))
}
