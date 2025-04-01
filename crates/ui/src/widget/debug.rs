use crate::animation::use_raf;
use crate::mutation_observer::{MutationObserver, ObserveOptions};
use crate::root::AppContext;
use crate::runtime::RUNTIME;
use crate::style::{StylePropertyKey, Styleable};
use crate::widget::dynamic::each;
use crate::widget::scroll_view::scroll_view;
use crate::{
    dynamic, fragment, text, view, Element, Fragment, Interactive, Renderer, Text, View, ViewId,
};
use peniko::Color;
use reactive::{
    create_rw_signal, on_cleanup, provide_context, use_context, RwSignal, SignalGet, SignalTrack,
    SignalUpdate,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::SystemTime;
use strum::IntoEnumIterator;

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
                view().children(text(move || format!(" {}", draw_tiles_signal.get())).style(|s| s.text_nowrap())),
            )),
        ))
}

fn observe_mutation(id: ViewId) -> RwSignal<bool> {
    let mutation = create_rw_signal(false);
    let observer = MutationObserver::new(move || {
        let now = SystemTime::now();
        dbg!("mutation");
        mutation.set(true);
        dbg!(SystemTime::now().duration_since(now).unwrap().as_millis());
    });
    observer.observe(
        id,
        ObserveOptions {
            attributes: true,
            child_list: true,
            subtree: false,
        },
    );
    on_cleanup(move || {
        observer.disconnect();
    });

    mutation
}

pub fn node_view(id: ViewId, depth: usize) -> Fragment {
    let ElementContext { exclude, selected } = use_context().unwrap();
    let AppContext {
        inspect_element, ..
    } = use_context().unwrap();
    if id == exclude {
        return fragment(());
    }

    let mutation = observe_mutation(id);
    let open = create_rw_signal(false);
    let visible = create_rw_signal(false);
    let hover = create_rw_signal(false);
    let padding_left = 8 * depth + 16;
    fragment(
        view().style(|s| s.flex_col().w_full()).children((
            view()
                .style(move |s| {
                    hover.track();
                    s.padding_left(padding_left)
                        .w_full()
                        .height(20)
                        .items_center()
                        .background(if selected.get() == id {
                            Color::from_rgb8(0, 0, 255).with_alpha(0.2)
                        } else if hover.get() {
                            Color::BLACK.with_alpha(0.1)
                        } else {
                            Color::TRANSPARENT
                        })
                })
                .on_click(move |_| {
                    if !open.get() {
                        open.set(true);
                        visible.set(true);
                    } else {
                        visible.set(!visible.get());
                    }
                    selected.set(id)
                })
                .on_mouse_enter(move |_| {
                    inspect_element.set(Some(id));
                    hover.set(true);
                })
                .on_mouse_leave(move |_| {
                    hover.set(false);
                })
                .children(
                    text(move || id.element().borrow().name().to_string())
                        .style(|s| s.color(Color::BLACK)),
                ),
            dynamic(move || {
                if !open.get() {
                    fragment(())
                } else {
                    fragment(
                        view()
                            .style(move |s| {
                                let s = s.flex_col().w_full();
                                
                                if visible.get() { s.flex() } else { s.hidden() }
                            })
                            .children(fragment((
                                each(
                                    move || {
                                        mutation.track();
                                        id.children().iter().copied().collect::<Vec<_>>()
                                    },
                                    move |id| node_view(id, depth + 1),
                                ),
                                dynamic(move || {
                                    if !(hover.get() || selected.get() == id) {
                                        fragment(())
                                    } else {
                                        fragment(view().style(move |s| {
                                            s.absolute()
                                                .left(padding_left)
                                                .top(0)
                                                .width(1)
                                                .h_full()
                                                .background(Color::BLACK.with_alpha(0.2))
                                        }))
                                    }
                                }),
                            ))),
                    )
                }
            }),
        )),
    )
}

