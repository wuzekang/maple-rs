use crate::character::Character;
use crate::character::ZMap;
use crate::cursor::{cursor, CursorState};
use crate::map;
use crate::map::{Ladder, PortalType};
use crate::sound::play_sound;
use crate::sprite::SpriteRenderer;
use glam::{vec2, Vec2};
use sdl3_sys::everything::*;
use std::sync::Arc;
use ::ui::element::Node;
use ::ui::event::{use_key, Event};
use ::ui::geometry::Rect;
use ::ui::reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use ::ui::style::Styleable;
use ::ui::{dynamic, fragment, input, view, Element, IntoElement, Renderer, ViewId};
use ::ui::{geometry, Drawable};

#[derive(Default)]
pub struct Cooldown {
    pub value: bool,
    delay: f32,
}

impl Cooldown {
    pub fn new() -> Self {
        Self {
            value: false,
            delay: 0.0,
        }
    }

    pub fn set(&mut self, delay: f32) {
        self.delay = delay;
        self.value = true;
    }

    pub fn update(&mut self, delta: f32) {
        if !self.value {
            return;
        }
        if delta > self.delay {
            self.delay = 0.0;
            self.value = false;
        } else {
            self.delay -= delta;
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
enum State {
    WALK,
    STAND,
    #[default]
    FALL,
    ALERT,
    PRONE,
    SWIM,
    CLIMB,
    DIED,
    SIT,
}

#[derive(Default, Clone)]
pub struct Camera {
    pub position: Vec2,
}

#[derive(Default)]
pub struct Player {
    avatar: Character,
    direction: Vec2,
    speed: Vec2,
    position: Vec2,
    flip: bool,
    state: State,
    foothold: i32,
    layer: i32,
    ladder: Option<Ladder>,
    climb_cooldown: Cooldown,
}

impl Player {
    pub fn walk_force(&self) -> f32 {
        14000.0
    }
    pub fn walk_drag(&self) -> f32 {
        8000.0
    }
    pub fn walk_speed(&self) -> f32 {
        125.0
    }

    pub fn climb_speed(&self) -> f32 {
        100.0
    }

    pub fn gravity_acc(&self) -> f32 {
        2000.0
    }

    pub fn fall_speed(&self) -> f32 {
        670.0
    }

    pub fn jump_speed(&self) -> f32 {
        555.0
    }

    pub fn update(
        &mut self,
        map: &map::Map,
        current_map: Option<RwSignal<(String, Option<String>)>>,
        delta: f32,
    ) {
        self.step(map, current_map, delta);
        if !(self.speed == Vec2::ZERO && matches!(self.state, State::CLIMB)) {
            self.avatar.tick(delta);
        }
        self.climb_cooldown.update(delta);
    }

    pub fn jump(&mut self) {
        play_sound("Sound/Game.img/Jump");
        self.avatar.set_action("jump");
        self.state = State::FALL;
    }

    pub fn climb(&mut self, ladder: &Ladder) {
        self.speed = Vec2::ZERO;
        self.position.x = ladder.x;
        self.position.y = self.position.y.clamp(ladder.y1, ladder.y2);
        self.avatar
            .set_action(if ladder.is_ladder { "ladder" } else { "rope" });
        self.ladder = Some(ladder.clone());
        self.state = State::CLIMB;
    }

    pub fn step(
        &mut self,
        map: &map::Map,
        current_map: Option<RwSignal<(String, Option<String>)>>,
        delta: f32,
    ) -> f32 {
        let delta = delta / 1000.0;

        if matches!(self.state, State::CLIMB) {
            self.flip = false;
        } else if self.direction.x > 0.0 {
            self.flip = true;
        } else if self.direction.x < 0.0 {
            self.flip = false;
        }

        // jump left right
        if input::key_pressed(SDL_SCANCODE_LALT)
            && self.direction.y <= 0.0
            && self.direction.x != 0.0
            && matches!(
                self.state,
                State::WALK | State::STAND | State::PRONE | State::CLIMB
            )
        {
            if self.ladder.is_some() {
                self.climb_cooldown.set(200.0);
                self.ladder = None;
            }
            self.speed.x = self.walk_force() * 8.0 / 1000.0 * self.direction.x;
            self.speed.y = -self.jump_speed();
            self.jump();
            return 0.0;
        }

        // jump up
        if input::key_pressed(SDL_SCANCODE_LALT)
            && self.direction.y <= 0.0
            && self.direction.x == 0.0
            && matches!(self.state, State::WALK | State::STAND | State::PRONE)
        {
            self.speed.x = 0.0;
            self.speed.y = -self.jump_speed();
            self.jump();
            return 0.0;
        }

        // jump down
        if matches!(self.state, State::WALK | State::STAND | State::PRONE)
            && input::key_pressed(SDL_SCANCODE_LALT)
            && self.direction.y > 0.0
        {
            let foothold = map.footholds.values().find(|item| {
                !item.is_wall()
                    && item.id != self.foothold
                    && self.position.x >= item.start.x
                    && self.position.x <= item.end.x
                    && item.ground(self.position.x) > self.position.y
            });
            if let Some(_foothold) = foothold {
                self.position.y += 1.0;
                self.jump();
                return 0.0;
            }
        }

        // climb
        if matches!(self.state, State::WALK | State::STAND)
            && !self.climb_cooldown.value
            && self.direction.y != 0.0
        {
            let ladder = map.ladders.iter().find(|item| {
                let hor = self.position.x >= item.x - 10.0 && self.position.x < item.x + 10.0;
                let y = self.position.y + self.direction.y * 5.0;
                let ver = y >= item.y1 && y < item.y2;
                hor && ver
            });
            if let Some(ladder) = ladder {
                self.climb(ladder);
                return 0.0;
            }
        }

        if matches!(self.state, State::WALK | State::STAND | State::FALL)
            && input::key_pressed(SDL_SCANCODE_LCTRL)
        {
            self.avatar.set_action("swingO1");
        }

        if matches!(self.state, State::WALK | State::STAND | State::CLIMB)
            && self.direction.y < 0.0
            && current_map.is_some()
        {
            for portal in &map.portals {
                if !matches!(portal.pt, PortalType::REGULAR | PortalType::INVISIBLE) {
                    continue;
                }
                let lt = portal.position + vec2(-25.0, -100.0);
                let rb = portal.position + vec2(25.0, 25.0);
                let rect = Rect::from((lt, rb - lt));
                if rect.contains(self.position) {
                    current_map
                        .unwrap()
                        .set((format!("{:0>9}", portal.tm), Some(portal.tn.clone())));
                }
            }
        }

        match self.state {
            State::WALK => {
                if self.direction.y > 0.0 {
                    self.avatar.set_action("prone");
                    self.state = State::PRONE;
                    return 0.0;
                }

                if self.direction.x == 0.0 {
                    self.avatar.set_action("stand1");
                    self.state = State::STAND;
                    return 0.0;
                }

                let speed = self.speed.x
                    + self.direction.x * (self.walk_force() - self.walk_drag()) * delta;
                self.speed.x = speed.clamp(-self.walk_speed(), self.walk_speed());

                let x = self.position.x + self.speed.x * delta;

                let foothold = &map.footholds[&self.foothold];
                self.position.x = x.clamp(foothold.start.x, foothold.end.x);
                self.position.x = self.position.x.clamp(map.wall.left, map.wall.right);

                if !foothold.is_wall() {
                    self.position.y = foothold.ground(self.position.x);
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
                        self.avatar.set_action("jump");
                        self.state = State::FALL;
                    } else {
                        let foothold = &map.footholds[&next];
                        if foothold.is_blocking(self.position.y) {
                            return 0.0;
                        } else if foothold.is_wall() {
                            self.avatar.set_action("jump");
                            self.state = State::FALL;
                        } else {
                            self.foothold = foothold.id;
                        }
                    }
                }
            }
            State::STAND => {
                if self.direction.y > 0.0 {
                    self.avatar.set_action("prone");
                    self.state = State::PRONE;
                } else if self.direction.x != 0.0 {
                    self.avatar.set_action("walk1");
                    self.state = State::WALK;
                } else if self.speed.x != 0.0 {
                    let speed = self.speed.x - self.speed.x.signum() * self.walk_drag() * delta;
                    let speed = if self.speed.x > 0.0 {
                        speed.max(0.0)
                    } else {
                        speed.min(0.0)
                    };
                    self.speed.x = speed;
                    self.position += self.speed * delta;
                    self.position.x = self.position.x.clamp(map.wall.left, map.wall.right);
                }
            }
            State::FALL => {
                let prev = self.position;
                self.position += self.speed * delta;
                self.position.x = self.position.x.clamp(map.wall.left, map.wall.right);
                self.speed.y += self.gravity_acc() * delta;
                self.speed.y = self.speed.y.min(self.fall_speed());

                for foothold in map.footholds.values() {
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
                            self.speed.x = 0.0;
                            return 0.0;
                        }
                    }
                }

                if self.direction.y < 0.0 && !self.climb_cooldown.value {
                    for ladder in map.ladders.iter() {
                        if let Some(p) = geometry::intersect(
                            &vec2(ladder.x - 10.0, ladder.y1 - 5.0),
                            &vec2(ladder.x - 10.0, ladder.y2 + 5.0),
                            &prev,
                            &self.position,
                        )
                        .or_else(|| {
                            geometry::intersect(
                                &vec2(ladder.x + 10.0, ladder.y1 - 5.0),
                                &vec2(ladder.x + 10.0, ladder.y2 + 5.0),
                                &prev,
                                &self.position,
                            )
                        }) {
                            self.position.x = ladder.x;
                            self.position.y = p.y;
                            self.climb(ladder);
                            return 0.0;
                        }
                    }
                }

                if self.speed.y > 0.0 {
                    for foothold in map.footholds.values() {
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
                            self.speed = Vec2::ZERO;
                            self.avatar.set_action("stand1");
                            self.foothold = foothold.id;
                            self.layer = foothold.layer;
                            self.state = State::STAND;
                            break;
                        }
                    }
                }
            }
            State::ALERT => {}
            State::PRONE => {
                if self.direction.y <= 0.0 {
                    self.avatar.set_action("stand1");
                    self.state = State::STAND;
                }
            }
            State::SWIM => {}
            State::CLIMB => {
                let ladder = self.ladder.unwrap();
                self.speed.y = self.direction.y * 100.0;
                self.position += self.speed * delta;

                if self.position.y < ladder.y1 {
                    if ladder.uf {
                        self.position.y = ladder.y1 - 5.0;
                        self.avatar.set_action("jump");
                        self.state = State::FALL;
                    } else {
                        self.position.y = self.position.y.max(ladder.y1);
                    }
                }
                if self.position.y > ladder.y2 {
                    self.position.y = ladder.y2;
                    self.avatar.set_action("jump");
                    self.state = State::FALL;
                }
            }
            State::DIED => {}
            State::SIT => {}
        }

