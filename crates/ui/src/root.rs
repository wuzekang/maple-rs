use crate::element::{Element, IntoElement};
use crate::event::{
    Event, EventIterator, FocusEvent, FocusEventType, LifecycleEvent, LifecycleEventType,
    MouseEventType,
};
use crate::geometry::Rect;
use crate::render::renderer::Renderer;
use crate::resource::Resource;
use crate::runtime::RUNTIME;
use crate::style::{compute_layout, Cursor, StyleComputeContext, Styleable};
use crate::view_id::ViewId;
use crate::widget::focus_trap::FocusTrap;
use crate::{input, Bounds, Drawable};
use bumpalo::Bump;
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{provide_context, use_context, RwSignal, Scope, SignalGet, SignalUpdate};
use sdl3_sys::everything::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::mem;
use std::rc::Rc;
use taffy::{prelude::TaffyMaxContent, Point, Size};
use std::time::Duration;


#[derive(Clone)]
pub struct AppContext {
    root: ViewId,
    hovered: Rc<RefCell<HashSet<ViewId>>>,
    pub focused: Rc<RefCell<Option<ViewId>>>,
    disposed: Rc<RefCell<Vec<Scope>>>,
    mounted: Rc<RefCell<Vec<(ViewId, ViewId)>>>,
    unmounted: Rc<RefCell<Vec<(ViewId, ViewId)>>>,
    queue: Rc<RefCell<Vec<Event>>>,
    style_dirty: Rc<RefCell<HashSet<ViewId>>>,
    pub inspect_element: RwSignal<Option<ViewId>>,
}

impl AppContext {
    pub fn new(root: ViewId) -> Self {
        Self {
            root,
            hovered: Rc::new(RefCell::new(Default::default())),
            focused: Rc::new(RefCell::new(None)),
            disposed: Rc::new(RefCell::new(vec![])),
            mounted: Rc::new(RefCell::new(vec![])),
            unmounted: Rc::new(RefCell::new(vec![])),
            queue: Rc::new(RefCell::new(vec![])),
            style_dirty: Rc::new(RefCell::new(Default::default())),
            inspect_element: RwSignal::new(None),
        }
    }

    pub fn focus(&self, target: ViewId) {
        if let Some(focused) = self.focused.borrow().as_ref() {
            if target == *focused {
                return;
            }
        }

        let focused = mem::take(&mut *self.focused.borrow_mut());
        let _ = self.focused.borrow_mut().insert(target);
        let prev = HashSet::<ViewId>::from_iter(focused);
        let next = HashSet::<ViewId>::from_iter(*self.focused.borrow());
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

    pub fn dispatch(&self, event: Event) {
        let root = self.root;
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
                let entered = &*self.hovered.borrow() - &prev;
                let leaved = &prev - &*self.hovered.borrow();

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
        } else if let Event::TextInput(event) = event {
            let id = self.focused.borrow().unwrap_or(root);
            id.dispatch_event(&mut Event::TextInput(event), false);
        } else if let Event::Focus(event) = event {
            self.focus(event.target);
        }
    }

    pub fn mount(&self, id: ViewId, parent: ViewId) {
        self.mounted.borrow_mut().push((id, parent));
    }

    pub fn request_style(&self, id: ViewId) {
        if !id.state().borrow().mounted {
            return;
        }
        self.style_dirty.borrow_mut().insert(id);
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
        let mut focused = HashSet::<ViewId>::from_iter(focused);
        for (id, _) in self.unmounted.borrow().iter() {
            self.hovered.borrow_mut().remove(id);
            self.style_dirty.borrow_mut().remove(id);
            if self.inspect_element.get_untracked() == Some(*id) {
                self.inspect_element.set(None);
            }
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

    pub fn process_queue(&self) {
        let queue = mem::take(&mut *self.queue.borrow_mut());
        for event in queue.into_iter() {
            self.dispatch(event);
        }
    }
}

struct CursorElement {
    root: ViewId,
    cursor: Cursor,
    image: Option<Box<dyn Drawable>>,
    target: Option<ViewId>,
    position: Vec2,
    pub inspect: bool,
    pub inspect_element: RwSignal<Option<ViewId>>,
    pub cursor_visible: bool,
}

impl CursorElement {
    fn new(root: ViewId, inspect_element: RwSignal<Option<ViewId>>) -> Self {
        Self {
            root,
            cursor: Cursor::DEFAULT,
            image: Default::default(),
            target: Default::default(),
            position: Default::default(),
            inspect: false,
            inspect_element,
            cursor_visible: false,
        }
    }

    fn set_inspect(&mut self, inspect: bool) {
        self.inspect = inspect;
    }

    fn event(&mut self, event: &mut Event) {

        if event.is_mouse_enter().is_some() {
            self.cursor_visible = true;
        } else if event.is_mouse_leave().is_some() {
            self.cursor_visible = false;
        }

        if let Some(event) = event.is_mouse_move() {
            let position = event.motion;
            self.position = position;
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
                                SDL_SetCursor(cursor.cursor());
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
            }
        }
    }
}

impl Drawable for CursorElement {
    fn draw(&self, cx: &mut Renderer) {
        if let Some(id) = self.inspect_element.get_untracked() {
            let layout = id.layout();
            let state = id.state();
            let viewport = state.borrow().viewport;
            let location = layout.location + viewport;
            let size = layout.size;
            let position = vec2(location.x, location.y);
            let size = vec2(size.width, size.height);
            cx.fill_rect(Color::BLACK.with_alpha(0.2), Rect::from((position, size)));
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

        if self.cursor_visible {
            if let Some(image) = &mut self.image {
                image.set_bounds(Bounds {
                    position: self.position,
                    size: image.size(),
                });
                image.update(delta);
            }
        }

        true
    }
}

pub struct Root {
    size: RwSignal<Vec2>,
    painter: Rc<RefCell<Renderer>>,
    renderer: *mut SDL_Renderer,
    app_context: AppContext,
    bump: Bump,
    id: ViewId,
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

