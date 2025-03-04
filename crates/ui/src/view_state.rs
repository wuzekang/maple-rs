use crate::event::{Event};
use crate::style::{Style, StyleBuilder};
use slotmap::{DefaultKey, SlotMap};
use std::rc::Rc;
use taffy::Point;
use crate::Texture;

#[derive(Clone)]
pub struct Layer {
    pub texture: Rc<Texture>,
    pub blend_texture: Rc<Texture>,
}

pub struct ViewState {
    pub style: Style,
    pub styles: Vec<Option<StyleBuilder>>,
    pub style_dirty: bool,
    pub inherited_style_dirty: bool,
    pub viewport: Point<f32>,
    pub layer: Option<Layer>,
    pub listeners: SlotMap<DefaultKey, Rc<Box<dyn Fn(&mut Event)>>>,
    pub mounted: bool,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            viewport: Point::ZERO,
            style: Default::default(),
            styles: Default::default(),
            style_dirty: true,
            inherited_style_dirty: true,
            layer: None,
            listeners: Default::default(),
            mounted: false,
        }
    }
}