        0.0
    }
}

pub struct MainScene {
    id: ViewId,
    size: Vec2,
    camera: Camera,
    pub camera_signal: RwSignal<Camera>,
    cursor_state: RwSignal<CursorState>,
    current_map: Option<RwSignal<(String, Option<String>)>>,
    player: Option<Player>,
    map: map::Map,
}

impl MainScene {
    pub async fn resource(map_name: &str, spawn: Option<String>, reader: std::sync::Arc<wz_splitter::reader::SplitWzReader>) -> Result<(Player, map::Map), Box<dyn std::error::Error>> {
        use crate::wz::WzSplitReaderExt;
        
        dbg!(map_name);
        let map = map::Map::new(reader.clone(), map_name.to_string()).await.map_err(|_| "Failed to create map")?;
        let spawn = spawn.unwrap_or("sp".to_string());
        let position = map.portals.iter().fold(None, |acc: Option<Vec2>, item| {
            if item.pn != spawn {
                return acc;
            }
            if let Some(prev) = acc {
                if item.position.length() > prev.length() {
                    Some(item.position + vec2(0.0, -10.0))
                } else {
                    acc
                }
            } else {
                Some(item.position + vec2(0.0, -10.0))
            }
        });
        
        let z_map_node = reader.get_node("zmap.img").await.map_err(|_| "Failed to load zmap")?;
        let z_map: Arc<ZMap> = Arc::new(z_map_node.try_into().map_err(|_| "Failed to parse zmap")?);

        // 并行加载所有角色部件
        let character_paths = [
            "00002000",
            "00012000", 
            "Hair/00030020",
            "Coat/01040002",
            "Pants/01060002",
            "Shoes/01072005",
            "Face/00020000",
            "Weapon/01302000",
        ];
        
        let character_futures = character_paths.iter().map(|path| {
            let reader = reader.clone();
            let path = path.to_string();
            async move {
                reader.get_node(&format!("Character/{path}.img")).await
                    .map_err(|_| format!("Failed to load character part: {}", path))
            }
        });
        
        let character_nodes = futures::future::try_join_all(character_futures).await?;

        let player = Player {
            avatar: Character::new(character_nodes, z_map),
            position: position.unwrap_or_default(),
            direction: Vec2::ZERO,
            speed: Vec2::ZERO,
            ..Default::default()
        };

        Ok((player, map))
    }

