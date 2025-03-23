use crate::geometry::Rect;
use crate::render::command::{
    Command, LineCommand, RectCommand, RenderTextureCommand, RenderTextureNineGridCommand,
    RenderTextureRotatedCommand,
};
use crate::render::layer::{Layer, Tile};
use crate::runtime::RUNTIME;
use crate::style::dimension;
use crate::{OffsetEditor, ViewId};
use cosmic_text::{Edit, FontSystem, Metrics, Placement, SwashCache};
use glam::{vec2, Vec2, Vec4Swizzles};
use hashbrown::HashSet;
use image::DynamicImage;
use peniko::Color;
use sdl3_sys::everything::*;
use std::rc::Rc;
use std::{cmp, collections::HashMap, sync::Arc};
use taffy::prelude::length;
use taffy::{AvailableSpace, LengthPercentage, Size};
use unicode_segmentation::UnicodeSegmentation;

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

    pub fn ptr(&self) -> *mut SDL_Surface {
        self.surface
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

    pub fn clear(&self, color: Color) {
        let [r, g, b, a] = color.components;
        unsafe {
            SDL_ClearSurface(self.surface, r, g, b, a);
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

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum RenderFlag {
    LayerDebug,
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

    pub fn border(&self) -> dimension::Rect<dimension::LengthPercentage> {
        dimension::Rect {
            left: dimension::length(self.left_width as f32).into(),
            right: dimension::length(self.right_width as f32).into(),
            top: dimension::length(self.top_height as f32).into(),
            bottom: dimension::length(self.bottom_height as f32).into(),
        }
    }
}

impl Drawable for NineGridTexture {
    fn draw(&self, painter: &mut Renderer) {
        painter.render_texture_nine_grid(&self, self.bounds.position, Some(self.bounds.size));
    }

    fn size(&self) -> Vec2 {
        self.texture.size
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }
}

#[derive(Clone)]
struct TextLocation {
    pub placement: Placement,
    pub index: usize,
    pub location: Vec2,
}

struct TextTexture {
    renderer: *mut SDL_Renderer,
    texture: Texture,
    line_height: f32,
    offset: Vec2,
}

impl TextTexture {
    pub fn new(renderer: *mut SDL_Renderer) -> Self {
        Self {
            renderer,
            texture: Texture::new(
                renderer,
                SDL_PixelFormat::ABGR8888,
                SDL_TextureAccess::TARGET,
                1024,
                1024,
            )
            .blend_mode_blend(),
            line_height: 0.0,
            offset: Vec2::ZERO,
        }
    }

    fn allocate(&mut self, size: Vec2) -> Option<Vec2> {
        if size.x > self.texture.size.x || size.y > self.texture.size.y {
            return None;
        }

        if size.x + self.offset.x <= self.texture.size.x
            && size.y.max(self.line_height) + self.offset.y <= self.texture.size.y
        {
            let offset = self.offset;
            self.offset.x += size.x;
            self.line_height = self.line_height.max(size.y);
            return Some(offset);
        }

        self.offset.y += self.line_height;
        self.offset.x = 0.0;
        self.line_height = 0.0;
        if size.y + self.offset.y < self.texture.size.y {
            let offset = self.offset;
            self.offset.x += size.x;
            self.line_height = size.y;
            return Some(offset);
        }

        None
    }

    pub fn insert(&mut self, image: cosmic_text::SwashImage) -> Option<TextLocation> {
        let size = vec2(image.placement.width as f32, image.placement.height as f32);
        let location = self.allocate(size);
        if location.is_none() {
            return None;
        }
        let location = location.unwrap();

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
            SDL_SetTextureBlendMode(texture, SDL_BLENDMODE_NONE);
            SDL_SetTextureScaleMode(texture, SDL_SCALEMODE_LINEAR);
        }
        let prev = unsafe { SDL_GetRenderTarget(self.renderer) };
        unsafe { SDL_SetRenderTarget(self.renderer, self.texture.ptr()) };
        unsafe {
            SDL_RenderTexture(
                self.renderer,
                texture,
                std::ptr::null_mut(),
                &Rect::from((location, size)).into(),
            );
        };

        unsafe { SDL_SetRenderTarget(self.renderer, prev) };

        Some(TextLocation {
            placement: image.placement,
            index: 0,
            location,
        })
    }
}

pub struct DynamicImageDrawable {
    pub size: Vec2,
    pub image: Arc<DynamicImage>,
    pub bounds: Bounds,
}

#[derive(Default, Clone, Debug)]
pub struct Bounds {
    pub position: Vec2,
    pub size: Vec2,
}

pub trait Drawable {
    fn draw(&self, ctx: &mut Renderer);
    fn size(&self) -> Vec2;
    fn set_bounds(&mut self, bounds: Bounds) {}
    fn update(&mut self, delta: f32) -> bool {
        false
    }
}

impl DynamicImageDrawable {
    pub fn new(image: Arc<DynamicImage>) -> Self {
        DynamicImageDrawable {
            size: vec2(image.width() as f32, image.height() as f32),
            image,
            bounds: Default::default(),
        }
    }
}

impl Drawable for DynamicImageDrawable {
    fn draw(&self, ctx: &mut Renderer) {
        let texture = ctx.texture(&self.image);
        ctx.render_texture_flip(
            &texture,
            SDL_FlipMode::NONE,
            Vec2::ZERO,
            self.size,
            Vec2::ZERO,
        );
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }
}

#[derive(Clone)]
pub struct Texture {
    pub texture: *mut SDL_Texture,
    pub size: Vec2,
}

impl Texture {
    pub fn new(
        renderer: *mut SDL_Renderer,
        format: SDL_PixelFormat,
        access: SDL_TextureAccess,
        width: i32,
        height: i32,
    ) -> Self {
        let texture = unsafe { SDL_CreateTexture(renderer, format, access, width, height) };
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
        }
        Texture {
            texture,
            size: vec2(image.width() as f32, image.height() as f32),
        }
    }

    pub fn alpha(&self, alpha: f32) -> &Self {
        unsafe { SDL_SetTextureAlphaModFloat(self.texture, alpha) };
        self
    }

    pub fn color(&self, color: Color) -> &Self {
        let [r, g, b, _] = color.components;
        unsafe { SDL_SetTextureColorModFloat(self.texture, r, g, b) };
        self
    }

    pub fn ptr(&self) -> *mut SDL_Texture {
        self.texture
    }

    pub fn scale_mode_nearest(self) -> Self {
        unsafe {
            SDL_SetTextureScaleMode(self.ptr(), SDL_ScaleMode::NEAREST);
        }
        self
    }

    pub fn blend_mode_none(self) -> Self {
        unsafe {
            SDL_SetTextureBlendMode(self.ptr(), SDL_BLENDMODE_NONE);
        }
        self
    }

    pub fn blend_mode_blend(self) -> Self {
        unsafe {
            SDL_SetTextureBlendMode(self.ptr(), SDL_BLENDMODE_BLEND);
        }
        self
    }

    pub fn blend_mode(self, mode: SDL_BlendMode) -> Self {
        unsafe {
            SDL_SetTextureBlendMode(self.ptr(), mode);
        }
        self
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTexture(self.texture);
        }
    }
}

