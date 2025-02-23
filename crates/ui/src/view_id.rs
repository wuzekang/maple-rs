use crate::event::{Event, EventTarget, EventType, FocusEvent};
use crate::style::{
    PointerEvents, Style, StyleComputeContext, StyleProperty, StylePropertyKey, TaffyStyleProperty,
    TaffyStylePropertyKey,
};
use crate::{element::Element, runtime::RUNTIME, view_state::ViewState};
use glam::Vec2;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use taffy::{NodeId, TaffyTree};

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
        let id = RUNTIME.with_borrow_mut(|r| {
            r.taffy
                .borrow_mut()
                .new_leaf(taffy::Style::DEFAULT)
                .unwrap()
        });
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
        listener: Box<dyn (Fn(&mut dyn Event) -> ()) + 'static>,
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
        if self.rect().contains(location)
            && self.state().borrow().style.pointer_events != PointerEvents::None
        {
            *target = Some(*self);
        }
        for child in self.children() {
            child.event_capture(location, target);
        }
    }

    pub fn dispatch_event(&self, event: &mut dyn Event, bubble: bool) {
        let state = self.state();
        let listeners = state.borrow().listeners.clone();
        for (_, listener) in listeners {
            listener(event);
        }
        if bubble && event.propagation() {
            if let Some(parent) = self.parent() {
                event.set_current_target(parent);
                parent.dispatch_event(event, bubble);
            }
        }
    }

    pub fn compute_style(&self, ctx: &mut StyleComputeContext) {
        ctx.push();

        let state = self.state();
        let node = self.node();

        let mut style_props = HashMap::<StylePropertyKey, StyleProperty>::new();
        let mut taffy_style_props = HashMap::<TaffyStylePropertyKey, TaffyStyleProperty>::new();

        let styles = state.borrow().styles.clone();
        for style_builder in styles.into_iter().rev() {
            if let Some(style_builder) = style_builder {
                for (key, value) in style_builder.taffy_style_props.into_iter().rev() {
                    if !taffy_style_props.contains_key(&key) {
                        taffy_style_props.insert(key, value);
                    }
                }
                for (key, value) in style_builder.style_props.into_iter().rev() {
                    if !style_props.contains_key(&key) {
                        style_props.insert(key, value);
                    }
                }
            }
        }

        for (key, value) in TaffyStyleProperty::initial() {
            if !taffy_style_props.contains_key(&key) {
                taffy_style_props.insert(key, value);
            }
        }

        let mut taffy_style = taffy::Style::default();
        for (_, value) in taffy_style_props {
            value.assign_to(&mut taffy_style);
        }

        self.taffy()
            .borrow_mut()
            .set_style(node, taffy_style)
            .unwrap();

        let mut style = Style::default();

        for (key, value) in StyleProperty::initial() {
            if !style_props.contains_key(&key) {
                value.assign_to(&mut style);
            }
        }

        for (key, value) in ctx.style.iter() {
            if !style_props.contains_key(key) && value.inherited() {
                value.assign_to(&mut style);
            }
        }

        for (_, value) in style_props.iter() {
            value.assign_to(&mut style);
        }



        state.borrow_mut().style = style;

        for (key, value) in style_props {
            if value.inherited() {
                if ctx.style.contains_key(&key) {
                    ctx.style.remove(&key);
                }
                ctx.style.insert(key, value);
            }
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

    pub(crate) fn mounted(&self) {
        self.dispatch_event(
            &mut FocusEvent {
                r#type: EventType::Mounted,
                event_target: EventTarget::new(*self),
            } as &mut dyn Event,
            false,
        );
    }

    pub(crate) fn unmounted(&self) {
        self.dispatch_event(
            &mut FocusEvent {
                r#type: EventType::Unmounted,
                event_target: EventTarget::new(*self),
            } as &mut dyn Event,
            false,
        );
    }
}
