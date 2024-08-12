use ab_glyph::FontVec;
use glam::{vec2, Vec2, Vec4};
use image::DynamicImage;
use sdl_sys::{
    self, SDL_CreateTexture, SDL_FRect, SDL_FlipMode, SDL_GetRendererFromTexture, SDL_PixelFormat,
    SDL_RenderTextureRotated, SDL_Renderer, SDL_SetTextureBlendMode, SDL_SetTextureScaleMode,
    SDL_Texture, SDL_UpdateTexture,
};
use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use crate::sprite::Sprite;

pub struct Surface {
    pub surface: *mut sdl_sys::SDL_Surface,
    pub width: i32,
    pub height: i32,
}

impl Surface {
    pub fn new(width: i32, height: i32) -> Self {
        unsafe {
            Self {
                surface: sdl_sys::SDL_CreateSurface(
                    width,
                    height,
                    sdl_sys::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888,
                ),
                width,
                height,
            }
        }
    }

    pub fn blit(&self, dest: &Surface, x: i32, y: i32) {
        unsafe {
            sdl_sys::SDL_BlitSurface(
                self.surface,
                &sdl_sys::SDL_Rect {
                    x: 0,
                    y: 0,
                    w: self.width,
                    h: self.height,
                },
                dest.surface,
                &sdl_sys::SDL_Rect {
                    x,
                    y,
                    w: self.width,
                    h: self.height,
                },
            );
        }
    }
}

impl From<Arc<DynamicImage>> for Surface {
    fn from(value: Arc<DynamicImage>) -> Self {
        let w = value.width() as i32;
        let h = value.height() as i32;
        let bytes = match value.borrow() {
            DynamicImage::ImageRgba8(data) => data,
            _ => &value.clone().to_rgba8(),
        };
        Self {
            surface: unsafe {
                sdl_sys::SDL_CreateSurfaceFrom(
                    w,
                    h,
                    sdl_sys::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888,
                    bytes.as_ptr() as *mut core::ffi::c_void,
                    w * 4,
                )
            },
            width: value.width() as i32,
            height: value.height() as i32,
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            sdl_sys::SDL_DestroySurface(self.surface);
        }
    }
}

pub struct Texture {
    pub texture: *mut SDL_Texture,
    pub renderer: *mut SDL_Renderer,
    pub size: Vec2,
}

impl Texture {
    pub fn from_surface(surface: &Surface, renderer: *mut SDL_Renderer) -> Self {
        let texture = unsafe { sdl_sys::SDL_CreateTextureFromSurface(renderer, surface.surface) };
        Self {
            texture,
            renderer,
            size: vec2(surface.width as f32, surface.height as f32),
        }
    }

    pub fn from_image(image: &DynamicImage, renderer: *mut SDL_Renderer) -> Self {
        let texture = unsafe {
            SDL_CreateTexture(
                renderer,
                SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888,
                sdl_sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STATIC,
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
            sdl_sys::SDL_SetTextureScaleMode(
                texture,
                sdl_sys::SDL_ScaleMode::SDL_SCALEMODE_NEAREST,
            );
        }
        Texture {
            texture,
            renderer,
            size: vec2(image.width() as f32, image.height() as f32),
        }
    }

    pub fn ptr(&self) -> *mut SDL_Texture {
        self.texture
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            sdl_sys::SDL_DestroyTexture(self.texture);
        }
    }
}

pub struct NineGridTexture {
    pub texture: Texture,
    pub left_width: i32,
    pub middle_width: i32,
    pub right_width: i32,
    pub top_height: i32,
    pub middle_height: i32,
    pub bottom_height: i32,
}

impl NineGridTexture {
    pub fn new(
        (lt, t, rt, lm, rm, lb, b, rb): (
            &Surface,
            &Surface,
            &Surface,
            &Surface,
            &Surface,
            &Surface,
            &Surface,
            &Surface,
        ),
        renderer: *mut SDL_Renderer,
    ) -> Self {
        let left_width = lt.width.min(lb.width).min(lm.width);
        let middle_width = t.width.min(b.width);
        let right_width = rt.width.min(rb.width).min(rm.width);

        let top_height = lt.height.min(t.height).min(rt.height);
        let middle_height = lm.height.min(rm.height);
        let bottom_height = lb.height.min(b.height).min(rb.height);

        let dest = Surface::new(
            left_width + middle_width + right_width,
            top_height + middle_height + bottom_height,
        );

        lt.blit(&dest, 0, 0);
        t.blit(&dest, left_width, 0);
        rt.blit(&dest, left_width + middle_width, 0);
        lm.blit(&dest, 0, top_height);
        rm.blit(&dest, left_width + middle_width, top_height);
        lb.blit(&dest, 0, top_height + middle_height);
        b.blit(&dest, left_width, top_height + middle_height);
        rb.blit(&dest, left_width + middle_width, top_height + middle_height);

        Self {
            texture: Texture::from_surface(&dest, renderer),
            left_width,
            middle_width,
            right_width,
            top_height,
            middle_height,
            bottom_height,
        }
    }

