use crate::sprite::SpriteAnimation;
use crate::wz::Node;
use crate::map::{Foothold, Wall};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use glam::Vec2;
use ui::geometry;
use rand::Rng;

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
    pub no_flip: i32,
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
            no_flip: value
                .try_get("noFlip")
                .map(TryInto::try_into)
                .transpose()?
                .unwrap_or_default(),
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

#[derive(Debug, Copy, Clone, Default)]
pub enum State {
    STAND,
    #[default]
    MOVE,
    FALL,
    JUMP,
}

pub struct Mob {
    info: MobInfo,
    pub(crate) actions: HashMap<String, SpriteAnimation>,
    pub position: Vec2,
    pub direction: Vec2,
    pub velocity: Vec2,
    pub state: State,
    pub foothold: i32,
    pub layer: i32,
    pub min_x: f32,
    pub max_x: f32,
    pub flip: bool,
    pub change_probability: f32,
    pub last_change: Instant,
    pub change_delay: Duration,
    pub can_jump: bool,
    pub jump_probability: f32,
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
            position: Vec2::ZERO,
            direction: Vec2::ZERO,
            velocity: Vec2::ZERO,
            state: State::default(),
            foothold: 0,
            layer: 0,
            min_x: 0.0,
            max_x: 0.0,
            flip: false,
            change_probability: 0.02,
            last_change: Instant::now() - Duration::from_millis(300),
            change_delay: Duration::from_millis(300),
            can_jump: false,
            jump_probability: 0.05,
        })
    }
}

impl Mob {
    pub fn walk_speed(&self) -> f32 {
        if self.info.speed == self.default_speed() as i32 {
            self.default_speed()
        } else {
            (self.info.speed as f32) + self.speed_factor()
        }
    }

    pub fn default_speed(&self) -> f32 {
        125.0
    }
    
    pub fn speed_factor(&self) -> f32 {
        90.0
    }

    pub fn walk_force(&self) -> f32 {
        14000.0
    }
    
    pub fn walk_drag(&self) -> f32 {
        8000.0
    }
        
    pub fn fall_speed(&self) -> f32 {
        670.0
    }

    pub fn gravity_acc(&self) -> f32 {
        2000.0
    }

    pub fn jump_speed(&self) -> f32 {
        555.0
    }
    
    pub fn set_random_direction(&mut self) {
        let mut rng = rand::rng();
        let rand = rng.random::<f32>();

        if matches!(self.state, State::FALL | State::JUMP) {
            return;
        }

        if rand < self.jump_probability && self.can_jump {
            self.state = State::JUMP;
        } else if rand < 0.2 {
            self.direction.x = 0.0;
            self.state = State::STAND;
        } else if rand < 0.6 {
            self.direction.x = -1.0;
            self.state = State::MOVE;
        } else {
            self.direction.x = 1.0;
            self.state = State::MOVE;
        }
    }

    pub fn change_direction(&mut self) {
        let now = Instant::now();
        
        if now.duration_since(self.last_change) > self.change_delay {
            self.set_random_direction();
            self.last_change = now;
        }
    }

