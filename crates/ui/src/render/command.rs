use crate::geometry::Rect;
use crate::Renderer;
use glam::Vec2;
use peniko::Color;
use sdl3_sys::everything::*;

fn bin_clip(clip: Rect, mut src_rect: Rect, dst_rect: Rect) -> Option<(Rect, Rect)> {
    if !clip.intersect(&dst_rect) {
        return None;
    }
    let rect = clip.intersect_rect(&dst_rect);
    src_rect.x += (rect.x - dst_rect.x) / dst_rect.width * src_rect.width;
    src_rect.y += (rect.y - dst_rect.y) / dst_rect.height * src_rect.height;
    src_rect.width *= rect.width / dst_rect.width;
    src_rect.height *= rect.height / dst_rect.height;
    Some((src_rect, rect - clip.location()))
}

pub trait Command {
    fn bounds(&self) -> Rect;
    fn clip(&self, clip_rect: Rect) -> Option<Box<dyn Command>>;
    fn execute(&self, ctx: &mut Renderer);
}

#[derive(Clone)]
pub struct RenderTextureCommand {
    pub texture: *mut SDL_Texture,
    pub color: Option<Color>,
    pub tiled: bool,
    pub src_rect: Rect,
    pub dst_rect: Rect,
}

impl Command for RenderTextureCommand {
    fn bounds(&self) -> Rect {
        self.dst_rect
    }

    fn clip(&self, clip_rect: Rect) -> Option<Box<dyn Command>> {
        let (src_rect, dst_rect) = bin_clip(clip_rect, self.src_rect, self.dst_rect)?;
        Some(Box::new(Self {
            texture: self.texture,
            color: self.color,
            tiled: self.tiled,
            src_rect,
            dst_rect,
        }))
    }

    fn execute(&self, ctx: &mut Renderer) {
        unsafe {
            if let Some(color) = self.color {
                let [r, g, b, a] = color.components;
                SDL_SetTextureColorModFloat(self.texture, r, g, b);
                SDL_SetTextureAlphaModFloat(self.texture, a);
            }
            if self.tiled {
                // 计算源纹理的尺寸
                let src_width = self.src_rect.width;
                let src_height = self.src_rect.height;

                // 计算需要重复的次数
                let x_count = (self.dst_rect.width / src_width).ceil() as i32;
                let y_count = (self.dst_rect.height / src_height).ceil() as i32;

                // 遍历绘制每个纹理块
                for y in 0..y_count {
                    for x in 0..x_count {
                        // 计算当前块的目标位置
                        let x_offset = x as f32 * src_width;
                        let y_offset = y as f32 * src_height;

                        // 计算当前块需要绘制的宽度和高度
                        let tile_width = src_width.min(self.dst_rect.width - x_offset);
                        let tile_height = src_height.min(self.dst_rect.height - y_offset);

                        // 计算源纹理中需要绘制的区域
                        let tile_src_width = src_width * (tile_width / src_width);
                        let tile_src_height = src_height * (tile_height / src_height);

                        let tile_src_rect = Rect::new(
                            self.src_rect.x,
                            self.src_rect.y,
                            tile_src_width,
                            tile_src_height,
                        );

                        let tile_dst_rect = Rect::new(
                            self.dst_rect.x + x_offset,
                            self.dst_rect.y + y_offset,
                            tile_width,
                            tile_height,
                        );

                        SDL_RenderTexture(
                            ctx.renderer,
                            self.texture,
                            &tile_src_rect.into(),
                            &tile_dst_rect.into(),
                        );
                    }
                }
            } else {
                SDL_RenderTexture(
                    ctx.renderer,
                    self.texture,
                    &self.src_rect.into(),
                    &(self.dst_rect).into(),
                );
            }
        }
    }
}

pub struct RectCommand {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub rect: Rect,
}

impl RectCommand {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            fill: None,
            stroke: None,
        }
    }
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }
}

impl Command for RectCommand {
    fn bounds(&self) -> Rect {
        self.rect
    }

    fn clip(&self, clip_rect: Rect) -> Option<Box<dyn Command>> {
        if self.stroke.is_none() && self.fill.is_none() {
            return None;
        }
        if !self.rect.intersect(&clip_rect) {
            return None;
        }
        let rect = self.rect.intersect_rect(&clip_rect);

        Some(Box::new(Self {
            fill: self.fill,
            stroke: self.stroke,
            rect: rect - clip_rect.location(),
        }))
    }

