use crate::character::Character;
use crate::character::ZMap;
use crate::map;
use crate::sprite::SpriteRenderer;
use crate::wz;
use glam::{vec2, Vec2};
use sdl3_sys::everything::*;
use std::sync::Arc;
use ui::element::Node;
use ui::event::{use_key, Event};
use ui::geometry;
use ui::peniko::Color;
use ui::reactive::{create_rw_signal, use_context, RwSignal, SignalGet, SignalUpdate};
use ui::style::dimension::length;
use ui::style::{StyleTrigger, Styleable};
use ui::taffy::Position;
use ui::{dynamic, fragment, input, view, Drawable, Element, IntoElement, Renderer, ViewId};

#[derive(Default, Clone)]
pub struct Camera {
    pub position: Vec2,
    pub size: Vec2,
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
    id: ViewId,
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    size: Vec2,
    camera: Camera,
    pub camera_signal: RwSignal<Camera>,
    player: Option<Player>,
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
        let z_map: Arc<ZMap> = Arc::new(base.at_path("zmap.img").unwrap().try_into().unwrap());

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
            speed: Vec2::ONE * 500.0,
            ..Default::default()
        };

        (player, map)
    }

    pub fn new(map: map::Map) -> Self {
        let text_visible = create_rw_signal(false);
        use_key(SDLK_T, move || {
            text_visible.set(!text_visible.get());
        });
        let camera_signal = create_rw_signal(Camera::default());

        let window = use_context().unwrap();
        let renderer = use_context().unwrap();

        let size = vec2(800.0, 600.0);

        let t = map.info.vr_top.unwrap() as f32;
        let b = map.info.vr_bottom.unwrap() as f32;
        let l = map.info.vr_left.unwrap() as f32;
        let r = map.info.vr_right.unwrap() as f32;
        let vr_size = vec2(r - l, b - t);

        let mut texts = vec![];
        for layer in &map.layers {
            for item in &layer.objects {
                for sprite in &item.sprites {
                    let path = sprite.path.clone();
                    let position = item.position;
                    texts.push(
                        view()
                            // .composite()
                            .style(move |s| {
                                s.absolute()
                                    .left(position.x - l)
                                    .top(position.y - t)
                                    .font_size(12.0)
                                    .line_height(14.0)
                                    .bg_white()
                            })
                            .children(ui::text({ move || path.clone() })),
                    );
                }
            }
        }

        let texts = fragment(texts).into_element();

        let camera = Camera {
            size,
            ..Default::default()
        };

        let id = view()
            .style(move |s| {
                s.absolute()
                    .left(0)
                    .top(0)
                    .width(size.x)
                    .height(size.y)
            })
            .children(
                view()
                    .style(move |s| {
                        let camera = camera_signal.get();
                        s.absolute()
                            .left(0)
                            .top(0)
                            .width(vr_size.x)
                            .height(vr_size.y)
                            .translate_x(-camera.position.x + l)
                            .translate_y(-camera.position.y + t)
                    })
                    .composite()
                    .children((dynamic({
                        move || {
                            if text_visible.get() {
                                texts.clone()
                            } else {
                                Node::Fragment(vec![])
                            }
                        }
                    }),)),
            )
            .id();

        Self {
            id,
            size,
            window,
            renderer,
            camera,
            player: None,
            map,
            camera_signal,
        }
    }

    pub fn set_camera_position(&mut self, position: Vec2) {
        self.camera.position = position;
        self.camera_signal.set(self.camera.clone());
    }

    pub fn move_camera_position(&mut self, offset: Vec2) {
        self.camera.position += offset;
        self.camera_signal.set(self.camera.clone());
    }

    pub fn set_player(&mut self, player: Player) {
        self.player = Some(player);
    }
}

