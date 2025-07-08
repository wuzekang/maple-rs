use crate::ui::async_image::AsyncImage;
use crate::ui::button;
use crate::wz::WzSplitReaderExt;
use crate::WzSplitReaderContext;
use ::ui::peniko::Color;
use ::ui::reactive::use_context;
use ::ui::style::dimension::{length, percent};
use ::ui::style::Styleable;
use ::ui::taffy::{AlignItems, Display, FlexDirection, JustifyContent, Position};
use ::ui::view_tuple::ViewTuple;
use ::ui::{dynamic, fragment, lazy, text, view, IntoElement, View};

pub fn level_no<F>(_value: F) -> View
where
  F: Fn() -> i32 + 'static,
{
  view()
    .style(|s| {
      s.justify_content(JustifyContent::FlexStart)
        .align_items(AlignItems::FlexStart)
        .gap_row(1.0)
        .gap_column(1.0)
    })
    .children((
      AsyncImage::new("UI/Basic.img/LevelNo/1"),
      AsyncImage::new("UI/Basic.img/LevelNo/8"),
    ))
}

pub fn bracket_wrap(children: impl ViewTuple) -> View {
  view()
    .style(|s| {
      s.justify_content(JustifyContent::FlexStart)
        .align_items(AlignItems::Center)
        .gap_row(1.0)
        .gap_column(1.0)
    })
    .children(fragment((
      AsyncImage::new("UI/StatusBar.img/number/Lbracket"),
      fragment(children),
      AsyncImage::new("UI/StatusBar.img/number/Rbracket"),
    )))
}

pub fn status_bar_number(f: impl (Fn() -> String) + 'static) -> View {
  view()
    .style(|s| {
      s.justify_content(JustifyContent::FlexStart)
        .align_items(AlignItems::FlexStart)
    })
    .children(dynamic(move || {
      f()
        .chars()
        .filter_map(|ch: char| {
          if ch.is_ascii_digit() {
            Some(format!(
              "UI/StatusBar.img/number/{}",
              ch as usize - '0' as usize
            ))
          } else if ch == '/' {
            Some("UI/StatusBar.img/number/slash".to_string())
          } else if ch == '%' {
            Some("UI/StatusBar.img/number/percent".to_string())
          } else {
            None
          }
        })
        .map(AsyncImage::new)
        .collect::<Vec<_>>()
    }))
}

pub fn status_bar() -> impl IntoElement {
  let reader = use_context::<WzSplitReaderContext>().map(|ctx| ctx.reader.clone());

  lazy(
    move || {
      let reader = reader.clone();
      async move {
        if let Some(reader) = reader {
          match reader.get_node("UI/StatusBar.img").await {
            Ok(img) => Some(img),
            Err(_) => None,
          }
        } else {
          None
        }
      }
    },
    move |img: Option<crate::wz::Node>| {
      if let Some(img) = img {
        fragment(
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
              AsyncImage::new("UI/StatusBar.img/base/backgrnd"),
              view()
                .children(AsyncImage::new("UI/StatusBar.img/base/backgrnd2"))
                .style(|s| {
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
                      AsyncImage::new("UI/StatusBar.img/base/box"),
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
                            .children(AsyncImage::new("UI/StatusBar.img/base/iconBlue")),
                          view()
                            .style(|s| {
                              s.width(length(20.0))
                                .justify_content(JustifyContent::Center)
                                .align_items(AlignItems::Center)
                            })
                            .children(AsyncImage::new("UI/StatusBar.img/base/iconMemo")),
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
                          .children(level_no_async(|| 18)),
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
                            (text(|| "魔法师")
                              .style(|s| s.color(Color::WHITE).font_size(12.0).line_height(15.0))),
                            bracket_wrap(
                              text(|| "魔法师")
                                .style(|s| s.color(Color::WHITE).font_size(12.0).line_height(15.0)),
                            ),
                          )),
                        text(|| "三个榔头")
                          .style(|s| s.color(Color::WHITE).font_size(12.0).line_height(15.0)),
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
                  AsyncImage::new("UI/StatusBar.img/gauge/bar"),
                  view()
                    .children(AsyncImage::new("UI/StatusBar.img/gauge/graduation"))
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
            )),
        )
      } else {
        fragment(())
      }
    },
  )
}

pub fn level_no_async<F>(value: F) -> impl IntoElement
where
  F: Fn() -> i32 + 'static,
{
  view()
    .style(|s| {
      s.justify_content(JustifyContent::FlexStart)
        .align_items(AlignItems::FlexStart)
        .gap_row(1.0)
        .gap_column(1.0)
    })
    .children(dynamic(move || {
      let val = value();
      let digit1 = (val / 10) % 10;
      let digit2 = val % 10;

      fragment((
        AsyncImage::new(format!("UI/Basic.img/LevelNo/{}", digit1)),
        AsyncImage::new(format!("UI/Basic.img/LevelNo/{}", digit2)),
      ))
    }))
}
