use crate::{
    element::Element,
    sdl::{ImageTexture, Painter},
    style::{Style, StyleBuilder},
    view_id::ViewId,
};
use glam::vec2;
use image::DynamicImage;
use reactive::{create_effect, RwSignal, SignalUpdate, SignalWith};
use sdl3_sys::surface::SDL_FlipMode;
use taffy::{AvailableSpace, Size};

enum ImageState {
    None,
    Loading(DynamicImage),
    Loaded(ImageTexture),
}

pub struct Image {
    id: ViewId,
    state: RwSignal<ImageState>,
}
impl Image {
    pub fn new<F: (Fn() -> DynamicImage) + 'static>(f: F) -> Self {
        let id = ViewId::new();
        let state = RwSignal::new(ImageState::None);

        create_effect(move |_| {
            let content = f();
            state.update(|v| {
                *v = ImageState::Loading(content);
            });
        });
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
            state.borrow_mut().style = Style {
                background: style.background,
                color: style.color,
            };
        });
        self
    }
}

impl Element for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&self, cx: &Painter) {
        let layout = self.id.get_layout().unwrap();
        let state = self.id.state();
        let viewport = state.borrow().viewport;

        let location = layout.location + viewport;
        let size = layout.size;

        if let Some(texture) = self.state.with_untracked(|s| match s {
            ImageState::None => None,
            ImageState::Loading(image) => {
                let texture = cx.create_image_texture(image);
                texture.draw(
                    vec2(location.x, location.y),
                    vec2(0.0, 0.0),
                    0xFF,
                    Some(vec2(size.width, size.height)),
                    SDL_FlipMode::NONE,
                );
                Some(texture)
            }
            ImageState::Loaded(texture) => {
                texture.draw(
                    vec2(location.x, location.y),
                    vec2(0.0, 0.0),
                    0xFF,
                    Some(vec2(size.width, size.height)),
                    SDL_FlipMode::NONE,
                );
                None
            }
        }) {
            self.state.update(|s| {
                *s = ImageState::Loaded(texture);
            });
        };
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        if let Some(image_size) = self.state.with_untracked(|s| match s {
            ImageState::None => None,
            ImageState::Loading(image) => Some(vec2(image.width() as f32, image.height() as f32)),
            ImageState::Loaded(texture) => Some(texture.size),
        }) {
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
            taffy::Size::ZERO
        }
    }
}