        let dpr = unsafe { SDL_GetWindowPixelDensity(window) };
        let app_context = AppContext::new(id);
        let painter = Rc::new(RefCell::new(Renderer::new(
            renderer,
            dpr,
            size.get_untracked(),
        )));

        provide_context(window);
        provide_context(renderer);
        provide_context(painter.clone());
        provide_context(id);
        provide_context(app_context.clone());

        let cursor_element = CursorElement::new(id, app_context.inspect_element);

        let view = view
            .style(move |s| s.width(size.get().x).height(size.get().y))
            .children(f);

        view.into_element();

        let bump = Bump::new();

        Self {
            id,
            size,
            painter,
            renderer,
            bump,
            app_context,
            cursor_element,
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
                    if event.key.key == SDLK_D {
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
        self.cursor_element.event(&mut event);
        self.app_context.dispatch(event);
    }

    pub fn compute_style(&self, nodes: Vec<ViewId>) {
        let mut ctx = StyleComputeContext::new(&self.bump);
        for id in nodes.iter() {
            if ctx.visited.contains(id) {
                continue;
            }
            id.compute_style(&mut ctx);
        }
    }

    pub fn compute_layout(&mut self) {
        let taffy = RUNTIME.with_borrow_mut(|s| s.taffy.clone());
        let ctx = self.painter.clone();
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
                        .map({
                            let ctx = ctx.clone();
                            move |e| {
                                e.borrow().measure(
                                    &mut ctx.borrow_mut(),
                                    known_dimensions,
                                    available_space,
                                )
                            }
                        })
                        .unwrap_or_default()
                },
            )
            .unwrap();

        compute_layout(&mut taffy.borrow_mut(), self.id.node(), Point::ZERO);
    }

    pub fn paint(&mut self) {
        self.id.paint(&mut self.painter.borrow_mut());
        self.cursor_element.draw(&mut self.painter.borrow_mut());
    }

    pub async fn launch(&mut self) {
        let renderer = self.renderer;
        let mut events = EventIterator::new();

        unsafe {
            SDL_SetRenderVSync(renderer, 1);
            //
            // unsafe {
            //     SDL_SetRenderScale(renderer, dpr, dpr);
            // }

            let mut exited = false;
            let mut prev = SDL_GetTicksNS();
            while !exited {

                async_runtime::yield_now().await;
                Resource::try_recv();

                self.app_context.process_queue();

                for event in &mut events {
                    self.dispatch_event(event);
                    if SDL_EventType(event.r#type) == SDL_EventType::QUIT {
                        exited = true;
                    }
                }

                let current = SDL_GetTicksNS();
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

                if !self.app_context.mounted.borrow().is_empty()
                    || !self.app_context.unmounted.borrow().is_empty()
                {
                    let observers = RUNTIME.with_borrow(|r| {
                        let values = r
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
                                    v.iter()
                                        .map(|(_, v, options)| (v.clone(), options.clone()))
                                        .collect::<Vec<_>>(),
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
                        .app_context
                        .mounted
                        .borrow()
                        .iter()
                        .chain(self.app_context.unmounted.borrow().iter())
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

                    while !stack.is_empty() {
                        let id = stack.pop_back().unwrap();
                        let parent = parents.get(&id).cloned().or_else(|| id.parent());
                        if let Some(parent) = parent {
                            if !added.contains(&parent) {
                                added.insert(parent);
                                stack.push_front(parent);
                            }
                        }
                        if let Some(callbacks) = observers.get(&id) {
                            for (callback, options) in callbacks {
                                if options.child_list
                                    && (options.subtree || has_children.contains(&id))
                                {
                                    callback();
                                }
                            }
                        }
                    }
                }

                for (id, _) in self.app_context.mounted.borrow().iter() {
                    self.app_context.style_dirty.borrow_mut().insert(*id);
                }

                self.app_context.perform_detach();

                let mut nodes = Vec::with_capacity(self.app_context.style_dirty.borrow().len());

                for view_id in self.app_context.style_dirty.borrow().iter() {
                    nodes.push((view_id.index_path(self.id), *view_id))
                }

                nodes.sort_by_key(|(path, _)| path.clone());

                let mut ordered = Vec::with_capacity(nodes.len());
                for (_, view_id) in nodes {
                    ordered.push(view_id);
                }

                self.app_context.style_dirty.borrow_mut().clear();

                self.compute_style(ordered);
                self.compute_layout();

                self.cursor_element.update(delta);

                self.app_context.perform_attach();

                SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
                SDL_RenderClear(renderer);

                self.paint();

                self.painter.borrow_mut().present();
                self.bump.reset();
            }
            
        }
        Scope::current().dispose()
    }
}
