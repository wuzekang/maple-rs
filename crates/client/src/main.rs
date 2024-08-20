use character::{Character, ZMap};
use glam::{vec2, Vec2, Vec2Swizzles};
use image::DynamicImage;
use map::world_map::WorldMap;
use sdl::{NineGridTexture, Renderer, Surface, Texture};
use sdl_sys::{
    self, SDL_CreateRenderer, SDL_CreateWindow, SDL_Delay, SDL_Event, SDL_EventType, SDL_FRect,
    SDL_GetKeyboardState, SDL_GetTicks, SDL_PollEvent, SDL_RenderClear, SDL_RenderPresent,
    SDL_Scancode::{
        self, SDL_SCANCODE_DOWN, SDL_SCANCODE_LEFT, SDL_SCANCODE_RIGHT, SDL_SCANCODE_UP,
    },
    SDL_SetRenderDrawColor, SDL_SetRenderVSync,
};
use slotmap::{DefaultKey, SlotMap};
use sprite::Sprite;
use ui::Button;
use wz::Node;

use std::{borrow::Borrow, error::Error, mem::MaybeUninit, sync::Arc};
use taffy::prelude::*;

mod character;
mod layout;
mod map;
mod npc;
mod sdl;
mod sprite;
mod timer;
mod ui;
mod wz;

pub fn intersect(p1: &Vec2, p2: &Vec2, p3: &Vec2, p4: &Vec2) -> Option<Vec2> {
    if (f32::max(p1.x, p2.x)) < f32::min(p3.x, p4.x)
        || (f32::max(p1.y, p2.y)) < f32::min(p3.y, p4.y)
        || (f32::max(p3.x, p4.x)) < f32::min(p1.x, p2.x)
        || (f32::max(p3.y, p4.y)) < f32::min(p1.y, p2.y)
    {
        return None;
    }

    if (((p1.x - p3.x) * (p4.y - p3.y) - (p1.y - p3.y) * (p4.x - p3.x))
        * ((p2.x - p3.x) * (p4.y - p3.y) - (p2.y - p3.y) * (p4.x - p3.x)))
        > 0.0
        || (((p3.x - p1.x) * (p2.y - p1.y) - (p3.y - p1.y) * (p2.x - p1.x))
            * ((p4.x - p1.x) * (p2.y - p1.y) - (p4.y - p1.y) * (p2.x - p1.x)))
            > 0.0
    {
        return None;
    }

    let base_x = (p4.x - p3.x) * (p1.y - p2.y) - (p2.x - p1.x) * (p3.y - p4.y);
    if base_x == 0.0 {
        return None;
    }
    let x = ((p1.y - p3.y) * (p2.x - p1.x) * (p4.x - p3.x) + p3.x * (p4.y - p3.y) * (p2.x - p1.x)
        - p1.x * (p2.y - p1.y) * (p4.x - p3.x))
        / base_x;

    let base_y = (p1.x - p2.x) * (p4.y - p3.y) - (p2.y - p1.y) * (p3.x - p4.x);
    if base_y == 0.0 {
        return None;
    }
    let y = (p2.y * (p1.x - p2.x) * (p4.y - p3.y) + (p4.x - p2.x) * (p4.y - p3.y) * (p1.y - p2.y)
        - p4.y * (p3.x - p4.x) * (p2.y - p1.y))
        / base_y;

    Some(Vec2::new(x, y))
}

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
            if SDL_PollEvent(self.event.as_mut_ptr()) == 1 {
                Some(&*self.event.as_ptr())
            } else {
                None
            }
        }
    }
}

#[derive(Default)]
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

struct World {
    window: *mut sdl_sys::SDL_Window,
    renderer: *mut sdl_sys::SDL_Renderer,
    sprite_renderer: Renderer,
    size: Vec2,
    dpr: f32,
    ticks: u64,
    delta: f32,
    camera: Camera,
}

impl World {
    pub fn new() -> Self {
        unsafe {
            sdl_sys::SDL_Init(sdl_sys::SDL_INIT_VIDEO);
        }

        // let size = vec2(1024.0, 768.0);
        let size = vec2(800.0, 600.0);
        let window =
            unsafe { SDL_CreateWindow(c"Maple RS".as_ptr(), size.x as i32, size.y as i32, 0x2000) };
        let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

        let dpr = unsafe { sdl_sys::SDL_GetWindowPixelDensity(window) };

        unsafe {
            sdl_sys::SDL_SetRenderScale(renderer, dpr, dpr);
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
        }
    }

    pub fn tick(&mut self) {
        let now = unsafe { SDL_GetTicks() };
        self.delta = (now - self.ticks) as f32;
        self.ticks = now;
    }
}

