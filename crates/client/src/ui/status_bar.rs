use crate::ui::button;
use crate::WzBase;
use glam::vec2;
use image::DynamicImage;
use std::sync::Arc;
use ::ui::peniko::Color;
use ::ui::reactive::{use_context, SignalGet};
use ::ui::style::dimension::{length, percent};
use ::ui::style::Styleable;
use ::ui::taffy::{AlignItems, Display, FlexDirection, JustifyContent, Position};
use ::ui::view_tuple::ViewTuple;
use ::ui::{dynamic, fragment, text, view, Image, IntoElement, View};

pub fn level_no<F>(_value: F) -> View
where
    F: Fn() -> i32 + 'static,
{
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Basic.img/LevelNo").unwrap();
    let images: Vec<Arc<DynamicImage>> = (0..9)
        .map(|i| node.get(&i.to_string()).try_into().unwrap())
        .collect();
    view()
        .style(|s| {
            s.justify_content(JustifyContent::FlexStart)
                .align_items(AlignItems::FlexStart)
                .gap_row(1.0)
                .gap_column(1.0)
        })
        .children((Image::new(images[1].clone()), Image::new(images[8].clone())))
}

pub fn bracket_wrap(children: impl ViewTuple) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/StatusBar.img/number").unwrap();
    let left: Arc<DynamicImage> = node.get("Lbracket").try_into().unwrap();
    let right: Arc<DynamicImage> = node.get("Rbracket").try_into().unwrap();
    view()
        .style(|s| {
            s.justify_content(JustifyContent::FlexStart)
                .align_items(AlignItems::Center)
                .gap_row(1.0)
                .gap_column(1.0)
        })
        .children(fragment((
            Image::new(left),
            fragment(children),
            Image::new(right),
        )))
}

pub fn status_bar_number(f: impl (Fn() -> String) + 'static) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/StatusBar.img/number").unwrap();
    let images: Vec<Arc<DynamicImage>> = (0..10)
        .map(|i| node.get(&i.to_string()).try_into().unwrap())
        .collect();
    let slash: Arc<DynamicImage> = node.get("slash").try_into().unwrap();
    let percent: Arc<DynamicImage> = node.get("percent").try_into().unwrap();

    view()
        .style(|s| {
            s.justify_content(JustifyContent::FlexStart)
                .align_items(AlignItems::FlexStart)
        })
        .children(dynamic(move || {
            f().chars()
                .filter_map(|ch: char| {
                    if ch.is_ascii_digit() {
                        Some(images[ch as usize - '0' as usize].clone())
                    } else if ch == '/' {
                        Some(slash.clone())
                    } else if ch == '%' {
                        Some(percent.clone())
                    } else {
                        None
                    }
                })
                .map(Image::new)
                .collect::<Vec<_>>()
        }))
}

