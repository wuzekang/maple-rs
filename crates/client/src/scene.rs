use crate::character::Character;
use crate::character::ZMap;
use crate::map;
use crate::math;
use crate::sprite::SpriteRenderer;
use crate::wz;
use glam::{vec2, Vec2};
use sdl3_sys::everything::{SDL_GetKeyboardState, SDL_Renderer, SDL_Scancode, SDL_Window};
use std::sync::Arc;
use ui::event::{Event, EventType, KeyboardEvent};
use ui::reactive::{RwSignal, SignalGet, SignalUpdate};
use ui::Drawable;

#[derive(Default, Clone)]
pub struct Camera {
    pub position: Vec2,
    pub direction: Vec2,
    pub speed: Vec2,
}

#[derive(Default)]
pub struct Player {
    avatar: Character,
    position: Vec2,
    direction: Vec2,
    speed: Vec2,
    flip: bool,
    foothold: i32,
}

pub struct MainScene {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    size: Vec2,
    camera: Camera,
    camera_signal: RwSignal<Camera>,
    player: Player,
    state: *const bool,
    map: map::Map,
}

impl MainScene {
    pub fn resource(map_name: &str) -> (Player, map::Map) {
        let base = wz::resolve_base().unwrap();
        let mut map = map::Map::new(base.clone(), map_name.to_string()).unwrap();
        let position = map.portals.iter().fold(None, |acc: Option<Vec2>, item| {
            if item.pn != "sp" {
                return acc;
            }
            if let Some(prev) = acc {
                if item.position.length() > prev.length() {
                    Some(item.position)
                } else {
                    acc
                }
            } else {
                Some(item.position)
            }
        });
        let z_map: Arc<ZMap> = Arc::new(base.at_path("zmap.img").unwrap().into());

        let player = Player {
            avatar: Character::new(
                [
                    "00002000",
                    "00012000",
                    "Hair/00030000",
                    "Coat/01040036",
                    "Pants/01060026",
                    "Shoes/01071000",
                    "Face/00020000",
                ]
                .iter()
                .map(|path| base.at_path(&format!("Character/{path}.img")).unwrap())
                .collect(),
                z_map,
            ),
            position: position.unwrap_or_default(),
            direction: Vec2::ZERO,
            speed: Vec2::ONE * 40.0,
            ..Default::default()
        };

        (player, map)
    }

    pub fn new(
        window: *mut SDL_Window,
        renderer: *mut SDL_Renderer,
        size: Vec2,
        camera_signal: RwSignal<Camera>,
        player: Player,
        map: map::Map,
    ) -> Self {
        Self {
            size,
            window,
            renderer,
            camera: Camera {
                speed: Vec2::ONE * 40.0,
                ..Default::default()
            },
            player,
            state: unsafe { SDL_GetKeyboardState(std::ptr::null_mut() as *mut core::ffi::c_int) },
            map,
            camera_signal,
        }
    }

