use crate::element::{Element, IntoElement};
use crate::event::{
    BlurEvent, Event, EventEmitter, EventTarget, EventType, FocusEvent, KeyboardEvent,
    MouseMotionEvent, TextInputEvent, UnhandledEvent,
};
use crate::runtime::RUNTIME;
use crate::sdl::{PollEvent, Renderer};
use crate::style::{Cursor, StyleComputeContext, Styleable};
use crate::view::View;
use crate::view_id::ViewId;
use crate::{Bounds, Drawable};
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{provide_context, RwSignal, Scope, SignalGet, SignalUpdate};
use sdl3_sys::everything::{
    SDL_Delay, SDL_GetTicks, SDL_HideCursor, SDL_RenderClear, SDL_RenderPresent,
    SDL_SetRenderDrawColor, SDL_SetRenderVSync, SDL_ShowCursor,
};
use sdl3_sys::mouse::SDL_GetMouseState;
use sdl3_sys::{
    events::{SDL_Event, SDL_EventType},
    everything::*,
    render::{SDL_GetRenderWindow, SDL_Renderer},
    video::SDL_GetWindowSize,
};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::CStr;
use std::mem;
use std::mem::MaybeUninit;
use std::rc::Rc;
use taffy::{
    prelude::{length, TaffyMaxContent},
    NodeId, Point, Size, TaffyTree,
};

fn compute_layout(taffy: &mut TaffyTree, parent: NodeId, viewport: Point<f32>) {
    let children = taffy.children(parent).unwrap();
    for child in children {
        let id = ViewId(child);
        id.state().borrow_mut().viewport = viewport;
        let location = taffy.layout(child).unwrap().location;
        let viewport = viewport + location;
        compute_layout(taffy, child, viewport);
    }
}

#[derive(Default, Clone)]
pub struct EventDispatcher {
    hovered: Rc<RefCell<HashSet<ViewId>>>,
    focused: Rc<RefCell<Option<ViewId>>>,
    disposed: Rc<RefCell<Vec<Scope>>>,
    mounted: Rc<RefCell<Vec<ViewId>>>,
    unmounted: Rc<RefCell<Vec<ViewId>>>,
}

impl EventDispatcher {
    pub fn focus(&self, target: ViewId) {
        if let Some(focused) = self.focused.borrow().as_ref() {
            if target == *focused {
                return;
            }
        }
        let focused = mem::take(&mut *self.focused.borrow_mut());
        let _ = self.focused.borrow_mut().insert(target);
        let prev = HashSet::<ViewId>::from_iter(focused.into_iter());
        let next = HashSet::<ViewId>::from_iter(self.focused.borrow().into_iter());
        for id in &prev - &next {
            let mut blur_event = BlurEvent {
                r#type: EventType::Blur,
                event_target: EventTarget::new(id),
            };
            blur_event
                .target()
                .dispatch_event(&mut blur_event as &mut dyn Event, false);
        }

        for id in &next - &prev {
            let mut focus_event = FocusEvent {
                r#type: EventType::Focus,
                event_target: EventTarget::new(id),
            };
            focus_event
                .target()
                .dispatch_event(&mut focus_event as &mut dyn Event, false);
        }
    }

