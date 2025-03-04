use crate::event::Event;
use crate::style::{compute_style_recursive, PointerEvents, StyleComputeContext};
use crate::view_state::Layer;
use crate::{element::Element, runtime::RUNTIME, view_state::ViewState, Renderer};
use glam::{vec2, Vec2};
use reactive::on_cleanup;
use sdl3_sys::everything::*;
use std::cmp::PartialEq;
use std::mem::MaybeUninit;
use std::{cell::RefCell, rc::Rc};
use taffy::Overflow::Hidden;
use taffy::{LengthPercentage, NodeId, Point, TaffyTree};

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

        on_cleanup(move || {
            RUNTIME.with_borrow(|s| s.taffy.borrow_mut().remove(id).unwrap());
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
        listener: Box<dyn (Fn(&mut Event) -> ()) + 'static>,
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
        for child in self.children() {
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
        for child in self.children() {
            child.update(delta);
        }
    }

    pub fn compute_style(&self, ctx: &mut StyleComputeContext) {
        compute_style_recursive(*self, ctx);
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

        ctx.translate(vec2(location.x, location.y));
        ctx.translate(vec2(translate.x, translate.y));

        if overflow.x == Hidden || overflow.y == Hidden {
            ctx.clip(Some((Vec2::ZERO, size)));
        }

        if opacity < 1.0 {
            let layer = self.state().borrow().layer.clone();
            if layer.is_none() {
                let mode = unsafe {
                    SDL_ComposeCustomBlendMode(
                        SDL_BlendFactor::ONE,
                        SDL_BlendFactor::ONE_MINUS_SRC_ALPHA,
                        SDL_BlendOperation::ADD,
                        SDL_BlendFactor::ONE,
                        SDL_BlendFactor::ONE_MINUS_SRC_ALPHA,
                        SDL_BlendOperation::ADD,
                    )
                };

                let texture = Rc::new(
                    ctx.create_texture(size)
                        .blend_mode(mode)
                        .scale_mode_nearest(),
                );

                let mode = unsafe {
                    SDL_ComposeCustomBlendMode(
                        SDL_BlendFactor::ZERO,
                        SDL_BlendFactor::SRC_ALPHA,
                        SDL_BlendOperation::ADD,
                        SDL_BlendFactor::ZERO,
                        SDL_BlendFactor::SRC_ALPHA,
                        SDL_BlendOperation::ADD,
                    )
                };

                let blend_texture =
                    Rc::new(ctx.create_streaming_texture(Vec2::ONE).blend_mode(mode));
                self.state().borrow_mut().layer = Some(Layer {
                    texture,
                    blend_texture,
                });
            }

            let Layer {
                texture,
                blend_texture,
            } = self.state().borrow().layer.clone().unwrap();

            ctx.with_target(&texture, |ctx| {
                ctx.save();
                ctx.reset();

                ctx.clear_texture(size);
                self.element().borrow().paint(ctx);
                for child in self.children() {
                    child.paint(ctx);
                }

                unsafe {
                    let mut surface = MaybeUninit::uninit();

                    SDL_LockTextureToSurface(
                        blend_texture.ptr(),
                        &SDL_Rect {
                            x: 0,
                            y: 0,
                            w: 1,
                            h: 1,
                        },
                        surface.as_mut_ptr(),
                    );

                    let surface = surface.assume_init();

                    SDL_WriteSurfacePixel(surface, 0, 0, 0, 0, 0, (opacity * 255.0) as u8);

                    SDL_UnlockTexture(blend_texture.ptr());

                    SDL_DestroySurface(surface);
                }

                ctx.render_texture(
                    &blend_texture,
                    Vec2::ZERO,
                    Vec2::ZERO,
                    255,
                    Some(size),
                    SDL_FLIP_NONE,
                );

                ctx.restore();
            });

            ctx.render_texture(
                &texture,
                Vec2::ZERO,
                Vec2::ZERO,
                255,
                None,
                SDL_FlipMode::NONE,
            );
        } else {
            self.element().borrow().paint(ctx);
            for child in self.children() {
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
}