    pub fn new(map: map::Map, current_map: Option<RwSignal<(String, Option<String>)>>) -> Self {
        let text_visible = create_rw_signal(false);
        use_key(SDLK_T, move || {
            text_visible.set(!text_visible.get());
        });
        let camera_signal = create_rw_signal(Camera::default());
        let cursor_state = create_rw_signal(CursorState::Idle);
        let bgm = map.info.bgm.split("/").collect::<Vec<_>>();
        play_sound(&format!("Sound/{}.img/{}", bgm[0], bgm[1]));

        let size = vec2(800.0, 600.0);

        let t = map.info.vr_top.unwrap() as f32;
        let b = map.info.vr_bottom.unwrap() as f32;
        let l = map.info.vr_left.unwrap() as f32;
        let r = map.info.vr_right.unwrap() as f32;
        let vr_size = vec2(r - l, b - t);

        let mut texts = vec![];
        for layer in &map.layers {
            for item in &layer.objects {
                for sprite in &item.sprites {
                    let path = sprite.path.clone();
                    let position = item.position;
                    texts.push(
                        view()
                            // .composite()
                            .style(move |s| {
                                s.absolute()
                                    .left(position.x - l)
                                    .top(position.y - t)
                                    .font_size(12.0)
                                    .line_height(14.0)
                                    .bg_white()
                            })
                            .children(ui::text(move || path.clone())),
                    );
                }
            }
        }

        let texts = fragment(texts).into_element();

        let camera = Camera {
            ..Default::default()
        };

        let id = view()
            .style(move |s| s.absolute().left(0).top(0).width(size.x).height(size.y))
            .style(move |s| s.cursor(cursor(cursor_state.get())))
            .children(
                view()
                    .style(move |s| {
                        let camera = camera_signal.get();
                        s.absolute()
                            .left(0)
                            .top(0)
                            .width(vr_size.x)
                            .height(vr_size.y)
                            .translate_x(-camera.position.x + l)
                            .translate_y(-camera.position.y + t)
                    })
                    .children((
                        view()
                            .style(|s| s.absolute().left(0).top(0).w_full().h_full())
                            .children(()),
                        view()
                            .composite()
                            .style(|s| s.absolute().left(0).top(0).w_full().h_full())
                            .children(dynamic({
                                move || {
                                    if text_visible.get() {
                                        texts.clone()
                                    } else {
                                        Node::Fragment(vec![])
                                    }
                                }
                            })),
                    )),
            )
            .id();

        Self {
            id,
            size,
            camera,
            player: None,
            cursor_state,
            map,
            camera_signal,
            current_map,
        }
    }