    pub fn event(&mut self, event: &mut dyn Event) {
        let Self { state, player, .. } = self;

        let pressed_left = unsafe { *state.offset(SDL_Scancode::LEFT.0 as isize) };
        let pressed_right = unsafe { *state.offset(SDL_Scancode::RIGHT.0 as isize) };
        let pressed_up = unsafe { *state.offset(SDL_Scancode::UP.0 as isize) };
        let pressed_down = unsafe { *state.offset(SDL_Scancode::DOWN.0 as isize) };

        if let Some(event) = event.as_any_mut().downcast_ref::<KeyboardEvent>() {
            match event.r#type {
                EventType::KeyDown => match event.scancode {
                    SDL_Scancode::LEFT => {
                        player.direction.x = -1.0;
                    }
                    SDL_Scancode::RIGHT => {
                        player.direction.x = 1.0;
                    }
                    SDL_Scancode::UP => {
                        player.direction.y = -1.0;
                    }
                    SDL_Scancode::DOWN => {
                        player.direction.y = 1.0;
                    }
                    _ => {}
                },

                EventType::KeyUp => match event.scancode {
                    SDL_Scancode::LEFT => {
                        player.direction.x = if pressed_right { 1.0 } else { 0.0 };
                    }
                    SDL_Scancode::RIGHT => {
                        player.direction.x = if pressed_left { -1.0 } else { 0.0 };
                    }
                    SDL_Scancode::UP => {
                        player.direction.y = if pressed_down { 1.0 } else { 0.0 };
                    }
                    SDL_Scancode::DOWN => {
                        player.direction.y = if pressed_up { -1.0 } else { 0.0 };
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn update(&mut self, delta: f32) {
        player_move(self);

        {
            let Self {
                camera,
                camera_signal,
                map,
                size,
                ..
            } = self;
            if camera.position != camera_signal.get().position {
                camera_signal.set(camera.clone());
            }

            let camera_position = camera.position;

            for item in &mut map.backgrounds {
                update_back(delta, camera_position, *size, item);
            }
        }

        let Self { map, player, .. } = self;

        for layer in &mut map.layers {
            for item in &mut layer.objects {
                item.timer.tick(delta);
            }
        }

        map.portal_timer.tick(delta);

        for item in &map.life {
            if item.r#type == "n" {
                let npc = map.npc.get_mut(&item.id).unwrap();
                if npc.actions.len() == 0 {
                    continue;
                }
                let action = npc.actions.get_mut("stand").unwrap();
                action.timer.tick(delta);
            }
        }

        player.avatar.tick(delta);
    }
}

impl Drawable for MainScene {
    fn draw(&self, renderer: &ui::Renderer) {
        let Self {
            size,
            camera,
            map,
            player,
            ..
        } = self;

        let sprite_renderer = &SpriteRenderer::new(renderer);
        let camera_position = camera.position;

        for item in &map.backgrounds {
            if !item.front {
                draw_back(camera_position, *size, sprite_renderer, item);
            }
        }

        for layer in &map.layers {
            for item in &layer.objects {
                let sprite = &item.sprites[item.timer.index];
                sprite_renderer.draw_flip(sprite, item.position - camera_position, item.flip);
            }

            for item in &layer.tiles {
                sprite_renderer.draw(&item.tile, item.position - camera_position);
            }
        }

        let sprite = &map.helper.pv[map.portal_timer.index];
        for item in map.portals.iter() {
            if item.pn == "sp" {
                continue;
            }
            sprite_renderer.draw(&sprite, item.position - camera_position);
        }

        for item in &map.life {
            if item.r#type == "n" {
                let npc = map.npc.get(&item.id).unwrap();
                if npc.actions.len() == 0 {
                    continue;
                }
                let action = npc.actions.get("stand").unwrap();
                let sprite = &action.frames[action.timer.index];
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(item.x as f32, item.cy as f32) - camera_position,
                    item.f == 1,
                );
            }
        }

        for sprite in player.avatar.frame() {
            sprite_renderer.draw_flip(&sprite, player.position - camera.position, player.flip)
        }
    }

    fn size(&self) -> Vec2 {
        self.size
    }
}

fn player_move(context: &mut MainScene) {
    let MainScene {
        player,
        size,
        camera,
        map,
        ..
    } = context;

    let world_size = *size;

    if player.direction.x > 0.0 && !player.flip || player.direction.x < 0.0 && player.flip {
        player.flip = !player.flip;
    }

    if player.foothold == 0 {
        player.avatar.set_action("jump");
        let prev = player.position;
        player.position += vec2(0.0, 0.5);
        for (i, fh) in map.footholds.iter() {
            if let Some(p) = math::intersect(&fh.start, &fh.end, &prev, &player.position) {
                player.position = p;
                player.foothold = *i;
                player.avatar.set_action("stand1");
                break;
            }
        }
    } else {
        let fh = map.footholds.get(&player.foothold).unwrap();

        if player.direction.x == 0.0 {
            player.avatar.set_action("stand1");
        } else {
            player.avatar.set_action("walk1");
        }

        if player.direction.y > 0.0 {
            player.avatar.set_action("prone");
        }

        let direction = player.direction;
        let speed = player.speed;
        player.position += direction * speed;
    }

    camera.position = player.position - world_size / 2.0;
}

fn update_back(delta: f32, camera_position: Vec2, size: Vec2, item: &mut map::MapBackground) {
    let offset = camera_position + size / 2.0;

    match item.r#type {
        4 | 6 => {
            item.offset_x += item.rx as f32 * 5.0 * delta / 1000.0;
            item.offset_y = item.y + offset.y * (item.ry + 100) as f32 / 100.0;
        }
        5 | 7 => {
            item.offset_x = item.x + offset.x * (item.rx + 100) as f32 / 100.0;
            item.offset_y += item.ry as f32 * 5.0 * delta / 1000.0;
        }
        _ => {
            item.offset_x = item.x + offset.x * (item.rx + 100) as f32 / 100.0;
            item.offset_y = item.y + offset.y * (item.ry + 100) as f32 / 100.0;
        }
    }

    match &mut item.sprite {
        map::BackgroundSprite::SpriteAnimation(animation) => {
            animation.tick(delta);
        }
        _ => {}
    };
}

fn draw_back(
    camera_position: Vec2,
    size: Vec2,
    sprite_renderer: &SpriteRenderer,
    item: &map::MapBackground,
) {
    let sprite = item.sprite.current_frame();
    let w = sprite.image.width() as f32;
    let h = sprite.image.height() as f32;
    let cw = if item.cx > 0 { item.cx as f32 } else { w };
    let ch = if item.cy > 0 { item.cy as f32 } else { h };

    let x = item.offset_x;
    let y = item.offset_y;
    let lb = x - sprite.origin.x;
    let rb = lb + w;
    let tb = y - sprite.origin.y;
    let bb = tb + h;

    let hs = f32::ceil((camera_position.x - rb) / cw) as i32;
    let he = f32::ceil((camera_position.x + size.x - rb) / cw) as i32 + 1;

    let vs = f32::ceil((camera_position.y - bb) / ch) as i32;
    let ve = f32::ceil((camera_position.y + size.y - bb) / ch) as i32 + 1;

    match item.r#type {
        1 | 4 => {
            for i in hs..he {
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(x + i as f32 * cw, y) - camera_position,
                    item.flip,
                );
            }
        }
        2 | 5 => {
            for i in vs..ve {
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(x, y + i as f32 * ch) - camera_position,
                    item.flip,
                );
            }
        }
        3 | 6 | 7 => {
            for i in vs..ve {
                for j in hs..he {
                    sprite_renderer.draw_flip(
                        sprite,
                        vec2(x + j as f32 * cw, y + i as f32 * ch) - camera_position,
                        item.flip,
                    );
                }
            }
        }
        _ => {
            sprite_renderer.draw_flip(sprite, vec2(x, y) - camera_position, item.flip);
        }
    }
}