    pub fn step(&mut self, delta: f32, footholds: &HashMap<i32, Foothold>, wall: &Wall) {
        let delta = delta / 1000.0;
        let mut rng = rand::rng();
        
        if rng.random::<f32>() < self.change_probability {
            self.change_direction();
        }

        if self.info.no_flip != 0 {
            self.flip = false;
        } else if self.direction.x != 0.0 {
            self.flip = self.direction.x > 0.0;
        }
        
        match self.state {
            State::STAND => {
                if self.direction.x != 0.0 {
                    self.state = State::MOVE;
                } else if self.velocity.x != 0.0 {
                    let speed = self.velocity.x - self.velocity.x.signum() * self.walk_drag() * delta;
                    let speed = if self.velocity.x > 0.0 {
                        speed.max(0.0)
                    } else {
                        speed.min(0.0)
                    };
                    self.velocity.x = speed;
                    self.position += self.velocity * delta;
                    self.position.x = self.position.x.clamp(wall.left, wall.right);
                }
            }
            State::MOVE => {
                let speed = self.velocity.x
                + self.direction.x * (self.walk_force() - self.walk_drag()) * delta;
                self.velocity.x = speed.clamp(-self.walk_speed(), self.walk_speed());

                let x = self.position.x + self.velocity.x * delta;
                
                let foothold = &footholds[&self.foothold];
                self.position.x = x.clamp(foothold.start.x, foothold.end.x);

                if !foothold.is_wall() {
                    self.position.y = foothold.ground(self.position.x);
                }
                
                if x < self.min_x {
                    self.position.x = self.min_x;
                    self.direction.x = 1.0;
                } else if x > self.max_x {
                    self.position.x = self.max_x;
                    self.direction.x = -1.0;
                }

                let mut next = None;
                if self.direction.x < 0.0 && x < foothold.start.x {
                    next = Some(foothold.prev);
                }
                if self.direction.x > 0.0 && x > foothold.end.x {
                    next = Some(foothold.next);
                }

                if let Some(next) = next {
                    if next == 0 {
                        self.position.x = x;
                        self.state = State::FALL;
                    } else {
                        let foothold = &footholds[&next];
                        if foothold.is_wall() && foothold.is_blocking(self.position.y) {
                            self.direction.x = -self.direction.x;
                        } else if foothold.is_wall() && !foothold.is_blocking(self.position.y) {
                            self.state = State::FALL;
                            self.foothold = foothold.id;
                        } else if !foothold.is_wall() {
                            if foothold.ground(self.position.x) >= self.position.y {
                                self.foothold = foothold.id;
                            } else {
                                self.state = State::FALL;
                                self.foothold = foothold.id;
                            }
                        }
                    }
                }
            }
            State::FALL => {
                let prev = self.position;
                self.position += self.velocity * delta;
                self.position.x = self.position.x.clamp(wall.left, wall.right);
                self.velocity.y += self.gravity_acc() * delta;
                self.velocity.y = self.velocity.y.min(self.fall_speed());

                for foothold in footholds.values() {
                    if foothold.layer == self.layer
                        && (foothold.is_blocking(prev.y) || foothold.is_blocking(self.position.y))
                    {
                        if let Some(p) = geometry::intersect(
                            &foothold.start,
                            &foothold.end,
                            &prev,
                            &self.position,
                        ) {
                            self.position.x = p.x - self.direction.x * 1.0;
                            self.position.y = p.y;
                            self.velocity.x = 0.0;
                       }
                    }
                }

                if self.velocity.y > 0.0 {
                    for foothold in footholds.values() {
                        if foothold.is_wall() {
                            continue;
                        }
                        if let Some(p) = geometry::intersect(
                            &foothold.start,
                            &foothold.end,
                            &prev,
                            &self.position,
                        ) {
                            self.position = p;
                            self.velocity = Vec2::ZERO;
                            self.foothold = foothold.id;
                            self.layer = foothold.layer;
                            self.state = State::STAND;
                            break;
                        }
                    }
                }
            }
            State::JUMP => {
                let jump_x = if self.direction.x != 0.0 {
                    self.walk_force() * 8.0 / 1000.0 * self.direction.x
                } else {
                    0.0
                };
            
                let x = self.position.x + jump_x;            
                if x < self.min_x || x > self.max_x {
                    self.state = State::MOVE;
                    return;
                }
            
                self.velocity.x = jump_x;
                self.velocity.y = -self.jump_speed();
                self.state = State::FALL;
            }
        }
    }

    pub fn spawn_mob(mut self, x: f32, y: f32, foothold: i32, layer: i32, min_x: f32, max_x: f32) -> Self {
        self.position = Vec2::new(x, y);
        self.foothold = foothold;
        self.layer = layer;
        self.min_x = min_x;
        self.max_x = max_x;
        self.can_jump = self.actions.contains_key("jump");
        self.change_direction();
        self
    }
    
    pub fn get_current_animation(&self) -> &str {
        match self.state {
            State::STAND => "stand",
            State::MOVE => "move",
            State::FALL => "move",
            State::JUMP => "jump",
        }
    }

    pub fn update(&mut self, delta: f32, footholds: &HashMap<i32, Foothold>, wall: &Wall) {
        self.step(delta, footholds, wall);

        let animation = self.get_current_animation().to_string();
        if let Some(animation) = self.actions.get_mut(&animation) {
            animation.tick(delta);
        }
    }
}