    pub fn set_camera_position(&mut self, position: Vec2) {
        self.camera.position = position;
        self.camera_signal.set(self.camera.clone());
    }

    pub fn set_player(&mut self, player: Player) {
        self.player = Some(player);
    }
}

impl Element for MainScene {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "MainScene".to_string()
    }

    fn update(&mut self, delta: f32) {
        player_move(self, delta);

        {
            let Self {
                camera, map, size, ..
            } = self;

            let camera_position = camera.position;

            for item in &mut map.backgrounds {
                update_back(delta, camera_position, *size, item);
            }
        }

        let Self { map, .. } = self;

        for layer in &mut map.layers {
            for item in &mut layer.objects {
                item.update(delta);
            }
        }

        map.portal_timer.tick(delta);
        for mob in map.mobs.values_mut() {
            if let Some(action) = mob.actions.get_mut("move") {
                action.update(delta);
            }
        }

        let mut clickable = false;
        for item in &map.life {
            if item.r#type == "n" {
                let npc = map.npc.get_mut(&item.id).unwrap();
                if npc.actions.is_empty() {
                    continue;
                }
                let action = npc.actions.get_mut("stand").unwrap();
                action.timer.tick(delta);
                let frame = &action.frames[action.timer.index];
                if Rect::from((
                    vec2(item.x as f32, item.cy as f32) - self.camera.position - frame.origin,
                    frame.size,
                ))
                .contains(input::mouse_position())
                {
                    clickable = true;
                }
            }
        }

        if clickable && matches!(self.cursor_state.get_untracked(), CursorState::Idle) {
            self.cursor_state.set(CursorState::LClick);
        }
        if !clickable && matches!(self.cursor_state.get_untracked(), CursorState::LClick) {
            self.cursor_state.set(CursorState::Idle);
        }
    }

    fn event(&mut self, event: &mut Event) {
        let Self { player, .. } = self;

        if let Some(player) = player.as_mut() {
            let pressed_left = input::key_pressed(SDL_Scancode::LEFT);
            let pressed_right = input::key_pressed(SDL_Scancode::RIGHT);
            let pressed_up = input::key_pressed(SDL_Scancode::UP);
            let pressed_down = input::key_pressed(SDL_Scancode::DOWN);

            if let Some(event) = event.is_key_down() {
                match event.scancode {
                    SDL_Scancode::LEFT => {
                        player.direction.x = -1.0;
                    }
                    SDL_Scancode::RIGHT => {
                        player.direction.x = 1.0;
                    }
                    SDL_Scancode::UP => {
                        player.direction.y = -1.0;
                    }
                    SDL_Scancode::DOWN => {
                        player.direction.y = 1.0;
                    }
                    _ => {}
                }
            } else if let Some(event) = event.is_key_up() {
                match event.scancode {
                    SDL_Scancode::LEFT => {
                        player.direction.x = if pressed_right { 1.0 } else { 0.0 };
                    }
                    SDL_Scancode::RIGHT => {
                        player.direction.x = if pressed_left { -1.0 } else { 0.0 };
                    }
                    SDL_Scancode::UP => {
                        player.direction.y = if pressed_down { 1.0 } else { 0.0 };
                    }
                    SDL_Scancode::DOWN => {
                        player.direction.y = if pressed_up { -1.0 } else { 0.0 };
                    }
                    _ => {}
                }
            }
        }
    }

    fn paint(&self, renderer: &mut Renderer) {
        let Self {
            size,
            camera,
            map,
            player,
            ..
        } = self;

        let sprite_renderer = &mut SpriteRenderer::new(renderer);
        let camera_position = camera.position;

        for item in &map.backgrounds {
            if !item.front {
                draw_back(camera_position, *size, sprite_renderer, item);
            }
        }

        for layer in &map.layers {
            for item in &layer.objects {
                let sprite = &item.sprites[item.timer.index];
                sprite_renderer.draw_flip(sprite, item.position - camera_position, item.flip);
            }

            for item in &layer.tiles {
                sprite_renderer.draw(&item.tile, item.position - camera_position);
            }
        }

        let sprite = &map.helper.pv[map.portal_timer.index];

        for item in map.portals.iter() {
            if matches!(item.pt, PortalType::REGULAR | PortalType::SCRIPTED) {
                sprite_renderer.draw(sprite, item.position - camera_position);
            }
            sprite_renderer
                .renderer
                .render_debug_text(item.position - camera_position, &format!("{:?}", item.pt))
        }

        for item in &map.life {
            if item.r#type == "n" {
                let npc = map.npc.get(&item.id).unwrap();
                if npc.actions.is_empty() {
                    continue;
                }
                let action = npc.actions.get("stand").unwrap();
                let sprite = &action.frames[action.timer.index];
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(item.x as f32, item.cy as f32) - camera_position,
                    item.f == 1,
                );
            }
            if item.r#type == "m" {
                let npc = map.mobs.get(&item.id).unwrap();
                if npc.actions.is_empty() {
                    continue;
                }
                let action = npc.actions.get("move").unwrap();

                let sprite = &action.frames[action.timer.index];
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(item.x as f32, item.cy as f32) - camera_position,
                    item.f == 1,
                );
            }
        }

        if let Some(player) = player.as_ref() {
            for sprite in player.avatar.frame() {
                sprite_renderer.draw_flip(&sprite, player.position - camera_position, player.flip)
            }
        }

        // for foothold in map.footholds.values() {
        //     let start = foothold.start - camera_position;
        //     let end = foothold.end - camera_position;
        //     renderer.line(Color::new([1.0, 0.0, 0.0, 1.0]), start, end);
        //
        //     let center = (start + end) / 2.0;
        //     unsafe {
        //         SDL_RenderDebugText(
        //             renderer.renderer,
        //             center.x,
        //             center.y,
        //             format!("{}\0", foothold.id).as_ptr() as *const c_char,
        //         );
        //     }
        // }

        // let t = map.info.vr_top.unwrap() as f32 - camera_position.y;
        // let b = map.info.vr_bottom.unwrap() as f32 - camera_position.y;
        // let l = map.info.vr_left.unwrap() as f32 - camera_position.x;
        // let r = map.info.vr_right.unwrap() as f32 - camera_position.x;
        // let vr_size = vec2(r - l, b - t);
        // renderer.set_color(Color::RED);
        // renderer.lines(&[vec2(l, t), vec2(r, t), vec2(r, b), vec2(l, b), vec2(l, t)]);

        // if world_size.x > vr_size.x {
        //     let len = (world_size.x - vr_size.x) / 2.0;
        //     renderer.fill_rect(Color::BLACK, Vec2::ZERO, vec2(len, world_size.y));
        //     renderer.fill_rect(
        //         Color::BLACK,
        //         vec2(world_size.x - len, 0.0),
        //         vec2(len, world_size.y),
        //     );
        // }
        // if world_size.y > vr_size.y {
        //     let len = (world_size.y - vr_size.y) / 2.0;
        //     renderer.fill_rect(Color::BLACK, Vec2::ZERO, vec2(world_size.x, len));
        //     renderer.fill_rect(
        //         Color::BLACK,
        //         vec2(0.0, world_size.y - len),
        //         vec2(world_size.x, len),
        //     );
        // }
    }
}

