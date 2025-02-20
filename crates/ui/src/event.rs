use crate::{Element, ViewId};
use glam::{vec2, Vec2};
use sdl3_sys::events::{SDL_Event, SDL_EventType};
use slotmap::DefaultKey;
use std::cell::{Cell, RefCell};
use std::cmp::PartialEq;
use std::rc::Rc;
use sdl3_sys::everything::{SDL_MouseMotionEvent, SDL_MouseWheelEvent};

#[derive(Clone)]
pub struct EventEmitter {
    listeners: Rc<RefCell<slotmap::SlotMap<DefaultKey, Box<dyn Fn()>>>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Default::default(),
        }
    }

    pub fn emit(&self) {
        for (_, f) in self.listeners.borrow().iter() {
            f();
        }
    }

    pub fn on(&self, f: impl Fn() + 'static) -> DefaultKey {
        self.listeners.borrow_mut().insert(Box::new(f))
    }

    pub fn off(&self, key: DefaultKey) {
        self.listeners.borrow_mut().remove(key);
    }
}

#[derive(Eq, PartialEq,Copy,Clone)]
pub enum EventType {
    MouseEnter,
    MouseLeave,
    MouseMove,
    MouseWheel,
    MouseDown,
    MouseUp,
    Click,
    None,
}

pub struct Event<'a> {
    pub event_type: EventType,
    pub event: &'a SDL_Event,
    pub target: ViewId,
    pub current_target: Cell<ViewId>,
}

impl Event<'_> {
    pub fn client(&self) -> Vec2 {
        unsafe {
            Vec2 {
                x: self.event.motion.x,
                y: self.event.motion.y,
            }
        }
    }

    pub fn offset(&self) -> Vec2 {
        let location = self.current_target.get().layout().unwrap().location;
        let viewport = self.current_target.get().state().borrow().viewport;
        self.client() - vec2(viewport.x, viewport.y) - vec2(location.x, location.y)
    }

    pub fn in_view_rect(&self) -> bool {
        let id = self.target;
        let layout = id.layout().unwrap();
        let viewport = id.state().borrow().viewport;

        let x = unsafe { self.event.button.x } - viewport.x;
        let y = unsafe { self.event.button.y } - viewport.y;

        let left = layout.location.x;
        let top = layout.location.y;
        let right = left + layout.size.width;
        let bottom = top + layout.size.height;

        x >= left && x < right && y >= top && y < bottom
    }

    pub fn is_pointer_event(&self) -> bool {
        match self.sdl_event_type() {
            SDL_EventType::MOUSE_MOTION
            | SDL_EventType::MOUSE_BUTTON_DOWN
            | SDL_EventType::MOUSE_BUTTON_UP
            | SDL_EventType::MOUSE_WHEEL => true,
            _ => false,
        }
    }

    pub fn sdl_event_type(&self) -> SDL_EventType {
        SDL_EventType(unsafe { self.event.r#type })
    }
}



pub trait Interactive: Sized + Element {
    fn on_event<F>(self, event_type: EventType, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        let _ = self.id().add_event_listener(Box::new(move |event| {
            if event.event_type == event_type {
                f(event)
            }
        }));
        self
    }

    fn on_click<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        self.on_event(EventType::MouseDown, move |event| {
            f(event);
        })
    }

    fn on_mouse_move<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        self.on_event(EventType::MouseMove, move |event| {
            f(event);
        })
    }

    fn on_mouse_enter<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        let id = self.id();
        self.on_event(EventType::MouseEnter, move |event| {
            f(event);
        })
    }
    fn on_mouse_leave<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        let id = self.id();
        self.on_event(EventType::MouseLeave, move |event| {
            f(event);
        })
    }
}
