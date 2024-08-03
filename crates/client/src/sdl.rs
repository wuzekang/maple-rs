use ab_glyph::FontVec;
use glam::{vec2, Vec2};
use image::DynamicImage;
use sdl_sys::{
    self, SDL_CreateTexture, SDL_FRect, SDL_FlipMode, SDL_PixelFormat, SDL_RenderTextureRotated,
    SDL_Renderer, SDL_SetTextureBlendMode, SDL_SetTextureScaleMode, SDL_Texture, SDL_UpdateTexture,
};
use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use crate::sprite::Sprite;

pub struct Texture {
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    size: Vec2,
}

impl Texture {
    pub fn render(&self, position: Vec2, origin: Vec2, alpha: i32, flip: SDL_FlipMode::Type) {
        unsafe {
            sdl_sys::SDL_SetTextureAlphaMod(self.texture, alpha as u8);
            SDL_RenderTextureRotated(
                self.renderer,
                self.texture,
                std::ptr::null(),
                &SDL_FRect {
                    x: if flip == SDL_FlipMode::SDL_FLIP_HORIZONTAL {
                        position.x - (self.size.x - origin.x)
                    } else {
                        position.x - origin.x
                    },
                    y: if flip == SDL_FlipMode::SDL_FLIP_VERTICAL {
                        position.y - (self.size.y - origin.y)
                    } else {
                        position.y - origin.y
                    },
                    w: self.size.x,
                    h: self.size.y,
                },
                0.0,
                std::ptr::null(),
                flip,
            );
        }
    }
}

pub struct SpriteRenderer {
    dpr: f32,
    font: FontVec,
    renderer: *mut SDL_Renderer,
    textures: HashMap<*const DynamicImage, Texture>,
    text_textures: HashMap<String, Texture>,
}

unsafe impl Send for SpriteRenderer {}
unsafe impl Sync for SpriteRenderer {}

impl SpriteRenderer {
    pub fn new(dpr: f32, renderer: *mut SDL_Renderer) -> Self {
        let font =
            // FontRef::try_from_slice(include_bytes!("../../.././Data/WenQuanYiMicroHei.ttf"))?;
        FontVec::try_from_vec(include_bytes!("../../.././Data/SourceHanSerifSC-Regular.otf").to_vec()).unwrap();
        // FontVec::try_from_vec(include_bytes!("../../.././Data/SimSun.ttf").to_vec()).unwrap();
        // FontVec::try_from_vec(include_bytes!("../../.././Data/NSimSun.ttf").to_vec()).unwrap();
        Self {
            dpr,
            font,
            renderer,
            textures: Default::default(),
            text_textures: Default::default(),
        }
    }
    pub fn texture(&mut self, image: &Arc<DynamicImage>) -> &Texture {
        self.textures.entry(Arc::as_ptr(image)).or_insert_with(|| {
            let texture = unsafe {
                SDL_CreateTexture(
                    self.renderer,
                    SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888,
                    sdl_sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STATIC,
                    image.width() as i32,
                    image.height() as i32,
                )
            };
            unsafe {
                let bytes = match image.borrow() {
                    DynamicImage::ImageRgba8(data) => data,
                    _ => &image.clone().to_rgba8(),
                };
                SDL_UpdateTexture(
                    texture,
                    std::ptr::null(),
                    bytes.as_ptr() as *const core::ffi::c_void,
                    image.width() as i32 * 4,
                );
                sdl_sys::SDL_SetTextureScaleMode(
                    texture,
                    sdl_sys::SDL_ScaleMode::SDL_SCALEMODE_NEAREST,
                    // sdl_sys::SDL_ScaleMode::SDL_SCALEMODE_BEST,
                    // sdl_sys::SDL_ScaleMode::SDL_SCALEMODE_LINEAR,
                );
            }
            Texture {
                renderer: self.renderer,
                texture,
                size: vec2(image.width() as f32, image.height() as f32),
            }
        })
    }

    pub fn draw(&mut self, sprite: &Sprite, position: Vec2) {
        self.draw_flip(sprite, position, false);
    }

    pub fn draw_text(&mut self, text: &str, position: Vec2) {
        if text.is_empty() {
            return;
        }
        let texture = self
            .text_textures
            .entry(text.to_string())
            .or_insert_with(|| unsafe {
                let image = crate::layout::draw_image(&self.font, 14.0 * self.dpr, text);

                let text_texture = SDL_CreateTexture(
                    self.renderer,
                    SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888,
                    sdl_sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STATIC,
                    image.width() as i32,
                    image.height() as i32,
                );
                SDL_UpdateTexture(
                    text_texture,
                    std::ptr::null(),
                    image.as_ptr() as *const core::ffi::c_void,
                    image.width() as i32 * 4,
                );
                SDL_SetTextureScaleMode(
                    text_texture,
                    sdl_sys::SDL_ScaleMode::SDL_SCALEMODE_NEAREST,
                );
                SDL_SetTextureBlendMode(text_texture, sdl_sys::SDL_BLENDMODE_BLEND);

                Texture {
                    renderer: self.renderer,
                    texture: text_texture,
                    size: vec2(
                        image.width() as f32 / self.dpr,
                        image.height() as f32 / self.dpr,
                    ),
                }
            });
        unsafe {
            sdl_sys::SDL_SetRenderDrawColor(self.renderer, 255, 255, 255, 255);
            sdl_sys::SDL_RenderFillRect(
                self.renderer,
                &SDL_FRect {
                    x: position.x - 4.0,
                    y: position.y - 2.0,
                    w: texture.size.x + 8.0,
                    h: texture.size.y + 4.0,
                },
            );
        }
        texture.render(position, Vec2::ZERO, 255, SDL_FlipMode::SDL_FLIP_NONE);
    }

    pub fn draw_flip(&mut self, sprite: &Sprite, position: Vec2, flip: bool) {
        self.texture(&sprite.image).render(
            position,
            sprite.origin,
            sprite.alpha,
            if flip {
                SDL_FlipMode::SDL_FLIP_HORIZONTAL
            } else {
                SDL_FlipMode::SDL_FLIP_NONE
            },
        )
    }
}
