use glam::vec2;
use peniko::Color;
use sdl3_sys::everything::*;
use ui::{style::Styleable, view, DraggableBox, Element, Interactive, Root};

fn main() {
  async_runtime::block_on(async {
    unsafe {
      SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO);
    }
    let window = unsafe {
      SDL_CreateWindow(
        c"Maple RS".as_ptr(),
        800,
        600,
        // SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_BORDERLESS | SDL_WINDOW_MAXIMIZED | SDL_WINDOW_FULLSCREEN,
        SDL_WINDOW_HIGH_PIXEL_DENSITY,
      )
    };
    let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

    let app = || {
      view()
        .style(|s| {
          s.w_full()
            .h_full()
            .background(Color::from_rgb8(240, 240, 240))
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
        })
        .children((
          // 标题
          view()
            .style(|s| {
              s.font_size(24.0)
                .color(Color::from_rgb8(50, 50, 50))
                .margin_bottom(20.0)
            })
            .children("拖动测试"),
          // 第一个可拖动的盒子
          DraggableBox::new(
            vec2(100.0, 100.0),
            vec2(100.0, 100.0),
            Color::from_rgb8(255, 100, 100),
          )
          .into_view(),
          // 第二个可拖动的盒子
          DraggableBox::new(
            vec2(250.0, 150.0),
            vec2(120.0, 80.0),
            Color::from_rgb8(100, 255, 100),
          )
          .into_view(),
          // 第三个可拖动的盒子
          DraggableBox::new(
            vec2(400.0, 200.0),
            vec2(80.0, 120.0),
            Color::from_rgb8(100, 100, 255),
          )
          .into_view(),
          // 说明文字
          view()
            .style(|s| {
              s.font_size(16.0)
                .color(Color::from_rgb8(100, 100, 100))
                .margin_top(40.0)
                .text_center()
            })
            .children("点击并拖动上面的彩色方块来测试拖动功能"),
        ))
    };

    Root::new(app, renderer).launch().await;
  })
}
