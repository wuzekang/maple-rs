use crate::event::{Event, Interactive};
use crate::{
    element::Element,
    sdl::{ImageTexture, Painter},
    style::StyleBuilder,
    view_id::ViewId,
};
use glam::Vec2;
use image::DynamicImage;
use reactive::{create_effect, use_context};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use taffy::{AvailableSpace, Size};

enum ImageState {
    None,
    Loading(DynamicImage),
    Loaded(ImageTexture),
}

pub trait Drawable {
    fn draw(&self, id: ViewId);
    fn size(&self) -> Vec2;
    fn update(&mut self) {}
    fn event(&mut self, event: &Event) {}
}

pub trait IntoDrawable: Sized {
    fn into_drawable(self) -> Rc<RefCell<Box<dyn Drawable>>>;
}

impl<T: Drawable + 'static> IntoDrawable for Box<dyn (Fn() -> T) + 'static> {
    fn into_drawable(self) -> Rc<RefCell<Box<dyn Drawable>>> {
        let image: Rc<RefCell<Box<dyn Drawable>>> = Rc::new(RefCell::new(Box::new(self())));

        create_effect({
            let image = image.clone();
            move |_| *image.borrow_mut() = Box::new(self())
        });
        image.clone()
    }
}

impl<T: Drawable + 'static> IntoDrawable for T {
    fn into_drawable(self) -> Rc<RefCell<Box<dyn Drawable>>> {
        Rc::new(RefCell::new(Box::new(self)))
    }
}

impl IntoDrawable for Rc<RefCell<Box<dyn Drawable>>> {
    fn into_drawable(self) -> Rc<RefCell<Box<dyn Drawable>>> {
        self
    }
}

impl IntoDrawable for Arc<DynamicImage> {
    fn into_drawable(self) -> Rc<RefCell<Box<dyn Drawable>>> {
        let renderer: *mut sdl3_sys::render::SDL_Renderer = use_context().unwrap();
        Rc::new(RefCell::new(Box::new(ImageTexture::new(renderer, &self))))
    }
}

pub struct Image {
    id: ViewId,
    state: Rc<RefCell<Box<dyn Drawable>>>,
}
impl Image {
    pub fn new(image: impl IntoDrawable) -> Self {
        let id = ViewId::new();
        let state = image.into_drawable();
        let s = state.clone();
        let _ = id.add_event_listener(Box::new(move |event| {
            s.borrow_mut().event(&event);
        }));
        Self { id, state }
    }

    pub fn style<F: Fn(StyleBuilder) -> StyleBuilder + 'static>(self, f: F) -> Self {
        create_effect(move |_| {
            let id = self.id;
            let state = id.state();
            let node = id.node();
            let style = f(StyleBuilder::default());
            id.taffy()
                .borrow_mut()
                .set_style(node, style.taffy_style.clone())
                .unwrap();
            state.borrow_mut().style = style.style.clone();
        });
        self
    }
}

impl Element for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&self, cx: &Painter) {
        self.state.borrow().draw(self.id);
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let image_size = self.state.borrow().size();
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
    }
}

impl Interactive for Image {}
