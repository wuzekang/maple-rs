use crate::event::Event;
use crate::geometry::Rect;
use crate::render::layer::Layer;
use crate::style::{compute_style_recursive, PointerEvents, StyleComputeContext, StyleTrigger};
use crate::{element::Element, runtime::RUNTIME, view_state::ViewState, Renderer};
use glam::{vec2, Vec2};
use reactive::on_cleanup;
use slotmap::DefaultKey;
use std::cmp::{Ordering, PartialEq};
use std::{cell::RefCell, rc::Rc};
use taffy::{LengthPercentage, NodeId, Point, TaffyTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(pub NodeId);

impl PartialOrd for ViewId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(DefaultKey::from(self.0).cmp(&other.0.into()))
    }
}

impl Ord for ViewId {
    fn cmp(&self, other: &Self) -> Ordering {
        DefaultKey::from(self.0).cmp(&other.0.into())
    }
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

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

        on_cleanup(move || {
            RUNTIME.with_borrow(|s| s.taffy.borrow_mut().remove(id).unwrap());
        });

        Self(id)
    }

    pub fn node(&self) -> NodeId {
        self.0
    }

    pub fn parent(&self) -> Option<ViewId> {
        RUNTIME.with_borrow(|r| r.taffy.borrow().parent(self.0).map(ViewId))
    }

    pub fn children(&self) -> Rc<Vec<ViewId>> {
        self.state().borrow().children.clone()
    }

    pub fn set_children(&self, elements: Vec<ViewId>) {
        for (index, child) in elements.iter().enumerate() {
            child.state().borrow_mut().index = index;
        }
        self.state().borrow_mut().children = Rc::new(elements.clone());
        let children = elements.into_iter().map(|item| item.0).collect::<Vec<_>>();
        self.taffy()
            .borrow_mut()
            .set_children(self.0, &children)
            .unwrap();
    }

    pub fn taffy(&self) -> Rc<RefCell<TaffyTree>> {
        RUNTIME.with_borrow(|s| s.taffy.clone())
    }

    pub fn state(&self) -> Rc<RefCell<ViewState>> {
        RUNTIME.with_borrow_mut(|s| s.states.get(self.0.into()).unwrap().clone())
    }

    pub fn element(&self) -> Rc<RefCell<dyn Element>> {
        RUNTIME.with_borrow(|s| s.elements.get(self.0.into()).unwrap().clone())
    }

    pub fn set_element(&self, element: Rc<RefCell<dyn Element>>) {
        RUNTIME.with_borrow_mut(|s| s.elements.insert(self.0.into(), element));
    }

    pub fn layout(&self) -> taffy::Layout {
        self.taffy().borrow_mut().layout(self.0).cloned().unwrap()
    }

    pub fn add_event_listener(
        &self,
        listener: Box<dyn (Fn(&mut Event)) + 'static>,
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

    pub fn event_capture(&self, location: Vec2, target: &mut ViewId) {
        if self.bounding_rect().contains(location)
            && self.state().borrow().style.pointer_events != PointerEvents::None
        {
            *target = *self;
        }
        for child in self.children().iter() {
            child.event_capture(location, target);
        }
    }

    pub fn dispatch_event(&self, event: &mut Event, bubble: bool) {
        event.set_current_target(*self);
        let element = self.element();
        element.borrow_mut().event(event);

        let state = self.state();
        let listeners = state.borrow().listeners.clone();
        for (_, listener) in listeners {
            listener(event);
        }

        if bubble && event.propagation() {
            if let Some(parent) = self.parent() {
                parent.dispatch_event(event, bubble);
            }
        }
    }

    pub fn update(&self, delta: f32) {
        self.element().borrow_mut().update(delta);
        for child in self.children().iter() {
            child.update(delta);
        }
    }

    pub fn compute_style(&self, ctx: &mut StyleComputeContext) {
        compute_style_recursive(self, ctx);
    }

    pub fn paint(&self, ctx: &mut Renderer) {
        ctx.save();
        let layout = self.layout();
        let location = layout.location;
        let opacity = self.state().borrow().style.opacity;
        let overflow = self.taffy().borrow().style(self.0).unwrap().overflow;

        let size = vec2(layout.size.width, layout.size.height);

        let translate = match self.state().borrow().style.translate {
            Point { x, y } => Point {
                x: match x {
                    LengthPercentage::Length(value) => value,
                    LengthPercentage::Percent(value) => value * layout.size.width,
                },
                y: match y {
                    LengthPercentage::Length(value) => value,
                    LengthPercentage::Percent(value) => value * layout.size.height,
                },
            },
        };

        ctx.translate(vec2(location.x, location.y) + vec2(translate.x, translate.y));

        let clip_x = overflow.x == taffy::Overflow::Hidden
            || overflow.x == taffy::Overflow::Clip
            || overflow.x == taffy::Overflow::Scroll;
        let clip_y = overflow.y == taffy::Overflow::Hidden
            || overflow.y == taffy::Overflow::Clip
            || overflow.y == taffy::Overflow::Scroll;
        let clip_rect = Rect {
            x: if clip_x { 0.0 } else { f32::NEG_INFINITY },
            y: if clip_y { 0.0 } else { f32::NEG_INFINITY },
            width: if clip_x { size.x } else { f32::INFINITY },
            height: if clip_y { size.y } else { f32::INFINITY },
        };

        if opacity < 1.0 || self.state().borrow().composite {
            if self.state().borrow().repaint {
                self.state().borrow_mut().repaint = false;

                let layer = Layer::new(Rect::from((ctx.translate, size)));

                ctx.save();
                ctx.clip = Rect::INFINITY;
                ctx.clip(&clip_rect);
                ctx.layer(&layer);
                self.element().borrow().paint(ctx);
                for child in self.children().iter() {
                    child.paint(ctx);
                }
                ctx.restore();

                self.state().borrow_mut().layer = Some(layer);
            }

            ctx.clip(&clip_rect);
            self.state()
                .borrow()
                .layer
                .as_ref()
                .unwrap()
                .render(ctx, opacity);
        } else {
            self.state().borrow_mut().repaint = false;
            ctx.clip(&clip_rect);
            self.element().borrow().paint(ctx);
            for child in self.children().iter() {
                child.paint(ctx);
            }
        }

        ctx.restore();
    }

    pub fn bounding_rect(&self) -> Rect {
        let layout = self.layout();
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

    pub fn index_path(&self, root: ViewId) -> Vec<usize> {
        let mut path = vec![];
        let mut current = *self;
        while current != root {
            path.push(current.state().borrow().index);
            current = current.parent().unwrap();
        }
        path.reverse();
        path
    }

    pub fn request_repaint(&self, trigger: StyleTrigger) {
        if trigger == StyleTrigger::None {
            return;
        }
        if !self.state().borrow().mounted {
            return;
        }
        let current = if trigger == StyleTrigger::Composite {
            self.parent()
        } else {
            Some(*self)
        };
        if current.is_none() {
            return;
        }

        let mut current = current.unwrap();
        while current.state().borrow().layer.is_none() {
            if current.state().borrow_mut().repaint {
                return;
            }
            current.state().borrow_mut().repaint = true;
            if current.parent().is_none() {
                return;
            }
            current = current.parent().unwrap();
        }
        current.state().borrow_mut().repaint = true;
    }
}