struct RendererState {
    translate: Vec2,
    alpha: f32,
    clip: Rect,
    layer: Option<Layer>,
}

pub struct Renderer {
    pub dpr: f32,
    scale: f32,
    pub renderer: *mut SDL_Renderer,
    text_locations: HashMap<cosmic_text::CacheKey, TextLocation>,
    text_textures: TextTexture,
    textures: HashMap<*const DynamicImage, Arc<Texture>>,
    pub(crate) translate: Vec2,
    pub(crate) clip: Rect,
    pub(crate) layer: Option<Layer>,
    size: Vec2,
    pub texture_blend_mode: SDL_BlendMode,
    states: Vec<RendererState>,
    transparent: Texture,
    pub alpha: f32,
    pub draw_calls: (usize, usize),
    pub draw_tiles: (usize, usize),
    pub flags: HashSet<RenderFlag>,
}

impl Renderer {
    pub fn new(renderer: *mut SDL_Renderer, dpr: f32, size: Vec2) -> Self {
        // unsafe {
        //     SDL_SetRenderLogicalPresentation(
        //         renderer,
        //         size.x as i32,
        //         size.y as i32,
        //         SDL_LOGICAL_PRESENTATION_STRETCH,
        //     )
        // };

        unsafe {
            SDL_SetRenderScale(renderer, dpr, dpr);
            SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);
        };

