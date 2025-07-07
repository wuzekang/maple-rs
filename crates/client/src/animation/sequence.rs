use crate::sprite::ASpriteAnimation;
use glam::Vec2;
use ::ui::{Bounds, Drawable, Renderer};

pub struct SequenceAnimation {
    pub index: usize,
    pub animations: Vec<ASpriteAnimation>,
    pub complete: bool,
    pub bounds: Bounds,
    pub listeners: Vec<Box<dyn Fn()>>,
}

impl SequenceAnimation {
    pub fn new(animations: Vec<ASpriteAnimation>) -> Self {
        Self {
            index: 0,
            animations,
            complete: false,
            bounds: Bounds::default(),
            listeners: Vec::new(),
        }
    }

    pub fn on_complete(&mut self, f: impl Fn() + 'static) {
        self.listeners.push(Box::new(f));
    }
}

impl Drawable for SequenceAnimation {
    fn draw(&self, painter: &mut Renderer) {
        let animation = &self.animations[self.index];
        animation.draw(painter);
    }

    fn size(&self) -> Vec2 {
        self.animations[self.index].size()
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, delta: f32) -> bool {
        if self.complete {
            return false;
        }
        if self.animations.is_empty() {
            return false;
        }
        let animation = &mut self.animations[self.index];
        let updated = animation.update(delta);
        let index = self.index;
        if animation.complete {
            self.index += 1;
        }
        if self.index >= self.animations.len() {
            self.complete = true;
            self.index = self.animations.len() - 1;
            for f in self.listeners.iter() {
                f();
            }
        }
        self.animations[self.index].set_bounds(self.bounds.clone());

        self.index != index || updated
    }
}