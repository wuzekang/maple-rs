use crate::element::{Element, IntoElement};
use crate::event::{Event, EventEmitter, EventType};
use crate::runtime::RUNTIME;
use crate::sdl::{PollEvent, Renderer};
use crate::style::{Cursor, StyleComputeContext};
use crate::view::View;
use crate::view_id::ViewId;
use crate::{Bounds, Drawable};
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{provide_context, RwSignal, Scope, SignalGet, SignalUpdate};
use sdl3_sys::everything::{SDL_Delay, SDL_GetTicks, SDL_HideCursor, SDL_RenderClear, SDL_RenderPresent, SDL_SetRenderDrawColor, SDL_SetRenderVSync, SDL_ShowCursor};
use sdl3_sys::mouse::SDL_GetMouseState;
use sdl3_sys::{
    events::{SDL_Event, SDL_EventType},
    everything::SDL_Window,
    render::{SDL_GetRenderWindow, SDL_Renderer},
    video::SDL_GetWindowSize,
};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
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
pub struct EventManager {
    hovered: Rc<RefCell<HashSet<ViewId>>>,
    entered: Rc<RefCell<HashSet<ViewId>>>,
    leaved: Rc<RefCell<HashSet<ViewId>>>,
    removed: Rc<RefCell<Vec<(ViewId, Scope)>>>,
}

impl EventManager {
    pub fn handle_event(&self, event: &SDL_Event, root: ViewId) {
        let mut event = Event {
            event_type: EventType::None,
            event,
            target: root,
            current_target: root.into(),
        };

        let mut target: Option<ViewId> = None;
        if event.is_pointer_event() {
            root.event_capture(event.client(), &mut target);
        }

        if let Some(target) = target {
            event.target = target;
        }

        if event.sdl_event_type() == SDL_EventType::MOUSE_MOTION {
            let prev = mem::take(&mut *self.hovered.borrow_mut());
            let mut node = event.target;
            self.hovered.borrow_mut().insert(node);
            while let Some(parent) = node.parent() {
                self.hovered.borrow_mut().insert(parent);
                node = parent;
            }
            *self.entered.borrow_mut() = &*self.hovered.borrow() - &prev;
            *self.leaved.borrow_mut() = &prev - &*self.hovered.borrow();

            event.event_type = EventType::MouseEnter;
            for item in self.entered.take().iter() {
                self.entered.borrow_mut().remove(&item);
                item.dispatch_event(&event, false)
            }
            self.entered.borrow_mut().clear();

            event.event_type = EventType::MouseLeave;
            for item in self.leaved.take().iter() {
                item.dispatch_event(&event, false)
            }
            self.leaved.borrow_mut().clear();

            event.event_type = EventType::MouseMove;
            event.target.dispatch_event(&event, true);
        } else if event.sdl_event_type() == SDL_EventType::MOUSE_BUTTON_DOWN {
            event.event_type = EventType::MouseDown;
            event.target.dispatch_event(&event, true);
        } else {
            event.target.dispatch_event(&event, false);
        }

        //
    }

    pub fn remove(&self, view: ViewId, scope: Scope) {
        self.removed.borrow_mut().push((view, scope));
    }

    pub fn perform_remove(&self) {
        for (id, scope) in self.removed.borrow().iter() {
            self.hovered.borrow_mut().remove(&id);
            self.entered.borrow_mut().remove(&id);
            self.leaved.borrow_mut().remove(&id);
            let child = id.remove();
            for item in child {
                self.hovered.borrow_mut().remove(&item);
                self.entered.borrow_mut().remove(&item);
                self.leaved.borrow_mut().remove(&item);
            }
            scope.dispose();
        }
        self.removed.borrow_mut().clear();
    }
}

struct CursorElement {
    root: ViewId,
    position: Vec2,
    cursor: Cursor,
    image: Option<Box<dyn Drawable>>,
    target: Option<ViewId>,
    inspect: bool,
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
                    Cursor::System(_) => {
                        unsafe { SDL_ShowCursor() };
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
    event_manager: EventManager,
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
        let update_event = EventEmitter::new();
        let event_manager: EventManager = Default::default();

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

    pub fn dispatch_event(&self, event: &SDL_Event) {
        unsafe {
            if SDL_EventType(event.r#type) == SDL_EventType::WINDOW_RESIZED.into() {
                self.size
                    .set(vec2(event.window.data1 as f32, event.window.data2 as f32));
            }
        }

        self.event_manager.handle_event(event, self.view.id());
        self.event_manager.perform_remove();
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

                let current = unsafe { SDL_GetTicks() };
                self.update(current - prev);
                prev = current;

                self.compute_style();
                self.compute_layout();

                SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
                SDL_RenderClear(renderer);

                self.paint();

                SDL_RenderPresent(renderer);
                SDL_Delay(16);
            }
        }
    }
}
