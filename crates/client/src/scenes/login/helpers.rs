use crate::sprite::{Sprite, SpriteDrawable};
use crate::wz;
use glam::Vec2;
use sdl3_sys::everything::SDL_FlipMode;
use ::ui::geometry::Rect;
use ::ui::widget::image::IntoDrawable as UIIntoDrawable;
use ::ui::{Bounds, Drawable, Renderer};

pub trait IntoDrawable {
    fn into_drawable(self) -> Box<dyn Drawable>;
}

impl IntoDrawable for wz::Node {
    fn into_drawable(self) -> Box<dyn Drawable> {
        let sprite: Sprite = self.try_into().unwrap();
        Box::new(SpriteDrawable {
            sprite,
            bounds: Bounds::default(),
        })
    }
}

pub struct ClipImage {
    pub sprite: Sprite,
    pub clip: Vec2,
}

impl ClipImage {
    pub fn new(node: wz::Node) -> Self {
        let sprite: Sprite = node.try_into().unwrap();

        Self {
            clip: sprite.size,
            sprite,
        }
    }
}

impl Drawable for ClipImage {
    fn draw(&self, ctx: &mut Renderer) {
        let sprite = &self.sprite;
        let texture = ctx.texture(&sprite.image);
        let clip = self.clip.min(texture.size).max(Vec2::ZERO);
        ctx.render_texture_rotated(
            &texture,
            Rect::from((Vec2::ZERO, clip)),
            Rect::from((-sprite.origin, clip)),
            0.0,
            None,
            SDL_FlipMode::NONE,
        );
    }

    fn size(&self) -> Vec2 {
        self.sprite.size
    }
}