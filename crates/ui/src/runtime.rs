use crate::{element::Element, view_state::ViewState};
use cosmic_text::{FontSystem, SwashCache};
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
    pub elements: SecondaryMap<DefaultKey, Rc<RefCell<Box<dyn Element>>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            taffy: Rc::new(RefCell::new(TaffyTree::new())),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            states: Default::default(),
            elements: Default::default(),
        }
    }
}
