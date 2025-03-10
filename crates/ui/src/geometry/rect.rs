use glam::{vec2, Vec2};
use sdl3_sys::everything::{SDL_FRect, SDL_Rect};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const INFINITY: Rect = Rect {
        x: f32::NEG_INFINITY,
        y: f32::NEG_INFINITY,
        width: f32::INFINITY,
        height: f32::INFINITY,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn intersect_rect(&self, rhs: &Self) -> Rect {
        // 计算两个矩形的左右上下边界
        let left_lhs = self.x;
        let right_lhs = self.x + self.width;
        let top_lhs = self.y;
        let bottom_lhs = self.y + self.height;

        let left_b = rhs.x;
        let right_b = rhs.x + rhs.width;
        let top_b = rhs.y;
        let bottom_b = rhs.y + rhs.height;

        // 计算相交区域的边界
        let left = left_lhs.max(left_b);
        let right = right_lhs.min(right_b);
        let top = top_lhs.max(top_b);
        let bottom = bottom_lhs.min(bottom_b);

        // 计算宽度和高度，并确保非负
        let width = (right - left).max(0.0);
        let height = (bottom - top).max(0.0);

        Rect {
            x: left,
            y: top,
            width,
            height,
        }
    }

    pub fn intersect(&self, rhs: &Self) -> bool {
        self.x.max(rhs.x) < (self.x + self.width).min(rhs.x + rhs.width)
            && self.y.max(rhs.y) < (self.y + self.height).min(rhs.y + rhs.height)
    }

    pub fn location(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let Vec2 { x, y } = point;
        let left = self.x;
        let top = self.y;
        let right = self.x + self.width;
        let bottom = self.y + self.height;
        x >= left && x < right && y >= top && y < bottom
    }
}

impl Add<Vec2> for Rect {
    type Output = Rect;
    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl Sub<Vec2> for Rect {
    type Output = Rect;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl AddAssign<Vec2> for Rect {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<Vec2> for Rect {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl From<(Vec2, Vec2)> for Rect {
    #[inline]
    fn from((position, size): (Vec2, Vec2)) -> Self {
        Rect {
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        }
    }
}

impl Into<SDL_Rect> for Rect {
    fn into(self) -> SDL_Rect {
        SDL_Rect {
            x: self.x as i32,
            y: self.y as i32,
            w: self.width as i32,
            h: self.height as i32,
        }
    }
}

impl Into<SDL_FRect> for Rect {
    fn into(self) -> SDL_FRect {
        SDL_FRect {
            x: self.x,
            y: self.y,
            w: self.width,
            h: self.height,
        }
    }
}