    pub fn handle_event(&self, event: &mut dyn Any, root: ViewId) {
        if let Some(event) = event.downcast_mut::<MouseMotionEvent>() {
            let mut target: Option<ViewId> = None;
            root.event_capture(event.client(), &mut target);
            if let Some(target) = target {
                event.set_target(target);
            }

            if event.r#type() == EventType::MouseMove {
                let prev = mem::take(&mut *self.hovered.borrow_mut());
                let mut node = event.target();
                self.hovered.borrow_mut().insert(node);
                while let Some(parent) = node.parent() {
                    self.hovered.borrow_mut().insert(parent);
                    node = parent;
                }
                let mut entered = &*self.hovered.borrow() - &prev;
                let mut leaved = &prev - &*self.hovered.borrow();

                event.r#type = EventType::MouseEnter;
                for item in entered.iter() {
                    item.dispatch_event(event, false)
                }

                event.r#type = EventType::MouseLeave;
                for item in leaved.iter() {
                    item.dispatch_event(event, false)
                }

                event.r#type = EventType::MouseMove;
                event.target().dispatch_event(event as &mut dyn Event, true);
            } else if event.r#type() == EventType::MouseDown {
                event.r#type = EventType::MouseDown;
                event.target().dispatch_event(event as &mut dyn Event, true);
                self.focus(event.target());
            }
        } else if let Some(event) = event.downcast_mut::<TextInputEvent>() {
            if (event.r#type() == EventType::TextInput) {
                let id = self.focused.borrow().unwrap_or(root);
                event.set_target(id);
                event.r#type = EventType::TextInput;
                event
                    .target()
                    .dispatch_event(event as &mut dyn Event, false);
            }
        } else if let Some(event) = event.downcast_mut::<KeyboardEvent>() {
            if event.r#type() == EventType::KeyDown || event.r#type == EventType::KeyUp {
                let id = self.focused.borrow().unwrap_or(root);
                event.set_target(id);
                event.target().dispatch_event(event as &mut dyn Event, true);
            }
        } else {
            // event.target().dispatch_event(event, false);
        }
    }

    pub fn dispose(&self, scope: Scope) {
        self.disposed.borrow_mut().push(scope);
    }

    pub fn mount(&self, view: ViewId) {
        self.mounted.borrow_mut().push(view);
    }

    pub fn unmount(&self, view: ViewId) {
        self.unmounted.borrow_mut().push(view);
    }

    pub fn perform_remove(&self) {
        let focused = mem::take(&mut *self.focused.borrow_mut());
        let mut focused = HashSet::<ViewId>::from_iter(focused.into_iter());
        for id in self.unmounted.borrow().iter() {
            self.hovered.borrow_mut().remove(&id);
            focused.remove(id);
        }
        for item in focused.into_iter() {
            let _ = self.focused.borrow_mut().insert(item);
        }

        for id in self.unmounted.borrow().iter() {
            id.unmounted();
        }

        for scope in self.disposed.borrow().iter() {
            scope.dispose();
        }

        for id in self.unmounted.borrow().iter() {
            id.remove();
        }

        self.unmounted.borrow_mut().clear();
    }
}

struct CursorElement {
    root: ViewId,
    position: Vec2,
    cursor: Cursor,
    image: Option<Box<dyn Drawable>>,
    target: Option<ViewId>,
    pub inspect: bool,
}

impl CursorElement {
    fn new(root: ViewId) -> Self {
        Self {
            root,
            cursor: Cursor::system_default(),
            position: Default::default(),
            image: Default::default(),
            target: Default::default(),
            inspect: false,
        }
    }

    fn set_inspect(&mut self, inspect: bool) {
        self.inspect = inspect;
    }
}

impl Drawable for CursorElement {
    fn draw(&self, cx: &Renderer) {
        if self.inspect {
            if let Some(id) = self.target {
                let layout = id.layout().unwrap();
                let state = id.state();
                let viewport = state.borrow().viewport;
                let location = layout.location + viewport;
                let size = layout.size;
                let position = vec2(location.x, location.y);
                let size = vec2(size.width, size.height);
                cx.fill_rect(Color::BLUE.multiply_alpha(0.5), position, size);
            }
        }

        if let Some(cursor) = self.image.as_ref() {
            cursor.draw(cx);
        }
    }

    fn size(&self) -> Vec2 {
        Vec2::ZERO
    }