pub fn status_bar() -> View {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/StatusBar.img").unwrap();
    let background: Arc<DynamicImage> = img
        .at_path("base")
        .unwrap()
        .get("backgrnd")
        .try_into()
        .unwrap();
    let background2: Arc<DynamicImage> = img
        .at_path("base")
        .unwrap()
        .get("backgrnd2")
        .try_into()
        .unwrap();

    let gauge = img.at_path("gauge").unwrap();
    let graduation: Arc<DynamicImage> = gauge.get("graduation").try_into().unwrap();
    let bar: Arc<DynamicImage> = gauge.get("bar").try_into().unwrap();

    let base_box: Arc<DynamicImage> = img.at_path("base/box").unwrap().try_into().unwrap();

    let icon_memo: Arc<DynamicImage> = img.at_path("base/iconMemo").unwrap().try_into().unwrap();
    let icon_blue: Arc<DynamicImage> = img.at_path("base/iconBlue").unwrap().try_into().unwrap();

    view()
        .style(|s| {
            s.position(Position::Absolute)
                .display(Display::Block)
                .left(length(0.0))
                .right(length(0.0))
                .width(percent(1.0))
                .bottom(length(0.0))
        })
        .children((
            Image::new(background),
            Image::new(background2).style(|s| {
                s.position(Position::Absolute)
                    .left(length(4.0))
                    .bottom(length(0.0))
            }),
            crate::ui::chat::chat_box(),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .justify_content(JustifyContent::FlexEnd)
                        .align_items(AlignItems::FlexStart)
                        .top(length(8.0))
                        .right(length(4.0))
                })
                .children((
                    view()
                        .style(|s| {
                            s.margin_right(length(3.0))
                                .justify_content(JustifyContent::FlexStart)
                                .align_items(AlignItems::FlexStart)
                        })
                        .children((
                            Image::new(base_box),
                            view()
                                .style(|s| {
                                    s.position(Position::Absolute)
                                        .width(percent(1.0))
                                        .height(percent(1.0))
                                        .justify_content(JustifyContent::SpaceBetween)
                                        .align_items(AlignItems::Stretch)
                                })
                                .children((
                                    view()
                                        .style(|s| {
                                            s.width(length(20.0))
                                                .justify_content(JustifyContent::Center)
                                                .align_items(AlignItems::Center)
                                        })
                                        .children(Image::new(icon_blue)),
                                    view()
                                        .style(|s| {
                                            s.width(length(20.0))
                                                .justify_content(JustifyContent::Center)
                                                .align_items(AlignItems::Center)
                                        })
                                        .children(Image::new(icon_memo)),
                                )),
                        )),
                    view()
                        .style(|s| {
                            s.justify_content(JustifyContent::FlexStart)
                                .align_items(AlignItems::FlexStart)
                                .gap_row(2.0)
                        })
                        .children((
                            button(img.get("EquipKey")),
                            button(img.get("InvenKey")),
                            button(img.get("StatKey")),
                            button(img.get("SkillKey")),
                            button(img.get("KeySet")),
                            button(img.get("QuickSlot")),
                        )),
                )),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .left(length(2.0))
                        .right(length(4.0))
                        .bottom(length(1.0))
                        .height(length(34.0))
                        .justify_content(JustifyContent::FlexStart)
                        .align_items(AlignItems::Center)
                })
                .children((view()
                    .style(|s| {
                        s.width(length(208.0))
                            .justify_content(JustifyContent::FlexStart)
                            .align_items(AlignItems::Center)
                    })
                    .children((
                        // level card
                        view()
                            .style(|s| {
                                s.margin_left(length(3.0))
                                    .width(length(74.0))
                                    .height(length(30.0))
                            })
                            .children(
                                // level no
                                view()
                                    .style(|s| {
                                        s.position(Position::Absolute)
                                            .left(length(27.0))
                                            .bottom(length(8.0))
                                            .width(length(47.0))
                                            .height(length(11.0))
                                            .justify_content(JustifyContent::Center)
                                            .align_items(AlignItems::Center)
                                    })
                                    .children(level_no(|| 18)),
                            ),
                        // job name
                        view()
                            .style(|s| {
                                s.margin_left(length(8.0))
                                    .flex_grow(1.0)
                                    .height(length(30.0))
                                    .flex_direction(FlexDirection::Column)
                            })
                            .children((
                                view()
                                    .style(|s| {
                                        s.justify_content(JustifyContent::FlexStart)
                                            .align_items(AlignItems::FlexStart)
                                            .gap_column(length(2.0))
                                    })
                                    .children((
                                        (text(|| "魔法师").style(|s| {
                                            s.color(Color::WHITE).font_size(12.0).line_height(15.0)
                                        })),
                                        bracket_wrap(text(|| "魔法师").style(|s| {
                                            s.color(Color::WHITE).font_size(12.0).line_height(15.0)
                                        })),
                                    )),
                                text(|| "三个榔头").style(|s| {
                                    s.color(Color::WHITE).font_size(12.0).line_height(15.0)
                                }),
                            )),
                    )),)),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .display(Display::Block)
                        .left(length(218.0))
                        .bottom(length(1.0))
                })
                .children((
                    Image::new(bar),
                    Image::new(graduation)
                        .style(|s| s.position(Position::Absolute).bottom(length(0.0))),
                    view()
                        .style(|s| {
                            s.position(Position::Absolute)
                                .display(Display::Block)
                                .top(length(3.0))
                                .left(length(19.0))
                        })
                        .children(bracket_wrap(status_bar_number(|| "124/274".to_string()))),
                    view()
                        .style(|s| {
                            s.position(Position::Absolute)
                                .display(Display::Block)
                                .top(length(3.0))
                                .left(length(131.0))
                        })
                        .children(bracket_wrap(status_bar_number(|| "118/617".to_string()))),
                    view()
                        .style(|s| {
                            s.position(Position::Absolute)
                                .display(Display::Block)
                                .top(length(3.0))
                                .left(length(248.0))
                        })
                        .children(bracket_wrap(status_bar_number(|| "6839/13716".to_string()))),
                )),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .justify_content(JustifyContent::SpaceBetween)
                        .right(length(4.0))
                        .bottom(length(1.0))
                        .width(length(224.0))
                        .height(length(34.0))
                })
                .children((
                    button(img.get("BtShop")),
                    button(img.get("BtNPT")),
                    button(img.get("BtMenu")),
                    button(img.get("BtShort")),
                )),
        ))
}