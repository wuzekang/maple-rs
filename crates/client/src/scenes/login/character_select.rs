use crate::scenes::login::{LoginContext, LoginStep};
use crate::sprite::SpriteAnimation;
use crate::ui::button;
use crate::{wz::WzSplitReaderExt, WzSplitReaderContext};
use ::ui::event::Interactive;
use ::ui::reactive::{create_rw_signal, use_context, SignalGet, SignalUpdate};
use ::ui::style::{StyleBuilder, Styleable};
use ::ui::{dynamic, fragment, lazy, view, Image, IntoElement};

pub fn select_character_view() -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();
  let ctx: LoginContext = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move { reader.get_node("UI/Login.img/CharSelect").await.ok() }
    },
    move |node_opt: Option<crate::wz::Node>| {
      let Some(node) = node_opt else {
        return fragment(());
      };
      fragment(
        (view().style(|s| s.w_full().h_full()).children((
          fragment(
            (0..3)
              .map(|i| {
                view()
                  .style(move |s| s.absolute().left(252 + i * 125).top(370))
                  .children((
                    Image::new(
                      SpriteAnimation::try_from(node.at_path("character/1").unwrap()).unwrap(),
                    )
                    .style(|s| s.absolute()),
                    Image::new(
                      SpriteAnimation::try_from(node.at_path("character/0").unwrap()).unwrap(),
                    )
                    .style(|s| s.absolute()),
                  ))
              })
              .collect::<Vec<_>>(),
          ),
          view()
            .style(|s| s.block().absolute().left(576).top(147))
            .children((
              button(node.get("BtSelect")).on_click(move |_| {
                ctx.start();
              }),
              button(node.get("BtNew"))
                .style(|s| s.margin_top(8))
                .on_click(move |_| ctx.scroll_to(LoginStep::SelectRace)),
              button(node.get("BtDelete")).style(|s| s.margin_top(14)),
            )),
        ))),
      )
    },
  )
}

pub fn select_race_view() -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();
  let ctx: LoginContext = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move { reader.get_node("UI/Login.img/RaceSelect").await.ok() }
    },
    move |node_opt: Option<crate::wz::Node>| {
      let Some(node) = node_opt else {
        return fragment(());
      };
      let selected = create_rw_signal("normal");

      let position = |race_type: &str| match race_type {
        "knight" => |s: StyleBuilder| s.absolute().left(104).top(59),
        "normal" => |s: StyleBuilder| s.absolute().left(293).top(59),
        "aran" => |s: StyleBuilder| s.absolute().left(511).top(60),
        _ => |s: StyleBuilder| s,
      };

      fragment(
        view().style(|s| s.w_full().h_full()).children((
          Image::new(SpriteAnimation::try_from(node.get("textGL")).unwrap())
            .style(|s| s.absolute().left(315 + 91).top(32 + 19)),
          button(node.at_path("knight/BtKnight").unwrap())
            .style(position("knight"))
            .on_click(move |_| selected.set("knight")),
          button(node.at_path("normal/BtNormal").unwrap())
            .style(position("normal"))
            .on_click(move |_| selected.set("normal")),
          button(node.at_path("aran/BtAran").unwrap())
            .style(position("aran"))
            .on_click(move |_| selected.set("aran")),
          dynamic({
            let node = node.clone();
            move || {
              let selected = selected.get();
              fragment((
                Image::new(
                  SpriteAnimation::try_from(
                    node.at_path(&format!("{selected}/OnAnimation")).unwrap(),
                  )
                  .unwrap(),
                )
                .style(position(selected)),
                Image::new(
                  SpriteAnimation::try_from(node.at_path(&format!("{selected}/text")).unwrap())
                    .unwrap(),
                )
                .style(|s| s.absolute().left(118).top(291)),
              ))
            }
          }),
          button(node.at_path("BtSelect").unwrap())
            .style(|s| s.absolute().left(530).top(402))
            .on_mouse_up(move |_| match selected.get() {
              "knight" => ctx.scroll_to(LoginStep::CreateKnight),
              "normal" => ctx.scroll_to(LoginStep::CreateAdventure),
              "aran" => ctx.scroll_to(LoginStep::CreateAran),
              _ => {}
            }),
        )),
      )
    },
  )
}
