use crate::{Element, ViewId};
use glam::{vec2, Vec2};
use sdl3_sys::events::{SDL_Event, SDL_EventType};
pub struct Event<'a> {
    pub event: &'a SDL_Event,
    pub target: ViewId,
}

impl Event<'_> {
    pub fn client(&self) -> Vec2 {
        unsafe {
            Vec2 {
                x: self.event.button.x,
                y: self.event.button.y,
            }
        }
    }

    pub fn offset(&self) -> Vec2 {
        let location = self.target.get_layout().unwrap().location;
        let viewport = self.target.state().borrow().viewport;
        self.client() - vec2(viewport.x, viewport.y) - vec2(location.x, location.y)
    }

    pub fn in_view_rect(&self) -> bool {
        let id = self.target;
        let layout = id.get_layout().unwrap();
        let viewport = id.state().borrow().viewport;

        let x = unsafe { self.event.button.x } - viewport.x;
        let y = unsafe { self.event.button.y } - viewport.y;

        let left = layout.location.x;
        let top = layout.location.y;
        let right = left + layout.size.width;
        let bottom = top + layout.size.height;

        x >= left && x < right && y >= top && y < bottom
    }
}

pub trait Interactive: Sized + Element {
    fn on_event<F>(self, event_type: SDL_EventType, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        let event_type = event_type.0.clone();
        let _ = self.id().add_event_listener(Box::new(move |e| {
            if unsafe{e.event.r#type} == event_type {
                f(e)
            }
        }));
        self
    }

    fn on_click<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        self.on_event(SDL_EventType::MOUSE_BUTTON_DOWN, move |event| {
            if event.in_view_rect() {
                f(event);
            }
        })
    }

    fn on_mouse_move<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        self.on_event(SDL_EventType::MOUSE_MOTION, move |event| {
            if event.in_view_rect() {
                f(event);
            }
        })
    }

    fn on_mouse_enter<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        let id = self.id();
        self.on_event(SDL_EventType::MOUSE_MOTION, move |event| {
            let state = id.state();
            if !state.borrow().hovered && event.in_view_rect() {
                state.borrow_mut().hovered = true;
                f(event);
            }
        })
    }
    fn on_mouse_leave<F>(self, f: F) -> Self
    where
        F: (Fn(&Event) -> ()) + 'static,
    {
        let id = self.id();
        self.on_event(SDL_EventType::MOUSE_MOTION, move |event| {
            let state = id.state();
            if state.borrow().hovered && !event.in_view_rect() {
                state.borrow_mut().hovered = false;
                f(event);
            }
        })
    }
}