        let surface = Surface::new(1, 1);
        surface.clear(Color::TRANSPARENT);
        let transparent = Texture::from_surface(&surface, renderer).blend_mode_none();

        Self {
            dpr,
            scale: dpr,
            renderer,
            text_locations: HashMap::new(),
            text_textures: TextTexture::new(renderer),
            textures: Default::default(),
            translate: Vec2::ZERO,
            clip: Rect::INFINITY,
            layer: None,
            size,
            texture_blend_mode: SDL_BLENDMODE_BLEND,
            states: Default::default(),
            transparent,
            alpha: 1.0,
            draw_calls: (0, 0),
            draw_tiles: (0, 0),
            flags: HashSet::new(),
        }
    }

    pub fn translate(&mut self, delta: Vec2) {
        self.clip -= delta;
        self.translate += delta;
    }

    pub fn clip(&mut self, rect: &Rect) {
        self.clip = self.clip.intersect_rect(rect);
    }

    pub fn layer(&mut self, layer: &Layer) {
        self.layer = Some(layer.clone());
    }

    fn command(&mut self, command: impl Command + 'static) {
        if let Some(layer) = self.layer.as_ref() {
            layer.command(Box::new(command));
        } else {
            command.execute(self);
            self.draw_calls.0 += 1;
        }
    }

    pub fn save(&mut self) {
        self.states.push(RendererState {
            translate: self.translate,
            alpha: self.alpha,
            clip: self.clip,
            layer: self.layer.clone(),
        });
    }

    pub fn restore(&mut self) {
        if let Some(state) = self.states.pop() {
            self.translate = state.translate;
            self.alpha = state.alpha;
            self.clip = state.clip;
            self.layer = state.layer;
        }
    }

    pub fn reset(&mut self) {
        self.translate = vec2(0.0, 0.0);
        self.alpha = 1.0;
        self.clip = Rect::INFINITY;
        self.layer = None;
    }

    pub fn blend_mode_blend(&self) {
        unsafe {
            SDL_SetRenderDrawBlendMode(self.renderer, SDL_BLENDMODE_BLEND);
        }
    }

    pub fn blend_mode_none(&self) {
        unsafe {
            SDL_SetRenderDrawBlendMode(self.renderer, SDL_BLENDMODE_NONE);
        }
    }

    pub fn texture_blend_mode_blend(&mut self) {
        self.texture_blend_mode = SDL_BLENDMODE_BLEND;
    }

    pub fn texture_blend_mode_none(&mut self) {
        self.texture_blend_mode = SDL_BLENDMODE_NONE;
    }

    // pub fn set_color(&self, color: Color) {
    //     unsafe {
    //         SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a);
    //     }
    // }

    // pub fn lines(&mut self, points: &[Vec2]) {
    //     let count = points.len() as i32;
    //     let points = points
    //         .iter()
    //         .map(|p| {
    //             let p = p + self.translate;
    //             SDL_FPoint { x: p.x, y: p.y }
    //         })
    //         .collect::<Vec<_>>();
    //     unsafe { SDL_RenderLines(self.renderer, points.as_ptr(), count) };
    //     self.draw_calls += 1;
    // }

    pub fn fill_selection(
        &mut self,
        color: Color,
        location: Vec2,
        size: Vec2,
        editor: &OffsetEditor,
        dpr: f32,
    ) {
        let offset = editor.offset;
        let mut f = |x: i32, y: i32, w: u32, h: u32| {
            let mut x = x as f32 + offset.x;
            let mut y = y as f32 + offset.y;
            let mut w = w as f32;
            let mut h = h as f32;
            w = w.min(size.x * dpr - x);
            h = h.min(size.y * dpr - y);
            if w <= 0.0 || h <= 0.0 || x + w <= 0.0 || y + h <= 0.0 {
                return;
            }
            if x < 0.0 {
                w += x;
                x = 0.0;
            }
            if y < 0.0 {
                h += y;
                y = 0.0;
            }
            self.fill_rect(
                color,
                Rect::from((
                    vec2(location.x + x / dpr, location.y + y / dpr),
                    vec2(w, h) / dpr,
                )),
            );
        };
        let selection_bounds = editor.editor.selection_bounds();
        editor.editor.with_buffer(|buffer| {
            for run in buffer.layout_runs() {
                let line_i = run.line_i;
                let line_y = run.line_y;
                let line_top = run.line_top;
                let line_height = run.line_height;

                // Highlight selection
                if let Some((start, end)) = selection_bounds {
                    if line_i >= start.line && line_i <= end.line {
                        let mut range_opt = None;
                        for glyph in run.glyphs.iter() {
                            // Guess x offset based on characters
                            let cluster = &run.text[glyph.start..glyph.end];
                            let total = cluster.grapheme_indices(true).count();
                            let mut c_x = glyph.x;
                            let c_w = glyph.w / total as f32;
                            for (i, c) in cluster.grapheme_indices(true) {
                                let c_start = glyph.start + i;
                                let c_end = glyph.start + i + c.len();
                                if (start.line != line_i || c_end > start.index)
                                    && (end.line != line_i || c_start < end.index)
                                {
                                    range_opt = match range_opt.take() {
                                        Some((min, max)) => Some((
                                            cmp::min(min, c_x as i32),
                                            cmp::max(max, (c_x + c_w) as i32),
                                        )),
                                        None => Some((c_x as i32, (c_x + c_w) as i32)),
                                    };
                                } else if let Some((min, max)) = range_opt.take() {
                                    f(
                                        min,
                                        line_top as i32,
                                        cmp::max(0, max - min) as u32,
                                        line_height as u32,
                                    );
                                }
                                c_x += c_w;
                            }
                        }

                        if run.glyphs.is_empty() && end.line > line_i {
                            // Highlight all of internal empty lines
                            range_opt = Some((0, buffer.size().0.unwrap_or(0.0) as i32));
                        }

                        if let Some((mut min, mut max)) = range_opt.take() {
                            if end.line > line_i {
                                // Draw to end of line
                                if run.rtl {
                                    min = 0;
                                } else {
                                    max = buffer.size().0.unwrap_or(0.0) as i32;
                                }
                            }
                            f(
                                min,
                                line_top as i32,
                                cmp::max(0, max - min) as u32,
                                line_height as u32,
                            );
                        }
                    }
                }
            }
        });
    }

    pub fn with_target(&mut self, texture: &Texture, f: impl FnOnce(&mut Self)) {
        let prev = unsafe { SDL_GetRenderTarget(self.renderer) };
        unsafe {
            SDL_SetRenderTarget(self.renderer, texture.ptr());
            SDL_SetRenderScale(self.renderer, self.dpr, self.dpr);
        };
        f(self);
        unsafe { SDL_SetRenderTarget(self.renderer, prev) };
    }

    pub fn scale(&self, scale: f32) -> &Self {
        let scale = scale * self.dpr;
        unsafe { SDL_SetRenderScale(self.renderer, scale, scale) };
        self
    }

    pub fn measure_text(
        &self,
        id: ViewId,
        buffer: &mut cosmic_text::Buffer,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        prune: bool,
    ) -> Size<f32> {
        let dpr = self.dpr;

        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width * dpr),
        });

        let height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width * dpr),
        });

        let style = id.state().borrow().style.clone();
        let metrics = Metrics::new(
            style.font_size.unwrap_or(20.0) * dpr,
            style.line_height.unwrap_or(24.0) * dpr,
        );

        RUNTIME.with_borrow_mut(|s| {
            buffer.set_metrics_and_size(
                &mut s.font_system,
                metrics,
                width_constraint,
                height_constraint,
            );
            buffer.set_wrap(&mut s.font_system, style.text_wrap.into());
            buffer.shape_until_scroll(&mut s.font_system, prune);
        });

        let (width, total_lines) = buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            });

        let height = total_lines as f32 * buffer.metrics().line_height;

        let size = Size {
            width: if let Some(max_width) = width_constraint {
                max_width.min(width)
            } else {
                width
            },
            height: if let Some(max_height) = height_constraint {
                max_height.min(height)
            } else {
                height
            },
        };

        Size {
            width: size.width / dpr,
            height: size.height / dpr,
        }
    }

    pub fn fill_text(
        &mut self,
        color: Color,
        location: Vec2,
        size: Vec2,
        offset: Vec2,
        swash_cache: &mut SwashCache,
        font_system: &mut FontSystem,
        buffer: &cosmic_text::Buffer,
    ) {
        let dpr = self.dpr;
        for run in buffer.layout_runs() {
            // self.line(0.0, location.y + run.line_top, 1000.0, location.y + run.line_top);
            // self.line(0.0, location.y + run.line_y, 1000.0, location.y + run.line_y);
            let y = run.line_top + offset.y;
            if y + run.line_height <= 0.0 || y >= size.y {
                continue;
            }

            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);
                let texture = {
                    self.text_locations
                        .entry(physical_glyph.cache_key)
                        .or_insert_with(|| {
                            let image = swash_cache
                                .get_image_uncached(font_system, physical_glyph.cache_key)
                                .unwrap();
                            self.text_textures.insert(image).unwrap()
                        })
                        .clone()
                };

                let x = (physical_glyph.x + texture.placement.left) as f32 + offset.x;
                let y = (run.line_y as i32 + physical_glyph.y - texture.placement.top) as f32
                    + offset.y;

                if (x + texture.placement.width as f32) <= 0.0 {
                    continue;
                }
                if x >= size.x {
                    break;
                }

                let sx = (-x).max(0.0);
                let sy = (-y).max(0.0);

                let w = (texture.placement.width as f32).min(size.x - x) - sx;
                let h = (texture.placement.height as f32).min(size.x - y) - sy;

                let x = x + sx;
                let y = y + sy;

                let src_rect = Rect::from((texture.location + vec2(sx, sy), vec2(w, h)));
                self.render_texture(
                    self.text_textures.texture.ptr(),
                    src_rect,
                    Rect::from((location + vec2(x, y) / dpr, vec2(w, h) / dpr)),
                    Some(color),
                );
            }
        }
    }

    pub fn texture(&mut self, image: &Arc<DynamicImage>) -> Arc<Texture> {
        self.textures
            .entry(Arc::as_ptr(image))
            .or_insert_with(|| {
                Texture::from_image(image, self.renderer)
                    .blend_mode_blend()
                    .scale_mode_nearest()
                    .into()
            })
            .clone()
    }

    pub fn render_texture_alpha(
        &mut self,
        texture: &Texture,
        position: Vec2,
        origin: Vec2,
        alpha: i32,
        size: Option<Vec2>,
        flip: SDL_FlipMode,
    ) {
        let size = size.unwrap_or(texture.size);
        unsafe {
            SDL_SetTextureAlphaMod(texture.ptr(), alpha as u8);
            self.render_texture_flip(texture, flip, position, size, origin);
        }
    }

    pub fn render_texture_flip(
        &mut self,
        texture: &Texture,
        flip: SDL_FlipMode,
        position: Vec2,
        size: Vec2,
        origin: Vec2,
    ) {
        self.render_texture_rotated(
            texture,
            Rect::from((Vec2::ZERO, texture.size)),
            Rect::from((
                vec2(
                    if flip == SDL_FlipMode::HORIZONTAL {
                        position.x - (size.x - origin.x)
                    } else {
                        position.x - origin.x
                    },
                    if flip == SDL_FlipMode::VERTICAL {
                        position.y - (size.y - origin.y)
                    } else {
                        position.y - origin.y
                    },
                ),
                size,
            )),
            0.0,
            None,
            flip,
        );
    }

    pub fn fill_rect(&mut self, color: Color, rect: Rect) {
        if !rect.intersect(&self.clip) {
            return;
        }
        let rect = rect.intersect_rect(&self.clip) + self.translate;
        self.command(RectCommand::new(rect).fill(color));
    }

    pub fn stroke_rect(&mut self, color: Color, rect: Rect) {
        if !rect.intersect(&self.clip) {
            return;
        }
        let rect = rect.intersect_rect(&self.clip) + self.translate;
        self.command(RectCommand::new(rect).stroke(color));
    }

    pub fn line(&mut self, color: Color, p1: Vec2, p2: Vec2) {
        let p1 = p1 + self.translate;
        let p2 = p2 + self.translate;
        self.command(LineCommand {
            color,
            from: p1,
            to: p2,
        });
    }

    pub fn render_texture(
        &mut self,
        texture: *mut SDL_Texture,
        src_rect: Rect,
        dst_rect: Rect,
        color: Option<Color>,
    ) {
        if let Some((src_rect, dst_rect)) = self.clip_rect(src_rect, dst_rect, SDL_FlipMode::NONE) {
            self.command(RenderTextureCommand {
                texture,
                src_rect,
                dst_rect,
                color,
            });
        }
    }

    pub fn render_texture_rotated(
        &mut self,
        texture: &Texture,
        src_rect: Rect,
        dst_rect: Rect,
        angle: f64,
        center: Option<Vec2>,
        flip: SDL_FlipMode,
    ) {
        if let Some((src_rect, dst_rect)) = self.clip_rect(src_rect, dst_rect, flip) {
            self.command(RenderTextureRotatedCommand {
                texture: texture.ptr(),
                src_rect,
                dst_rect,
                angle,
                center,
                flip,
            });
        }
    }

    pub fn render_texture_nine_grid(
        &mut self,
        texture: &NineGridTexture,
        position: Vec2,
        size: Option<Vec2>,
    ) {
        let size = size.unwrap_or(texture.texture.size);
        let position = position + self.translate;
        self.command(RenderTextureNineGridCommand {
            texture: texture.ptr(),
            src_rect: Rect::from((Vec2::ZERO, texture.texture.size)),
            dst_rect: Rect::from((position, size)),
            left: texture.left_width as f32,
            right: texture.right_width as f32,
            top: texture.top_height as f32,
            bottom: texture.bottom_height as f32,
            scale: 1.0,
        });
    }

    fn clip_rect(
        &self,
        mut src_rect: Rect,
        dst_rect: Rect,
        flip: SDL_FlipMode,
    ) -> Option<(Rect, Rect)> {
        if !self.clip.intersect(&dst_rect) {
            return None;
        }
        let rect = self.clip.intersect_rect(&dst_rect);
        let origin_rect = src_rect;

        src_rect.x += (rect.x - dst_rect.x) / dst_rect.width * src_rect.width;
        src_rect.y += (rect.y - dst_rect.y) / dst_rect.height * src_rect.height;
        src_rect.width *= (rect.width / dst_rect.width);
        src_rect.height *= (rect.height / dst_rect.height);
        if flip == SDL_FlipMode::HORIZONTAL {
            src_rect.x = origin_rect.width - src_rect.x - src_rect.width;
        }
        if flip == SDL_FlipMode::VERTICAL {
            src_rect.y = origin_rect.height - src_rect.y - src_rect.height;
        }
        Some((src_rect, rect + self.translate))
    }

    pub fn create_texture(&self, size: Vec2) -> Texture {
        Texture::new(
            self.renderer,
            SDL_PixelFormat::ABGR8888,
            SDL_TextureAccess::TARGET,
            size.x as i32,
            size.y as i32,
        )
    }

    pub fn create_streaming_texture(&self, size: Vec2) -> Texture {
        Texture::new(
            self.renderer,
            SDL_PixelFormat::ARGB8888,
            SDL_TextureAccess::STREAMING,
            size.x as i32,
            size.y as i32,
        )
    }

    pub fn clear(&self) {
        unsafe {
            SDL_SetRenderDrawColor(self.renderer, 0, 0, 0, 0);
            SDL_RenderClear(self.renderer);
        }
    }

    pub fn present(&mut self) {
        self.draw_calls.1 = self.draw_calls.0;
        self.draw_calls.0 = 0;

        self.draw_tiles.1 = self.draw_tiles.0;
        self.draw_tiles.0 = 0;

        unsafe {
            SDL_RenderPresent(self.renderer);
        }
    }
}
