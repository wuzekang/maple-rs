use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{RwSignal, SignalGet, SignalUpdate};
use runtime::RUNTIME;
use sdl::{Painter, PollEvent};
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

pub mod dynamic;
pub mod element;
pub mod fragment;
pub mod image;
pub mod runtime;
pub mod sdl;
pub mod style;
pub mod text;
pub mod view;
pub mod view_id;
pub mod view_state;
pub mod view_tuple;
pub mod root;

pub use crate::dynamic::dynamic;
pub use crate::image::Image;
pub use crate::view::view;
pub use crate::element::{Element, IntoElement};
pub use crate::fragment::Fragment;
pub use crate::text::Text;
pub use crate::view::View;
pub use crate::view_id::ViewId;
pub use crate::root::Root;
pub use taffy;
pub use peniko;
pub use reactive;