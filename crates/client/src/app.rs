use crate::scene::MainScene;
use crate::scenes::login::login_scene;
use crate::scenes::logo_scene;
use crate::ui::{dialog, status_bar, world_map_window};
use crate::WzBase;
use ::ui::event::{use_event, Interactive};
use ::ui::reactive::{use_context, RwSignal, SignalGet, SignalUpdate};
use ::ui::style::Styleable;
use ::ui::Element;
use ::ui::{dynamic, fragment, view, widget::debug::debug, IntoElement};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
enum Stage {
  Pause,
  Logo,
  Login,
  Main,
}

pub fn app() -> impl IntoElement {
  let stage = RwSignal::new(Stage::Main);
  let open = RwSignal::new(false);
  // 000010000
  // 910000000
  let current_map: RwSignal<(String, Option<String>)> =
    RwSignal::new(("000010000".to_string(), None));

  fragment((
    view()
      .style(move |s| {
        s.cursor(crate::cursor::CursorState::Idle)
          .absolute()
          .overflow_clip()
          .width(800)
          .height(600)
          .bg_black()
      })
      .children(dynamic(move || match stage.get() {
        Stage::Pause => fragment(
          view()
            .style(|s| s.w_full().h_full().bg_white())
            .on_click(move |_| stage.set(Stage::Logo)),
        ),
        Stage::Logo => fragment(logo_scene(move || stage.set(Stage::Login))),
        Stage::Login => fragment(login_scene(move || stage.set(Stage::Main))),
        Stage::Main => fragment((
          map_scene(current_map),
          status_bar(),
          world_map_window(open, current_map),
          dialog(),
        )),
      })),
    // debug().style(|s| s.absolute().top(0).right(0).width(800).height(600)),
  ))
}

pub fn map_scene(map_name: RwSignal<(String, Option<String>)>) -> impl IntoElement {
  let (_map_name, spawn) = map_name.get();
  let WzBase { node: base } = use_context().unwrap();
  let (player, map) = MainScene::resource(&_map_name, spawn, base).unwrap();
  let main_scene = Rc::new(RefCell::new(MainScene::new(map, Some(map_name))));
  main_scene.borrow_mut().set_player(player);
  use_event({
    let main_scene = main_scene.clone();
    move |event| {
      main_scene.borrow_mut().event(&mut *event);
    }
  });
  fragment(main_scene)
}
