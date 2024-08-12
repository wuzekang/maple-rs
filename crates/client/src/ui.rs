use crate::{sprite::SpriteAnimation, wz::Node};

pub struct Button {
    pub disabled: SpriteAnimation,
    pub mouse_over: SpriteAnimation,
    pub normal: SpriteAnimation,
    pub pressed: SpriteAnimation,
}

impl From<Node> for Button {
    fn from(node: Node) -> Self {
        Self {
            disabled: node.get("disabled").into(),
            mouse_over: node.get("mouseOver").into(),
            normal: node.get("normal").into(),
            pressed: node.get("pressed").into(),
        }
    }
}

pub struct FlexView {}
