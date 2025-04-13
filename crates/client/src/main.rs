use glam::vec2;
use sdl3_sys::everything::*;
use std::error::Error;
use ui::{reactive::provide_context, Root};
use wz::Node;

mod app;
mod character;
mod cursor;
mod login;
mod map;
mod mob;
mod npc;
mod scene;
mod sound;
mod sprite;
mod timer;
mod wz;

#[derive(Clone)]
struct WzBase {
    pub node: Node,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO);
    }

    // let size = vec2(1920.0, 1080.0);
    // let size = vec2(1366.0, 768.0);
    // let size = unsafe {
    //     let mut num = MaybeUninit::<i32>::uninit();
    //     let displays = SDL_GetDisplays(num.as_mut_ptr());
    //     let mode = SDL_GetCurrentDisplayMode(*displays.offset(0));
    //     let w = (*mode).w;
    //     let h = (*mode).h;
    //     vec2(w as f32, h as f32)
    // };

    // let size = vec2(1600.0, 600.0);
    let size = vec2(800.0, 600.0);
    // let size = vec2(1024.0, 768.0);
    // let size = vec2(1366.0, 1024.0);

    let window = unsafe {
        SDL_CreateWindow(
            c"Maple RS".as_ptr(),
            size.x as i32,
            size.y as i32,
            // SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_BORDERLESS | SDL_WINDOW_MAXIMIZED | SDL_WINDOW_FULLSCREEN,
            SDL_WINDOW_HIGH_PIXEL_DENSITY,
        )
    };
    let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

    provide_context(WzBase {
        node: wz::resolve_base().unwrap(),
    });

    Root::new(app::app, renderer).launch();

    Ok(())
}