pub fn attribute_view(selected: RwSignal<ViewId>) -> View {
    fn label(text: &str) -> View {
        let text = text.to_string();
        view()
            .style(|s| {
                s.height(20)
                    .absolute()
                    .left(0)
                    .top(0)
                    .justify_center()
                    .items_center()
                    .padding([0, 8])
            })
            .children(crate::text(move || text.clone()))
    }

    fn value(
        taffy::Rect {
            left,
            right,
            top,
            bottom,
        }: taffy::Rect<f32>,
    ) -> Fragment {
        fn content(value: f32) -> Text {
            text(move || {
                if value == 0.0 {
                    " - ".to_string()
                } else {
                    format!(" {} ", value)
                }
            })
        }

        fragment((
            view()
                .style(|s| {
                    s.absolute()
                        .justify_center()
                        .items_center()
                        .left(0)
                        .top(0)
                        .w_full()
                        .height(20)
                })
                .children(content(left)),
            view()
                .style(|s| {
                    s.absolute()
                        .justify_center()
                        .items_center()
                        .right(0)
                        .top(0)
                        .h_full()
                        .width(20)
                })
                .children(content(top)),
            view()
                .style(|s| {
                    s.absolute()
                        .justify_center()
                        .items_center()
                        .left(0)
                        .bottom(0)
                        .w_full()
                        .height(20)
                })
                .children(content(right)),
            view()
                .style(|s| {
                    s.absolute()
                        .justify_center()
                        .items_center()
                        .left(0)
                        .top(0)
                        .h_full()
                        .width(20)
                })
                .children(content(bottom)),
        ))
    }

    view()
        .style(|s| {
            s.width(400)
                .h_full()
                .bg_white()
                .justify_stretch()
                .font_size(12.0)
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
        })
        .children(dynamic(move || {
            let id = selected.get();
            let taffy::Layout {
                margin,
                padding,
                border,
                
                size,
                ..
            } = id.layout();

            let style = create_rw_signal(id.state().borrow().style.clone());

            (
                view().style(|s| s).children(
                    // margin
                    view()
                        .style(|s| s.background(Color::from_rgb8(247, 203, 158)).padding(20))
                        .children((
                            label("margin"),
                            value(margin),
                            // border
                            view()
                                .style(|s| {
                                    s.background(Color::from_rgb8(254, 236, 188)).padding(20)
                                })
                                .children((
                                    label("border"),
                                    value(border),
                                    // padding
                                    view()
                                        .style(|s| {
                                            s.background(Color::from_rgb8(195, 222, 184))
                                                .padding(20)
                                        })
                                        .children((
                                            label("padding"),
                                            value(padding),
                                            view()
                                                .style(|s| {
                                                    s.background(Color::from_rgb8(159, 195, 229))
                                                        .width(200)
                                                        .height(20)
                                                        .justify_center()
                                                        .items_center()
                                                })
                                                .children(text(move || {
                                                    format!("{}x{}", size.width, size.height)
                                                })),
                                        )),
                                )),
                        )),
                ),
                scroll_view((view()
                    .style(|s| s.padding([0, 4]).flex_col().w_full())
                    .children(dynamic(move || {
                        fragment(
                            StylePropertyKey::iter()
                                .map(|key| {
                                    view()
                                        .style(|s| {
                                            s.w_full().line_height(20.0).height(20).font_size(12.0)
                                        })
                                        .children((view().children(text(move || {
                                            format!("{:?}", key.value(&style.get()))
                                        })),))
                                })
                                .collect::<Vec<_>>(),
                        )
                    })),))
                .style(|s| s.margin([4, 0]).w_full().h_full()),
            )
        }))
}

#[derive(Clone, Copy)]
struct ElementContext {
    exclude: ViewId,
    selected: RwSignal<ViewId>,
}

pub fn debug() -> View {
    let root: ViewId = use_context().unwrap();
    let mutation = observe_mutation(root);

    let element = view();
    let selected = create_rw_signal(root);
    provide_context(ElementContext {
        exclude: element.id(),
        selected,
    });

    element.children(fragment((
        dynamic(move || {
            if !mutation.get() {
                fragment(())
            } else {
                fragment(view().style(|s| s.w_full().h_full()).children((
                    scroll_view(node_view(root, 0)).style(|s| {
                        s.padding_left(8)
                            .w_full()
                            .h_full()
                            .background(Color::BLACK.with_alpha(0.1))
                            .color(Color::BLACK)
                    }),
                    attribute_view(selected),
                )))
            }
        }),
        monitor(),
    )))
}
