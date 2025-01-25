use scene::EventEmitter;
use glam::{vec2, Vec2Swizzles};
use sdl3_sys::{
    events::{SDL_Event, SDL_EventType, SDL_PollEvent},
    init::{SDL_Init, SDL_INIT_VIDEO},
    render::{
        SDL_CreateRenderer, SDL_RenderClear, SDL_RenderPresent, SDL_SetRenderDrawColor,
        SDL_SetRenderVSync,
    },
    timer::SDL_Delay,
    video::SDL_CreateWindow,
};
use std::{error::Error, mem::MaybeUninit};
use ui::image::IntoDrawable;
use ui::{
    reactive::{provide_context, SignalGet, SignalUpdate},
    taffy::prelude::*,
    Drawable, Element, IntoElement, Root,
};
use wz::Node;

mod character;
mod scene;
mod map;
mod math;
mod npc;
mod sdl;
mod sprite;
mod timer;
mod ui_view;
mod wz;

struct PollEvent {
    event: MaybeUninit<SDL_Event>,
}

unsafe impl Send for PollEvent {}
unsafe impl Sync for PollEvent {}

impl PollEvent {
    fn new() -> Self {
        Self {
            event: MaybeUninit::uninit(),
        }
    }
}

impl<'a> Iterator for &'a mut PollEvent {
    type Item = &'a SDL_Event;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if SDL_PollEvent(self.event.as_mut_ptr()) {
                Some(&*self.event.as_ptr())
            } else {
                None
            }
        }
    }
}

#[derive(Clone)]
struct WzBase {
    pub node: Node,
}

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        SDL_Init(SDL_INIT_VIDEO);
    }

    let size = vec2(800.0, 600.0);
    let window =
        unsafe { SDL_CreateWindow(c"Maple RS".as_ptr(), size.x as i32, size.y as i32, 0x2000) };
    let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

    provide_context(window);
    let update_event = EventEmitter::new();
    provide_context(update_event.clone());
    provide_context(WzBase {
        node: wz::resolve_base().unwrap(),
    });

    let root = Root::new(ui_view::ui_view, renderer);

    let mut events = PollEvent::new();

    unsafe {
        SDL_SetRenderVSync(renderer, 1);

        let mut exited = false;

        while !exited {
            for event in &mut events {
                root.dispatch_event(event);
                match SDL_EventType(event.r#type) {
                    SDL_EventType::QUIT => {
                        exited = true;
                    }
                    _ => {}
                }
            }

            update_event.emit();

            root.layout();

            SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
            SDL_RenderClear(renderer);

            root.paint();

            SDL_RenderPresent(renderer);
            SDL_Delay(16);
        }
    }

    Ok(())
}
