use crate::element::{Element, IntoElement};
use crate::event::{
    Event, EventIterator, FocusEvent, FocusEventType, LifecycleEvent, LifecycleEventType,
    MouseEventType,
};
use crate::resource::Resource;
use crate::runtime::RUNTIME;
use crate::sdl::Renderer;
use crate::style::dimension::length;
use crate::style::{compute_layout, Cursor, StyleComputeContext, Styleable};
use crate::view_id::ViewId;
use crate::widget::focus_trap::FocusTrap;
use crate::widget::view::View;
use crate::{fragment, input, Bounds, Drawable, Interactive};
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{provide_context, RwSignal, Scope, SignalGet, SignalUpdate};
use sdl3_sys::everything::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::mem;
use std::rc::Rc;
use taffy::{prelude::TaffyMaxContent, Point, Size};

#[derive(Default, Clone)]
pub struct EventDispatcher {
    hovered: Rc<RefCell<HashSet<ViewId>>>,
    pub focused: Rc<RefCell<Option<ViewId>>>,
    disposed: Rc<RefCell<Vec<Scope>>>,
    mounted: Rc<RefCell<Vec<(ViewId, ViewId)>>>,
    unmounted: Rc<RefCell<Vec<(ViewId, ViewId)>>>,
    queue: Rc<RefCell<Vec<Event>>>,
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
            let mut blur_event = Event::Focus(FocusEvent {
                r#type: FocusEventType::Blur,
                target: id,
            });
            id.dispatch_event(&mut blur_event, false);
        }

        for id in &next - &prev {
            let mut focus_event = Event::Focus(FocusEvent {
                r#type: FocusEventType::Focus,
                target: id,
            });
            id.dispatch_event(&mut focus_event, false);
        }
    }

    pub fn dispatch(&self, mut event: Event, root: ViewId) {
        if let Event::Mouse(mut event) = event {
            root.event_capture(event.client(), &mut event.target);
            let target = event.target;

            if event.r#type == MouseEventType::MouseMove {
                let prev = mem::take(&mut *self.hovered.borrow_mut());
                let mut node = event.target;
                self.hovered.borrow_mut().insert(node);
                while let Some(parent) = node.parent() {
                    self.hovered.borrow_mut().insert(parent);
                    node = parent;
                }
                let mut entered = &*self.hovered.borrow() - &prev;
                let mut leaved = &prev - &*self.hovered.borrow();

                event.r#type = MouseEventType::MouseEnter;
                let mut event = Event::Mouse(event);
                for item in entered.iter() {
                    item.dispatch_event(&mut event, false)
                }

                let mut event = match event {
                    Event::Mouse(event) => event,
                    _ => unreachable!(),
                };
                event.r#type = MouseEventType::MouseLeave;
                let mut event = Event::Mouse(event);
                for item in leaved.iter() {
                    item.dispatch_event(&mut event, false)
                }

                let mut event = match event {
                    Event::Mouse(event) => event,
                    _ => unreachable!(),
                };
                event.r#type = MouseEventType::MouseMove;
                let target = event.target;
                let mut event = Event::Mouse(event);
                target.dispatch_event(&mut event, true);
            } else if event.r#type == MouseEventType::MouseLeave {
                let hovered = mem::take(&mut *self.hovered.borrow_mut());
                let mut event = Event::Mouse(event);
                for item in hovered.iter() {
                    item.dispatch_event(&mut event, false)
                }
            } else if event.r#type == MouseEventType::MouseDown {
                let mut event = Event::Mouse(event);
                target.dispatch_event(&mut event, true);
                self.focus(target);
            } else if event.r#type == MouseEventType::MouseUp {
                let mut event = Event::Mouse(event);
                target.dispatch_event(&mut event, true);
            } else if event.r#type == MouseEventType::MouseWheel {
                let mut event = Event::Mouse(event);
                target.dispatch_event(&mut event, true);
            }
        } else if let Event::Keyboard(mut event) = event {
            let id = self.focused.borrow().unwrap_or(root);
            event.target = id;
            id.dispatch_event(&mut Event::Keyboard(event), true);
            return;
        } else if let Event::TextInput(mut event) = event {
            let id = self.focused.borrow().unwrap_or(root);
            id.dispatch_event(&mut Event::TextInput(event), false);
        } else if let Event::Focus(event) = event {
            self.focus(event.target);
        }
    }

    pub fn mount(&self, id: ViewId, parent: ViewId) {
        self.mounted.borrow_mut().push((id, parent));
    }

    pub fn unmount(&self, id: ViewId, parent: ViewId) {
        self.unmounted.borrow_mut().push((id, parent));
    }

    pub fn dispose(&self, scope: Scope) {
        self.disposed.borrow_mut().push(scope);
    }

    pub fn perform_attach(&self) {
        for (id, _) in mem::take(&mut *self.mounted.borrow_mut()) {
            id.dispatch_event(
                &mut Event::Lifecycle(LifecycleEvent {
                    r#type: LifecycleEventType::Attach,
                }),
                false,
            );
        }
    }

    pub fn perform_detach(&self) {
        let focused = mem::take(&mut *self.focused.borrow_mut());
        let mut focused = HashSet::<ViewId>::from_iter(focused.into_iter());
        for (id, _) in self.unmounted.borrow().iter() {
            self.hovered.borrow_mut().remove(&id);
            if focused.remove(id) {
                id.dispatch_event(
                    &mut Event::Focus(FocusEvent {
                        r#type: FocusEventType::Blur,
                        target: *id,
                    }),
                    false,
                );
            }
        }
        for item in focused.into_iter() {
            let _ = self.focused.borrow_mut().insert(item);
        }

        for (id, _) in self.unmounted.borrow().iter() {
            id.dispatch_event(
                &mut Event::Lifecycle(LifecycleEvent {
                    r#type: LifecycleEventType::Detach,
                }),
                false,
            );
        }

        for scope in self.disposed.borrow().iter() {
            scope.dispose();
        }

        self.unmounted.borrow_mut().clear();
        self.disposed.borrow_mut().clear();
    }

    pub fn queue(&self, event: Event) {
        self.queue.borrow_mut().push(event);
    }

    pub fn process_queue(&self, root: ViewId) {
        let queue = mem::take(&mut *self.queue.borrow_mut());
        for event in queue.into_iter() {
            self.dispatch(event, root);
        }
    }
}