struct EventContext {
    pub listeners: SlotMap<DefaultKey, fn(&SDL_Event) -> ()>,
}

impl EventContext {
    pub fn new() -> Self {
        Self {
            listeners: SlotMap::new(),
        }
    }
}

struct WorldMapWindow {
    open: bool,
    world_map: WorldMap,
    title: sprite::Sprite,

    btn_close: ui::Button,
    map_image: Vec<Sprite>,

    background: NineGridTexture,
}

impl WorldMapWindow {
    pub fn new(world: &World, base: Node) -> Self {
        let world_map_node = base.at_path("UI/UIWindow.img/WorldMap").unwrap();
        let title: sprite::Sprite = world_map_node.get("title").into();
        let border: Vec<Arc<DynamicImage>> = base
            .at_path("UI/UIWindow.img/WorldMap/Border")
            .unwrap()
            .into();

        let map_image: Vec<Sprite> = base
            .at_path("Map/MapHelper.img/worldMap/mapImage")
            .unwrap()
            .into();

        let world_map =
            map::world_map::WorldMap::from(base.at_path("Map/WorldMap/WorldMap.img").unwrap());

        let btn_close = ui::Button::from(base.at_path("UI/Basic.img/BtClose").unwrap());

        let surfaces = border
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<Surface>>();

        let background = sdl::NineGridTexture::new(
            (
                &surfaces[0],
                &surfaces[1],
                &surfaces[2],
                &surfaces[3],
                &surfaces[4],
                &surfaces[5],
                &surfaces[6],
                &surfaces[7],
            ),
            world.renderer,
        );

        Self {
            open: true,
            world_map,
            title,
            btn_close,
            map_image,
            background,
        }
        // let tooltip_bg: DynamicImage = image::load_from_memory(include_bytes!("./tooltip.png"))
        //     .unwrap()
        //     .into();

        // let tooltip_tex = NineGridTexture {
        //     texture: Texture::from_image(&tooltip_bg, world.renderer),
        //     left_width: 4,
        //     middle_width: 0,
        //     right_width: 4,
        //     top_height: 4,
        //     middle_height: 0,
        //     bottom_height: 4,
        // };
    }

