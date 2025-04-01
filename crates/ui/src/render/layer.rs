use crate::geometry::Rect;
use crate::render::command::Command;
use crate::{RenderFlag, Renderer, Texture};
use glam::{vec2, Vec2};
use peniko::Color;
use sdl3_sys::blendmode::{SDL_BlendFactor, SDL_BlendOperation, SDL_ComposeCustomBlendMode};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const TILE_WIDTH: usize = 256;
const TILE_HEIGHT: usize = 256;

pub struct Tile {
    dirty: Cell<bool>,
    texture: Option<Texture>,
    commands: Vec<Box<dyn Command>>,
}

impl Default for Tile {
    fn default() -> Self {
        Self::new()
    }
}

impl Tile {
    pub fn new() -> Self {
        Self {
            dirty: true.into(),
            texture: None,
            commands: Vec::new(),
        }
    }

    pub fn render(&mut self, ctx: &mut Renderer, alpha: f32) {
        if self.commands.is_empty() {
            return;
        }

        if self.dirty.get() {
            self.dirty.set(false);

            if self.texture.is_none() {
                let mode = unsafe {
                    SDL_ComposeCustomBlendMode(
                        SDL_BlendFactor::ONE,
                        SDL_BlendFactor::ONE_MINUS_SRC_ALPHA,
                        SDL_BlendOperation::ADD,
                        SDL_BlendFactor::ONE,
                        SDL_BlendFactor::ONE_MINUS_SRC_ALPHA,
                        SDL_BlendOperation::ADD,
                    )
                };

                let texture = ctx
                    .create_texture(vec2(TILE_WIDTH as f32, TILE_HEIGHT as f32) * ctx.dpr)
                    .blend_mode(mode)
                    .scale_mode_nearest();

                self.texture = Some(texture);
            }

            let texture = self.texture.as_ref().unwrap();

            ctx.with_target(texture, |ctx| {
                ctx.save();
                ctx.reset();
                ctx.clear();
                for command in self.commands.iter() {
                    command.execute(ctx);
                    if ctx.flags.contains(&RenderFlag::LayerDebug) {
                        ctx.fill_rect(Color::BLACK.with_alpha(0.2), command.bounds())
                    }
                }
                ctx.restore();
            });
        }

        let texture = self.texture.as_ref().unwrap();

        ctx.render_texture(
            texture.texture,
            Rect::from((Vec2::ZERO, texture.size)),
            Rect::from((Vec2::ZERO, texture.size / ctx.dpr)),
            Some(Color::new([alpha, alpha, alpha, alpha])),
        );

        ctx.draw_tiles.0 += 1;

        if ctx.flags.contains(&RenderFlag::LayerDebug) {
            ctx.stroke_rect(
                Color::BLACK.with_alpha(0.5),
                Rect::from((Vec2::ZERO, texture.size / ctx.dpr)),
            );
        }
    }
}

#[derive(Clone)]
pub struct Layer {
    pub bounds: Rect,
    pub tiles: Rc<RefCell<Vec<Tile>>>,
}

impl Layer {
    pub fn new(bounds: Rect) -> Self {
        let width = bounds.width as usize;
        let height = bounds.height as usize;
        let size = width.div_ceil(TILE_WIDTH) * height.div_ceil(TILE_HEIGHT);
        let mut tiles = Vec::with_capacity(size);
        for _ in 0..size {
            tiles.push(Tile::new());
        }
        Self {
            bounds,
            tiles: Rc::new(RefCell::new(tiles)),
        }
    }

    pub fn command(&self, command: Box<dyn Command + 'static>) {
        let location = self.bounds.location();
        let mut tiles = self.tiles.borrow_mut();
        let clip_rect = (command.bounds() - location)
            .intersect_rect(&Rect::from((Vec2::ZERO, self.bounds.size())));
        if clip_rect.width == 0.0 || clip_rect.height == 0.0 {
            return;
        }
        for (i, x, y) in self.clip_tiles(clip_rect) {
            if let Some(command) = command.clip(Rect::from((
                location + vec2((x * TILE_WIDTH) as f32, (y * TILE_HEIGHT) as f32),
                vec2(TILE_WIDTH as f32, TILE_HEIGHT as f32),
            ))) {
                tiles[i].commands.push(command);
            }
        }
    }

    pub fn render(&self, ctx: &mut Renderer, alpha: f32) {
        let clip_rect = ctx
            .clip
            .intersect_rect(&Rect::from((Vec2::ZERO, self.bounds.size())));

        if clip_rect.width == 0.0 || clip_rect.height == 0.0 {
            return;
        }

        let mut tiles = self.tiles.borrow_mut();

        if ctx.flags.contains(&RenderFlag::LayerDebug) {
            ctx.stroke_rect(
                Color::new([0.0, 0.0, 1.0, 1.0]),
                Rect::from((Vec2::ONE, self.bounds.size() - Vec2::ONE * 2.0)),
            );
        }

        for (i, x, y) in self.clip_tiles(clip_rect) {
            ctx.save();
            ctx.translate(Vec2::new((x * TILE_WIDTH) as f32, (y * TILE_HEIGHT) as f32));
            tiles[i].render(ctx, alpha);
            ctx.restore();
        }
    }

    pub fn clip_tiles(&self, clip_rect: Rect) -> TileIterator {
        TileIterator::new(clip_rect, self.bounds)
    }
}

pub struct TileIterator {
    x: usize,
    y: usize,
    end_x: usize,
    end_y: usize,
    columns: usize,
    // rows: usize,
}

impl TileIterator {
    pub fn new(clip_rect: Rect, bounds: Rect) -> Self {
        let Rect {
            x,
            y,
            width,
            height,
        } = clip_rect;
        let start_x = x as usize / TILE_WIDTH;
        let end_x = ((x + width) as usize).div_ceil(TILE_WIDTH);
        let start_y = y as usize / TILE_HEIGHT;
        let end_y = ((y + height) as usize).div_ceil(TILE_HEIGHT);

        let columns = (bounds.width as usize).div_ceil(TILE_WIDTH);
        // let rows = (bounds.height as usize).div_ceil(TILE_HEIGHT);

        // dbg!(
        //     clip_rect,
        //     start_x,
        //     end_x,
        //     start_y,
        //     end_y,
        //     (end_x - start_x + 1) * (end_y - start_y + 1),
        //     columns * rows
        // );

        Self {
            x: start_x,
            y: start_y,
            end_x,
            end_y,
            columns,
            // rows,
        }
    }
}

impl Iterator for TileIterator {
    type Item = (usize, usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.end_x {
            self.y += 1;
            self.x = 0;
        }
        if self.y >= self.end_y {
            return None;
        }
        let x = self.x;
        self.x += 1;
        Some((x + self.columns * self.y, x, self.y))
    }
}
