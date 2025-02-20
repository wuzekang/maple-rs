use crate::event::{Event, EventType};
use crate::style::Style;
use slotmap::{DefaultKey, SlotMap};
use std::rc::Rc;
use taffy::Point;

pub struct ViewState {
    pub style: Style,
    pub viewport: Point<f32>,
    pub listeners: SlotMap<DefaultKey, Rc<Box<dyn Fn(&Event)>>>,
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
