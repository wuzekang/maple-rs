use crate::timer::Timer;
use crate::wz::Node;
use glam::{vec2, Vec2};
use image::DynamicImage;
use std::cell::Cell;
use std::sync::Arc;

pub struct Sprite {
    pub path: String,
    pub image: Arc<DynamicImage>,
    pub size: Vec2,
    pub origin: Vec2,
    pub a0: i32,
    pub a1: i32,
    pub alpha: Cell<i32>,
    pub z: i32,
    pub delay: i32,
}

pub struct SpriteAnimation {
    pub frames: Vec<Sprite>,
    pub timer: Timer,
}

impl From<Node> for SpriteAnimation {
    fn from(node: Node) -> Self {
        let frames: Vec<Sprite> = node.into();
        Self {
            timer: Timer::new(frames.iter().map(|frame| frame.delay as f32).collect()),
            frames,
        }
    }
}

impl SpriteAnimation {
    pub fn tick(&self, delta: f32) -> &Sprite {
        self.timer.tick(delta);
        let sprite = &self.frames[self.timer.index.get()];
        let p = self.timer.progress();
        sprite
            .alpha
            .set(((1.0 - p) * sprite.a0 as f32 + p * sprite.a1 as f32) as i32);
        sprite
    }
}

impl From<Node> for Sprite {
    fn from(node: Node) -> Self {
        let image: Arc<DynamicImage> = node.clone().into();
        Self {
            path: node.path(),
            origin: node.get("origin").into(),
            z: node.try_get("z").map(Into::into).unwrap_or(0),
            delay: node.try_get("delay").map(Into::into).unwrap_or(100),
            a0: node.try_get("a0").map(Into::into).unwrap_or(0),
            a1: node.try_get("a1").map(Into::into).unwrap_or(0),
            alpha: 255.into(),
            size: vec2(image.width() as f32, image.height() as f32),
            image,
        }
    }
}