struct CursorElement {
    root: ViewId,
    position: Vec2,
    cursor: Cursor,
    image: Option<Box<dyn Drawable>>,
    target: Option<ViewId>,
    pub inspect: bool,
    pub cursor_visible: bool,
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
            cursor_visible: false,
        }
    }

    fn set_inspect(&mut self, inspect: bool) {
        self.inspect = inspect;
    }
}

impl Drawable for CursorElement {
    fn draw(&self, cx: &mut Renderer) {
        if self.inspect {
            if let Some(id) = self.target {
                let layout = id.layout();
                let state = id.state();
                let viewport = state.borrow().viewport;
                let location = layout.location + viewport;
                let size = layout.size;
                let position = vec2(location.x, location.y);
                let size = vec2(size.width, size.height);
                cx.fill_rect(Color::BLUE.multiply_alpha(0.5), position, size);
            }
        }

        if self.cursor_visible {
            if let Some(cursor) = self.image.as_ref() {
                cursor.draw(cx);
            }
        }
    }

    fn size(&self) -> Vec2 {
        Vec2::ZERO
    }

    fn update(&mut self, delta: f32) -> bool {
        let position = input::mouse_position();

        let mut target = self.root;
        self.root.event_capture(position, &mut target);
        self.target = Some(target);

        if let Some(target) = self.target {
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
        true
    }
}

pub struct Root {
    size: RwSignal<Vec2>,
    painter: Renderer,
    renderer: *mut SDL_Renderer,
    window: *mut SDL_Window,
    event_dispatcher: EventDispatcher,
    id: ViewId,
    current_cursor: Cursor,
    cursor_element: CursorElement,
}

impl Root {
    pub fn new<F, E>(f: F, renderer: *mut SDL_Renderer) -> Self
    where
        F: 'static + Fn() -> E,
        E: IntoElement + 'static,
    {
        let window = unsafe { SDL_GetRenderWindow(renderer) };
        let size = RwSignal::new(unsafe {
            let mut x = 0;
            let mut y = 0;
            SDL_GetWindowSize(window, &mut x, &mut y);
            vec2(x as f32, y as f32)
        });

        let view = FocusTrap::new();
        let id = view.id();
        id.state().borrow_mut().mounted = true;

        let event_manager: EventDispatcher = Default::default();

        provide_context(window);
        provide_context(renderer);
        provide_context(id);
        provide_context(event_manager.clone());

        let cursor_element = CursorElement::new(id);

        let view = view
            .style(move |s| s.width(length(size.get().x)).height(length(size.get().y)))
            .children(f);

        view.into_element();

        let dpr = unsafe { SDL_GetWindowPixelDensity(window) };

        Self {
            id,
            size,
            painter: Renderer::new(renderer, dpr, size.get_untracked()),
            renderer,
            window,
            event_dispatcher: event_manager,
            cursor_element,
            current_cursor: Cursor::system_default(),
        }
    }