    pub fn ptr(&self) -> *mut SDL_Texture {
        self.texture.ptr()
    }

    pub fn draw(&self, offset: Vec2, size: Vec2) {
        unsafe {
            sdl_sys::SDL_RenderTexture9Grid(
                self.texture.renderer,
                self.ptr(),
                std::ptr::null(),
                self.left_width as f32,
                self.right_width as f32,
                self.top_height as f32,
                self.bottom_height as f32,
                1.0,
                &SDL_FRect {
                    x: offset.x,
                    y: offset.y,
                    w: size.x,
                    h: size.y,
                },
            );
        }
    }

    pub fn border_size(&self) -> Vec2 {
        vec2(
            (self.left_width + self.right_width) as f32,
            (self.top_height + self.bottom_height) as f32,
        )
    }
}

pub struct Renderer {
    dpr: f32,
    font: FontVec,
    renderer: *mut SDL_Renderer,
    textures: HashMap<*const DynamicImage, Arc<Texture>>,
    text_textures: HashMap<String, Arc<Texture>>,
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(dpr: f32, renderer: *mut SDL_Renderer) -> Self {
        let font =
            // FontRef::try_from_slice(include_bytes!("../../.././Data/WenQuanYiMicroHei.ttf"))?;
        // FontVec::try_from_vec(include_bytes!("../../.././Data/SourceHanSerifSC-Regular.otf").to_vec()).unwrap();
        FontVec::try_from_vec(include_bytes!("../../.././Data/simsun.ttc").to_vec()).unwrap();
        // FontVec::try_from_vec(include_bytes!("../../.././Data/NSimSun.ttf").to_vec()).unwrap();
        Self {
            dpr,
            font,
            renderer,
            textures: Default::default(),
            text_textures: Default::default(),
        }
    }

    pub fn texture(&mut self, image: &Arc<DynamicImage>) -> Arc<Texture> {
        self.textures
            .entry(Arc::as_ptr(image))
            .or_insert_with(|| Texture::from_image(image, self.renderer).into())
            .clone()
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
                    texture: text_texture,
                    renderer: self.renderer,
                    size: vec2(
                        image.width() as f32 / self.dpr,
                        image.height() as f32 / self.dpr,
                    ),
                }
                .into()
            })
            .clone();
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
        self.render_texture(
            &texture,
            position,
            Vec2::ZERO,
            255,
            None,
            SDL_FlipMode::SDL_FLIP_NONE,
        );
    }

    pub fn draw_flip(&mut self, sprite: &Sprite, position: Vec2, flip: bool) {
        let texture = self.texture(&sprite.image);
        self.render_texture(
            &texture,
            position,
            sprite.origin,
            sprite.alpha,
            None,
            if flip {
                SDL_FlipMode::SDL_FLIP_HORIZONTAL
            } else {
                SDL_FlipMode::SDL_FLIP_NONE
            },
        )
    }

    pub fn render_rect(&self, rect: &SDL_FRect) {
        unsafe {
            sdl_sys::SDL_RenderRect(self.renderer, rect);
        }
    }

    pub fn render_texture(
        &self,
        texture: &Texture,
        position: Vec2,
        origin: Vec2,
        alpha: i32,
        size: Option<Vec2>,
        flip: SDL_FlipMode::Type,
    ) {
        let size = size.unwrap_or(texture.size);
        unsafe {
            sdl_sys::SDL_SetTextureAlphaMod(texture.texture, alpha as u8);
            SDL_RenderTextureRotated(
                self.renderer,
                texture.texture,
                std::ptr::null(),
                &SDL_FRect {
                    x: if flip == SDL_FlipMode::SDL_FLIP_HORIZONTAL {
                        position.x - (texture.size.x - origin.x)
                    } else {
                        position.x - origin.x
                    },
                    y: if flip == SDL_FlipMode::SDL_FLIP_VERTICAL {
                        position.y - (texture.size.y - origin.y)
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
