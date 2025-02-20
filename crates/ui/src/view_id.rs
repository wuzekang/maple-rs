use crate::event::Event;
use crate::style::StyleComputeContext;
use crate::{element::Element, runtime::RUNTIME, view_state::ViewState};
use glam::Vec2;
use std::cmp::PartialEq;
use std::{cell::RefCell, rc::Rc};
use taffy::{NodeId, Style, TaffyTree};

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn contains(&self, point: Vec2) -> bool {
        let Vec2 { x, y } = point;
        let left = self.x;
        let top = self.y;
        let right = self.x + self.width;
        let bottom = self.y + self.height;
        x >= left && x < right && y >= top && y < bottom
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(pub NodeId);

impl ViewId {
    pub fn new() -> Self {
        let id =
            RUNTIME.with_borrow_mut(|r| r.taffy.borrow_mut().new_leaf(Style::DEFAULT).unwrap());
        RUNTIME.with_borrow_mut(|s| {
            s.states
                .entry(id.into())
                .unwrap()
                .or_insert_with(|| Rc::new(RefCell::new(ViewState::new())))
                .clone()
        });
        Self(id)
    }

    pub fn node(&self) -> NodeId {
        self.0
    }

    pub fn parent(&self) -> Option<ViewId> {
        RUNTIME.with_borrow(|r| r.taffy.borrow().parent(self.0).map(|item| ViewId(item)))
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

    pub fn remove(&self) -> Vec<ViewId> {
        let mut vec = Vec::new();
        for child in self.children() {
            vec.append(&mut child.remove());
        }
        self.taffy().borrow_mut().remove(self.0);
        vec.push(*self);
        vec
    }

    pub fn taffy(&self) -> Rc<RefCell<TaffyTree>> {
        RUNTIME.with_borrow(|s| s.taffy.clone())
    }

    pub fn state(&self) -> Rc<RefCell<ViewState>> {
        RUNTIME.with_borrow_mut(|s| s.states.get(self.0.into()).unwrap().clone())
    }

    pub fn element(&self) -> Rc<dyn Element> {
        RUNTIME.with_borrow(|s| s.elements.get(self.0.into()).cloned().unwrap())
    }

    pub fn layout(&self) -> Option<taffy::Layout> {
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

    pub fn event_capture(&self, location: Vec2, target: &mut Option<ViewId>) {
        if self.rect().contains(location) {
            *target = Some(*self);
        }
        for child in self.children() {
            child.event_capture(location, target);
        }
    }

    pub fn dispatch_event(&self, event: &Event, bubble: bool) {
        let state = self.state();
        let listeners = state.borrow().listeners.clone();
        for (_, listener) in listeners {
            listener(event);
        }
        if bubble {
            if let Some(parent) = self.parent() {
                event.current_target.set(parent);
                parent.dispatch_event(event, bubble);
            }
        }
    }

    pub fn compute_style(&self, ctx: &mut StyleComputeContext) {
        ctx.push();
        let state = self.state();
        let style = state.borrow().style.clone();
        if !style.cursor.is_inherit() {
            ctx.style.cursor = style.cursor.clone();
        } else {
            state.borrow_mut().style.cursor = ctx.style.cursor.clone();
        }
        for child in self.children() {
            child.compute_style(ctx);
        }
        ctx.pop();
    }

    pub fn rect(&self) -> Rect {
        let layout = self.layout().unwrap();
        let viewport = self.state().borrow().viewport;
        let location = layout.location + viewport;
        let size = layout.size;

        Rect {
            x: location.x,
            y: location.y,
            width: size.width,
            height: size.height,
        }
    }
}
