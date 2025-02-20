use cosmic_text::{FontSystem, Placement, SwashCache};
use glam::{vec2, Vec2};
use image::DynamicImage;
use peniko::Color;
use sdl3_sys::{
    blendmode::SDL_BLENDMODE_BLEND,
    events::{SDL_Event, SDL_PollEvent},
    pixels::SDL_PixelFormat,
    rect::{SDL_FRect, SDL_Rect},
    render::{
        SDL_CreateTexture, SDL_CreateTextureFromSurface, SDL_DestroyTexture, SDL_RenderFillRect,
        SDL_RenderTexture, SDL_RenderTexture9Grid, SDL_RenderTextureRotated, SDL_Renderer,
        SDL_SetRenderDrawBlendMode, SDL_SetRenderDrawColor, SDL_SetTextureAlphaMod,
        SDL_SetTextureBlendMode, SDL_SetTextureColorMod, SDL_SetTextureScaleMode, SDL_Texture,
        SDL_TextureAccess, SDL_UpdateTexture,
    },
    surface::{
        SDL_BlitSurface, SDL_CreateSurface, SDL_CreateSurfaceFrom, SDL_DestroySurface,
        SDL_FlipMode, SDL_ScaleMode, SDL_Surface,
    },
};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap, mem::MaybeUninit, sync::Arc};
use taffy::prelude::length;
use taffy::{LengthPercentage, Rect};

pub struct Surface {
    pub surface: *mut SDL_Surface,
    pub width: i32,
    pub height: i32,
}

impl Surface {
    pub fn new(width: i32, height: i32) -> Self {
        unsafe {
            Self {
                surface: SDL_CreateSurface(width, height, SDL_PixelFormat::ABGR8888),
                width,
                height,
            }
        }
    }

    pub fn blit(&self, dest: &Surface, x: i32, y: i32) {
        unsafe {
            SDL_BlitSurface(
                self.surface,
                &SDL_Rect {
                    x: 0,
                    y: 0,
                    w: self.width,
                    h: self.height,
                },
                dest.surface,
                &SDL_Rect {
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
        let bytes = match &*value {
            DynamicImage::ImageRgba8(data) => data,
            _ => &value.clone().to_rgba8(),
        };
        Self {
            surface: unsafe {
                SDL_CreateSurfaceFrom(
                    w,
                    h,
                    SDL_PixelFormat::ABGR8888,
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
            SDL_DestroySurface(self.surface);
        }
    }
}

#[derive(Clone)]
pub struct NineGridTexture {
    pub texture: Rc<Texture>,
    pub left_width: i32,
    pub middle_width: i32,
    pub right_width: i32,
    pub top_height: i32,
    pub middle_height: i32,
    pub bottom_height: i32,
    pub bounds: Bounds,
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
            texture: Rc::new(Texture::from_surface(&dest, renderer)),
            left_width,
            middle_width,
            right_width,
            top_height,
            middle_height,
            bottom_height,
            bounds: Default::default(),
        }
    }

    pub fn ptr(&self) -> *mut SDL_Texture {
        self.texture.ptr()
    }

    pub fn border_size(&self) -> Vec2 {
        vec2(
            (self.left_width + self.right_width) as f32,
            (self.top_height + self.bottom_height) as f32,
        )
    }

    pub fn border(&self) -> Rect<LengthPercentage> {
        Rect {
            left: length(self.left_width as f32),
            right: length(self.right_width as f32),
            top: length(self.top_height as f32),
            bottom: length(self.bottom_height as f32),
        }
    }

    pub fn render(&self, painter: &Renderer, position: Vec2, size: Option<Vec2>) {
        let size = size.unwrap_or(self.texture.size);
        unsafe {
            SDL_RenderTexture9Grid(
                painter.renderer,
                self.ptr(),
                std::ptr::null(),
                self.left_width as f32,
                self.right_width as f32,
                self.top_height as f32,
                self.bottom_height as f32,
                1.0,
                &SDL_FRect {
                    x: position.x,
                    y: position.y,
                    w: size.x,
                    h: size.y,
                },
            );
        }
    }
}

impl Drawable for NineGridTexture {
    fn draw(&self, painter: &Renderer) {
        self.render(painter, self.bounds.position, Some(self.bounds.size));
    }

    fn size(&self) -> Vec2 {
        self.texture.size
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }
}

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
    pub texture: Texture,
    pub size: Vec2,
    pub flip: SDL_FlipMode,
    pub alpha: u8,
    pub bounds: Bounds,
}

#[derive(Default, Clone)]
pub struct Bounds {
    pub position: Vec2,
    pub size: Vec2,
}

pub trait Drawable {
    fn draw(&self, painter: &Renderer);
    fn size(&self) -> Vec2;
    fn set_bounds(&mut self, bounds: Bounds) {}
    fn update(&mut self, delta: u64) {}
}

impl ImageTexture {
    pub fn new(renderer: *mut SDL_Renderer, image: &DynamicImage) -> Self {
        let texture = Texture::new(
            renderer,
            SDL_PixelFormat::ABGR8888,
            SDL_TextureAccess::STATIC,
            image.width(),
            image.height(),
        );

        unsafe {
            let bytes = match image {
                DynamicImage::ImageRgba8(data) => data,
                _ => &image.clone().to_rgba8(),
            };
            SDL_UpdateTexture(
                texture.ptr(),
                std::ptr::null(),
                bytes.as_ptr() as *const core::ffi::c_void,
                image.width() as i32 * 4,
            );
            SDL_SetTextureScaleMode(texture.ptr(), SDL_ScaleMode::NEAREST);
        }

        ImageTexture {
            texture,
            size: vec2(image.width() as f32, image.height() as f32),
            flip: SDL_FlipMode::NONE,
            alpha: 255,
            bounds: Default::default(),
        }
    }
}

impl Drawable for ImageTexture {
    fn draw(&self, painter: &Renderer) {
        let origin = vec2(0.0, 0.0);
        self.texture.set_alpha_mod(self.alpha);
        painter.render_texture_rotated(
            &self.texture,
            self.flip,
            self.bounds.position,
            self.bounds.size,
            origin,
        );
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }
}

pub struct Texture {
    pub texture: *mut SDL_Texture,
    pub size: Vec2,
}

impl Texture {
    pub fn new(
        renderer: *mut SDL_Renderer,
        format: SDL_PixelFormat,
        access: SDL_TextureAccess,
        width: u32,
        height: u32,
    ) -> Self {
        let texture =
            unsafe { SDL_CreateTexture(renderer, format, access, width as i32, height as i32) };
        Self {
            texture,
            size: vec2(width as f32, height as f32),
        }
    }

    pub fn from_surface(surface: &Surface, renderer: *mut SDL_Renderer) -> Self {
        let texture = unsafe { SDL_CreateTextureFromSurface(renderer, surface.surface) };
        Self {
            texture,
            size: vec2(surface.width as f32, surface.height as f32),
        }
    }

    pub fn from_image(image: &DynamicImage, renderer: *mut SDL_Renderer) -> Self {
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
        Texture {
            texture,
            size: vec2(image.width() as f32, image.height() as f32),
        }
    }

    pub fn set_alpha_mod(&self, alpha: u8) -> bool {
        unsafe { SDL_SetTextureAlphaMod(self.texture, alpha) }
    }

    pub fn ptr(&self) -> *mut SDL_Texture {
        self.texture
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTexture(self.texture);
        }
    }
}

pub struct Renderer {
    renderer: *mut SDL_Renderer,
    text_textures: RefCell<HashMap<cosmic_text::CacheKey, TextTexture>>,
    textures: RefCell<HashMap<*const DynamicImage, Arc<Texture>>>,
}

impl Renderer {
    pub fn new(renderer: *mut SDL_Renderer) -> Self {
        Self {
            renderer,
            text_textures: Default::default(),
            textures: Default::default(),
        }
    }

