use crate::sprite::SpriteAnimation;
use crate::wz::Node;
use std::collections::HashMap;

pub struct MobInfo {
    pub ma_damage: i32,
    pub md_damage: i32,
    pub pa_damage: i32,
    pub pd_damage: i32,
    pub acc: i32,
    pub body_attack: i32,
    pub eva: i32,
    pub exp: i32,
    pub fs: f32,
    pub level: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub mob_type: i32,
    pub pushed: i32,
    pub speed: i32,
    pub summon_type: i32,
    pub undead: i32,
}

impl TryFrom<Node> for MobInfo {
    type Error = ();

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        Ok(Self {
            ma_damage: value.get("MADamage").try_into()?,
            md_damage: value.get("MDDamage").try_into()?,
            pa_damage: value.get("PADamage").try_into()?,
            pd_damage: value.get("PDDamage").try_into()?,
            acc: value.get("acc").try_into()?,
            body_attack: value.get("bodyAttack").try_into()?,
            eva: value.get("eva").try_into()?,
            exp: value.get("exp").try_into()?,
            fs: value.get("fs").try_into().unwrap_or_default(),
            level: value.get("level").try_into()?,
            max_hp: value.get("maxHP").try_into()?,
            max_mp: value.get("maxMP").try_into()?,
            mob_type: value
                .try_get("mobType")
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or_default(),
            pushed: value.get("pushed").try_into()?,
            speed: value
                .try_get("speed")
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or_default(),
            summon_type: value.get("summonType").try_into()?,
            undead: value.get("undead").try_into()?,
        })
    }
}

pub struct Mob {
    info: MobInfo,
    pub(crate) actions: HashMap<String, SpriteAnimation>,
}

impl TryFrom<Node> for Mob {
    type Error = ();
    fn try_from(value: Node) -> Result<Self, Self::Error> {
        let mut actions = HashMap::new();
        for (key, value) in value.children() {
            if key.to_string() == "info".to_string() {
                continue;
            }
            actions.insert(key.to_string(), SpriteAnimation::try_from(value)?);
        }
        Ok(Self {
            info: value.get("info").try_into()?,
            actions,
        })
    }
}
