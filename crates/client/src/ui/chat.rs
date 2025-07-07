use crate::ui::button;
use crate::WzBase;
use image::DynamicImage;
use std::sync::Arc;
use ::ui::event::{use_key, Interactive};
use ::ui::peniko::Color;
use ::ui::reactive::{create_rw_signal, use_context, SignalGet, SignalUpdate};
use ::ui::style::dimension::{length, percent};
use ::ui::style::Styleable;
use ::ui::taffy::{AlignItems, JustifyContent, Position};
use ::ui::{dynamic, fragment, text, view, Image, IntoElement, TextInput, View};

pub fn chat_box() -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let basic = base.at_path("UI/Basic.img").unwrap();
    let open = create_rw_signal(false);

    dynamic(move || {
        if !open.get() {
            use_key(13, move || {
                open.set(true);
            });

            fragment(
                view()
                    .style(|s| {
                        s.position(Position::Absolute)
                            .justify_content(JustifyContent::SpaceBetween)
                            .align_items(AlignItems::Center)
                            .top(length(6.0))
                            .left(length(0.0))
                            .padding_left(length(8.0))
                            .width(length(568.0))
                            .height(length(24.0))
                            .background([0x88, 0x88, 0x88])
                    })
                    .children((
                        (text(|| "欢迎来到冒险岛，现在开始你的旅程吧～！".to_string())
                            .style(|s| s.font_size(11.0).line_height(11.0).color(Color::WHITE))),
                        view()
                            .style(|s| s.align_items(AlignItems::Center))
                            .children((
                                view().style(|s| s.margin_right(length(3.0))).children(
                                    button(basic.get("BtMax")).on_click(move |_| open.set(true)),
                                ),
                                scroll_vertical(),
                            )),
                    )),
            )
        } else {
            // let mut input_ref = None;

            fragment(
                view()
                    .style(|s| {
                        s.position(Position::Absolute)
                            .align_items(AlignItems::Center)
                            .top(length(6.0))
                            .left(length(0.0))
                            .width(length(568.0))
                            .height(length(24.0))
                    })
                    .children((
                        TextInput::new()
                            // ._ref(&mut input_ref)
                            .style(|s| {
                                s.position(Position::Absolute)
                                    .left(length(4.0))
                                    .width(length(563.0))
                                    .height(length(22.0))
                            })
                            // .on_attach(move || {
                            //     input_ref.as_ref().unwrap().with(|input| {
                            //         input.focus();
                            //     });
                            // })
                            .on_key_down(move |event| {
                                if event.key == 13 {
                                    open.set(false);
                                }
                                event.propagation = false;
                            })
                            .on_key_up(move |event| event.propagation = false),
                        view()
                            .style(|s| {
                                s.position(Position::Absolute)
                                    .align_items(AlignItems::Center)
                                    .height(percent(1.0))
                                    .right(length(18.0))
                            })
                            .children(
                                button(basic.get("BtMin")).on_click(move |_| open.set(false)),
                            ),
                    )),
            )
        }
    })
}

pub fn scroll_vertical() -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Basic.img/VScr5/enabled").unwrap();
    let prev: Arc<DynamicImage> = node.get("prev0").try_into().unwrap();
    let next: Arc<DynamicImage> = node.get("next0").try_into().unwrap();
    view()
        .style(|s| s.width(length(15.0)).height(length(25.0)))
        .children((
            Image::new(prev).style(|s| s.position(Position::Absolute).top(length(0.0))),
            Image::new(next).style(|s| s.position(Position::Absolute).bottom(length(0.0))),
        ))
}