    pub fn render_texture_rotated(
        &self,
        texture: &Texture,
        flip: SDL_FlipMode,
        position: Vec2,
        size: Vec2,
        origin: Vec2,
    ) -> bool {
        unsafe {
            SDL_RenderTextureRotated(
                self.renderer,
                texture.ptr(),
                std::ptr::null(),
                &SDL_FRect {
                    x: if flip == SDL_FlipMode::HORIZONTAL {
                        position.x - (size.x - origin.x)
                    } else {
                        position.x - origin.x
                    },
                    y: if flip == SDL_FlipMode::VERTICAL {
                        position.y - (size.y - origin.y)
                    } else {
                        position.y - origin.y
                    },
                    w: size.x,
                    h: size.y,
                },
                0.0,
                std::ptr::null(),
                flip,
            )
        }
    }

    pub fn fill_rect(&self, color: Color, location: Vec2, size: Vec2) {
        unsafe {
            SDL_SetRenderDrawBlendMode(self.renderer, SDL_BLENDMODE_BLEND);
            SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a);
            SDL_RenderFillRect(
                self.renderer,
                &SDL_FRect {
                    x: location.x,
                    y: location.y,
                    w: size.x,
                    h: size.y,
                } as *const SDL_FRect,
            )
        };
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

    pub fn texture(&self, image: &Arc<DynamicImage>) -> Arc<Texture> {
        self.textures
            .borrow_mut()
            .entry(Arc::as_ptr(image))
            .or_insert_with(|| Texture::from_image(image, self.renderer).into())
            .clone()
    }

    pub fn render_texture(
        &self,
        texture: &Texture,
        position: Vec2,
        origin: Vec2,
        alpha: i32,
        size: Option<Vec2>,
        flip: SDL_FlipMode,
    ) {
        let size = size.unwrap_or(texture.size);
        unsafe {
            SDL_SetTextureAlphaMod(texture.texture, alpha as u8);
            self.render_texture_rotated(texture, flip, position, size, origin);
        }
    }
}
