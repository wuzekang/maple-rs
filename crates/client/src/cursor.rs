use crate::sprite::SpriteAnimation;
use crate::WzBase;
use glam::Vec2;
use sdl3_sys::everything::*;
use ui::reactive::use_context;
use ui::style::Cursor;
use ui::{input, Bounds, Drawable, Renderer};

#[derive(Debug, Copy, Clone)]
pub enum CursorState {
    Idle,
    LClick,
    Game,
    House,
    Grab,
    Gift,
    VScroll,
    HScroll,
    VScrollIdle,
    HScrollIdle,
    Grabbing,
    Clicking,
    RClick,
}

impl CursorState {
    fn index(&self) -> u8 {
        match self {
            CursorState::Idle => 0,
            CursorState::LClick => 1,
            CursorState::Game => 2,
            CursorState::House => 3,
            CursorState::Grab => 5,
            CursorState::Gift => 6,
            CursorState::VScroll => 7,
            CursorState::HScroll => 8,
            CursorState::VScrollIdle => 9,
            CursorState::HScrollIdle => 10,
            CursorState::Grabbing => 11,
            CursorState::Clicking => 12,
            CursorState::RClick => 13,
        }
    }

    fn clicking_state(&self) -> CursorState {
        match self {
            CursorState::Idle => CursorState::Clicking,
            CursorState::LClick => CursorState::Clicking,
            CursorState::Game => CursorState::Clicking,
            CursorState::House => CursorState::Clicking,
            CursorState::Grab => CursorState::Grabbing,
            CursorState::Gift => CursorState::Clicking,
            CursorState::VScroll => CursorState::Clicking,
            CursorState::HScroll => CursorState::Clicking,
            CursorState::VScrollIdle => CursorState::Clicking,
            CursorState::HScrollIdle => CursorState::Clicking,
            CursorState::Grabbing => CursorState::Grabbing,
            CursorState::Clicking => CursorState::Clicking,
            CursorState::RClick => CursorState::Clicking,
        }
    }
}

impl From<CursorState> for Cursor {
    fn from(val: CursorState) -> Self {
        cursor(val)
    }
}

pub fn cursor_image(state: CursorState) -> SpriteAnimation {
    let WzBase { node: base } = use_context().unwrap();
    let basic = base.at_path("UI/Basic.img").unwrap();
    let cursor: SpriteAnimation = basic
        .at_path(&format!("Cursor/{}", state.index()))
        .unwrap()
        .try_into()
        .unwrap();
    cursor
}

pub fn cursor(state: CursorState) -> Cursor {
    Cursor::from_drawable({
        let cursor = DefaultCursor::new(state);
        move || Box::new(cursor.clone())
    })
}

#[derive(Clone)]
pub struct DefaultCursor {
    idle_image: SpriteAnimation,
    clicking_image: SpriteAnimation,
    clicking: bool,
}

impl DefaultCursor {
    pub fn new(state: CursorState) -> Self {
        Self {
            clicking: false,
            clicking_image: cursor_image(state.clicking_state()),
            idle_image: cursor_image(state),
        }
    }
}

impl Drawable for DefaultCursor {
    fn draw(&self, painter: &mut Renderer) {
        if self.clicking {
            self.clicking_image.draw(painter);
        } else {
            self.idle_image.draw(painter);
        }
    }

    fn size(&self) -> Vec2 {
        if self.clicking {
            self.clicking_image.size()
        } else {
            self.idle_image.size()
        }
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.idle_image.set_bounds(bounds.clone());
        self.clicking_image.set_bounds(bounds);
    }

    fn update(&mut self, delta: f32) -> bool {
        let clicking = self.clicking;
        self.clicking = input::mouse_button_pressed(SDL_BUTTON_LMASK);
        let updated = if self.clicking {
            self.clicking_image.update(delta)
        } else {
            self.idle_image.update(delta)
        };
        updated || clicking != self.clicking
    }
}
