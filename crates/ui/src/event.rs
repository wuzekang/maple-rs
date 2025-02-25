use crate::root::EventDispatcher;
use crate::{Element, ViewId};
use glam::{vec2, Vec2};
use reactive::{on_cleanup, use_context};
use sdl3_sys::everything::*;
use std::any::Any;
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Clone)]
pub struct EventTarget {
    pub target: ViewId,
    pub current: Option<ViewId>,
    pub propagation: bool,
}

impl EventTarget {
    pub fn new(id: ViewId) -> Self {
        Self {
            target: id,
            current: Some(id),
            propagation: true,
        }
    }
}

pub trait Event: Any {
    fn r#type(&self) -> EventType;

    fn event_target_mut(&mut self) -> &mut EventTarget;
    fn event_target(&self) -> &EventTarget;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn set_target(&mut self, target: ViewId) {
        self.event_target_mut().target = target;
    }

    fn set_current_target(&mut self, current_target: ViewId) {
        self.event_target_mut().current = Some(current_target);
    }

    fn target(&self) -> ViewId {
        self.event_target().target
    }

    fn current_target(&self) -> Option<ViewId> {
        self.event_target().current
    }

    fn stop_propagation(&mut self) {
        self.event_target_mut().propagation = false;
    }

    fn propagation(&self) -> bool {
        self.event_target().propagation
    }
}

pub struct MouseMotionEvent {
    pub motion: Vec2,
    pub r#type: EventType,
    pub event_target: EventTarget,
}
impl MouseMotionEvent {
    pub fn new(id: ViewId, motion: Vec2) -> Self {
        Self {
            r#type: EventType::MouseMove,
            event_target: EventTarget::new(id),
            motion,
        }
    }

    pub fn client(&self) -> Vec2 {
        unsafe {
            Vec2 {
                x: self.motion.x,
                y: self.motion.y,
            }
        }
    }

    pub fn offset(&self) -> Vec2 {
        let target = self.current_target().unwrap();
        let location = target.layout().unwrap().location;
        let viewport = target.state().borrow().viewport;
        self.client() - vec2(viewport.x, viewport.y) - vec2(location.x, location.y)
    }

    pub fn in_view_rect(&self) -> bool {
        let id = self.target();
        let layout = id.layout().unwrap();
        let viewport = id.state().borrow().viewport;

        let x = unsafe { self.motion.x } - viewport.x;
        let y = unsafe { self.motion.y } - viewport.y;

        let left = layout.location.x;
        let top = layout.location.y;
        let right = left + layout.size.width;
        let bottom = top + layout.size.height;

        x >= left && x < right && y >= top && y < bottom
    }
}

