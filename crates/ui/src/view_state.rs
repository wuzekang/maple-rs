use crate::event::Event;
use crate::style::Style;
use sdl3_sys::events::SDL_EventType;
use slotmap::{DefaultKey, SlotMap};
use std::collections::HashMap;
use std::rc::Rc;
use taffy::Point;

pub struct ViewState {
    pub style: Style,
    pub viewport: Point<f32>,
    pub listeners: HashMap<SDL_EventType, SlotMap<DefaultKey, Rc<Box<dyn Fn(&Event)>>>>,
    pub hovered: bool,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            viewport: Point::ZERO,
            style: Default::default(),
            listeners: Default::default(),
            hovered: false,
        }
    }
}
