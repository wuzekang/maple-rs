use crate::animation::use_raf;
use crate::mutation_observer::MutationObserver;
use crate::runtime::RUNTIME;
use crate::style::Styleable;
use crate::widget::scroll_view::scroll_view;
use crate::{dynamic, fragment, text, view, Element, Fragment, Renderer, Text, View, ViewId};
use peniko::Color;
use reactive::{
    create_rw_signal, on_cleanup, provide_context, use_context, SignalGet, SignalRead, SignalUpdate,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::SystemTime;

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
            let fps = 1000.0 / (data.borrow().iter().sum::<f32>() / data.borrow().len() as f32);
            signal.set(fps);
        }
    });
    text(move || format!("{:.0}", signal.get().round())).style(|s| s.text_nowrap())
}

fn node_count_text() -> Text {
    let content = create_rw_signal("".to_string());
    use_raf(move |_| {
        let value = RUNTIME
            .with_borrow(|r| r.taffy.borrow().total_node_count())
            .to_string();
        if content.get_untracked() != value {
            content.set(value);
        }
    });
    text(move || content.get()).style(|s| s.text_nowrap())
}

pub fn monitor() -> View {
    let draw_calls_signal = create_rw_signal(0);
    let draw_tiles_signal = create_rw_signal(0);
    let ctx: Rc<RefCell<Renderer>> = use_context().unwrap();
    use_raf(move |_| {
        let draw_tiles = ctx.borrow().draw_tiles.1;
        let draw_calls = ctx.borrow().draw_calls.1;
        draw_tiles_signal.set(draw_tiles);
        draw_calls_signal.set(draw_calls);
    });
    view()
        .style(|s| {
            s.absolute()
                .right(0)
                .top(0)
                .padding_top(8)
                .padding_right(16)
                .padding_bottom(8)
                .padding_left(16)
                .background(Color::BLACK.with_alpha(0.5))
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
                .children(("FPS: ", "DC: ", "Node: ", "Tile: ")),
            view().style(|s| s.flex_col().items_end()).children((
                view().children(fps_text()),
                view().children(text(move || draw_calls_signal.get()).style(|s| s.text_nowrap())),
                view().children(node_count_text()),
                view().children(text(move || draw_tiles_signal.get()).style(|s| s.text_nowrap())),
            )),
        ))
}

pub fn node_view(id: ViewId) -> Fragment {
    // return fragment(());
    let Exclude(exclude) = use_context().unwrap();
    if id == exclude {
        return fragment(());
    }
    fragment(
        view().style(|s| s.flex_col()).children((
            view()
                .children(
                    (text(move || format!("{}", id.element().borrow().name()))
                        .style(|s| s.color(Color::BLACK))),
                )
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
        let now = SystemTime::now();
        mutation.set(true);
        dbg!(SystemTime::now().duration_since(now).unwrap().as_millis());
    });
    observer.observe(root);
    on_cleanup(move || {
        observer.disconnect();
    });
    let element = view();

    provide_context(Exclude(element.id()));

    element.children((
        scroll_view(view().children(dynamic(move || {
            if mutation.get() {
                node_view(root)
            } else {
                fragment(())
            }
        })))
        .style(|s| {
            s.padding_left(8)
                .w_full()
                .h_full()
                .background(Color::BLACK.with_alpha(0.1))
                .color(Color::BLACK)
        }),
        monitor(),
    ))
}
