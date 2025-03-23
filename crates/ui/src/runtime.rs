use crate::resource::Resource;
use crate::{element::Element, view_state::ViewState, ViewId};
use cosmic_text::{fontdb::Source, FontSystem, SwashCache};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{cell::RefCell, rc::Rc};
use taffy::TaffyTree;
use crate::mutation_observer::ObserveOptions;

thread_local! {
    pub static RUNTIME: RefCell<Runtime> = Default::default();
}

pub struct Runtime {
    pub taffy: Rc<RefCell<TaffyTree>>,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub states: SecondaryMap<DefaultKey, Rc<RefCell<ViewState>>>,
    pub elements: SecondaryMap<DefaultKey, Rc<RefCell<dyn Element + 'static>>>,
    pub animation_frame_callbacks: Rc<RefCell<HashMap<u64, Box<dyn Fn(f32) + 'static>>>>,
    pub sender: Sender<Resource>,
    pub receiver: Receiver<Resource>,
    pub resources: RefCell<SlotMap<DefaultKey, Rc<dyn Fn(Resource)>>>,
    pub mutation_observers: RefCell<SlotMap<DefaultKey, (ViewId, Rc<dyn Fn()>, ObserveOptions)>>,
}

impl Default for Runtime {
    fn default() -> Self {
        let font_system = FontSystem::new_with_fonts([Source::File("Data/simsun.ttc".into())]);
        let (sender, receiver) = channel::<Resource>();

        Self {
            taffy: Rc::new(RefCell::new(TaffyTree::new())),
            font_system,
            swash_cache: SwashCache::new(),
            states: Default::default(),
            elements: Default::default(),
            animation_frame_callbacks: Default::default(),
            sender,
            receiver,
            resources: Default::default(),
            mutation_observers: Default::default(),
        }
    }
}
