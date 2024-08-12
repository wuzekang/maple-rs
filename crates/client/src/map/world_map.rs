use glam::Vec2;
use indexmap::IndexMap;

use crate::{sprite::Sprite, wz::Node};

pub struct Item {
    pub r#type: i32,
    pub map_no: IndexMap<String, i32>,
    pub spot: Vec2,
    pub title: Option<String>,
    pub desc: Option<String>,
    pub path: Option<Sprite>,
}

impl From<Node> for Item {
    fn from(node: Node) -> Self {
        Self {
            r#type: node.get("type").into(),
            map_no: node.get("mapNo").into(),
            spot: node.get("spot").into(),
            title: node.try_get("title").map(Into::into),
            desc: node.try_get("desc").map(Into::into),
            path: node.try_get("path").map(Into::into),
        }
    }
}

pub struct Link {
    pub tool_tip: Option<String>,
    pub link_map: String,
    pub link_img: Sprite,
}

impl From<Node> for Link {
    fn from(node: Node) -> Self {
        Self {
            tool_tip: node.try_get("toolTip").map(Into::into),
            link_map: node.at_path("link/linkMap").unwrap().into(),
            link_img: node.at_path("link/linkImg").unwrap().into(),
        }
    }
}

pub struct WorldMap {
    pub base_img: Sprite,
    pub map_list: IndexMap<String, Item>,
    pub map_link: IndexMap<String, Link>,
}

impl From<Node> for WorldMap {
    fn from(node: Node) -> Self {
        Self {
            base_img: node.at_path("BaseImg/0").unwrap().into(),
            map_list: node.get("MapList").into(),
            map_link: node.get("MapLink").into(),
        }
    }
}