    pub fn dispatch_event(&mut self, event: &SDL_Event) {
        let event_type = unsafe { SDL_EventType(event.r#type) };

        unsafe {
            match event_type {
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

        if let Ok(event) = Event::try_from((event, self.id)) {
            self.handle_event(event)
        }
    }

    pub fn handle_event(&mut self, mut event: Event) {
        if event.is_mouse_enter().is_some() {
            self.cursor_element.cursor_visible = true;
        } else if event.is_mouse_leave().is_some() {
            self.cursor_element.cursor_visible = false;
        }
        self.event_dispatcher.dispatch(event, self.id);
    }

    pub fn compute_style(&self) {
        let mut ctx = StyleComputeContext::new();
        self.id.compute_style(&mut ctx);
    }

    pub fn compute_layout(&self) {
        let taffy = RUNTIME.with_borrow_mut(|s| s.taffy.clone());
        taffy
            .borrow_mut()
            .compute_layout_with_measure(
                self.id.node(),
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
                        .map(|e| e.borrow().measure(known_dimensions, available_space))
                        .unwrap_or_default()
                },
            )
            .unwrap();

        compute_layout(&mut taffy.borrow_mut(), self.id.node(), Point::ZERO);
    }

    pub fn paint(&mut self) {
        self.id.paint(&mut self.painter);
        self.cursor_element.draw(&mut self.painter);
    }

    pub fn launch(&mut self) {
        let renderer = self.renderer;
        let window = self.window;
        let mut events = EventIterator::new();

        unsafe {
            SDL_SetRenderVSync(renderer, 1);
            //
            // unsafe {
            //     SDL_SetRenderScale(renderer, dpr, dpr);
            // }

            let mut exited = false;
            let mut prev = unsafe { SDL_GetTicksNS() };
            while !exited {
                Resource::try_recv();

                self.event_dispatcher.process_queue(self.id);

                for event in &mut events {
                    self.dispatch_event(event);
                    match SDL_EventType(event.r#type) {
                        SDL_EventType::QUIT => {
                            exited = true;
                        }
                        _ => {}
                    }
                }

                let current = unsafe { SDL_GetTicksNS() };
                let delta = (current - prev) as f32 / 1000000.0;
                prev = current;

                let mut vec = RUNTIME
                    .with_borrow_mut(|s| mem::take(&mut *s.animation_frame_callbacks.borrow_mut()));

                for callback in vec.values() {
                    callback(delta)
                }

                RUNTIME.with_borrow_mut(|s| {
                    let pending = mem::take(&mut *s.animation_frame_callbacks.borrow_mut());
                    for (key, value) in pending {
                        vec.insert(key, value);
                    }
                    *s.animation_frame_callbacks.borrow_mut() = vec;
                });

                self.id.update(delta);

                if self.event_dispatcher.mounted.borrow().len() > 0
                    || self.event_dispatcher.unmounted.borrow().len() > 0
                {
                    let observers = RUNTIME.with_borrow(|r| {
                        let mut values = r
                            .mutation_observers
                            .borrow()
                            .values()
                            .cloned()
                            .collect::<Vec<_>>();

                        let observers = values
                            .chunk_by(|a, b| a.0 == b.0)
                            .map(|v| {
                                (
                                    v[0].0,
                                    v.into_iter().map(|(_, v)| v.clone()).collect::<Vec<_>>(),
                                )
                            })
                            .collect::<HashMap<_, _>>();

                        observers
                    });

                    let mut nodes = HashSet::new();
                    let mut parents = HashMap::new();
                    let mut has_children = HashSet::new();
                    let mut stack = VecDeque::new();
                    let mut added = HashSet::new();

                    for (id, parent) in self
                        .event_dispatcher
                        .mounted
                        .borrow()
                        .iter()
                        .chain(self.event_dispatcher.unmounted.borrow().iter())
                    {
                        nodes.insert(*id);
                        parents.insert(*id, *parent);
                        has_children.insert(*parent);
                    }

                    for id in &nodes {
                        if has_children.contains(id) {
                            continue;
                        }
                        stack.push_back(*id);
                    }

                    while stack.len() > 0 {
                        let id = stack.pop_back().unwrap();
                        let parent = parents.get(&id).cloned().or_else(|| id.parent());
                        if let Some(parent) = parent {
                            if !added.contains(&parent) {
                                added.insert(parent);
                                stack.push_front(parent);
                            }
                        }
                        if let Some(callbacks) = observers.get(&id) {
                            for callback in callbacks {
                                callback();
                            }
                        }
                    }
                }

                self.event_dispatcher.perform_detach();

                self.compute_style();
                self.compute_layout();

                self.cursor_element.update(delta);

                SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
                SDL_RenderClear(renderer);

                self.paint();

                SDL_RenderPresent(renderer);

                self.event_dispatcher.perform_attach();
            }
        }
    }
}