    fn update(&mut self, delta: u64) {
        let mut target = None;
        let position = unsafe {
            let mut x = MaybeUninit::uninit();
            let mut y = MaybeUninit::uninit();
            SDL_GetMouseState(x.as_mut_ptr(), y.as_mut_ptr());
            vec2(x.assume_init(), y.assume_init())
        };

        self.root.event_capture(position, &mut target);
        self.target = target;

        if let Some(target) = target {
            let state = target.state();
            let cursor = state.borrow().style.cursor.clone();
            if self.cursor != cursor {
                match &cursor {
                    Cursor::None => {
                        unsafe { SDL_HideCursor() };
                        self.image = None;
                    }
                    Cursor::System(cursor) => {
                        unsafe {
                            SDL_ShowCursor();
                            SDL_SetCursor(cursor.cursor);
                        };
                        self.image = None;
                    }
                    Cursor::Drawable(f) => {
                        unsafe { SDL_HideCursor() };
                        self.image = Some(f.0());
                    }
                    _ => {}
                }
                self.cursor = cursor
            }
        } else {
        }

        if let Some(image) = &mut self.image {
            image.set_bounds(Bounds {
                position,
                size: image.size(),
            });
            image.update(delta);
        }
    }
}

pub struct Root {
    view: View,
    size: RwSignal<Vec2>,
    painter: Renderer,
    update_event: EventEmitter,
    renderer: *mut SDL_Renderer,
    window: *mut SDL_Window,
    event_manager: EventDispatcher,
    id: ViewId,
    current_cursor: Cursor,
    cursor_element: CursorElement,
}

impl Root {
    pub fn new<F, E>(f: F, renderer: *mut SDL_Renderer) -> Self
    where
        F: 'static + Fn() -> E,
        E: IntoElement,
    {
        let window = unsafe { SDL_GetRenderWindow(renderer) };
        let size = RwSignal::new(unsafe {
            let mut x = 0;
            let mut y = 0;
            SDL_GetWindowSize(window, &mut x, &mut y);
            vec2(x as f32, y as f32)
        });

        let id = ViewId::new();
        id.state().borrow_mut().mounted = true;

        let update_event = EventEmitter::new();
        let event_manager: EventDispatcher = Default::default();

        provide_context(window);
        provide_context(renderer);
        provide_context(update_event.clone());
        provide_context(id);
        provide_context(event_manager.clone());

        let cursor_element = CursorElement::new(id);

        let view = View::new(id, f())
            .style(move |s| s.width(length(size.get().x)).height(length(size.get().y)));

        Self {
            id,
            view,
            size,
            painter: Renderer::new(renderer),
            update_event,
            renderer,
            window,
            event_manager,
            cursor_element,
            current_cursor: Cursor::system_default(),
        }
    }

