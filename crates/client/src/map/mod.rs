use std::cell::RefCell;
use glam::{vec2, Vec2};
use std::collections::HashMap;
use std::rc::Rc;
use wz_reader::node::Error;

use crate::npc::Npc;
use crate::sprite::{self, Sprite, SpriteAnimation};
use crate::timer::Timer;
use crate::{map, Context};
use crate::wz::Node;

pub mod world_map;

pub struct MapHelper {
    pub pv: Vec<Sprite>,
}

impl From<Node> for MapHelper {
    fn from(node: Node) -> Self {
        Self {
            pv: node.at_path("portal/game/pv").unwrap().into(),
        }
    }
}

pub struct MapTile {
    pub id: i32,
    pub tile: Sprite,
    pub position: Vec2,
}

#[derive(Debug)]
pub struct Portal {
    pub pn: String,
    pub pt: i32,
    pub position: Vec2,
}

impl From<Node> for Portal {
    fn from(node: Node) -> Self {
        Self {
            pn: node.get("pn").into(),
            pt: node.get("pt").into(),
            position: vec2(
                i32::from(node.get("x")) as f32,
                i32::from(node.get("y")) as f32,
            ),
        }
    }
}

#[derive(Default)]
pub struct Foothold {
    pub start: Vec2,
    pub end: Vec2,
    pub prev: i32,
    pub next: i32,
    pub page: i32,
    pub z_mass: i32,
}

pub struct MapInfo {
    vr_top: i32,
    vr_bottom: i32,
    vr_left: i32,
    vr_right: i32,
}

impl From<Node> for MapInfo {
    fn from(node: Node) -> Self {
        Self {
            vr_top: node.get("VRTop").into(),
            vr_bottom: node.get("VRBottom").into(),
            vr_left: node.get("VRLeft").into(),
            vr_right: node.get("VRRight").into(),
        }
    }
}

pub struct MapObject {
    id: i32,
    pub timer: Timer,
    pub flip: bool,
    pub sprites: Vec<Sprite>,
    pub position: Vec2,
    pub z: i32,
}

pub enum MapItem {
    Tile(MapTile),
    Object(MapObject),
}

impl MapItem {
    pub fn z(&self) -> i32 {
        match self {
            MapItem::Tile(item) => item.tile.z,
            MapItem::Object(item) => item.z,
        }
    }
}

pub struct MapLayer {
    pub tiles: Vec<MapTile>,
    pub objects: Vec<MapObject>,
}

// {
//     "a": 255,
//     "ani": 0,
//     "bS": "midForest",
//     "cx": 0,
//     "cy": 0,
//     "f": 0,
//     "front": 0,
//     "no": 0,
//     "rx": 0,
//     "ry": 0,
//     "type": 3,
//     "x": 0,
//     "y": 0
// }
pub struct MapBackground {
    // bS
    pub bs: String,
    pub front: bool,
    pub ani: i32,
    pub no: i32,
    pub flip: bool,
    pub x: f32,
    pub y: f32,
    pub rx: i32,
    pub ry: i32,
    pub r#type: i32,
    pub cx: i32,
    pub cy: i32,
    pub a: i32,
    pub sprite: BackgroundSprite,
    pub offset_x: f32,
    pub offset_y: f32,
}

pub struct MapLife {
    pub cy: i32,
    pub f: i32,
    pub fh: i32,
    pub id: String,
    pub rx0: i32,
    pub rx1: i32,
    pub r#type: String,
    pub x: i32,
    pub y: i32,
}

impl From<Node> for MapLife {
    fn from(node: Node) -> Self {
        Self {
            cy: node.get("cy").into(),
            f: node.try_get("f").map(Into::into).unwrap_or(0),
            fh: node.get("fh").into(),
            id: node.get("id").into(),
            rx0: node.get("rx0").into(),
            rx1: node.get("rx1").into(),
            r#type: node.get("type").into(),
            x: node.get("x").into(),
            y: node.get("y").into(),
        }
    }
}

pub enum BackgroundSprite {
    Sprite(Sprite),
    SpriteAnimation(SpriteAnimation),
}

impl MapBackground {
    pub fn new(root: Node, node: Node) -> Self {
        let bs: String = node.get("bS").into();
        let ani: i32 = node.get("ani").into();
        let no: i32 = node.get("no").into();

        let path = format!(
            "Map/Back/{}.img/{}/{}",
            bs,
            match ani {
                0 => "back",
                1 => "ani",
                2 => "spine",
                _ => panic!("unknown ani: {}", ani),
            },
            no
        );

        let back_node = root.at_path(&path).unwrap();

        let x = i32::from(node.get("x")) as f32;
        let y = i32::from(node.get("y")) as f32;
        let background = Self {
            sprite: if ani == 0 {
                BackgroundSprite::Sprite(Sprite::from(back_node))
            } else {
                BackgroundSprite::SpriteAnimation(SpriteAnimation::from(back_node))
            },
            offset_x: x,
            offset_y: y,
            bs,
            front: node.get("front").into(),
            ani,
            no,
            flip: node.get("f").into(),
            x,
            y,
            cx: node.get("cx").into(),
            cy: node.get("cy").into(),
            r#type: node.get("type").into(),
            rx: node.get("rx").into(),
            ry: node.get("ry").into(),
            a: node.get("a").into(),
        };
        // 0 无平铺
        // 1 水平平铺
        // 2 垂直平铺
        // 3 双向平铺
        // 4 水平平铺+水平滚动
        // 5 垂直平铺+垂直滚动
        // 6 双向平铺+水平滚动
        // 7 双向平铺+垂直滚动
        background
    }
}
pub struct Map {
    pub npc: HashMap<String, Npc>,
    pub life: Vec<MapLife>,
    pub backgrounds: Vec<MapBackground>,
    pub layers: Vec<MapLayer>,
    pub footholds: HashMap<i32, Foothold>,
    pub portals: Vec<Portal>,
    pub helper: MapHelper,
    pub portal_timer: Timer,
    pub info: MapInfo,
}

