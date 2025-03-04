use glam::Vec2;
use indexmap::IndexMap;

use crate::{sprite::Sprite, wz::Node};

pub struct Item {
    pub r#type: i32,
    pub map_no: Option<IndexMap<String, i32>>,
    pub spot: Vec2,
    pub title: Option<String>,
    pub desc: Option<String>,
    pub path: Option<Sprite>,
}

impl TryFrom<Node> for Item {
    type Error = ();

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            r#type: node.get("type").try_into()?,
            map_no: node.try_get("mapNo").map(TryInto::try_into).transpose()?,
            spot: node.get("spot").try_into()?,
            title: node.try_get("title").map(TryInto::try_into).transpose()?,
            desc: node.try_get("desc").map(TryInto::try_into).transpose()?,
            path: node.try_get("path").map(TryInto::try_into).transpose()?,
        })
    }
}

pub struct Link {
    pub tool_tip: Option<String>,
    pub link_map: String,
    pub link_img: Sprite,
}

impl TryFrom<Node> for Link {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            tool_tip: node.try_get("toolTip").and_then(|v| v.try_into().ok()),
            link_map: node.at_path("link/linkMap").unwrap().try_into()?,
            link_img: node.at_path("link/linkImg").unwrap().try_into()?,
        })
    }
}

pub struct WorldMap {
    pub base_img: Sprite,
    pub map_list: IndexMap<String, Item>,
    pub map_link: Option<IndexMap<String, Link>>,
}

impl TryFrom<Node> for WorldMap {
    type Error = ();
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            base_img: node.at_path("BaseImg/0").unwrap().try_into()?,
            map_list: node.get("MapList").try_into()?,
            map_link: node.try_get("MapLink").and_then(|v| v.try_into().ok()),
        })
    }
}
