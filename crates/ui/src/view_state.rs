use crate::event::Event;
use crate::style::{Style, StyleBuilder, StyleProperty, StylePropertyKey};
use crate::Texture;
use slotmap::{DefaultKey, SlotMap};
use std::collections::HashMap;
use std::rc::Rc;
use taffy::Point;

#[derive(Clone)]
pub struct Layer {
    pub texture: Rc<Texture>,
    pub blend_texture: Rc<Texture>,
}

pub struct ViewState {
    pub style: Style,
    pub styles: Vec<Option<StyleBuilder>>,
    pub style_dirty: bool,
    pub style_inherited_dirty: bool,
    pub style_cache: Option<HashMap<StylePropertyKey, StyleProperty>>,
    pub viewport: Point<f32>,
    pub layer: Option<Layer>,
    pub composite: bool,
    pub repaint: bool,
    pub listeners: SlotMap<DefaultKey, Rc<Box<dyn Fn(&mut Event)>>>,
    pub mounted: bool,
    pub tab_index: Option<i32>,
    pub index: usize,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            viewport: Point::ZERO,
            style: Default::default(),
            styles: Default::default(),
            style_dirty: true,
            style_inherited_dirty: true,
            style_cache: None,
            layer: None,
            composite: false,
            repaint: true,
            listeners: Default::default(),
            mounted: false,
            tab_index: None,
            index: 0,
        }
    }
}
