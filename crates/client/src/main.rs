use glam::{vec2, Vec2Swizzles};
use sdl3_sys::{
    init::{SDL_Init, SDL_INIT_VIDEO},
    render::SDL_CreateRenderer,
    video::SDL_CreateWindow,
};
use std::error::Error;
use ui::widget::image::IntoDrawable;
use ui::{
    reactive::{provide_context, SignalGet, SignalUpdate},
    taffy::prelude::*,
    Drawable, Element, IntoElement, Root,
};
use wz::Node;

mod character;
mod map;
mod math;
mod npc;
mod scene;
mod sprite;
mod timer;
mod ui_view;
mod wz;

#[derive(Clone)]
struct WzBase {
    pub node: Node,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        SDL_Init(SDL_INIT_VIDEO);
    }

    let size = vec2(800.0, 600.0);
    let window =
        unsafe { SDL_CreateWindow(c"Maple RS".as_ptr(), size.x as i32, size.y as i32, 0x2000) };
    let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

    provide_context(WzBase {
        node: wz::resolve_base().unwrap(),
    });

    Root::new(ui_view::ui_view, renderer).launch();

    Ok(())
}