impl Event for MouseMotionEvent {
    fn r#type(&self) -> EventType {
        self.r#type
    }

    fn event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }

    fn event_target(&self) -> &EventTarget {
        &self.event_target
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct FocusEvent {
    pub r#type: EventType,
    pub event_target: EventTarget,
}

impl Event for FocusEvent {
    fn r#type(&self) -> EventType {
        self.r#type
    }

    fn event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }

    fn event_target(&self) -> &EventTarget {
        &self.event_target
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct BlurEvent {
    pub r#type: EventType,
    pub event_target: EventTarget,
}

impl Event for BlurEvent {
    fn r#type(&self) -> EventType {
        self.r#type
    }

    fn event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }

    fn event_target(&self) -> &EventTarget {
        &self.event_target
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct TextInputEvent {
    pub r#type: EventType,
    pub event_target: EventTarget,
    pub text: String,
}

impl Event for TextInputEvent {
    fn r#type(&self) -> EventType {
        self.r#type
    }

    fn event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }

    fn event_target(&self) -> &EventTarget {
        &self.event_target
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct KeyboardEvent {
    pub r#type: EventType,
    pub event_target: EventTarget,
    pub key: SDL_Keycode,
    pub r#mod: SDL_Keymod,
    pub scancode: SDL_Scancode,
}

impl Debug for KeyboardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardEvent")
            .field("type", &self.r#type)
            .field("key", &self.key)
            .field("mod", &self.r#mod)
            .finish()
    }
}

impl Event for KeyboardEvent {
    fn r#type(&self) -> EventType {
        self.r#type
    }

    fn event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }

    fn event_target(&self) -> &EventTarget {
        &self.event_target
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct UnhandledEvent {
    pub r#type: EventType,
    pub event_target: EventTarget,
}

impl Event for UnhandledEvent {
    fn r#type(&self) -> EventType {
        self.r#type
    }

    fn event_target_mut(&mut self) -> &mut EventTarget {
        &mut self.event_target
    }

    fn event_target(&self) -> &EventTarget {
        &self.event_target
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
pub enum EventData {
    MouseMotion(MouseMotionEvent),
    TextInput(TextInputEvent),
    Keyboard(KeyboardEvent),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum EventType {
    MouseEnter,
    MouseLeave,
    MouseMove,
    MouseWheel,
    MouseDown,
    MouseUp,
    TextInput,
    KeyDown,
    KeyUp,
    Click,
    Focus,
    Blur,
    Attach,
    Detach,
    Unhandled,
}

impl EventType {
    pub fn is_pointer_event(&self) -> bool {
        match self {
            Self::MouseEnter => true,
            Self::MouseLeave => true,
            Self::MouseMove => true,
            Self::MouseWheel => true,
            Self::MouseDown => true,
            Self::MouseUp => true,
            _ => false,
        }
    }
}

pub trait Interactive: Sized + Element {
    fn focus(&self) {
        let ctx: EventDispatcher = use_context().unwrap();
        ctx.focus(self.id());
    }

    fn on_event<F>(self, r#type: EventType, f: F) -> Self
    where
        F: (Fn(&mut dyn Event) -> ()) + 'static,
    {
        let _ = self.id().add_event_listener(Box::new(move |event| {
            if r#type == event.r#type() {
                f(event)
            }
        }));
        self
    }

    fn on_click<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseMotionEvent) -> ()) + 'static,
    {
        self.on_event(EventType::MouseDown, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }

    fn on_mouse_move<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseMotionEvent) -> ()) + 'static,
    {
        self.on_event(EventType::MouseMove, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }

    fn on_mouse_enter<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseMotionEvent) -> ()) + 'static,
    {
        self.on_event(EventType::MouseEnter, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }
    fn on_mouse_leave<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseMotionEvent) -> ()) + 'static,
    {
        self.on_event(EventType::MouseLeave, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }

    fn on_focus<F>(self, f: F) -> Self
    where
        F: (Fn(&FocusEvent) -> ()) + 'static,
    {
        self.on_event(EventType::Focus, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }
    fn on_blur<F>(self, f: F) -> Self
    where
        F: (Fn(&BlurEvent) -> ()) + 'static,
    {
        self.on_event(EventType::Blur, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }

    fn on_text_input<F>(self, f: F) -> Self
    where
        F: (Fn(&TextInputEvent) -> ()) + 'static,
    {
        self.on_event(EventType::TextInput, move |event| {
            f(event.as_any_mut().downcast_ref().unwrap());
        })
    }

    fn on_key_down<F>(self, f: F) -> Self
    where
        F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
    {
        self.on_event(EventType::KeyDown, move |event| {
            f(event.as_any_mut().downcast_mut().unwrap());
        })
    }

    fn on_key_up<F>(self, f: F) -> Self
    where
        F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
    {
        self.on_event(EventType::KeyUp, move |event| {
            f(event.as_any_mut().downcast_mut().unwrap());
        })
    }

    fn on_attach<F>(self, f: F) -> Self
    where
        F: (Fn() -> ()) + 'static,
    {
        self.on_event(EventType::Attach, move |event| {
            f();
        })
    }

    fn on_detach<F>(self, f: F) -> Self
    where
        F: (Fn() -> ()) + 'static,
    {
        self.on_event(EventType::Detach, move |event| {
            f();
        })
    }
}

pub fn use_event<F>(r#type: Option<EventType>, f: F)
where
    F: (Fn(&mut dyn Event) -> ()) + 'static,
{
    let root: ViewId = use_context().unwrap();
    on_cleanup(root.add_event_listener(Box::new(move |event| {
        if r#type.map(|item| item == event.r#type()).unwrap_or(true) {
            f(event)
        }
    })))
}

pub fn use_keyboard_event<F>(r#type: EventType, f: F)
where
    F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
{
    use_event(Some(r#type), move |event| {
        f(event.as_any_mut().downcast_mut().unwrap());
    })
}

pub fn use_key_down_event<F>(f: F)
where
    F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
{
    use_keyboard_event(EventType::KeyDown, f)
}

pub fn use_key<F>(key: SDL_Keycode, f: F)
where
    F: (Fn() -> ()) + 'static,
{
    use_key_down_event(move |event| {
        if event.key == key {
            f()
        }
    })
}
