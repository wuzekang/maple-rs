mod character_create;
mod character_select;
mod helpers;
mod title;
mod world_select;

use crate::map;
use crate::scene::MainScene;
use crate::ui::async_image::AsyncImage;
use crate::{wz::WzSplitReaderExt, WzSplitReaderContext};
use ::ui::animation::use_raf;
use ::ui::geometry::CubicBezier;
use ::ui::reactive::{
  create_effect, create_ref, create_rw_signal, provide_context, use_context, Ref, RwSignal,
  SignalGet, SignalTrack, SignalUpdate,
};
use ::ui::style::Styleable;
use ::ui::widget::focus_trap::focus_trap;
use ::ui::{dynamic, fragment, lazy, view, Fragment, IntoElement};
use glam::vec2;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub use character_create::create_normal;
pub use character_select::{select_character_view, select_race_view};
pub use title::title_view;
pub use world_select::{world_select_sidebar, world_select_view};

#[derive(Debug, Copy, Clone)]
pub enum LoginStep {
  Title,
  SelectWorld,
  SelectCharacter,
  SelectRace,
  CreateKnight,
  CreateAdventure,
  CreateAran,
}

impl From<LoginStep> for usize {
  fn from(val: LoginStep) -> Self {
    match val {
      LoginStep::Title => 0,
      LoginStep::SelectWorld => 1,
      LoginStep::SelectCharacter => 2,
      LoginStep::SelectRace => 3,
      LoginStep::CreateKnight => 4,
      LoginStep::CreateAdventure => 5,
      LoginStep::CreateAran => 6,
    }
  }
}

impl LoginStep {
  fn view(&self) -> Fragment {
    let ctx: LoginContext = use_context().unwrap();

    match self {
      LoginStep::Title => fragment(title_view(move || ctx.scroll_to(LoginStep::SelectWorld))),
      LoginStep::SelectWorld => fragment(world_select_view(move || {
        ctx.scroll_to(LoginStep::SelectCharacter)
      })),
      LoginStep::SelectCharacter => fragment(select_character_view()),
      LoginStep::SelectRace => fragment(select_race_view()),
      LoginStep::CreateKnight => fragment(create_normal()),
      LoginStep::CreateAdventure => fragment(create_normal()),
      LoginStep::CreateAran => fragment(create_normal()),
    }
  }
}

#[derive(Copy, Clone)]
pub struct LoginContext {
  pub step: RwSignal<usize>,
  pub on_start: Ref<Box<dyn Fn()>>,
}

impl LoginContext {
  pub fn scroll_to(&self, step: LoginStep) {
    self.step.set(step.into());
  }

  pub fn start(&self) {
    self.on_start.with(|f| f());
  }
}

pub fn login_scene(on_enter: impl Fn() + Clone + 'static) -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move {
        let ui_img = reader.get_node("UI/Login.img").await.ok()?;
        let map = map::Map::new(reader, "login".to_string()).await.ok()?;
        Some((ui_img, map))
      }
    },
    move |data_opt: Option<(crate::wz::Node, map::Map)>| {
      let Some((img, map)) = data_opt else {
        return fragment(());
      };

      let step: RwSignal<usize> = create_rw_signal(LoginStep::Title.into());

      let ctx = LoginContext {
        step,
        on_start: create_ref(Box::new(on_enter.clone())),
      };

      provide_context(ctx);

      let steps = [
        LoginStep::Title,
        LoginStep::SelectWorld,
        LoginStep::SelectCharacter,
        LoginStep::SelectRace,
        LoginStep::CreateKnight,
        LoginStep::CreateAdventure,
        LoginStep::CreateAran,
      ];

      let len = steps.len();
      let size = vec2(800.0, 600.0);
      let compute_scroll_top = move || (len - step.get_untracked() - 1) as f32 * size.y;
      let scroll_top = create_rw_signal(compute_scroll_top());

      create_effect(move |_| {
        step.track();
        let easing = CubicBezier::new(0.17, 0.0, 0.26, 1.09);
        let from = scroll_top.get_untracked();
        let to = compute_scroll_top();
        let duration = 600.0;
        let elapsed = Cell::new(0.0);
        use_raf(move |delta| {
          elapsed.set(elapsed.get() + delta);
          let x = elapsed.get().min(duration) / duration;
          let y = easing.solve(x as f64);
          let value = y as f32 * (to - from) + from;
          scroll_top.set(value)
        });
      });

      let scene = Rc::new(RefCell::new(MainScene::new(
        map,
        None,
      )));

      create_effect({
        let scene = scene.clone();
        move |_| {
          scene.borrow_mut().set_camera_position(
            vec2(28.0, -8.0) - size / 2.0 + vec2(0.0, scroll_top.get() - size.y * (len - 1) as f32),
          );
        }
      });

      fragment((
        scene,
        view()
          .style(|s| s.flex_col())
          .style(move |s| s.translate_y(-scroll_top.get()).translate_x(0.0))
          .children(
            steps
              .into_iter()
              .rev()
              .map(move |i| {
                let index: usize = i.into();
                focus_trap()
                  .active(move || step.get() == index)
                  .style(move |s| s.width(size.x).height(size.y))
                  .children(|| i.view())
              })
              .collect::<Vec<_>>(),
          ),
        view()
          .children(AsyncImage::new("UI/Login.img/Common/frame"))
          .style(move |s| {
            s.pointer_events_none()
              .absolute()
              .left(0)
              .top(0)
              .width(size.x)
              .height(size.y)
          }),
        dynamic(move || {
          if step.get() >= 1 {
            fragment(world_select_sidebar(step, move |_| {
              step.set(0);
            }))
          } else {
            fragment(())
          }
        }),
      ))
    },
  )
}
