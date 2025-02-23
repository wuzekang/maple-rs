use crate::event::Interactive;
use crate::sdl::{Bounds, Drawable};
use crate::{
    element::Element,
    sdl::{ImageTexture, Renderer},
    style::StyleBuilder,
    view_id::ViewId,
};
use glam::{vec2, Vec2};
use image::DynamicImage;
use reactive::{create_effect, use_context};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use taffy::{AvailableSpace, Size};
use crate::style::Styleable;

enum ImageState {
    None,
    Loading(DynamicImage),
    Loaded(ImageTexture),
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
        let renderer: *mut sdl3_sys::render::SDL_Renderer = use_context().unwrap();
        Box::new(ImageTexture::new(renderer, &self))
    }
}

impl Drawable for Rc<RefCell<dyn Drawable>> {
    fn draw(&self, painter: &Renderer) {
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
        Self {
            id,
            drawable: Rc::new(RefCell::new(Some(drawable.into_drawable()))),
        }
    }

    pub fn dynamic<T, D>(f: T) -> Self
    where
        T: Fn() -> D + 'static,
        D: IntoDrawable,
    {
        let drawable: Rc<RefCell<Option<Box<dyn Drawable>>>> = Default::default();
        create_effect({
            let drawable = drawable.clone();
            move |_| *drawable.borrow_mut() = Some(f().into_drawable())
        });
        Self {
            id: ViewId::new(),
            drawable,
        }
    }


}

impl Element for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&self, cx: &Renderer) {
        if let Some(drawable) = self.drawable.borrow_mut().as_mut() {
            let id = self.id();
            let layout = id.layout().unwrap();
            let state = id.state();
            let viewport = state.borrow().viewport;
            let location = layout.location + viewport;
            let size = layout.size;
            let position = vec2(location.x, location.y);
            let size = vec2(size.width, size.height);
            drawable.set_bounds(Bounds { position, size });
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