use crate::event::Event;
use crate::{element::Element, runtime::RUNTIME, view_state::ViewState};
use sdl3_sys::events::SDL_Event;
use std::{cell::RefCell, rc::Rc};
use taffy::{NodeId, Style, TaffyTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewId(pub NodeId);

impl ViewId {
    pub fn new() -> Self {
        Self(RUNTIME.with_borrow_mut(|r| r.taffy.borrow_mut().new_leaf(Style::DEFAULT).unwrap()))
    }

    pub fn node(&self) -> NodeId {
        self.0
    }

    pub fn children(&self) -> Vec<ViewId> {
        RUNTIME.with_borrow(|r| {
            r.taffy
                .borrow()
                .children(self.0)
                .unwrap_or_default()
                .into_iter()
                .map(|item| ViewId(item))
                .collect::<Vec<_>>()
        })
    }

    pub fn set_children(&self, elements: Vec<ViewId>) {
        let children = elements.into_iter().map(|item| item.0).collect::<Vec<_>>();

        self.taffy().borrow_mut().set_children(self.0, &children);
    }

    pub fn remove(&self) {
        self.taffy().borrow_mut().remove(self.0);
    }

    pub fn taffy(&self) -> Rc<RefCell<TaffyTree>> {
        RUNTIME.with_borrow(|s| s.taffy.clone())
    }

    pub fn state(&self) -> Rc<RefCell<ViewState>> {
        RUNTIME.with_borrow_mut(|s| {
            s.states
                .entry(self.0.into())
                .unwrap()
                .or_insert_with(|| Rc::new(RefCell::new(ViewState::new())))
                .clone()
        })
    }

    pub fn element(&self) -> Rc<RefCell<Box<dyn Element>>> {
        RUNTIME.with_borrow(|s| s.elements.get(self.0.into()).cloned().unwrap())
    }

    pub fn get_layout(&self) -> Option<taffy::Layout> {
        self.taffy().borrow_mut().layout(self.0).cloned().ok()
    }

    pub fn add_event_listener(
        &self,
        listener: Box<dyn (Fn(&Event) -> ()) + 'static>,
    ) -> Box<dyn Fn()> {
        let key = self
            .state()
            .borrow_mut()
            .listeners
            .insert(Rc::new(listener));

        let id = *self;

        Box::new(move || {
            id.state().borrow_mut().listeners.remove(key);
        })
    }

    pub fn dispatch_event(&self, event: &SDL_Event) {
        let event_ = Event {
            event,
            target: *self,
        };
        let state = self.state();
        let listeners = state.borrow().listeners.clone();
        for (_, listener) in listeners {
            listener(&event_);
        }
        for child in self.children() {
            child.dispatch_event(event);
        }
    }

    pub fn mark_dirty(&self) {
        self.taffy().borrow_mut().mark_dirty(self.0).unwrap();
    }
}
