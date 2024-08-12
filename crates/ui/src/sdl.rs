use cosmic_text::{FontSystem, Placement, SwashCache};
use glam::{vec2, Vec2};
use image::DynamicImage;
use peniko::Color;
use sdl3_sys::{
    blendmode::SDL_BLENDMODE_BLEND,
    events::{SDL_Event, SDL_PollEvent},
    pixels::SDL_PixelFormat,
    rect::SDL_FRect,
    render::{
        SDL_CreateTexture, SDL_DestroyTexture, SDL_RenderFillRect, SDL_RenderTexture,
        SDL_RenderTextureRotated, SDL_Renderer, SDL_SetRenderDrawColor, SDL_SetTextureAlphaMod,
        SDL_SetTextureBlendMode, SDL_SetTextureColorMod, SDL_SetTextureScaleMode, SDL_Texture,
        SDL_TextureAccess, SDL_UpdateTexture,
    },
    surface::{SDL_FlipMode, SDL_ScaleMode},
};
use std::{cell::RefCell, collections::HashMap, mem::MaybeUninit};

pub struct PollEvent {
    event: MaybeUninit<SDL_Event>,
}

unsafe impl Send for PollEvent {}
unsafe impl Sync for PollEvent {}

impl PollEvent {
    pub fn new() -> Self {
        Self {
            event: MaybeUninit::uninit(),
        }
    }
}

impl<'a> Iterator for &'a mut PollEvent {
    type Item = &'a SDL_Event;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if SDL_PollEvent(self.event.as_mut_ptr()) {
                Some(&*self.event.as_ptr())
            } else {
                None
            }
        }
    }
}

struct TextTexture {
    pub placement: Placement,
    pub texture: *mut SDL_Texture,
}

pub struct ImageTexture {
    pub texture: *mut SDL_Texture,
    pub renderer: *mut SDL_Renderer,
    pub size: glam::Vec2,
}

impl Drop for ImageTexture {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTexture(self.texture);
        }
    }
}

impl ImageTexture {
    pub fn new(renderer: *mut SDL_Renderer, image: &DynamicImage) -> Self {
        let texture = unsafe {
            SDL_CreateTexture(
                renderer,
                SDL_PixelFormat::ABGR8888,
                SDL_TextureAccess::STATIC,
                image.width() as i32,
                image.height() as i32,
            )
        };
        unsafe {
            let bytes = match image {
                DynamicImage::ImageRgba8(data) => data,
                _ => &image.clone().to_rgba8(),
            };
            SDL_UpdateTexture(
                texture,
                std::ptr::null(),
                bytes.as_ptr() as *const core::ffi::c_void,
                image.width() as i32 * 4,
            );
            SDL_SetTextureScaleMode(texture, SDL_ScaleMode::NEAREST);
        }
        ImageTexture {
            texture,
            renderer,
            size: vec2(image.width() as f32, image.height() as f32),
        }
    }

    pub fn draw(
        &self,
        position: Vec2,
        origin: Vec2,
        alpha: i32,
        size: Option<Vec2>,
        flip: SDL_FlipMode,
    ) {
        let size = size.unwrap_or(self.size);
        unsafe {
            SDL_SetTextureAlphaMod(self.texture, alpha as u8);
            SDL_RenderTextureRotated(
                self.renderer,
                self.texture,
                std::ptr::null(),
                &SDL_FRect {
                    x: if flip == SDL_FlipMode::HORIZONTAL {
                        position.x - (self.size.x - origin.x)
                    } else {
                        position.x - origin.x
                    },
                    y: if flip == SDL_FlipMode::VERTICAL {
                        position.y - (self.size.y - origin.y)
                    } else {
                        position.y - origin.y
                    },
                    w: size.x,
                    h: size.y,
                },
                0.0,
                std::ptr::null(),
                flip,
            );
        }
    }
}

pub struct Painter {
    renderer: *mut SDL_Renderer,
    text_textures: RefCell<HashMap<cosmic_text::CacheKey, TextTexture>>,
}

impl Painter {
    pub fn new(renderer: *mut SDL_Renderer) -> Self {
        Self {
            renderer,
            text_textures: Default::default(),
        }
    }

    pub fn fill_rect(&self, color: Color, location: taffy::Point<f32>, size: taffy::Size<f32>) {
        unsafe {
            SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a);
            SDL_RenderFillRect(
                self.renderer,
                &SDL_FRect {
                    x: location.x,
                    y: location.y,
                    w: size.width,
                    h: size.height,
                } as *const SDL_FRect,
            )
        };
    }

    pub fn create_image_texture(&self, image: &DynamicImage) -> ImageTexture {
        ImageTexture::new(self.renderer, image)
    }

    pub fn fill_text(
        &self,
        color: Color,
        location: taffy::Point<f32>,
        swash_cache: &mut SwashCache,
        font_system: &mut FontSystem,
        buffer: &cosmic_text::Buffer,
    ) {
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);
                let mut text_textures = self.text_textures.borrow_mut();
                let texture = text_textures
                    .entry(physical_glyph.cache_key)
                    .or_insert_with(|| {
                        let image = swash_cache
                            .get_image_uncached(font_system, physical_glyph.cache_key)
                            .unwrap();

                        let texture = unsafe {
                            SDL_CreateTexture(
                                self.renderer,
                                SDL_PixelFormat::ARGB8888,
                                SDL_TextureAccess::STATIC,
                                image.placement.width.try_into().unwrap(),
                                image.placement.height.try_into().unwrap(),
                            )
                        };

                        let data = image
                            .data
                            .iter()
                            .map(|v| ((*v as u32) << 24) | 0xFF_FF_FF)
                            .collect::<Vec<_>>();

                        unsafe {
                            SDL_UpdateTexture(
                                texture,
                                std::ptr::null(),
                                data.as_ptr() as *const core::ffi::c_void,
                                image.placement.width as i32 * 4,
                            );
                            SDL_SetTextureScaleMode(texture, SDL_ScaleMode::NEAREST);
                            SDL_SetTextureBlendMode(texture, SDL_BLENDMODE_BLEND);
                        };

                        TextTexture {
                            placement: image.placement,
                            texture,
                        }
                    });

                let x = physical_glyph.x + texture.placement.left;
                let y = run.line_y as i32 + physical_glyph.y - texture.placement.top;

                unsafe {
                    SDL_SetTextureColorMod(texture.texture, color.r, color.g, color.b);
                    SDL_RenderTexture(
                        self.renderer,
                        texture.texture,
                        std::ptr::null(),
                        &SDL_FRect {
                            x: location.x + x as f32,
                            y: location.y + y as f32,
                            w: texture.placement.width as f32,
                            h: texture.placement.height as f32,
                        },
                    )
                };
            }
        }
    }
}
