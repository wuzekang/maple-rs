use crate::style::Style;
use sdl3_sys::events::SDL_Event;
use std::rc::Rc;
use taffy::{NodeId, Point, TaffyTree};

pub struct ViewState {
    pub style: Style,
    pub viewport: Point<f32>,
    pub listeners: Vec<Rc<Box<dyn Fn(&SDL_Event)>>>,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            viewport: Point::ZERO,
            style: Default::default(),
            listeners: vec![],
        }
    }
}