impl Element for MainScene {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "MainScene".to_string()
    }

    fn update(&mut self, delta: f32) {
        // self.id.request_repaint(StyleTrigger::Paint);

        player_move(self, delta);

        {
            let Self {
                camera,
                camera_signal,
                map,
                size,
                ..
            } = self;

            let camera_position = camera.position;

            for item in &mut map.backgrounds {
                update_back(delta, camera_position, *size, item);
            }
        }

        let Self { map, player, .. } = self;

        for layer in &mut map.layers {
            for item in &mut layer.objects {
                item.update(delta);
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

        if let Some(player) = self.player.as_mut() {
            player.avatar.tick(delta);
        }
    }

    fn event(&mut self, event: &mut Event) {
        let Self { player, .. } = self;

        if let Some(player) = player.as_mut() {
            let pressed_left = input::key_pressed(SDL_Scancode::LEFT);
            let pressed_right = input::key_pressed(SDL_Scancode::RIGHT);
            let pressed_up = input::key_pressed(SDL_Scancode::UP);
            let pressed_down = input::key_pressed(SDL_Scancode::DOWN);

            if let Some(event) = event.is_key_down() {
                match event.scancode {
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
                }
            } else if let Some(event) = event.is_key_up() {
                match event.scancode {
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
                }
            }
        }
    }

    fn paint(&self, renderer: &mut Renderer) {
        let Self {
            size,
            camera,
            map,
            player,
            ..
        } = self;

        let sprite_renderer = &mut SpriteRenderer::new(renderer);
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
            if (item.pt != 7) {
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

        if let Some(player) = player.as_ref() {
            for sprite in player.avatar.frame() {
                sprite_renderer.draw_flip(&sprite, player.position - camera_position, player.flip)
            }
        }

        let t = map.info.vr_top.unwrap() as f32 - camera_position.y;
        let b = map.info.vr_bottom.unwrap() as f32 - camera_position.y;
        let l = map.info.vr_left.unwrap() as f32 - camera_position.x;
        let r = map.info.vr_right.unwrap() as f32 - camera_position.x;
        let vr_size = vec2(r - l, b - t);
        // renderer.set_color(Color::RED);
        // renderer.lines(&[vec2(l, t), vec2(r, t), vec2(r, b), vec2(l, b), vec2(l, t)]);

        // if world_size.x > vr_size.x {
        //     let len = (world_size.x - vr_size.x) / 2.0;
        //     renderer.fill_rect(Color::BLACK, Vec2::ZERO, vec2(len, world_size.y));
        //     renderer.fill_rect(
        //         Color::BLACK,
        //         vec2(world_size.x - len, 0.0),
        //         vec2(len, world_size.y),
        //     );
        // }
        // if world_size.y > vr_size.y {
        //     let len = (world_size.y - vr_size.y) / 2.0;
        //     renderer.fill_rect(Color::BLACK, Vec2::ZERO, vec2(world_size.x, len));
        //     renderer.fill_rect(
        //         Color::BLACK,
        //         vec2(0.0, world_size.y - len),
        //         vec2(world_size.x, len),
        //     );
        // }
    }
}

fn player_move(context: &mut MainScene, delta: f32) {
    let MainScene {
        player,
        size,
        camera,
        camera_signal,
        map,
        ..
    } = context;
    if player.is_none() {
        return;
    }
    let player = player.as_mut().unwrap();

    let world_size = *size;

    if player.direction.x > 0.0 && !player.flip || player.direction.x < 0.0 && player.flip {
        player.flip = !player.flip;
    }

    if player.foothold == 0 {
        player.avatar.set_action("jump");
        let prev = player.position;
        player.position += vec2(0.0, 200.0) * delta / 1000.0;
        for (i, fh) in map.footholds.iter() {
            if let Some(p) = geometry::intersect(&fh.start, &fh.end, &prev, &player.position) {
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
        player.position += direction * speed * delta / 1000.0;
    }

    camera.position = player.position - world_size / 2.0;
    let vr_left = map.info.vr_left.unwrap() as f32;
    let vr_right = map.info.vr_right.unwrap() as f32;
    let vr_top = map.info.vr_top.unwrap() as f32;
    let vr_bottom = map.info.vr_bottom.unwrap() as f32;
    let vr_size = vec2(vr_right - vr_left, vr_bottom - vr_top);

    if vr_size.x < world_size.x {
        camera.position.x = vr_left - (world_size.x - vr_size.x) / 2.0;
    } else {
        camera.position.x = (player.position.x - world_size.x / 2.0)
            .max(vr_left)
            .min(vr_right - world_size.x);
    }
    if vr_size.y < world_size.y {
        camera.position.y = vr_top - (world_size.y - vr_size.y) / 2.0;
    } else {
        camera.position.y = (player.position.y - world_size.y + 240.0)
            .max(vr_top)
            .min(vr_bottom - world_size.y);
    }
    if camera.position != camera_signal.get().position {
        camera_signal.set(camera.clone());
    }
}

fn update_back(delta: f32, camera_position: Vec2, size: Vec2, item: &mut map::MapBackground) {
    let camera_position = camera_position;
    let size = size;
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
    sprite_renderer: &mut SpriteRenderer,
    item: &map::MapBackground,
) {
    let camera_position = camera_position;

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
