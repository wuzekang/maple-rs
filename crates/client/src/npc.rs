use std::collections::HashMap;

use crate::{sprite::SpriteAnimation, wz::Node};

pub struct NPCInfo {
    pub speak: HashMap<String, String>,
}

impl From<Node> for NPCInfo {
    fn from(node: Node) -> Self {
        Self {
            speak: node.get("speak").into(),
        }
    }
}

pub struct Npc {
    pub info: NPCInfo,
    pub actions: HashMap<String, SpriteAnimation>,
}

impl From<Node> for Npc {
    fn from(node: Node) -> Self {
        let info: NPCInfo = node.get("info").into();
        let actions: HashMap<String, SpriteAnimation> = node
            .children()
            .into_iter()
            .filter(|(k, _)| k.as_str() != "info")
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect();
        Self { info, actions }
    }
}