    fn execute(&self, ctx: &mut Renderer) {
        let Self { fill, stroke, rect } = self;
        unsafe {
            if let Some(fill) = fill {
                let [r, g, b, a] = fill.components;
                SDL_SetRenderDrawColorFloat(ctx.renderer, r, g, b, a);
                SDL_RenderFillRect(ctx.renderer, &(*rect).into());
            }
            if let Some(stroke) = stroke {
                let [r, g, b, a] = stroke.components;
                SDL_SetRenderDrawColorFloat(ctx.renderer, r, g, b, a);
                SDL_RenderRect(ctx.renderer, &(*rect).into());
            }
        };
    }
}

pub struct LineCommand {
    pub color: Color,
    pub from: Vec2,
    pub to: Vec2,
}

impl Command for LineCommand {
    fn bounds(&self) -> Rect {
        let min = self.from.min(self.to);
        let max = self.from.max(self.to);
        Rect::from((min, max - min))
    }

    fn clip(&self, clip_rect: Rect) -> Option<Box<dyn Command>> {
        let location = clip_rect.location();
        Some(Box::new(Self {
            color: self.color,
            from: self.from - location,
            to: self.to - location,
        }))
    }

    fn execute(&self, ctx: &mut Renderer) {
        let from = self.from;
        let to = self.to;
        unsafe {
            let [r, g, b, a] = self.color.components;
            SDL_SetRenderDrawColorFloat(ctx.renderer, r, g, b, a);
            SDL_RenderLine(ctx.renderer, from.x, from.y, to.x, to.y);
        }
    }
}

pub struct RenderTextureRotatedCommand {
    pub texture: *mut SDL_Texture,
    pub src_rect: Rect,
    pub dst_rect: Rect,
    pub angle: f64,
    pub center: Option<Vec2>,
    pub flip: SDL_FlipMode,
}

impl Command for RenderTextureRotatedCommand {
    fn bounds(&self) -> Rect {
        self.dst_rect
    }

    fn clip(&self, clip_rect: Rect) -> Option<Box<dyn Command>> {
        let (src_rect, dst_rect) = bin_clip(clip_rect, self.src_rect, self.dst_rect)?;
        Some(Box::new(Self {
            texture: self.texture,
            src_rect,
            dst_rect,
            angle: self.angle,
            center: self.center,
            flip: self.flip,
        }))
    }

    fn execute(&self, ctx: &mut Renderer) {
        unsafe {
            SDL_RenderTextureRotated(
                ctx.renderer,
                self.texture,
                &self.src_rect.into(),
                &(self.dst_rect).into(),
                self.angle,
                if let Some(center) = self.center {
                    &SDL_FPoint {
                        x: center.x,
                        y: center.y,
                    }
                } else {
                    std::ptr::null()
                },
                self.flip,
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderTextureNineGridCommand {
    pub texture: *mut SDL_Texture,
    pub src_rect: Rect,
    pub dst_rect: Rect,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub scale: f32,
}

impl Command for RenderTextureNineGridCommand {
    fn bounds(&self) -> Rect {
        self.dst_rect
    }

    fn clip(&self, clip_rect: Rect) -> Option<Box<dyn Command>> {
        if !clip_rect.intersect(&self.dst_rect) {
            return None;
        }
        let dst_rect = clip_rect.intersect_rect(&self.dst_rect);
        let left = self.left.min(dst_rect.x - self.dst_rect.x);
        let top = self.top.min(dst_rect.y - self.dst_rect.y);
        let right = self
            .right
            .min(self.dst_rect.x + self.dst_rect.width - dst_rect.x - dst_rect.width);
        let bottom = self
            .bottom
            .min(self.dst_rect.y + self.dst_rect.height - dst_rect.y - dst_rect.height);

        let dst_rect = dst_rect - clip_rect.location();

        Some(Box::new(Self {
            src_rect: Rect::new(
                self.src_rect.x + left,
                self.src_rect.y + top,
                self.src_rect.width - left - right,
                self.src_rect.height - top - bottom,
            ),
            left: self.left - left,
            top: self.top - top,
            right: self.right - right,
            bottom: self.bottom - bottom,
            dst_rect,
            ..self.clone()
        }))
    }

    fn execute(&self, ctx: &mut Renderer) {
        unsafe {
            SDL_RenderTexture9Grid(
                ctx.renderer,
                self.texture,
                &self.src_rect.into(),
                self.left,
                self.right,
                self.top,
                self.bottom,
                self.scale,
                &(self.dst_rect).into(),
            );
        }
    }
}