fn player_move(context: &mut MainScene, delta: f32) {
    let MainScene {
        player,
        size,
        camera,
        camera_signal,
        map,
        current_map,
        ..
    } = context;
    if player.is_none() {
        return;
    }
    let player = player.as_mut().unwrap();

    let world_size = *size;

    player.update(map, *current_map, delta);

    let vr_left = map.info.vr_left.unwrap() as f32;
    let vr_right = map.info.vr_right.unwrap() as f32;
    let vr_top = map.info.vr_top.unwrap() as f32;
    let vr_bottom = map.info.vr_bottom.unwrap() as f32;
    let vr_size = vec2(vr_right - vr_left, vr_bottom - vr_top);

    let mut next = Vec2::ZERO;
    if vr_size.x < world_size.x {
        next.x = vr_left - (world_size.x - vr_size.x) / 2.0;
    } else {
        next.x = (player.position.x - world_size.x / 2.0)
            .max(vr_left)
            .min(vr_right - world_size.x);
    }
    if vr_size.y < world_size.y {
        next.y = vr_top - (world_size.y - vr_size.y) / 2.0;
    } else {
        next.y = (player.position.y - world_size.y + 240.0)
            .max(vr_top)
            .min(vr_bottom - world_size.y);
    }

    let prev = camera.position;
    let offset = next - prev;
    camera.position += vec2(
        if offset.x.abs() >= 5.0 {
            offset.x * 12.0 / 800.0
        } else {
            0.0
        },
        if offset.y.abs() >= 5.0 {
            offset.y * 12.0 / 600.0
        } else {
            0.0
        },
    );

    if camera.position != camera_signal.get().position {
        camera_signal.set(camera.clone());
    }
}