    pub fn render(&self, world: &mut World, mouse: Vec2) {
        if !self.open {
            return;
        }

        let tex = &self.background;
        let content_size = vec2(640.0, 470.0);
        let window_size = content_size + tex.border_size();
        let window_offset = (world.size - window_size) / 2.0;
        tex.draw(window_offset, window_size);

        let frame = &self.btn_close.normal.frames[0];
        world.sprite_renderer.draw(
            &self.title,
            window_offset + vec2(tex.left_width as f32 + 4.0, 9.5),
        );
        world.sprite_renderer.draw(
            frame,
            window_offset
                + vec2(window_size.x, 0.0)
                + vec2(-(tex.right_width as f32 + frame.image.width() as f32), 6.0),
        );

        let content_position = window_offset
            + vec2(tex.left_width as f32, tex.top_height as f32)
            + (content_size / 2.0);

        world
            .sprite_renderer
            .draw(&self.world_map.base_img, content_position);

        for (_, item) in self.world_map.map_link.iter() {
            let lt = content_position - item.link_img.origin;
            let rb = lt + item.link_img.size;
            if (mouse.cmpge(lt).all()) && mouse.cmplt(rb).all() {
                let pt = mouse - lt;

                let pixel = item
                    .link_img
                    .image
                    .as_rgba8()
                    .unwrap()
                    .get_pixel(pt.x as u32, pt.y as u32);

                if pixel.0[3] > 0 {
                    world.sprite_renderer.draw(&item.link_img, content_position);
                    break;
                }
            }

            // let size = vec2(
            //     item.link_img.image.width() as f32,
            //     item.link_img.image.height() as f32,
            // );
            // world.sprite_renderer.render_rect(&SDL_FRect {
            //     x: p.x,
            //     y: p.y,
            //     w: size.x,
            //     h: size.y,
            // })
        }

        for (_, item) in self.world_map.map_list.iter() {
            // if let Some(path) = &item.path {
            //     world.sprite_renderer.draw(path, content_position);
            // }
            let spot_image = &self.map_image[3];
            world
                .sprite_renderer
                .draw(spot_image, content_position + item.spot);
            // world.sprite_renderer.render_rect(&SDL_FRect {
            //     x: item.spot.x + content_position.x,
            //     y: item.spot.y + content_position.y,
            //     w: 20.0,
            //     h: 20.0,
            // })
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut tree: TaffyTree<()> = TaffyTree::new();

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
    let mut player = Player {
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

    let mut world = World::new();
    let mut events = PollEvent::new();
    let mut cx = EventContext::new();

    cx.listeners.insert(|event| unsafe {
        println!("l1 {:?}", event.type_);
    });
    cx.listeners.insert(|event| unsafe {
        println!("l2 {:?}", event.type_);
    });
    let state = unsafe { SDL_GetKeyboardState(std::ptr::null_mut() as *mut core::ffi::c_int) };

    let world_map_window = WorldMapWindow::new(&world, base);
    unsafe {
        SDL_SetRenderVSync(world.renderer, 1);

        let mut exited = false;

        while !exited {
            {
                world.tick();

                let camera = &mut world.camera;

                let pressed_left = *state.offset(SDL_SCANCODE_LEFT as isize) != 0;
                let pressed_right = *state.offset(SDL_SCANCODE_RIGHT as isize) != 0;
                let pressed_up = *state.offset(SDL_SCANCODE_UP as isize) != 0;
                let pressed_down = *state.offset(SDL_SCANCODE_DOWN as isize) != 0;

                let prev = player.direction;
                for event in &mut events {
                    for (_, listener) in &cx.listeners {
                        listener(event);
                    }
                    match event.type_ as SDL_EventType::Type {
                        SDL_EventType::SDL_EVENT_QUIT => {
                            exited = true;
                        }
                        SDL_EventType::SDL_EVENT_KEY_DOWN => match event.key.scancode {
                            SDL_Scancode::SDL_SCANCODE_LEFT => {
                                player.direction.x = -1.0;
                            }
                            SDL_Scancode::SDL_SCANCODE_RIGHT => {
                                player.direction.x = 1.0;
                            }
                            SDL_Scancode::SDL_SCANCODE_UP => {
                                player.direction.y = -1.0;
                            }
                            SDL_Scancode::SDL_SCANCODE_DOWN => {
                                player.direction.y = 1.0;
                            }
                            _ => {}
                        },

                        SDL_EventType::SDL_EVENT_KEY_UP => match event.key.scancode {
                            SDL_Scancode::SDL_SCANCODE_LEFT => {
                                player.direction.x = if pressed_right { 1.0 } else { 0.0 };
                            }
                            SDL_Scancode::SDL_SCANCODE_RIGHT => {
                                player.direction.x = if pressed_left { -1.0 } else { 0.0 };
                            }
                            SDL_Scancode::SDL_SCANCODE_UP => {
                                player.direction.y = if pressed_down { 1.0 } else { 0.0 };
                            }
                            SDL_Scancode::SDL_SCANCODE_DOWN => {
                                player.direction.y = if pressed_up { -1.0 } else { 0.0 };
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                if player.direction.x > 0.0 && !player.flip
                    || player.direction.x < 0.0 && player.flip
                {
                    player.flip = !player.flip;
                }

                if player.foothold == 0 {
                    player.avatar.set_action("jump");
                    let prev = player.position;
                    player.position += vec2(0.0, 0.5);
                    for (i, fh) in map.footholds.iter() {
                        if let Some(p) = intersect(&fh.start, &fh.end, &prev, &player.position) {
                            println!("foothold: {p:?}");
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

                camera.position = player.position - world.size / 2.0;
                // camera.position = player.position;
            }

            let mut mouse_x = MaybeUninit::<f32>::uninit();
            let mut mouse_y = MaybeUninit::<f32>::uninit();
            sdl_sys::SDL_GetMouseState(mouse_x.as_mut_ptr(), mouse_y.as_mut_ptr());

            let mouse = { vec2(mouse_x.assume_init() as f32, mouse_y.assume_init() as f32) };

            SDL_SetRenderDrawColor(world.renderer, 0, 0, 0, 255);
            SDL_RenderClear(world.renderer);

            fn draw_back(world: &mut World, item: &mut map::MapBackground) {
                let World {
                    delta,
                    size,
                    sprite_renderer,
                    ..
                } = world;
                let delta = *delta;

                let offset = world.camera.position + *size / 2.0;
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
                    map::Drawable::Sprite(sprite) => sprite,
                    map::Drawable::SpriteAnimation(animation) => animation.tick(delta),
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

                let hs = f32::ceil((world.camera.position.x - rb) / cw) as i32;
                let he = f32::ceil((world.camera.position.x + size.x - rb) / cw) as i32 + 1;

                let vs = f32::ceil((world.camera.position.y - bb) / ch) as i32;
                let ve = f32::ceil((world.camera.position.y + size.y - bb) / ch) as i32 + 1;

                match item.r#type {
                    1 | 4 => {
                        for i in hs..he {
                            sprite_renderer.draw_flip(
                                sprite,
                                vec2(x + i as f32 * cw, y) - world.camera.position,
                                item.flip,
                            );
                        }
                    }
                    2 | 5 => {
                        for i in vs..ve {
                            sprite_renderer.draw_flip(
                                sprite,
                                vec2(x, y + i as f32 * ch) - world.camera.position,
                                item.flip,
                            );
                        }
                    }
                    3 | 6 | 7 => {
                        for i in vs..ve {
                            for j in hs..he {
                                sprite_renderer.draw_flip(
                                    sprite,
                                    vec2(x + j as f32 * cw, y + i as f32 * ch)
                                        - world.camera.position,
                                    item.flip,
                                );
                            }
                        }
                    }
                    _ => {
                        sprite_renderer.draw_flip(
                            sprite,
                            vec2(x, y) - world.camera.position,
                            item.flip,
                        );
                    } // _ => {}
                }

                // unsafe {
                //     sdl_sys::SDL_RenderRect(
                //         renderer,
                //         &SDL_FRect {
                //             x: x - sprite.origin.x - world.camera.position.x,
                //             y: y - sprite.origin.y - world.camera.position.y,
                //             w,
                //             h,
                //         },
                //     );
                // }
                // sprite_renderer.draw(
                //     &sprite.image,
                //     sprite.origin,
                //     vec2(x as f32, y as f32) - world.camera.position,
                // );
            }

            // for item in &mut map.backgrounds {
            //     if !item.front {
            //         draw_back(&mut world, item);
            //     }
            // }

            // let delta = world.delta;

            // {
            //     let sprite_renderer = &mut world.sprite_renderer;

            //     for layer in &mut map.layers {
            //         for item in &mut layer.objects {
            //             item.timer.tick(delta);
            //             let sprite = &item.sprites[item.timer.index];
            //             sprite_renderer.draw_flip(
            //                 sprite,
            //                 item.position - world.camera.position,
            //                 item.flip,
            //             );
            //         }

            //         for item in &mut layer.tiles {
            //             sprite_renderer.draw(&item.tile, item.position - world.camera.position);
            //         }
            //     }
            // }

            // map.portal_timer.tick(delta);
            // let sprite = &map.helper.pv[map.portal_timer.index];
            // for item in map.portals.iter() {
            //     if item.pn == "sp" {
            //         continue;
            //     }
            //     world
            //         .sprite_renderer
            //         .draw(&sprite, item.position - world.camera.position);
            // }

            // {
            //     let sprite_renderer = &mut world.sprite_renderer;
            //     for item in &mut map.life {
            //         if item.r#type == "n" {
            //             let npc = map.npc.get_mut(&item.id).unwrap();
            //             let action = npc.actions.get_mut("stand").unwrap();
            //             action.timer.tick(delta);
            //             let sprite = &action.frames[action.timer.index];
            //             sprite_renderer.draw_flip(
            //                 sprite,
            //                 vec2(item.x as f32, item.cy as f32) - world.camera.position,
            //                 item.f == 1,
            //             );
            //         }
            //     }
            // }

            // {
            //     let sprite_renderer = &mut world.sprite_renderer;
            //     player.avatar.tick(delta);
            //     for sprite in player.avatar.frame() {
            //         sprite_renderer.draw_flip(
            //             &sprite,
            //             player.position - world.camera.position,
            //             player.flip,
            //         )
            //     }
            // }

            // for item in &mut map.backgrounds {
            //     if item.front {
            //         draw_back(&mut world, item);
            //     }
            // }

            // {
            //     let sprite_renderer = &mut world.sprite_renderer;
            //     for layer in &mut map.layers {
            //         for item in &mut layer.objects {
            //             for sprite in &item.sprites {
            //                 sprite_renderer.draw_text(
            //                     &format!("{}", sprite.path),
            //                     item.position - world.camera.position,
            //                 );
            //             }
            //         }
            //     }
            // }

            // world_map_window.render(&mut world, mouse);
            // tooltip_tex.draw(vec2(50.0, 50.0), vec2(400.0, 400.0));
            // sdl_sys::SDL_RenderTexture(
            //     world.renderer,
            //     tex.texture,
            //     std::ptr::null(),
            //     &SDL_FRect {
            //         x: 0.0,
            //         y: 0.0,
            //         w: tex.size.x,
            //         h: tex.size.y,
            //     },
            // );
            SDL_RenderPresent(world.renderer);
            SDL_Delay(16);
        }
    }

    Ok(())
}
