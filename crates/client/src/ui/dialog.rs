use crate::npc::Npc;
use crate::sprite::Sprite;
use crate::ui::async_image::AsyncImage;
use crate::ui::button;
use crate::widget::tiled;
use crate::wz::WzSplitReaderExt;
use crate::WzSplitReaderContext;
use ::ui::event::Interactive;
use ::ui::peniko::Color;
use ::ui::reactive::{create_rw_signal, use_context, SignalGet, SignalUpdate};
use ::ui::style::dimension::percent;
use ::ui::style::Styleable;
use ::ui::{dynamic, fragment, lazy, text, view, Image, IntoElement};

pub fn dialog() -> impl IntoElement {
  let open = create_rw_signal(true);
  let reader = use_context::<WzSplitReaderContext>().map(|ctx| ctx.reader.clone());

  dynamic(move || {
    if !open.get() {
      return fragment(());
    }

    fragment(lazy(
      {
        let reader = reader.clone();
        move || {
          let reader = reader.clone();
          async move {
            if let Some(reader) = reader {
              match futures::try_join!(
                reader.get_node("UI/UIWindow.img/UtilDlgEx"),
                reader.get_node("Npc/9010000.img")
              ) {
                Ok((node, npc_node)) => {
                  let npc: Result<Npc, _> = npc_node.try_into();
                  if let Ok(npc) = npc {
                    let sprite = npc.actions["stand"].frames[0].clone();
                    Some((node, sprite))
                  } else {
                    None
                  }
                }
                Err(_) => None,
              }
            } else {
              None
            }
          }
        }
      },
      move |data: Option<(crate::wz::Node, Sprite)>| {
        if let Some((node, Sprite { image, origin, .. })) = data {
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
                    AsyncImage::new("UI/UIWindow.img/UtilDlgEx/t"),
                    view().children((
                      tiled(node.get("c")).style(|s| s.absolute().w_full().h_full()),
                      view().children((
                        view().children((view()
                          .style(|s| s.margin_top(117).margin_left(20))
                          .children((
                            view()
                              .style(|s| s.absolute().left(percent(0.5)).top(4))
                              .children(
                                Image::new(image)
                                  .style(move |s| s.translate_x(-origin.x).translate_y(-origin.y)),
                              ),
                            AsyncImage::new("UI/UIWindow.img/UtilDlgEx/bar"),
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
                            AsyncImage::new("UI/UIWindow.img/UtilDlgEx/notice"),
                          )),
                      )),
                    )),
                    AsyncImage::new("UI/UIWindow.img/UtilDlgEx/s"),
                    button(node.get("BtClose"))
                      .style(|s| s.absolute().left(9).bottom(8))
                      .on_click(move |_| open.set(false)),
                    button(node.get("BtOK"))
                      .style(|s| s.absolute().right(8).bottom(8))
                      .on_click(move |_| open.set(false)),
                  )),
              ),
          )
        } else {
          fragment(text(|| "Loading..."))
        }
      },
    ))
  })
}
