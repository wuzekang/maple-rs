use character::{Character, ZMap};
use glam::{vec2, Vec2, Vec2Swizzles};
use hecs::World;
use sdl::Renderer;
use sdl3_sys::{
    events::{SDL_Event, SDL_EventType, SDL_PollEvent},
    init::{SDL_Init, SDL_INIT_VIDEO},
    keyboard::SDL_GetKeyboardState,
    render::{
        SDL_CreateRenderer, SDL_RenderClear, SDL_RenderPresent, SDL_Renderer,
        SDL_SetRenderDrawColor, SDL_SetRenderScale, SDL_SetRenderVSync,
    },
    scancode::SDL_Scancode,
    timer::{SDL_Delay, SDL_GetTicks},
    video::{SDL_CreateWindow, SDL_GetWindowPixelDensity, SDL_Window},
};
use std::{error::Error, mem::MaybeUninit, sync::Arc};
use ui::image::IntoDrawable;
use ui::reactive::create_rw_signal;
use ui::{
    reactive::{provide_context, SignalGet, SignalUpdate},
    taffy::prelude::*,
    Drawable, Element, IntoElement, Root,
};
use wz::Node;

mod character;
mod map;
mod math;
mod npc;
mod sdl;
mod sprite;
mod timer;
mod ui_view;
mod wz;

struct PollEvent {
    event: MaybeUninit<SDL_Event>,
}

unsafe impl Send for PollEvent {}
unsafe impl Sync for PollEvent {}

