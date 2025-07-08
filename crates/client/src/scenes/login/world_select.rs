use crate::scenes::login::helpers::ClipImage;
use crate::sprite::SpriteAnimation;
use crate::timer::Repeat;
use crate::ui::async_image::AsyncImage;
use crate::ui::button;
use crate::{wz::WzSplitReaderExt, WzSplitReaderContext};
use ::ui::animation::use_raf;
use ::ui::event::{Interactive, MouseEvent};
use ::ui::reactive::{
  create_ref, create_rw_signal, use_context, RwSignal, SignalGet, SignalUpdate,
};
use ::ui::style::Styleable;
use ::ui::{dynamic, fragment, lazy, use_resource, view, Drawable, Image, IntoElement};
use async_runtime::sleep;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub fn notice_loading(
  on_cancel: impl Fn() + Clone + 'static,
  on_connected: impl Fn() + Clone + 'static,
) -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move { reader.get_node("UI/Login.img").await.ok() }
    },
    move |img_opt: Option<crate::wz::Node>| {
      let Some(img) = img_opt else {
        return fragment(());
      };
      let on_connected = on_connected.clone();
      use_resource(|| sleep(Duration::from_secs(2)), move |_| on_connected());
      fragment(
        view()
          .style(|s| {
            s.absolute()
              .left(0)
              .top(0)
              .w_full()
              .h_full()
              .justify_center()
              .items_center()
          })
          .children(
            view().children((
              AsyncImage::new("UI/Login.img/Notice/Loading/backgrnd"),
              button(img.at_path("Notice/Loading/BtCancel").unwrap())
                .style(|s| s.absolute().right(28).top(44))
                .on_click({
                  let on_cancel = on_cancel.clone();
                  move |_| on_cancel()
                }),
              Image::new(
                SpriteAnimation::try_from(img.at_path("Notice/Loading/bar").unwrap()).unwrap(),
              )
              .style(|s| s.absolute().bottom(38).right(42)),
            )),
          ),
      )
    },
  )
}

pub fn channel_option(
  i: i32,
  selected: impl Fn() -> bool + Clone + 'static,
  on_select: impl Fn() + Clone + 'static,
) -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move { reader.get_node("UI/Login.img").await.ok() }
    },
    move |img_opt: Option<crate::wz::Node>| {
      let Some(img) = img_opt else {
        return fragment(());
      };
      let on_select = on_select.clone();
      let selected = selected.clone();
      let progress = Rc::new(RefCell::new(ClipImage::new(
        img.at_path("WorldSelect/channel/chgauge").unwrap(),
      )));

      progress.borrow_mut().clip.x = i as f32 * 2.0;

      fragment(
        view().on_click(move |_| on_select()).children((
          AsyncImage::new(format!("UI/Login.img/WorldSelect/channel/{i}/normal")),
          Image::new(progress as Rc<RefCell<dyn Drawable>>)
            .style(|s| s.pointer_events_none().absolute().left(43).top(20)),
          dynamic({
            let img = img.clone();
            move || {
              if selected() {
                return fragment(
                  Image::new(
                    SpriteAnimation::try_from(img.at_path("WorldSelect/channel/chSelect").unwrap())
                      .unwrap()
                      .with_repeat(Repeat::Finite(1)),
                  )
                  .style(|s| s.pointer_events_none().absolute().left(20).top(7)),
                );
              }
              fragment(())
            }
          }),
        )),
      )
    },
  )
}

