use crate::sprite::{Sprite, SpriteRenderer};
use crate::timer::Timer;
use crate::wz::Node;
use glam::Vec2;
use image::DynamicImage;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;
use ui::{Drawable, Renderer};
use wz_reader::WzNodeCast;

#[derive(Debug)]
pub struct AvatarFramePart {
    pub origin: Vec2,
    pub map: HashMap<String, Vec2>,
    pub image: Arc<DynamicImage>,
    pub z: String,
}

impl TryFrom<Node> for AvatarFramePart {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let frame = AvatarFramePart {
            origin: node.get("origin").try_into()?,
            z: node.get("z").try_into()?,
            map: node.get("map").try_into()?,
            image: node.try_into()?,
        };
        Ok(frame)
    }
}

#[derive(Default)]
pub struct ZMap {
    pub layers: HashMap<String, i32>,
}

impl TryFrom<Node> for ZMap {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, ()> {
        Ok(Self {
            layers: node
                .children()
                .keys()
                .rev()
                .enumerate()
                .map(|(index, item)| (item.to_string(), index as i32))
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct AvatarFrame {
    pub parts: HashMap<String, AvatarFramePart>,
    pub delay: Option<i32>,
}

impl TryFrom<Node> for AvatarFrame {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, ()> {
        Ok(AvatarFrame {
            parts: node
                .children()
                .into_iter()
                .filter_map(|(key, body_node)| {
                    if body_node.wz_node.read().unwrap().try_as_png().is_none() {
                        None
                    } else {
                        Some((key.to_string(), AvatarFramePart::try_from(body_node).ok()?))
                    }
                })
                .collect(),
            delay: node.try_get("delay").and_then(|v| v.try_into().ok()),
        })
    }
}

#[derive(Debug)]
pub struct AvatarPart {
    pub info: AvatarPartInfo,
    pub variant: HashMap<String, Vec<AvatarFrame>>,
}

impl TryFrom<Node> for AvatarPart {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let info: AvatarPartInfo = node.get("info").try_into()?;
        Ok(Self {
            info,
            variant: node
                .children()
                .into_iter()
                .filter(|(key, _)| key.as_str() != "info")
                .filter_map(|(key, node)| {
                    Some((key.to_string(), {
                        let children = node.children();
                        if children.contains_key("0") {
                            children
                                .into_iter()
                                .filter_map(|(_, node)| {
                                    if node.has("action") {
                                        None
                                    } else {
                                        Some(node.try_into().ok()?)
                                    }
                                })
                                .collect()
                        } else {
                            vec![node.try_into().ok()?]
                        }
                    }))
                })
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct AvatarPartInfo {
    pub slot: String,
    // pub cash: bool,
}

impl TryFrom<Node> for AvatarPartInfo {
    type Error = ();
    fn try_from(value: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: value.get("islot").try_into()?,
            // cash: value.get("cash").into(),
        })
    }
}

#[derive(Default)]
pub struct Character {
    pub slots: HashMap<String, AvatarPart>,
    pub action: String,
    pub emotion: String,
    pub flip: bool,
    timer: Timer,
    z_map: Arc<ZMap>,
}

impl Character {
    pub fn new(parts: Vec<Node>, z_map: Arc<ZMap>) -> Self {
        let mut item = Self {
            slots: HashMap::new(),
            action: "stand1".to_string(),
            emotion: "default".to_string(),
            flip: false,
            timer: Timer::new(vec![]),
            z_map,
        };
        let len = parts.len();
        for part in parts {
            item.insert(part);
        }
        if len > 0 {
            item.timer = Timer::new(
                item.slots["Bd"].variant[&item.action]
                    .iter()
                    .map(|frame| frame.delay.unwrap() as f32)
                    .collect(),
            );
        }
        item
    }

    pub fn insert(&mut self, node: Node) {
        let part: AvatarPart = node.try_into().unwrap();
        self.slots.insert(part.info.slot.clone(), part);
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(delta)
    }

    pub fn set_action(&mut self, action: &str) {
        if self.action == action {
            return;
        }
        self.action = action.to_string();
        if self.slots["Bd"].variant[&self.action].len() > 1 {
            self.timer = Timer::new(
                self.slots["Bd"].variant[&self.action]
                    .iter()
                    .map(|frame| frame.delay.unwrap() as f32)
                    .collect(),
            );
        } else {
            self.timer = Timer::new(vec![]);
        }
    }

    pub fn frame(&self) -> Vec<Sprite> {
        let action = &self.action;
        let emotion = &self.emotion;
        let index = self.timer.index;

        let body = &self.slots["Bd"].variant[action][index].parts["body"].map;
        let arm = self.slots["Bd"].variant[action][index].parts.get("arm");
        let head = &self.slots["Hd"].variant[action][index].parts["head"].map;
        let offset = |slot: &str, part: &str, item: &HashMap<String, Vec2>| match slot {
            "Bd" => match part {
                "body" => Vec2::ZERO,
                _ => item["navel"] - body["navel"],
            },
            "Hd" => item["neck"] - body["neck"],
            "Fc" | "Hr" => item["brow"] - head["brow"] + head["neck"] - body["neck"],
            "Wp" => {
                let arm = &arm.unwrap().map;
                item["hand"] - arm["hand"] + arm["navel"] - body["navel"]
            }
            _ => item["navel"] - body["navel"],
        };

        let mut frame = Vec::<Sprite>::new();

        for (slot, item) in self.slots.iter() {
            if slot == "Fc" && matches!(action.as_str(), "ladder" | "rope") {
                continue;
            }
            let parts = if slot == "Fc" {
                item.variant[emotion][0].borrow()
            } else {
                item.variant[action][index].borrow()
            }
            .parts
            .iter();
            for (part, item) in parts {
                frame.push(Sprite {
                    a0: 0,
                    a1: 0,
                    alpha: 255,
                    path: "".to_string(),
                    image: item.image.clone(),
                    origin: item.origin + offset(slot, part, &item.map),
                    z: self.z_map.layers[&item.z],
                    delay: 0,
                    size: Vec2::new(item.image.width() as f32, item.image.height() as f32),
                })
            }
        }

        frame.sort_by_key(|item| item.z);
        frame
    }
}

impl Drawable for Character {
    fn draw(&self, ctx: &mut Renderer) {
        let mut sprite_renderer = SpriteRenderer::new(ctx);
        for sprite in &self.frame() {
            sprite_renderer.draw_flip(sprite, Vec2::ZERO, self.flip);
        }
    }

    fn size(&self) -> Vec2 {
        Vec2::ZERO
    }

    fn update(&mut self, delta: f32) -> bool {
        let index = self.timer.index;
        self.timer.tick(delta);
        index != self.timer.index
    }
}