    pub fn dispatch_event(&mut self, event: &SDL_Event) {
        unsafe {
            match SDL_EventType(event.r#type) {
                SDL_EventType::WINDOW_RESIZED => {
                    self.size
                        .set(vec2(event.window.data1 as f32, event.window.data2 as f32));
                }
                SDL_EventType::KEY_DOWN => {
                    if (event.key.key == SDLK_D) {
                        self.cursor_element
                            .set_inspect(!self.cursor_element.inspect);
                    }
                }
                _ => {}
            }
        }

        let id = self.view.id();

        unsafe {
            match SDL_EventType(event.r#type) {
                SDL_EventType::MOUSE_MOTION => {
                    self.event_manager.handle_event(
                        MouseMotionEvent {
                            r#type: EventType::MouseMove,
                            event_target: EventTarget::new(id),
                            motion: vec2(event.motion.x, event.motion.y),
                        }
                        .as_any_mut(),
                        self.view.id(),
                    );
                }
                SDL_EventType::MOUSE_BUTTON_DOWN => {
                    self.event_manager.handle_event(
                        MouseMotionEvent {
                            r#type: EventType::MouseDown,
                            event_target: EventTarget::new(id),
                            motion: vec2(event.motion.x, event.motion.y),
                        }
                        .as_any_mut(),
                        id,
                    );
                }
                SDL_EventType::MOUSE_BUTTON_UP => self.event_manager.handle_event(
                    MouseMotionEvent {
                        r#type: EventType::MouseUp,
                        event_target: EventTarget::new(id),
                        motion: vec2(event.motion.x, event.motion.y),
                    }
                    .as_any_mut(),
                    self.view.id(),
                ),
                SDL_EventType::KEY_DOWN => self.event_manager.handle_event(
                    KeyboardEvent {
                        r#type: EventType::KeyDown,
                        event_target: EventTarget::new(id),
                        key: event.key.key,
                        r#mod: event.key.r#mod,
                        scancode: event.key.scancode,
                    }
                    .as_any_mut(),
                    self.view.id(),
                ),
                SDL_EventType::KEY_UP => self.event_manager.handle_event(
                    KeyboardEvent {
                        r#type: EventType::KeyUp,
                        event_target: EventTarget::new(id),
                        key: event.key.key,
                        r#mod: event.key.r#mod,
                        scancode: event.key.scancode,
                    }
                    .as_any_mut(),
                    self.view.id(),
                ),
                SDL_EventType::TEXT_INPUT => self.event_manager.handle_event(
                    TextInputEvent {
                        r#type: EventType::TextInput,
                        event_target: EventTarget::new(id),
                        text: CStr::from_ptr(event.text.text)
                            .to_str()
                            .unwrap()
                            .to_string(),
                    }
                    .as_any_mut(),
                    self.view.id(),
                ),
                _ => self.event_manager.handle_event(
                    UnhandledEvent {
                        r#type: EventType::Unhandled,
                        event_target: EventTarget::new(id),
                    }
                    .as_any_mut(),
                    self.view.id(),
                ),
            }
        };
    }

    pub fn update(&mut self, delta: u64) {
        self.update_event.emit();
        self.cursor_element.update(delta);
    }

    // TODO: need to optimize performance
    pub fn compute_style(&self) {
        let mut ctx = StyleComputeContext::new();
        self.view.id().compute_style(&mut ctx);
    }

    pub fn compute_layout(&self) {
        let taffy = RUNTIME.with_borrow_mut(|s| s.taffy.clone());
        taffy
            .borrow_mut()
            .compute_layout_with_measure(
                self.view.id().node(),
                Size::MAX_CONTENT,
                move |known_dimensions, available_space, id, _, _| {
                    if let Size {
                        width: Some(width),
                        height: Some(height),
                    } = known_dimensions
                    {
                        return Size { width, height };
                    }

                    let element = RUNTIME.with_borrow_mut(|s| s.elements.get(id.into()).cloned());

                    element
                        .map(|e| e.measure(known_dimensions, available_space))
                        .unwrap_or_default()
                },
            )
            .unwrap();

        compute_layout(&mut taffy.borrow_mut(), self.view.id().node(), Point::ZERO);
    }

    pub fn paint(&self) {
        self.view.paint(&self.painter);
        self.cursor_element.draw(&self.painter);
    }

    pub fn launch(&mut self) {
        let renderer = self.renderer;
        let mut events = PollEvent::new();

        unsafe {
            SDL_SetRenderVSync(renderer, 1);

            let mut exited = false;
            let mut prev = unsafe { SDL_GetTicks() };
            while !exited {
                for event in &mut events {
                    self.dispatch_event(event);
                    match SDL_EventType(event.r#type) {
                        SDL_EventType::QUIT => {
                            exited = true;
                        }
                        _ => {}
                    }
                }

                self.event_manager.perform_remove();

                self.compute_style();
                self.compute_layout();

                let current = unsafe { SDL_GetTicks() };
                self.update(current - prev);
                prev = current;

                SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
                SDL_RenderClear(renderer);

                self.paint();

                SDL_RenderPresent(renderer);

                for id in mem::take(&mut *self.event_manager.mounted.borrow_mut()) {
                    id.mounted()
                }

                SDL_Delay(16);
            }
        }
    }
}
