use crate::{element::Element, view_state::ViewState};
use cosmic_text::{fontdb::Source, FontSystem, SwashCache};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{cell::RefCell, rc::Rc};
use taffy::{NodeId, TaffyTree};

thread_local! {
    pub static RUNTIME: RefCell<Runtime> = Default::default();
}

pub struct Runtime {
    pub taffy: Rc<RefCell<TaffyTree>>,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub states: SecondaryMap<DefaultKey, Rc<RefCell<ViewState>>>,
    pub elements: SecondaryMap<DefaultKey, Rc<dyn Element>>,
    pub animation_frame_callbacks: Rc<RefCell<SlotMap<DefaultKey, Box<dyn Fn(f32) + 'static>>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        let font_system = FontSystem::new_with_fonts([Source::File("Data/simsun.ttc".into())]);

        Self {
            taffy: Rc::new(RefCell::new(TaffyTree::new())),
            font_system,
            swash_cache: SwashCache::new(),
            states: Default::default(),
            elements: Default::default(),
            animation_frame_callbacks: Default::default(),
        }
    }
}