impl PollEvent {
    fn new() -> Self {
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

#[derive(Default, Clone)]
struct Camera {
    position: Vec2,
    direction: Vec2,
    speed: Vec2,
}

#[derive(Default)]
struct Player {
    avatar: Character,
    position: Vec2,
    direction: Vec2,
    speed: Vec2,
    flip: bool,
    foothold: i32,
}

struct Context {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    sprite_renderer: Renderer,
    size: Vec2,
    dpr: f32,
    ticks: u64,
    delta: f32,
    camera: Camera,
    world: World,
    events: Vec<SDL_Event>,
    state: *const bool,
}

impl Context {
    pub fn new() -> Self {
        unsafe {
            SDL_Init(SDL_INIT_VIDEO);
        }

        // let size = vec2(1024.0, 768.0);
        let size = vec2(800.0, 600.0);
        let window =
            unsafe { SDL_CreateWindow(c"Maple RS".as_ptr(), size.x as i32, size.y as i32, 0x2000) };
        let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

        let dpr = unsafe { SDL_GetWindowPixelDensity(window) };

        unsafe {
            SDL_SetRenderScale(renderer, dpr, dpr);
        }

        let sprite_renderer = sdl::Renderer::new(dpr, renderer);

        let ticks = unsafe { SDL_GetTicks() };

        Self {
            size,
            window,
            renderer,
            dpr,
            ticks,
            delta: 0.0,
            sprite_renderer,
            camera: Camera {
                speed: Vec2::ONE * 40.0,
                ..Default::default()
            },
            world: World::new(),
            events: Vec::new(),
            state: unsafe { SDL_GetKeyboardState(std::ptr::null_mut() as *mut core::ffi::c_int) },
        }
    }

    pub fn tick(&mut self) {
        let now = unsafe { SDL_GetTicks() };
        self.delta = (now - self.ticks) as f32;
        self.ticks = now;
    }
}

#[derive(Clone)]
struct WzBase {
    pub node: Node,
}

struct EventCollection(Vec<SDL_Event>);

unsafe impl Send for EventCollection {}
unsafe impl Sync for EventCollection {}

fn player_move_system(context: &mut Context, map: &map::Map) {
    context.tick();
    let state = context.state;
    let Context {
        world,
        events,
        size,
        camera,
        ..
    } = context;

    let world_size = *size;

    let pressed_left = unsafe { *state.offset(SDL_Scancode::LEFT.0 as isize) };
    let pressed_right = unsafe { *state.offset(SDL_Scancode::RIGHT.0 as isize) };
    let pressed_up = unsafe { *state.offset(SDL_Scancode::UP.0 as isize) };
    let pressed_down = unsafe { *state.offset(SDL_Scancode::DOWN.0 as isize) };

    let q = world.query_mut::<&mut Player>();
    let (_, player) = q.into_iter().next().unwrap();

    let prev = player.direction;

    for event in events {
        match SDL_EventType(unsafe { event.r#type }) {
            SDL_EventType::KEY_DOWN => match unsafe { event.key.scancode } {
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

            SDL_EventType::KEY_UP => match unsafe { event.key.scancode } {
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

        if prev.x == 0.0 && player.direction.x != 0.0 {
            player.avatar.set_action("walk1");
        }

        if prev.x != 0.0 && player.direction.x == 0.0 {
            player.avatar.set_action("stand1");
        }

        let direction = player.direction;
        let speed = player.speed;
        player.position += direction * speed;
    }

    camera.position = player.position - world_size / 2.0;
}

fn draw_back(world: &mut Context, item: &mut map::MapBackground) {
    let delta = world.delta;
    let camera_position = world.camera.position.clone();
    let size = world.size;
    let sprite_renderer = &world.sprite_renderer;
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

    let sprite = match &mut item.sprite {
        map::BackgroundSprite::Sprite(sprite) => sprite,
        map::BackgroundSprite::SpriteAnimation(animation) => animation.tick(delta),
    };
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
fn main() -> Result<(), Box<dyn Error>> {
    let base = wz::resolve_base().unwrap();
    let mut map = map::Map::new(&base, "002000000").unwrap();
    // let mut map = map::Map::new(&node, "222020111").unwrap();
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

    let mut context = Context::new();

    let player = Player {
        avatar: character::Character::new(
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

    context.world.spawn((player,));

    let mut events = PollEvent::new();

    provide_context(WzBase { node: base.clone() });

    let camera_signal = create_rw_signal(Camera::default());

    provide_context(camera_signal);

    let renderer = context.renderer.clone();
    let root = Root::new(ui_view::ui_view, renderer);

    unsafe {
        SDL_SetRenderVSync(renderer, 1);

        let mut exited = false;

        while !exited {
            let mut event_vec = vec![];
            for event in &mut events {
                root.dispatch_event(event);
                event_vec.push(event.clone());
                match SDL_EventType(event.r#type) {
                    SDL_EventType::QUIT => {
                        exited = true;
                    }
                    _ => {}
                }
            }
            context.events = event_vec;

            player_move_system(&mut context, &map);

            {
                let camera = &context.camera;
                if camera.position != camera_signal.get().position {
                    camera_signal.set(camera.clone());
                }
            }

            root.layout();

            SDL_SetRenderDrawColor(context.renderer, 0, 0, 0, 255);
            SDL_RenderClear(context.renderer);

            for item in &mut map.backgrounds {
                if !item.front {
                    draw_back(&mut context, item);
                }
            }

            let delta = context.delta;

            {
                let sprite_renderer = &context.sprite_renderer;

                for layer in &mut map.layers {
                    for item in &mut layer.objects {
                        item.timer.tick(delta);
                        let sprite = &item.sprites[item.timer.index.get()];
                        sprite_renderer.draw_flip(
                            sprite,
                            item.position - context.camera.position,
                            item.flip,
                        );
                    }

                    for item in &mut layer.tiles {
                        sprite_renderer.draw(&item.tile, item.position - context.camera.position);
                    }
                }
            }

            map.portal_timer.tick(delta);
            let sprite = &map.helper.pv[map.portal_timer.index.get()];
            for item in map.portals.iter() {
                if item.pn == "sp" {
                    continue;
                }
                context
                    .sprite_renderer
                    .draw(&sprite, item.position - context.camera.position);
            }

            {
                let sprite_renderer = &context.sprite_renderer;
                for item in &mut map.life {
                    if item.r#type == "n" {
                        let npc = map.npc.get_mut(&item.id).unwrap();
                        let action = npc.actions.get_mut("stand").unwrap();
                        action.timer.tick(delta);
                        let sprite = &action.frames[action.timer.index.get()];
                        sprite_renderer.draw_flip(
                            sprite,
                            vec2(item.x as f32, item.cy as f32) - context.camera.position,
                            item.f == 1,
                        );
                    }
                }
            }

            {
                let Context {
                    world,
                    sprite_renderer,
                    camera,
                    ..
                } = &mut context;
                let q = world.query_mut::<&mut Player>();
                let (_, player) = q.into_iter().next().unwrap();
                player.avatar.tick(delta);
                for sprite in player.avatar.frame() {
                    sprite_renderer.draw_flip(
                        &sprite,
                        player.position - camera.position,
                        player.flip,
                    )
                }
            }

            // tooltip_tex.draw(vec2(50.0, 50.0), vec2(400.0, 400.0));

            root.paint();
            SDL_RenderPresent(renderer);
            SDL_Delay(16);
        }
    }

    Ok(())
}
