use ::image::load_from_memory;
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{RwSignal, SignalGet, SignalUpdate};
use sdl3_sys::{
    events::{SDL_Event, SDL_EventType},
    init::{SDL_Init, SDL_Quit, SDL_INIT_VIDEO},
    render::{
        SDL_CreateRenderer, SDL_GetRenderWindow, SDL_RenderClear, SDL_RenderPresent, SDL_Renderer,
        SDL_SetRenderDrawColor, SDL_SetRenderScale, SDL_SetRenderVSync,
        SDL_RENDERER_VSYNC_ADAPTIVE,
    },
    timer::SDL_Delay,
    video::{SDL_CreateWindow, SDL_GetWindowPixelDensity, SDL_GetWindowSize},
};
use taffy::{
    prelude::{length, TaffyMaxContent},
    Dimension, NodeId, Point, Size, TaffyTree,
};
use ui::{
    dynamic, fragment, sdl::PollEvent, view, Element, Fragment, Image, IntoElement, Root, Text,
};

fn main() {
    if unsafe { SDL_Init(SDL_INIT_VIDEO) } {
        let size = vec2(800.0, 600.0);
        let window =
            unsafe { SDL_CreateWindow(c"Maple RS".as_ptr(), size.x as i32, size.y as i32, 0x2000) };
        let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };
        let dpr = unsafe { SDL_GetWindowPixelDensity(window) };

        let root = Root::new(counter_view, renderer);

        unsafe {
            SDL_SetRenderScale(renderer, dpr, dpr);
        }

        unsafe { SDL_SetRenderVSync(renderer, SDL_RENDERER_VSYNC_ADAPTIVE) };

        let mut exited = false;

        let mut events = PollEvent::new();

        while !exited {
            for event in &mut events {
                if unsafe { event.r#type } == 0x100 {
                    exited = true;
                } else {
                    root.dispatch_event(event);
                }
            }
            root.layout();

            unsafe { SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255) };
            unsafe { SDL_RenderClear(renderer) };
            root.paint();
            unsafe { SDL_RenderPresent(renderer) };
            unsafe { SDL_Delay(16) };
        }
        unsafe { SDL_Quit() };
    }
}

struct CardProps {
    title: Fragment,
}

fn card(CardProps { title }: CardProps) -> impl IntoElement {
    view(view(title))
}

fn counter_view() -> impl Element {
    let height = RwSignal::new(100.0);

    view((
        dynamic(move || {
            if height.get() > 50.0 {
                view(()).style(|s| {
                    s.width(length(10.0))
                        .height(length(10.0))
                        .background(Color::RED)
                })
            } else {
                view(()).style(|s| {
                    s.width(length(10.0))
                        .height(length(20.0))
                        .background(Color::GREEN)
                })
            }
        }),
        view((
            view(()).style(|s| {
                s.background(Color::RED)
                    .flex_grow(1.0)
                    .width(Dimension::Auto)
                    .height(length(20.0))
            }),
            view(()).style(|s| {
                s.background(Color::GREEN)
                    .flex_grow(1.0)
                    .width(Dimension::Auto)
                    .height(length(20.0))
            }),
            Fragment::new((
                view(()).style(|s| {
                    s.background(Color::BLUE)
                        .flex_grow(1.0)
                        .width(Dimension::Auto)
                        .height(length(20.0))
                }),
                card(CardProps {
                    title: Fragment::new(()),
                }),
            )),
        ))
        .style(move |s| {
            s.background(Color::BLACK)
                .width(length(100.0))
                .height(length(height.get()))
        })
        .on_click(move |event| match SDL_EventType(unsafe { event.r#type }) {
            SDL_EventType::MOUSE_BUTTON_DOWN => {
                height.set(if height.get() == 50.0 { 100.0 } else { 50.0 });
            }
            _ => {}
        }),
        view((
            view(()).style(|s| {
                s.width(length(100.0))
                    .height(length(10.0))
                    .background(Color::GREEN)
            }),
            view(()).style(|s| {
                s.width(length(100.0))
                    .height(length(10.0))
                    .background(Color::BROWN)
            }),
            view(
                Text::new(move || format!("ABCDEFG\nABCD\n{}", height.get()))
                    .style(|s| s.color(Color::RED)),
            )
            .style(|s| {
                s.flex_grow(1.0)
                    .justify_content(taffy::AlignContent::Center)
                    .align_items(taffy::AlignItems::Center)
                    .background(Color::BLUE)
            }),
        ))
        .style(|s| {
            s.background(Color::PURPLE)
                .width(length(100.0))
                .height(length(100.0))
                .flex_direction(taffy::FlexDirection::Column)
        }),
    ))
    .style(|s| s.flex_direction(taffy::FlexDirection::Column))
}
