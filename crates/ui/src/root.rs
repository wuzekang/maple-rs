use crate::dynamic::dynamic;
use crate::image::Image;
use crate::view::view;
use ::image::load_from_memory;
use crate::element::{Element, IntoElement};
use crate::fragment::Fragment;
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{RwSignal, SignalGet, SignalUpdate};
use crate::runtime::RUNTIME;
use crate::sdl::{Painter, PollEvent};
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
use crate::text::Text;
use crate::view::View;
use crate::view_id::ViewId;

fn compute_layout(taffy: &mut TaffyTree, parent: NodeId, viewport: Point<f32>) {
    let children = taffy.children(parent).unwrap();
    for child in children {
        let id = ViewId(child);
        id.state().borrow_mut().viewport = viewport;
        let location = taffy.layout(child).unwrap().location;
        let viewport = viewport + location;
        compute_layout(taffy, child, viewport);
    }
}

pub struct Root {
    view: View,
    size: RwSignal<Vec2>,
    painter: Painter,
}

impl Root {
    pub fn new<F, E>(f: F, renderer: *mut SDL_Renderer) -> Self
    where
        F: 'static + Fn() -> E,
        E: IntoElement,
    {
        let window = unsafe { SDL_GetRenderWindow(renderer) };
        let size = RwSignal::new(unsafe {
            let mut x = 0;
            let mut y = 0;
            SDL_GetWindowSize(window, &mut x, &mut y);
            vec2(x as f32, y as f32)
        });
        let view =
            view(f()).style(move |s| s.width(length(size.get().x)).height(length(size.get().y)));
        Self {
            view,
            size,
            painter: Painter::new(renderer),
        }
    }

    pub fn dispatch_event(&self, event: &SDL_Event) {
        unsafe {
            if SDL_EventType(event.r#type) == SDL_EventType::WINDOW_RESIZED.into() {
                self.size
                    .set(vec2(event.window.data1 as f32, event.window.data2 as f32));
            }
        }
        self.view.id().dispatch_event(event);
    }

    pub fn layout(&self) {
        let taffy = RUNTIME.with_borrow_mut(|s| s.taffy.clone());
        taffy
            .borrow_mut()
            .compute_layout_with_measure(
                self.view.id().node(),
                Size::MAX_CONTENT,
                move |known_dimensions, available_space, id, _, _| {
                    if let Size {
                        width: Some(width),
                        height: Some(height),
                    } = known_dimensions
                    {
                        return Size { width, height };
                    }

                    let element = RUNTIME.with_borrow_mut(|s| s.elements.get(id.into()).cloned());

                    element
                        .map(|e| e.borrow().measure(known_dimensions, available_space))
                        .unwrap_or_default()
                },
            )
            .unwrap();

        compute_layout(&mut taffy.borrow_mut(), self.view.id().node(), Point::ZERO);
    }

    pub fn paint(&self) {
        self.view.paint(&self.painter);
    }
}