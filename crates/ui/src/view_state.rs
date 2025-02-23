use crate::event::{Event, EventType};
use crate::style::{Style, StyleBuilder};
use slotmap::{DefaultKey, SlotMap};
use std::rc::Rc;
use taffy::Point;

pub struct ViewState {
    pub style: Style,
    pub styles: Vec<Option<StyleBuilder>>,
    pub viewport: Point<f32>,
    pub listeners: SlotMap<DefaultKey, Rc<Box<dyn Fn(&mut dyn Event)>>>,
    pub mounted: bool,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            viewport: Point::ZERO,
            style: Default::default(),
            styles: Default::default(),
            listeners: Default::default(),
            mounted: false,
        }
    }
}