fn update_back(delta: f32, camera_position: Vec2, size: Vec2, item: &mut map::MapBackground) {
    let camera_position = camera_position;
    let size = size;
    let offset = camera_position + size / 2.0;

    match item.r#type {
        4 | 6 => {
            item.offset_x += item.rx as f32 * 5.0 * delta / 1000.0;
            item.offset_y = item.y + offset.y * (item.ry + 100) as f32 / 100.0;
        }
        5 | 7 => {
            item.offset_x = item.x + offset.x * (item.rx + 100) as f32 / 100.0;
            item.offset_y += item.ry as f32 * 5.0 * delta / 1000.0;
        }
        _ => {
            item.offset_x = item.x + offset.x * (item.rx + 100) as f32 / 100.0;
            item.offset_y = item.y + offset.y * (item.ry + 100) as f32 / 100.0;
        }
    }

    if let map::BackgroundSprite::SpriteAnimation(animation) = &mut item.sprite {
        animation.tick(delta);
    };
}

fn draw_back(
    camera_position: Vec2,
    size: Vec2,
    sprite_renderer: &mut SpriteRenderer,
    item: &map::MapBackground,
) {
    let camera_position = camera_position;

    let sprite = item.sprite.current_frame();
    let w = sprite.image.width() as f32;
    let h = sprite.image.height() as f32;
    let cw = if item.cx > 0 { item.cx as f32 } else { w };
    let ch = if item.cy > 0 { item.cy as f32 } else { h };

    let x = item.offset_x;
    let y = item.offset_y;
    let lb = x - sprite.origin.x;
    let rb = lb + w;
    let tb = y - sprite.origin.y;
    let bb = tb + h;

    let hs = f32::ceil((camera_position.x - rb) / cw) as i32;
    let he = f32::ceil((camera_position.x + size.x - rb) / cw) as i32 + 1;

    let vs = f32::ceil((camera_position.y - bb) / ch) as i32;
    let ve = f32::ceil((camera_position.y + size.y - bb) / ch) as i32 + 1;

    match item.r#type {
        1 | 4 => {
            for i in hs..he {
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(x + i as f32 * cw, y) - camera_position,
                    item.flip,
                );
            }
        }
        2 | 5 => {
            for i in vs..ve {
                sprite_renderer.draw_flip(
                    sprite,
                    vec2(x, y + i as f32 * ch) - camera_position,
                    item.flip,
                );
            }
        }
        3 | 6 | 7 => {
            for i in vs..ve {
                for j in hs..he {
                    sprite_renderer.draw_flip(
                        sprite,
                        vec2(x + j as f32 * cw, y + i as f32 * ch) - camera_position,
                        item.flip,
                    );
                }
            }
        }
        _ => {
            sprite_renderer.draw_flip(sprite, vec2(x, y) - camera_position, item.flip);
        }
    }
}
