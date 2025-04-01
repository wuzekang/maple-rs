use crate::event::Event;
use crate::render::layer::Layer;
use crate::style::{Style, StyleBuilder, StyleProperty, StylePropertyKey};
use crate::ViewId;
use slotmap::{DefaultKey, SlotMap};
use std::collections::HashMap;
use std::rc::Rc;
use taffy::Point;

pub struct ViewState {
    pub style: Style,
    pub styles: Vec<Option<StyleBuilder>>,
    pub style_dirty: bool,
    pub style_cache: HashMap<StylePropertyKey, StyleProperty>,
    pub children: Rc<Vec<ViewId>>,
    pub viewport: Point<f32>,
    pub layer: Option<Layer>,
    pub composite: bool,
    pub repaint: bool,
    pub listeners: SlotMap<DefaultKey, Rc<Box<dyn Fn(&mut Event)>>>,
    pub mounted: bool,
    pub tab_index: Option<i32>,
    pub index: usize,
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            viewport: Point::ZERO,
            style: Default::default(),
            styles: Default::default(),
            style_dirty: true,
            style_cache: Default::default(),
            children: Default::default(),
            layer: None,
            composite: false,
            repaint: false,
            listeners: Default::default(),
            mounted: false,
            tab_index: None,
            index: 0,
        }
    }
}
