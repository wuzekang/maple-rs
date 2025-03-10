use crate::animation::use_raf;
use crate::event::Interactive;
use crate::sdl::{Bounds, Drawable};
use crate::style::Styleable;
use crate::{
    element::Element,
    sdl::{DynamicImageDrawable, Renderer},
    view_id::ViewId,
};
use glam::{vec2, Vec2};
use image::DynamicImage;
use reactive::{create_effect, use_context};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use taffy::{AvailableSpace, Size};

enum ImageState {
    None,
    Loading(DynamicImage),
    Loaded(DynamicImageDrawable),
}

pub trait IntoDrawable: Sized {
    fn into_drawable(self) -> Box<dyn Drawable>;
}

impl<T: Drawable + 'static> IntoDrawable for T {
    fn into_drawable(self) -> Box<dyn Drawable> {
        Box::new(self)
    }
}

impl IntoDrawable for Arc<DynamicImage> {
    fn into_drawable(self) -> Box<dyn Drawable> {
        Box::new(DynamicImageDrawable::new(self))
    }
}

impl Drawable for Rc<RefCell<dyn Drawable>> {
    fn draw(&self, painter: &mut Renderer) {
        self.borrow().draw(painter);
    }

    fn size(&self) -> Vec2 {
        self.borrow().size()
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.borrow_mut().set_bounds(bounds);
    }
}

pub struct Image {
    id: ViewId,
    drawable: Rc<RefCell<Option<Box<dyn Drawable>>>>,
}
impl Image {
    pub fn new(drawable: impl IntoDrawable) -> Self {
        let id = ViewId::new();
        let drawable = Rc::new(RefCell::new(Some(drawable.into_drawable())));
        use_raf({
            let drawable = drawable.clone();
            move |delta| {
                if let Some(drawable) = drawable.borrow_mut().as_mut() {
                    if (drawable.update(delta)) {
                        id.request_repaint();
                    }
                }
            }
        });
        Self { id, drawable }
    }

    pub fn dynamic<T, D>(f: T) -> Self
    where
        T: Fn() -> Option<D> + 'static,
        D: IntoDrawable,
    {
        let id = ViewId::new();
        let drawable: Rc<RefCell<Option<Box<dyn Drawable>>>> = Default::default();
        create_effect({
            let drawable = drawable.clone();
            move |_| {
                id.request_repaint();
                *drawable.borrow_mut() = f().map(IntoDrawable::into_drawable)
            }
        });
        use_raf({
            let drawable = drawable.clone();
            move |delta| {
                if let Some(drawable) = drawable.borrow_mut().as_mut() {
                     if (drawable.update(delta)) {
                         id.request_repaint();
                     }
                }
            }
        });
        Self {
            id,
            drawable,
        }
    }
}

impl Element for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "Image".to_string()
    }
    
    fn paint(&self, cx: &mut Renderer) {
        if let Some(drawable) = self.drawable.borrow_mut().as_mut() {
            let id = self.id();
            let layout = id.layout();
            let size = layout.size;
            let size = vec2(size.width, size.height);
            drawable.set_bounds(Bounds {
                position: Vec2::ZERO,
                size,
            });
            drawable.draw(cx);
        }
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        if let Some(image) = self.drawable.borrow().as_ref() {
            let image_size = image.size();
            match (known_dimensions.width, known_dimensions.height) {
                (Some(width), Some(height)) => Size { width, height },
                (Some(width), None) => Size {
                    width,
                    height: (width / image_size.x) * image_size.y,
                },
                (None, Some(height)) => Size {
                    width: (height / image_size.y) * image_size.x,
                    height,
                },
                (None, None) => Size {
                    width: image_size.x,
                    height: image_size.y,
                },
            }
        } else {
            Size::ZERO
        }
    }
}

impl Interactive for Image {}

impl Styleable for Image {}
