use crate::timer::{Repeat, Timer};
use crate::wz::Node;
use glam::{vec2, Vec2};
use image::DynamicImage;
use sdl3_sys::surface::SDL_FlipMode;
use std::sync::Arc;
use ui::{Bounds, Drawable, Renderer, Texture};

pub struct SpriteRenderer<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> SpriteRenderer<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn scale(&mut self, scale: f32) {
        self.renderer.scale(scale);
    }

    pub fn draw(&mut self, sprite: &Sprite, position: Vec2) {
        self.draw_flip(sprite, position, false);
    }

    pub fn draw_flip(&mut self, sprite: &Sprite, position: Vec2, flip: bool) {
        let texture = self.renderer.texture(&sprite.image);
        self.renderer.render_texture(
            &texture,
            position,
            sprite.origin,
            sprite.alpha,
            None,
            if flip {
                SDL_FlipMode::HORIZONTAL
            } else {
                SDL_FlipMode::NONE
            },
        )
    }

    pub fn draw_flip_once(&self, sprite: &ASprite, position: Vec2, flip: bool) {
        let image: Arc<DynamicImage> = sprite.node.clone().try_into().unwrap();
        let texture = Texture::from_image(&image, self.renderer.renderer);
        self.renderer.render_texture(
            &texture,
            position,
            sprite.origin,
            sprite.alpha,
            None,
            if flip {
                SDL_FlipMode::HORIZONTAL
            } else {
                SDL_FlipMode::NONE
            },
        )
    }
}

#[derive(Clone)]
pub struct Sprite {
    pub path: String,
    pub image: Arc<DynamicImage>,
    pub size: Vec2,
    pub origin: Vec2,
    pub a0: i32,
    pub a1: i32,
    pub alpha: i32,
    pub z: i32,
    pub delay: i32,
}

impl TryFrom<Node> for Sprite {
    fn try_from(node: Node) -> Result<Self, ()> {
        let image: Arc<DynamicImage> = node.clone().try_into()?;
        Ok(Self {
            path: node.path(),
            origin: node.get("origin").try_into()?,
            z: node.try_get("z").map(TryInto::try_into).unwrap_or(Ok(0))?,
            delay: node
                .try_get("delay")
                .map(TryInto::try_into)
                .unwrap_or(Ok(100))?,
            a0: node
                .try_get("a0")
                .map(TryInto::try_into)
                .unwrap_or(Ok(255))?,
            a1: node
                .try_get("a1")
                .map(TryInto::try_into)
                .unwrap_or(Ok(255))?,
            alpha: 255.into(),
            size: vec2(image.width() as f32, image.height() as f32),
            image,
        })
    }

    type Error = ();
}

pub struct SpriteDrawable {
    pub sprite: Sprite,
    pub bounds: Bounds,
}

impl Drawable for SpriteDrawable {
    fn draw(&self, renderer: &mut Renderer) {
        let mut sprite_renderer = SpriteRenderer::new(renderer);
        sprite_renderer.draw(&self.sprite, self.bounds.position);
    }

    fn size(&self) -> Vec2 {
        self.sprite.size
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, delta: f32) {}
}

#[derive(Clone)]
pub struct ASprite {
    pub node: Node,
    pub path: String,
    pub size: Vec2,
    pub origin: Vec2,
    pub a0: i32,
    pub a1: i32,
    pub alpha: i32,
    pub z: i32,
    pub delay: i32,
}

impl TryFrom<Node> for ASprite {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            node: node.clone(),
            path: node.path(),
            origin: node.get("origin").try_into()?,
            z: node
                .try_get("z")
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or(0),
            delay: node
                .try_get("delay")
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or(100),
            a0: node
                .try_get("a0")
                .map(TryInto::try_into)
                .unwrap_or(Ok(255))?,
            a1: node
                .try_get("a1")
                .map(TryInto::try_into)
                .unwrap_or(Ok(255))?,
            alpha: 255.into(),
            size: Vec2::ZERO,
        })
    }
}

#[derive(Clone)]
pub struct ASpriteAnimation {
    pub frames: Vec<ASprite>,
    pub timer: Timer,
    pub bounds: Bounds,
    pub complete: bool,
}

impl TryFrom<Node> for ASpriteAnimation {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let frames: Vec<ASprite> = node.try_into()?;
        Ok(Self {
            timer: Timer {
                repeat: Repeat::Finite(1),
                ..Timer::new(frames.iter().map(|frame| frame.delay as f32).collect())
            },
            frames,
            bounds: Bounds::default(),
            complete: false,
        })
    }
}

impl ASpriteAnimation {
    pub fn tick(&mut self, delta: f32) {
        if !self.complete {
            if !self.timer.tick(delta) {
                self.complete = true;
            }
        }
        let sprite = &mut self.frames[self.timer.index];
        let p = self.timer.progress();
        sprite.alpha = (((1.0 - p) * sprite.a0 as f32 + p * sprite.a1 as f32) as i32);
    }

    pub fn current_frame(&self) -> &ASprite {
        &self.frames[self.timer.index]
    }
}

impl Drawable for ASpriteAnimation {
    fn draw(&self, renderer: &mut Renderer) {
        let mut sprite_renderer = SpriteRenderer::new(renderer);
        let frame = self.current_frame();
        sprite_renderer.draw_flip_once(frame, self.bounds.position, false);
    }

    fn size(&self) -> Vec2 {
        Vec2::ZERO
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, delta: f32) {
        self.tick(delta);
    }
}

#[derive(Clone)]
pub struct SpriteAnimation {
    pub frames: Vec<Sprite>,
    pub timer: Timer,
    pub bounds: Bounds,
}

impl TryFrom<Node> for SpriteAnimation {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let frames: Vec<Sprite> = node.try_into()?;
        Ok(Self {
            timer: Timer::new(frames.iter().map(|frame| frame.delay as f32).collect()),
            frames,
            bounds: Bounds::default(),
        })
    }
}

impl SpriteAnimation {
    pub fn tick(&mut self, delta: f32) -> &Sprite {
        self.timer.tick(delta);
        let sprite = &mut self.frames[self.timer.index];
        let p = self.timer.progress();
        sprite.alpha = (((1.0 - p) * sprite.a0 as f32 + p * sprite.a1 as f32) as i32);
        sprite
    }

    pub fn current_frame(&self) -> &Sprite {
        &self.frames[self.timer.index]
    }

    pub fn with_repeat(mut self, repeat: Repeat) -> Self {
        self.timer.repeat = repeat;
        self
    }
}

impl Drawable for SpriteAnimation {
    fn draw(&self, renderer: &mut Renderer) {
        let mut sprite_renderer = SpriteRenderer::new(renderer);
        let frame = self.current_frame();
        sprite_renderer.draw(frame, self.bounds.position);
    }

    fn size(&self) -> Vec2 {
        self.current_frame().size
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, delta: f32) {
        self.tick(delta);
    }
}
