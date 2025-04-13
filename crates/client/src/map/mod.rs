use crate::npc::Npc;
use crate::sprite::{Sprite, SpriteAnimation};
use crate::timer::{Repeat, Timer};
use crate::wz::Node;
use glam::{vec2, FloatExt, Vec2};
use std::collections::HashMap;

pub mod world_map;

pub struct Wall {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub struct MapHelper {
    pub pv: Vec<Sprite>,
}

impl TryFrom<Node> for MapHelper {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            pv: node.at_path("portal/game/pv").or(Err(()))?.try_into()?,
        })
    }
}

#[allow(dead_code)]
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

impl TryFrom<Node> for Portal {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            pn: node.get("pn").try_into()?,
            pt: node.get("pt").try_into()?,
            position: vec2(
                i32::try_from(node.get("x"))? as f32,
                i32::try_from(node.get("y"))? as f32,
            ),
        })
    }
}

#[derive(Default, Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct Foothold {
    pub id: i32,
    pub start: Vec2,
    pub end: Vec2,
    pub prev: i32,
    pub next: i32,
    pub layer: i32,
    pub z_mass: i32,
}

impl Foothold {
    pub fn is_wall(&self) -> bool {
        self.start.x == self.end.x
    }
    pub fn is_blocking(&self, y: f32) -> bool {
        if !self.is_wall() {
            return false;
        }
        let t = y - 50.0;
        let b = y - 1.0;

        let ft = self.start.y.min(self.end.y);
        let fb = self.start.y.max(self.end.y);

        t.max(ft) <= b.min(fb)
    }
    pub fn ground(&self, x: f32) -> f32 {
        if self.is_wall() {
            self.start.y
        } else {
            self.start
                .y
                .lerp(self.end.y, (x - self.start.x) / (self.end.x - self.start.x))
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Ladder {
    pub is_ladder: bool,
    // page: i32,
    pub uf: bool,
    pub x: f32,
    pub y1: f32,
    pub y2: f32,
}

impl TryFrom<Node> for Ladder {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            is_ladder: node.get("l").try_into()?,
            // page: node.get("page").try_into()?,
            uf: node.get("uf").try_into()?,
            x: node.get("x").try_into()?,
            y1: node.get("y1").try_into()?,
            y2: node.get("y2").try_into()?,
        })
    }
}

#[derive(Debug)]
pub struct MapInfo {
    pub vr_top: Option<i32>,
    pub vr_bottom: Option<i32>,
    pub vr_left: Option<i32>,
    pub vr_right: Option<i32>,
    pub bgm: String,
}

impl TryFrom<Node> for MapInfo {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            vr_top: node.try_get("VRTop").and_then(|v| v.try_into().ok()),
            vr_bottom: node.try_get("VRBottom").and_then(|v| v.try_into().ok()),
            vr_left: node.try_get("VRLeft").and_then(|v| v.try_into().ok()),
            vr_right: node.try_get("VRRight").and_then(|v| v.try_into().ok()),
            bgm: node.try_get("bgm").and_then(|v| v.try_into().ok()).unwrap(),
        })
    }
}

#[allow(dead_code)]
pub struct MapObject {
    pub id: i32,
    pub timer: Timer,
    pub flip: bool,
    pub sprites: Vec<Sprite>,
    pub position: Vec2,
    pub z: i32,
}

impl MapObject {
    pub fn update(&mut self, delta: f32) {
        self.timer.tick(delta);
        let sprite = &mut self.sprites[self.timer.index];
        let p = self.timer.progress();
        sprite.alpha = ((1.0 - p) * sprite.a0 as f32 + p * sprite.a1 as f32) as i32;
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

impl TryFrom<Node> for MapLife {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            cy: node.get("cy").try_into()?,
            f: node.try_get("f").map(TryInto::try_into).unwrap_or(Ok(0))?,
            fh: node.get("fh").try_into()?,
            id: node.get("id").try_into()?,
            rx0: node.get("rx0").try_into()?,
            rx1: node.get("rx1").try_into()?,
            r#type: node.get("type").try_into()?,
            x: node.get("x").try_into()?,
            y: node.get("y").try_into()?,
        })
    }
}

