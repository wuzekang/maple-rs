use crate::wz::Node;
use glam::Vec2;
use image::DynamicImage;
use std::sync::Arc;
use ::ui::geometry::Rect;
use ::ui::{Element, Interactive, Renderer, ViewId};
use ::ui::style::Styleable;

pub struct Tiled {
    id: ViewId,
    image: Arc<DynamicImage>,
}

impl Tiled {
    pub fn new(node: Node) -> Self {
        let image: Arc<DynamicImage> = node.try_into().unwrap();
        Tiled {
            id: ViewId::new(),
            image,
        }
    }
}

pub fn tiled(node: Node) -> Tiled {
    Tiled::new(node)
}

impl Element for Tiled {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "Tiled".into()
    }

    fn paint(&self, ctx: &mut Renderer) {
        let rect = self.id.layout_rect();
        let texture = ctx.texture(&self.image);
        ctx.render_texture(
            texture.ptr(),
            Rect::from((Vec2::ZERO, texture.size)),
            rect,
            None,
            true,
        );
    }
}

impl Interactive for Tiled {}
impl Styleable for Tiled {}