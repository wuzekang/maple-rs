use glam::Vec2;
use image::DynamicImage;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;
use wz_reader::WzNodeCast;

use crate::sprite::Sprite;
use crate::wz::Node;

#[derive(Debug)]
pub struct AvatarFramePart {
    pub origin: Vec2,
    pub map: HashMap<String, Vec2>,
    pub image: Arc<DynamicImage>,
    pub z: String,
}

impl From<Node> for AvatarFramePart {
    fn from(node: Node) -> Self {
        let frame = AvatarFramePart {
            origin: node.get("origin").into(),
            z: node.get("z").into(),
            map: node.get("map").into(),
            image: node.into(),
        };
        frame
    }
}

#[derive(Default)]
pub struct ZMap {
    pub layers: HashMap<String, i32>,
}

impl From<Node> for ZMap {
    fn from(node: Node) -> Self {
        Self {
            layers: node
                .children()
                .keys()
                .rev()
                .enumerate()
                .map(|(index, item)| (item.to_string(), index as i32))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AvatarFrame {
    pub parts: HashMap<String, AvatarFramePart>,
    pub delay: Option<i32>,
}

impl From<Node> for AvatarFrame {
    fn from(node: Node) -> Self {
        AvatarFrame {
            parts: node
                .children()
                .into_iter()
                .filter_map(|(key, body_node)| {
                    if body_node.wz_node.read().unwrap().try_as_png().is_none() {
                        None
                    } else {
                        Some((key.to_string(), AvatarFramePart::from(body_node)))
                    }
                })
                .collect(),
            delay: node.try_get("delay").map(Into::into),
        }
    }
}

#[derive(Debug)]
pub struct AvatarPart {
    pub info: AvatarPartInfo,
    pub variant: HashMap<String, Vec<AvatarFrame>>,
}

impl From<Node> for AvatarPart {
    fn from(node: Node) -> Self {
        let info: AvatarPartInfo = node.get("info").into();
        Self {
            info,
            variant: node
                .children()
                .into_iter()
                .filter(|(key, _)| key.as_str() != "info")
                .map(|(key, node)| {
                    (key.to_string(), {
                        let children = node.children();
                        if children.contains_key("0") {
                            children
                                .into_iter()
                                .filter_map(|(_, node)| {
                                    if node.has("action") {
                                        None
                                    } else {
                                        Some(node.into())
                                    }
                                })
                                .collect()
                        } else {
                            vec![node.into()]
                        }
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AvatarPartInfo {
    pub slot: String,
    pub cash: bool,
}

impl From<Node> for AvatarPartInfo {
    fn from(value: Node) -> Self {
        Self {
            slot: value.get("islot").into(),
            cash: value.get("cash").into(),
        }
    }
}
#[derive(Debug, Default)]
pub struct Timer {
    elapsed: f32,
    intervals: Vec<f32>,
    total: f32,
    pub index: usize,
}

impl Timer {
    pub fn new(intervals: Vec<f32>) -> Self {
        Self {
            total: intervals.iter().sum(),
            elapsed: 0.0,
            intervals,
            index: 0,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        if self.intervals.is_empty() || self.total == 0.0 {
            return false;
        }
        let prev = self.index;
        self.elapsed += delta;
        self.elapsed %= self.total;
        while self.elapsed >= self.intervals[self.index] {
            self.elapsed -= self.intervals[self.index];
            self.index = (self.index + 1) % self.intervals.len();
        }
        self.index != prev
    }
}
#[derive(Default)]
pub struct Character {
    pub slots: HashMap<String, AvatarPart>,
    pub action: String,
    pub emotion: String,
    timer: Timer,
    z_map: Arc<ZMap>,
}

impl Character {
    pub fn new(parts: Vec<Node>, z_map: Arc<ZMap>) -> Self {
        let mut item = Self {
            slots: HashMap::new(),
            action: "stand1".to_string(),
            emotion: "default".to_string(),
            timer: Timer::new(vec![]),
            z_map,
        };
        for part in parts {
            item.insert(part);
        }
        item.timer = Timer::new(
            item.slots["Bd"].variant[&item.action]
                .iter()
                .map(|frame| frame.delay.unwrap() as f32)
                .collect(),
        );
        item
    }

    pub fn insert(&mut self, node: Node) {
        let part: AvatarPart = node.into();
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
        self.timer = Timer::new(
            self.slots["Bd"].variant[&self.action]
                .iter()
                .map(|frame| frame.delay.unwrap() as f32)
                .collect(),
        );
    }

    pub fn frame(&self) -> Vec<Sprite> {
        let action = &self.action;
        let emotion = &self.emotion;
        let index = self.timer.index;

        let body = &self.slots["Bd"].variant[action][index].parts["body"].map;
        let head = &self.slots["Hd"].variant[action][index].parts["head"].map;
        let offset = |slot: &str, part: &str, item: &HashMap<String, Vec2>| match slot {
            "Bd" => match part {
                "body" => Vec2::ZERO,
                _ => item["navel"] - body["navel"],
            },
            "Hd" => item["neck"] - body["neck"],
            "Fc" | "Hr" => item["brow"] - head["brow"] + head["neck"] - body["neck"],
            _ => item["navel"] - body["navel"],
        };

        let mut frame = Vec::<Sprite>::new();

        for (slot, item) in self.slots.iter() {
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
                    origin: item.origin + offset(&slot, &part, &item.map),
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