pub fn world_select_view(on_enter: impl Fn() + Clone + 'static) -> impl IntoElement {
  let on_enter = create_ref(on_enter.clone());
  let WzSplitReaderContext { reader } = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move { reader.get_node("UI/Login.img").await.ok() }
    },
    move |img_opt: Option<crate::wz::Node>| {
      let Some(img) = img_opt else {
        return fragment(());
      };
      let selected_world = create_rw_signal(None);
      let scroll_state = create_rw_signal(None);
      let loading = create_rw_signal(false);

      let worlds = (0..20)
        .map({
          let img = img.clone();
          move |i| {
            button(img.at_path(&format!("WorldSelect/BtWorld/{}", i)).unwrap()).on_click(
              move |_| {
                if selected_world.get() != Some(i) {
                  selected_world.set(Some(i));
                  scroll_state.set(Some(0));
                }
              },
            )
          }
        })
        .collect::<Vec<_>>();

      fragment(
        view()
          .style(|s| s.w_full().h_full())
          .on_click(move |event| {
            if event.current.map(|v| v == event.target).unwrap_or_default()
              && selected_world.get_untracked().is_some()
            {
              selected_world.set(None);
              scroll_state.set(Some(1));
            }
          })
          .children((
            view().style(|s| s.absolute().left(158).top(79)).children((
              dynamic({
                let img = img.clone();
                move || {
                  if let Some(i) = scroll_state.get() {
                    fragment(Image::new(
                      SpriteAnimation::try_from(
                        img.at_path(&format!("WorldSelect/scroll/{i}")).unwrap(),
                      )
                      .unwrap()
                      .with_repeat(Repeat::Finite(1)),
                    ))
                  } else {
                    fragment(())
                  }
                }
              }),
              dynamic({
                let img = img.clone();
                move || {
                  if let Some(world) = selected_world.get() {
                    let delay = 350.0;
                    let duration = 500.0;
                    let elapsed = Rc::new(RefCell::new(0.0));
                    let alpha = create_rw_signal(0.0);
                    use_raf(move |delta| {
                      *elapsed.borrow_mut() += delta;
                      let value = (*elapsed.borrow() - delay).max(0.0).min(duration) / duration;
                      if value != alpha.get_untracked() {
                        alpha.set(value);
                      }
                    });

                    let selected_channel = create_rw_signal(None);

                    fragment((view()
                      .style(|s| s.absolute().left(38).top(133).width(449).height(268))
                      .composite()
                      .style(move |s| s.opacity(alpha.get()))
                      .children((
                        view()
                          .children(AsyncImage::new(format!(
                            "UI/Login.img/WorldSelect/world/{world}"
                          )))
                          .style(|s| s.absolute().left(33).top(8)),
                        view()
                          .style(|s| {
                            s.absolute()
                              .left(35)
                              .top(65)
                              .width(374)
                              .height(158)
                              .flex_wrap()
                              .gap_row(2)
                              .gap_column(2)
                          })
                          .children(
                            (0..20)
                              .map(move |i| {
                                channel_option(
                                  i,
                                  move || {
                                    selected_channel
                                      .get()
                                      .map(|value| value == i)
                                      .unwrap_or(false)
                                  },
                                  move || {
                                    let current = selected_channel.get_untracked();
                                    selected_channel.set(Some(i));
                                    if current == Some(i) {
                                      loading.set(true)
                                    }
                                  },
                                )
                              })
                              .collect::<Vec<_>>(),
                          ),
                        button(img.at_path("WorldSelect/BtGoworld").unwrap())
                          .style(|s| s.absolute().right(22).bottom(2))
                          .on_click(move |_| {
                            loading.set(true);
                          }),
                      )),))
                  } else {
                    fragment(())
                  }
                }
              }),
            )),
            view()
              .style(|s| s.absolute().left(151).top(104).gap_row(1))
              .children(worlds),
            dynamic(move || {
              if loading.get() {
                fragment(notice_loading(
                  move || {
                    loading.set(false);
                  },
                  move || {
                    loading.set(false);
                    on_enter.with(|f| f());
                  },
                ))
              } else {
                fragment(())
              }
            }),
          )),
      )
    },
  )
}

pub fn world_select_sidebar(
  _step: RwSignal<usize>,
  on_back: impl (Fn(&MouseEvent)) + Clone + 'static,
) -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move { reader.get_node("UI/Login.img").await.ok() }
    },
    move |img_opt: Option<crate::wz::Node>| {
      let Some(img) = img_opt else {
        return fragment(());
      };
      let on_back = on_back.clone();
      fragment(
        view()
          .style(|s| s.absolute().left(0).top(0).width(0).height(0))
          .children((
            view()
              .children(AsyncImage::new("UI/Login.img/Common/step/1"))
              .style(|s| s.absolute().left(0).top(33)),
            button(img.at_path("WorldSelect/BtViewChoice").unwrap())
              .style(|s| s.absolute().left(0).top(125)),
            button(img.at_path("ViewAllChar/BtVAC").unwrap())
              .style(|s| s.absolute().left(0).top(375)),
            button(img.at_path("Common/BtStart").unwrap())
              .style(|s| s.absolute().left(0).top(425))
              .on_click(on_back),
          )),
      )
    },
  )
}
