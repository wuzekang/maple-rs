use crate::root::EventDispatcher;
use crate::runtime::RUNTIME;
use crate::{Element, ViewId};
use glam::{vec2, Vec2};
use reactive::{on_cleanup, use_context};
use sdl3_sys::everything::*;
use std::any::Any;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::ffi::CStr;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct EventIterator {
    event: MaybeUninit<SDL_Event>,
}

impl EventIterator {
    pub fn new() -> Self {
        Self {
            event: MaybeUninit::uninit(),
        }
    }
}

impl Default for EventIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Iterator for &'a mut EventIterator {
    type Item = &'a SDL_Event;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if SDL_PollEvent(self.event.as_mut_ptr()) {
                Some(&*self.event.as_ptr())
            } else {
                None
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum MouseEventType {
    MouseEnter,
    MouseLeave,
    MouseMove,
    MouseWheel,
    MouseDown,
    MouseUp,
    Click,
}

impl TryFrom<SDL_EventType> for MouseEventType {
    type Error = ();
    fn try_from(value: SDL_EventType) -> Result<Self, Self::Error> {
        match value {
            SDL_EventType::MOUSE_MOTION => Ok(Self::MouseMove),
            SDL_EventType::MOUSE_BUTTON_UP => Ok(Self::MouseUp),
            SDL_EventType::MOUSE_BUTTON_DOWN => Ok(Self::MouseDown),
            SDL_EventType::MOUSE_WHEEL => Ok(Self::MouseWheel),
            SDL_EventType::WINDOW_MOUSE_ENTER => Ok(Self::MouseEnter),
            SDL_EventType::WINDOW_MOUSE_LEAVE => Ok(Self::MouseLeave),
            _ => Err(()),
        }
    }
}

pub struct MouseEvent {
    pub r#type: MouseEventType,
    pub motion: Vec2,

    pub target: ViewId,
    pub current: Option<ViewId>,
    pub propagation: bool,
}
impl MouseEvent {
    pub fn client(&self) -> Vec2 {
        unsafe {
            Vec2 {
                x: self.motion.x,
                y: self.motion.y,
            }
        }
    }

    pub fn offset(&self) -> Vec2 {
        let target = self.current.unwrap();
        let location = target.layout().location;
        let viewport = target.state().borrow().viewport;
        self.client() - vec2(viewport.x, viewport.y) - vec2(location.x, location.y)
    }

    pub fn in_view_rect(&self) -> bool {
        let id = self.target;
        let layout = id.layout();
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

#[derive(PartialEq, Eq)]
pub enum KeyboardEventType {
    KeyDown,
    KeyUp,
}

impl TryFrom<SDL_EventType> for KeyboardEventType {
    type Error = ();
    fn try_from(value: SDL_EventType) -> Result<Self, Self::Error> {
        match value {
            SDL_EventType::KEY_UP => Ok(Self::KeyUp),
            SDL_EventType::KEY_DOWN => Ok(Self::KeyDown),
            _ => Err(()),
        }
    }
}

pub struct KeyboardEvent {
    pub r#type: KeyboardEventType,
    pub key: SDL_Keycode,
    pub r#mod: SDL_Keymod,
    pub scancode: SDL_Scancode,
    pub target: ViewId,
    pub current: Option<ViewId>,
    pub propagation: bool,
}

impl Debug for KeyboardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardEvent")
            .field("key", &self.key)
            .field("mod", &self.r#mod)
            .finish()
    }
}

#[derive(PartialEq, Eq)]
pub enum FocusEventType {
    Focus,
    Blur,
}
pub struct FocusEvent {
    pub r#type: FocusEventType,
    pub target: ViewId,
}

#[derive(PartialEq, Eq)]
pub enum LifecycleEventType {
    Attach,
    Detach,
}
pub struct LifecycleEvent {
    pub r#type: LifecycleEventType,
}

pub struct TextInputEvent {
    pub text: String,
}

pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Focus(FocusEvent),
    TextInput(TextInputEvent),
    Lifecycle(LifecycleEvent),
}

impl TryFrom<(&SDL_Event, ViewId)> for Event {
    type Error = ();
    fn try_from((event, id): (&SDL_Event, ViewId)) -> Result<Self, Self::Error> {
        let event_type = unsafe { SDL_EventType(event.r#type) };

        unsafe {
            if let Ok(r#type) = MouseEventType::try_from(event_type) {
                Ok(Event::Mouse(MouseEvent {
                    r#type,
                    target: id,
                    current: None,
                    propagation: true,
                    motion: vec2(event.motion.x, event.motion.y),
                }))
            } else if let Ok(r#type) = KeyboardEventType::try_from(event_type) {
                Ok(Event::Keyboard(KeyboardEvent {
                    r#type,
                    target: id,
                    current: None,
                    propagation: true,
                    key: event.key.key,
                    r#mod: event.key.r#mod,
                    scancode: event.key.scancode,
                }))
            } else {
                match event_type {
                    SDL_EventType::TEXT_INPUT => Ok(Event::TextInput(TextInputEvent {
                        text: unsafe {
                            CStr::from_ptr(event.text.text)
                                .to_str()
                                .unwrap()
                                .to_string()
                        },
                    })),
                    _ => Err(()),
                }
            }
        }
    }
}

impl Event {
    pub fn propagation(&self) -> bool {
        match self {
            Event::Mouse(event) => event.propagation,
            Event::Keyboard(event) => event.propagation,
            _ => false,
        }
    }
    pub fn set_current_target(&mut self, target: ViewId) {
        match self {
            Event::Mouse(event) => event.current = Some(target),
            Event::Keyboard(event) => event.current = Some(target),
            _ => (),
        }
    }

    pub fn is_mouse_event(&self) -> bool {
        match self {
            Self::Mouse(_) => true,
            _ => false,
        }
    }

    pub fn is_mouse(&mut self, r#type: MouseEventType) -> Option<&mut MouseEvent> {
        match self {
            Event::Mouse(event) => {
                if event.r#type == r#type {
                    Some(event)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_mouse_move(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::MouseMove)
    }
    pub fn is_mouse_down(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::MouseDown)
    }
    pub fn is_mouse_up(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::MouseUp)
    }
    pub fn is_mouse_enter(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::MouseEnter)
    }
    pub fn is_mouse_leave(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::MouseLeave)
    }
    pub fn is_mouse_wheel(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::MouseWheel)
    }
    pub fn is_click(&mut self) -> Option<&mut MouseEvent> {
        self.is_mouse(MouseEventType::Click)
    }

    pub fn is_keyboard_event(&mut self, r#type: KeyboardEventType) -> Option<&mut KeyboardEvent> {
        match self {
            Event::Keyboard(event) => {
                if event.r#type == r#type {
                    Some(event)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn is_key_down(&mut self) -> Option<&mut KeyboardEvent> {
        self.is_keyboard_event(KeyboardEventType::KeyDown)
    }
    pub fn is_key_up(&mut self) -> Option<&mut KeyboardEvent> {
        self.is_keyboard_event(KeyboardEventType::KeyUp)
    }

    pub fn is_focus(&mut self) -> Option<&mut FocusEvent> {
        match self {
            Event::Focus(event) => {
                if event.r#type == FocusEventType::Focus {
                    Some(event)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_blur(&mut self) -> Option<&mut FocusEvent> {
        match self {
            Event::Focus(event) => {
                if event.r#type == FocusEventType::Blur {
                    Some(event)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_text_input(&mut self) -> Option<&mut TextInputEvent> {
        if let Self::TextInput(event) = self {
            Some(event)
        } else {
            None
        }
    }

    pub fn is_lifecycle(&mut self, r#type: LifecycleEventType) -> Option<&mut LifecycleEvent> {
        if let Self::Lifecycle(event) = self {
            if event.r#type == r#type {
                return Some(event);
            }
        }
        None
    }

    pub fn is_attach(&mut self) -> Option<&mut LifecycleEvent> {
        self.is_lifecycle(LifecycleEventType::Attach)
    }

    pub fn is_detach(&mut self) -> Option<&mut LifecycleEvent> {
        self.is_lifecycle(LifecycleEventType::Detach)
    }
}

#[derive(Clone, Copy)]
pub struct ElementRef<T: Element> {
    id: ViewId,
    _marker: PhantomData<T>,
}

impl<T: Element> ElementRef<T> {
    pub fn with(&self, f: impl FnOnce(&T)) {
        let id = self.id;
        let element = RUNTIME.with_borrow(move |r| {
            r.elements.get(id.0.into()).unwrap().clone() as Rc<RefCell<dyn Any>>
        });
        f(element.borrow().downcast_ref::<T>().unwrap());
    }
}

pub trait Interactive: Sized + Element {
    fn _ref(self, r: &mut Option<ElementRef<Self>>) -> Self {
        *r = Some(ElementRef::<Self> {
            id: self.id(),
            _marker: Default::default(),
        });
        self
    }

    fn focus(&self) {
        let ctx: EventDispatcher = use_context().unwrap();
        ctx.queue(Event::Focus(FocusEvent {
            r#type: FocusEventType::Focus,
            target: self.id(),
        }));
    }

    fn on_event<F>(self, f: F) -> Self
    where
        F: (Fn(&mut Event) -> ()) + 'static,
    {
        let _ = self
            .id()
            .add_event_listener(Box::new(move |event| f(event)));
        self
    }

    fn on_mouse_event<F>(self, r#type: MouseEventType, f: F) -> Self
    where
        F: (Fn(&mut MouseEvent) -> ()) + 'static,
    {
        let _ = self.id().add_event_listener(Box::new(move |event| {
            if let Event::Mouse(event) = event {
                if event.r#type == r#type {
                    f(event)
                }
            }
        }));
        self
    }

    fn on_click<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseEvent) -> ()) + 'static,
    {
        self.on_mouse_event(MouseEventType::MouseDown, move |event| {
            f(event);
        })
    }

    fn on_mouse_move<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseEvent) -> ()) + 'static,
    {
        self.on_mouse_event(MouseEventType::MouseMove, move |event| {
            f(event);
        })
    }

    fn on_mouse_enter<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseEvent) -> ()) + 'static,
    {
        self.on_mouse_event(MouseEventType::MouseEnter, move |event| {
            f(event);
        })
    }
    fn on_mouse_leave<F>(self, f: F) -> Self
    where
        F: (Fn(&MouseEvent) -> ()) + 'static,
    {
        self.on_mouse_event(MouseEventType::MouseLeave, move |event| {
            f(event);
        })
    }

    fn on_focus<F>(self, f: F) -> Self
    where
        F: (Fn(&FocusEvent) -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::Focus(event) = event {
                if event.r#type == FocusEventType::Focus {
                    f(event);
                }
            }
        })
    }
    fn on_blur<F>(self, f: F) -> Self
    where
        F: (Fn(&FocusEvent) -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::Focus(event) = event {
                if event.r#type == FocusEventType::Blur {
                    f(event);
                }
            }
        })
    }

    fn on_text_input<F>(self, f: F) -> Self
    where
        F: (Fn(&TextInputEvent) -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::TextInput(event) = event {
                f(event);
            }
        })
    }

    fn on_key_down<F>(self, f: F) -> Self
    where
        F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::Keyboard(event) = event {
                if event.r#type == KeyboardEventType::KeyDown {
                    f(event);
                }
            }
        })
    }

    fn on_key_up<F>(self, f: F) -> Self
    where
        F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::Keyboard(event) = event {
                if event.r#type == KeyboardEventType::KeyUp {
                    f(event);
                }
            }
        })
    }

    fn on_attach<F>(self, f: F) -> Self
    where
        F: (Fn() -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::Lifecycle(event) = event {
                if event.r#type == LifecycleEventType::Attach {
                    f();
                }
            }
        })
    }

    fn on_detach<F>(self, f: F) -> Self
    where
        F: (Fn() -> ()) + 'static,
    {
        self.on_event(move |event| {
            if let Event::Lifecycle(event) = event {
                if event.r#type == LifecycleEventType::Detach {
                    f();
                }
            }
        })
    }
}

pub fn use_event<F>(f: F)
where
    F: (Fn(&mut Event) -> ()) + 'static,
{
    let root: ViewId = use_context().unwrap();
    on_cleanup(root.add_event_listener(Box::new(f)))
}

pub fn use_keyboard_event<F>(r#type: KeyboardEventType, f: F)
where
    F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
{
    use_event(move |event| {
        if let Event::Keyboard(event) = event {
            if event.r#type == r#type {
                f(event);
            }
        }
    })
}

pub fn use_key_down_event<F>(f: F)
where
    F: (Fn(&mut KeyboardEvent) -> ()) + 'static,
{
    use_keyboard_event(KeyboardEventType::KeyDown, f)
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