pub enum BackgroundSprite {
    Sprite(Sprite),
    SpriteAnimation(SpriteAnimation),
}

impl BackgroundSprite {
    pub fn current_frame(&self) -> &Sprite {
        match self {
            BackgroundSprite::Sprite(sprite) => sprite,
            BackgroundSprite::SpriteAnimation(animation) => animation.current_frame(),
        }
    }
}

impl MapBackground {
    pub fn new(root: Node, node: Node) -> Result<Self, ()> {
        let bs: String = node.get("bS").try_into()?;
        let ani: i32 = node.get("ani").try_into()?;
        let no: i32 = node.get("no").try_into()?;

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

        let back_node = root.at_path(&path).or(Err(()))?;

        let x = i32::try_from(node.get("x"))? as f32;
        let y = i32::try_from(node.get("y"))? as f32;
        let background = Self {
            sprite: if ani == 0 {
                BackgroundSprite::Sprite(Sprite::try_from(back_node)?)
            } else {
                BackgroundSprite::SpriteAnimation(SpriteAnimation::try_from(back_node)?)
            },
            offset_x: x,
            offset_y: y,
            bs,
            front: node.get("front").try_into()?,
            ani,
            no,
            flip: node.get("f").try_into()?,
            x,
            y,
            cx: node.get("cx").try_into()?,
            cy: node.get("cy").try_into()?,
            r#type: node.get("type").try_into()?,
            rx: node.get("rx").try_into()?,
            ry: node.get("ry").try_into()?,
            a: node.get("a").try_into()?,
        };
        // 0 无平铺
        // 1 水平平铺
        // 2 垂直平铺
        // 3 双向平铺
        // 4 水平平铺+水平滚动
        // 5 垂直平铺+垂直滚动
        // 6 双向平铺+水平滚动
        // 7 双向平铺+垂直滚动
        Ok(background)
    }
}
pub struct Map {
    pub npc: HashMap<String, Npc>,
    pub life: Vec<MapLife>,
    pub backgrounds: Vec<MapBackground>,
    pub layers: Vec<MapLayer>,
    pub footholds: HashMap<i32, Foothold>,
    pub wall: Wall,
    pub ladders: Vec<Ladder>,
    pub portals: Vec<Portal>,
    pub helper: MapHelper,
    pub portal_timer: Timer,
    pub info: MapInfo,
}

