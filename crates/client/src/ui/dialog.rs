use crate::npc::Npc;
use crate::sprite::Sprite;
use crate::ui::button;
use crate::widget::tiled;
use crate::WzBase;
use image::DynamicImage;
use std::sync::Arc;
use ::ui::peniko::Color;
use ::ui::reactive::{create_rw_signal, use_context, SignalGet, SignalUpdate};
use ::ui::event::Interactive;
use ::ui::style::dimension::percent;
use ::ui::style::Styleable;
use ::ui::{dynamic, fragment, text, view, Image, IntoElement};

pub fn dialog() -> impl IntoElement {
    let open = create_rw_signal(true);

    dynamic(move || {
        if !open.get() {
            return fragment(());
        }

        let WzBase { node: base } = use_context().ok_or(()).unwrap();
        let node = base.at_path("UI/UIWindow.img/UtilDlgEx").unwrap();
        let t: Arc<DynamicImage> = node.get("t").try_into().unwrap();
        let s: Arc<DynamicImage> = node.get("s").try_into().unwrap();

        let npc: Npc = base.at_path("Npc/9010000.img").unwrap().try_into().unwrap();

        let Sprite { image, origin, .. } = npc.actions["stand"].frames[0].clone();

        fragment(
            view()
                .style(|s| {
                    s.pointer_events_none()
                        .absolute()
                        .left(0)
                        .top(0)
                        .w_full()
                        .h_full()
                        .justify_center()
                        .items_center()
                })
                .children(
                    view()
                        .style(|s| s.pointer_events_auto().flex_col().items_stretch())
                        .children((
                            Image::new(t),
                            view().children((
                                tiled(node.get("c")).style(|s| s.absolute().w_full().h_full()),
                                view().children((
                                    view().children((view()
                                        .style(|s| s.margin_top(117).margin_left(20))
                                        .children((
                                            view()
                                                .style(|s| s.absolute().left(percent(0.5)).top(4))
                                                .children(Image::new(image).style(move |s| {
                                                    s.translate_x(-origin.x).translate_y(-origin.y)
                                                })),
                                            Image::new({
                                                let bar: Arc<DynamicImage> = node.get("bar").try_into().unwrap();
                                                bar
                                            }),
                                            view()
                                                .style(|s| {
                                                    s.absolute()
                                                        .left(0)
                                                        .top(0)
                                                        .w_full()
                                                        .h_full()
                                                        .color(Color::WHITE)
                                                        .justify_center()
                                                        .items_center()
                                                })
                                                .children(text(|| "Maple Administrator")),
                                        )),)),
                                    view()
                                        .style(|s| {
                                            s.margin_top(13)
                                                .margin_left(20)
                                                .margin_bottom(10)
                                                .flex_col()
                                        })
                                        .children((
                                            text(|| "Basic Configuration for MapleStory")
                                                .style(|s| s.margin_bottom(13)),
                                            Image::new({
                                                let notice: Arc<DynamicImage> = node.get("notice").try_into().unwrap();
                                                notice
                                            }),
                                        )),
                                )),
                            )),
                            Image::new(s),
                            button(node.get("BtClose"))
                                .style(|s| s.absolute().left(9).bottom(8))
                                .on_click(move |_| open.set(false)),
                            button(node.get("BtOK"))
                                .style(|s| s.absolute().right(8).bottom(8))
                                .on_click(move |_| open.set(false)),
                        )),
                ),
        )
    })
}