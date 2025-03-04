use crate::{sprite::SpriteAnimation, wz::Node};
use std::collections::HashMap;

pub struct NPCInfo {
    pub speak: Option<HashMap<String, String>>,
}

impl TryFrom<Node> for NPCInfo {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            speak: node.try_get("speak").map(TryInto::try_into).transpose()?,
        })
    }
}

pub struct Npc {
    pub info: NPCInfo,
    pub actions: HashMap<String, SpriteAnimation>,
}

impl TryFrom<Node> for Npc {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let info: NPCInfo = node.get("info").try_into()?;
        let actions: HashMap<String, SpriteAnimation> = node
            .children()
            .into_iter()
            .filter(|(k, _)| k.as_str() != "info")
            .filter_map(|(k, v)| Some((k.to_string(), v.try_into().ok()?)))
            .collect();
        Ok(Self { info, actions })
    }
}