impl Map {
    pub fn new(root: Node, name: String) -> Result<Self, ()> {
        let map_img = if name == "login" {
            root.at_path("UI/MapLogin.img").unwrap()
        } else {
            root.at_path(&format!("Map/Map/Map{}/{name}.img", &name[0..1]))
                .unwrap()
        };

        let children = map_img.get("back").children();
        let backgrounds: Vec<_> = (0..children.len())
            .filter_map(|i| {
                MapBackground::new(root.clone(), children[i.to_string().as_str()].clone()).ok()
            })
            .collect();

        let mut layers = vec![];
        for i in '0'..'7' {
            let mut tiles = vec![];
            let mut objects = vec![];

            let node = map_img.get(i.to_string().as_str());

            if let Some(obj) = node.try_get("obj") {
                for (id, item) in obj.children() {
                    let id = id.to_string().parse::<i32>().unwrap();
                    let flip: bool = item.get("f").try_into()?;
                    let x: i32 = item.get("x").try_into()?;
                    let y: i32 = item.get("y").try_into()?;
                    let z: i32 = item.get("z").try_into()?;

                    let path = format!(
                        "Map/Obj/{}.img/{}/{}/{}",
                        String::try_from(item.get("oS"))?,
                        String::try_from(item.get("l0"))?,
                        String::try_from(item.get("l1"))?,
                        String::try_from(item.get("l2"))?
                    );

                    let node = root.at_path(&path).unwrap();
                    let repeat = node
                        .try_get("repeat")
                        .map(i32::try_from)
                        .transpose()?
                        .map(|i| i != -1)
                        .unwrap_or(true);

                    let sprites: Vec<Sprite> = node.try_into().unwrap();
                    let mut timer =
                        Timer::new(sprites.iter().map(|item| item.delay as f32).collect());
                    if !repeat {
                        timer.repeat = Repeat::Finite(1)
                    }
                    objects.push(MapObject {
                        id,
                        flip,
                        position: vec2(x as f32, y as f32),
                        z,
                        timer,
                        sprites,
                    });
                }
            }

            if let Some(info) = node.try_get("info") {
                if info.has("tS") {
                    let ts: String = info.get("tS").try_into()?;
                    for (key, value) in node.get("tile").children().iter() {
                        let id = key.to_string().parse::<i32>().unwrap();
                        let x: i32 = value.get("x").try_into()?;
                        let y: i32 = value.get("y").try_into()?;
                        let no: i32 = value.get("no").try_into()?;
                        let u: String = value.get("u").try_into()?;
                        // let zm: i32 = value.get("zM").try_into()?;
                        let tile_path = format!("Map/Tile/{ts}.img/{u}/{no}");

                        tiles.push(MapTile {
                            id,
                            tile: root.at_path(&tile_path).unwrap().try_into().unwrap(),
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
        for (layer, val) in &map_img.get("foothold").children() {
            let layer = layer.to_string().parse::<i32>().unwrap();
            for (z_mass, val) in &val.children() {
                let z_mass = z_mass.to_string().parse::<i32>().unwrap();
                for (key, val) in &val.children() {
                    let x1: i32 = val.get("x1").try_into()?;
                    let x2: i32 = val.get("x2").try_into()?;
                    let y1: i32 = val.get("y1").try_into()?;
                    let y2: i32 = val.get("y2").try_into()?;
                    let next: i32 = val.get("next").try_into()?;
                    let prev: i32 = val.get("prev").try_into()?;
                    let id = key.to_string().parse::<i32>().unwrap();
                    footholds.insert(
                        id,
                        Foothold {
                            id,
                            start: vec2(x1 as f32, y1 as f32),
                            end: vec2(x2 as f32, y2 as f32),
                            next,
                            prev,
                            layer,
                            z_mass,
                        },
                    );
                }
            }
        }

        let ladders: Vec<Ladder> = map_img.get("ladderRope").try_into()?;

        let mut lt = Vec2::INFINITY;
        let mut rb = Vec2::NEG_INFINITY;
        for item in footholds.values() {
            lt = lt.min(item.start).min(item.end);
            rb = rb.max(item.start).max(item.end);
        }
        lt.y -= 320.0;
        rb.y += 160.0;

        let wall = Wall {
            left: lt.x + 25.0,
            right: rb.x - 25.0,
            top: lt.y,
            bottom: rb.y,
        };

        let helper: MapHelper = root.at_path("Map/MapHelper.img").or(Err(()))?.try_into()?;
        let life: Vec<MapLife> = map_img.get("life").try_into()?;
        let npc: HashMap<String, Npc> = life
            .iter()
            .filter(|item| item.r#type == "n")
            .filter_map(|item| {
                Some((
                    item.id.to_string(),
                    root.at_path(&format!("Npc/{}.img", item.id))
                        .unwrap()
                        .try_into()
                        .ok()?,
                ))
            })
            .collect();

        let mut info: MapInfo = map_img.get("info").try_into()?;
        info.vr_left = info.vr_left.or(Some(lt.x as i32));
        info.vr_top = info.vr_top.or(Some(lt.y as i32));
        info.vr_right = info.vr_right.or(Some(rb.x as i32));
        info.vr_bottom = info.vr_bottom.or(Some(rb.y as i32));

        Ok(Self {
            life,
            npc,
            backgrounds,
            layers,
            footholds,
            wall,
            ladders,
            portals: map_img.get("portal").try_into()?,
            info,
            portal_timer: Timer::new((1..helper.pv.len()).map(|_| 100.0).collect()),
            helper,
        })
    }
}