impl Map {
    pub fn new(root: &Node, name: &str) -> Result<Self, Error> {
        let map_img = root
            .at_path(&format!("Map/Map/Map{}/{name}.img", &name[0..1]))
            .unwrap();

        let children = map_img.get("back").children();
        let backgrounds: Vec<_> = (0..children.len())
            .into_iter()
            .map(|i| MapBackground::new(root.clone(), children[i.to_string().as_str()].clone()))
            .collect();

        let mut layers = vec![];
        for i in '0'..'7' {
            let mut tiles = vec![];
            let mut objects = vec![];

            let node = map_img.get(i.to_string().as_str());

            if let Some(obj) = node.try_get("obj") {
                for (id, item) in obj.children() {
                    let id = id.to_string().parse::<i32>().unwrap();
                    let flip: bool = item.get("f").into();
                    let x: i32 = item.get("x").into();
                    let y: i32 = item.get("y").into();
                    let z: i32 = item.get("z").into();

                    let path = format!(
                        "Map/Obj/{}.img/{}/{}/{}",
                        String::from(item.get("oS")),
                        String::from(item.get("l0")),
                        String::from(item.get("l1")),
                        String::from(item.get("l2"))
                    );

                    let sprites: Vec<Sprite> = root.at_path(&path).unwrap().into();
                    objects.push(MapObject {
                        id,
                        flip,
                        position: vec2(x as f32, y as f32),
                        z: z,
                        timer: Timer::new(sprites.iter().map(|item| item.delay as f32).collect()),
                        sprites,
                    });
                }
            }

            if let Some(info) = node.try_get("info") {
                if info.has("tS") {
                    let ts: String = info.get("tS").into();
                    for (key, value) in node.get("tile").children().iter() {
                        let id = key.to_string().parse::<i32>().unwrap();
                        let x: i32 = value.get("x").into();
                        let y: i32 = value.get("y").into();
                        let no: i32 = value.get("no").into();
                        let u: String = value.get("u").into();
                        let zm: i32 = value.get("zM").into();
                        let tile_path = format!("Map/Tile/{ts}.img/{u}/{no}");

                        tiles.push(MapTile {
                            id,
                            tile: root.at_path(&tile_path).unwrap().into(),
                            position: vec2(x as f32, y as f32),
                        });
                    }
                }
            }

            tiles.sort_by_key(|item| item.tile.z);
            objects.sort_by_key(|item| item.z);

            layers.push(MapLayer { tiles, objects });
        }

        let mut footholds = HashMap::<i32, Foothold>::new();
        for (page, val) in &map_img.get("foothold").children() {
            let page = page.to_string().parse::<i32>().unwrap();
            for (z_mass, val) in &val.children() {
                let z_mass = z_mass.to_string().parse::<i32>().unwrap();
                for (key, val) in &val.children() {
                    let x1: i32 = val.get("x1").into();
                    let x2: i32 = val.get("x2").into();
                    let y1: i32 = val.get("y1").into();
                    let y2: i32 = val.get("y2").into();
                    let next: i32 = val.get("next").into();
                    let prev: i32 = val.get("prev").into();
                    let key = key.to_string().parse::<i32>().unwrap();
                    footholds.insert(
                        key,
                        Foothold {
                            start: vec2(x1 as f32, y1 as f32),
                            end: vec2(x2 as f32, y2 as f32),
                            next,
                            prev,
                            page,
                            z_mass,
                        },
                    );
                }
            }
        }

        let helper: MapHelper = root.at_path("Map/MapHelper.img")?.into();
        let life: Vec<MapLife> = map_img.get("life").into();
        let npc: HashMap<String, Npc> = life
            .iter()
            .filter(|item| item.r#type == "n")
            .map(|item| {
                (
                    item.id.to_string(),
                    root.at_path(&format!("Npc/{}.img", item.id))
                        .unwrap()
                        .into(),
                )
            })
            .collect();

        Ok(Self {
            life,
            npc,
            backgrounds,
            layers,
            footholds,
            portals: map_img.get("portal").into(),
            info: map_img.get("info").into(),
            portal_timer: Timer::new((1..helper.pv.len()).into_iter().map(|_| 100.0).collect()),
            helper,
        })
    }
}



fn draw_back(world: &Rc<RefCell<Context>>, item: &mut map::MapBackground) {
    let delta = world.borrow().delta;
    let camera_position = world.borrow().camera.position.clone();
    let size = world.borrow().size;
    let sprite_renderer = &world.borrow().sprite_renderer;
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
                        vec2(x + j as f32 * cw, y + i as f32 * ch)
                            - camera_position,
                        item.flip,
                    );
                }
            }
        }
        _ => {
            sprite_renderer.draw_flip(
                sprite,
                vec2(x, y) - camera_position,
                item.flip,
            );
        }
    }

    // unsafe {
    //     SDL_RenderRect(
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
pub fn background_update_system(world: &mut hecs::World) {

}