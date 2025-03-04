use crate::app::button;
use crate::character::{Character, ZMap};
use crate::scene::MainScene;
use crate::sprite::{Sprite, SpriteAnimation, SpriteDrawable};
use crate::timer::Repeat;
use crate::WzBase;
use crate::{map, wz};
use glam::{vec2, Vec2, Vec4};
use image::DynamicImage;
use sdl3_sys::everything::SDL_FlipMode;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use ui::animation::use_raf;
use ui::event::MouseEvent;
use ui::peniko::Color;
use ui::reactive::{
    create_effect, create_ref, create_rw_signal, provide_context, use_context, RwSignal, SignalGet,
    SignalRead, SignalUpdate, SignalWith,
};
use ui::style::{StyleBuilder, Styleable, TextWrap};
use ui::widget::image::IntoDrawable;
use ui::{dynamic, fragment, view, Drawable, Element, IntoElement, Renderer, TextInput};
use ui::{text, View};
use ui::{use_resource, Interactive};
use ui::{Bounds, Image};

impl IntoDrawable for wz::Node {
    fn into_drawable(self) -> Box<dyn Drawable> {
        let sprite: Sprite = self.try_into().unwrap();
        Box::new(SpriteDrawable {
            sprite,
            bounds: Bounds::default(),
        })
    }
}

struct ClipImage {
    sprite: Sprite,
    clip: Vec2,
}

impl ClipImage {
    pub fn new(node: wz::Node) -> Self {
        let sprite: Sprite = node.try_into().unwrap();

        Self {
            clip: sprite.size,
            sprite,
        }
    }
}

impl Drawable for ClipImage {
    fn draw(&self, ctx: &mut Renderer) {
        let sprite = &self.sprite;
        let texture = ctx.texture(&sprite.image);
        let clip = self.clip.min(texture.size).max(Vec2::ZERO);
        ctx.render_texture_rotated(
            &texture,
            Vec4::from((Vec2::ZERO, clip)),
            Vec4::from((-sprite.origin, clip)),
            0.0,
            None,
            SDL_FlipMode::NONE,
        );
    }

    fn size(&self) -> Vec2 {
        self.sprite.size
    }
}

pub fn notice_loading(
    on_cancel: impl Fn() + 'static,
    on_connected: impl Fn() + 'static,
) -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();
    use_resource(|| sleep(Duration::from_secs(2)), move |_| on_connected());
    view(
        (view((
            Image::new(img.at_path("Notice/Loading/backgrnd").unwrap()),
            button(img.at_path("Notice/Loading/BtCancel").unwrap())
                .style(|s| s.absolute().right(28).top(44))
                .on_click(move |_| on_cancel()),
            Image::new(
                SpriteAnimation::try_from(img.at_path("Notice/Loading/bar").unwrap()).unwrap(),
            )
            .style(|s| s.absolute().bottom(38).right(42)),
        ))),
    )
    .style(|s| {
        s.absolute()
            .left(0)
            .top(0)
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
    })
}

pub fn channel_option(
    i: i32,
    selected: impl Fn() -> bool + 'static,
    on_select: impl Fn() + 'static,
) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();

    let progress = Rc::new(RefCell::new(ClipImage::new(
        img.at_path("WorldSelect/channel/chgauge").unwrap(),
    )));

    progress.borrow_mut().clip.x = i as f32 * 2.0;

    view((
        Image::new(
            img.at_path(&format!("WorldSelect/channel/{i}/normal"))
                .unwrap(),
        ),
        Image::new(progress as Rc<RefCell<dyn Drawable>>)
            .style(|s| s.pointer_events_none().absolute().left(43).top(20)),
        dynamic({
            let img = img.clone();
            move || {
                if selected() {
                    return fragment(
                        Image::new(
                            SpriteAnimation::try_from(
                                img.at_path("WorldSelect/channel/chSelect").unwrap(),
                            )
                            .unwrap()
                            .with_repeat(Repeat::Finite(1)),
                        )
                        .style(|s| s.pointer_events_none().absolute().left(20).top(7)),
                    );
                }
                return fragment(());
            }
        }),
    ))
    .on_click(move |_| on_select())
}

pub fn world_select_view(on_enter: impl Fn() + 'static) -> impl IntoElement {
    let on_enter = create_ref(on_enter);

    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();

    let selected_world = create_rw_signal(None);
    let scroll_state = create_rw_signal(None);
    let loading = create_rw_signal(false);

    let worlds = (0..20)
        .map({
            let img = img.clone();
            move |i| {
                button(img.at_path(&format!("WorldSelect/BtWorld/{}", i)).unwrap()).on_click(
                    move |_| {
                        if selected_world.get() != Some(i) {
                            selected_world.set(Some(i));
                            scroll_state.set(Some(0));
                        }
                    },
                )
            }
        })
        .collect::<Vec<_>>();

    view((
        view((
            dynamic({
                let img = img.clone();
                move || {
                    if let Some(i) = scroll_state.get() {
                        fragment(Image::new(
                            SpriteAnimation::try_from(
                                img.at_path(&format!("WorldSelect/scroll/{i}")).unwrap(),
                            )
                            .unwrap()
                            .with_repeat(Repeat::Finite(1)),
                        ))
                    } else {
                        fragment(())
                    }
                }
            }),
            dynamic({
                let img = img.clone();
                move || {
                    if let Some(world) = selected_world.get() {
                        let title: Arc<DynamicImage> = img
                            .at_path(&format!("WorldSelect/world/{world}"))
                            .unwrap()
                            .try_into()
                            .unwrap();

                        let delay = 350.0;
                        let duration = 250.0;
                        let mut elapsed = Rc::new(RefCell::new(0.0));
                        let alpha = create_rw_signal(0.0);
                        use_raf(move |delta| {
                            *elapsed.borrow_mut() += delta;
                            let value =
                                (*elapsed.borrow() - delay).max(0.0).min(duration) / duration;
                            if value != alpha.get_untracked() {
                                alpha.set(value);
                            }
                        });

                        let selected_channel = create_rw_signal(None);

                        fragment((view((
                            Image::new(title).style(|s| s.absolute().left(33).top(8)),
                            view(
                                (0..20)
                                    .map(move |i| {
                                        channel_option(
                                            i,
                                            move || {
                                                selected_channel
                                                    .get()
                                                    .map(|value| value == i)
                                                    .unwrap_or(false)
                                            },
                                            move || {
                                                let current = selected_channel.get_untracked();
                                                selected_channel.set(Some(i));
                                                if current == Some(i) {
                                                    loading.set(true)
                                                }
                                            },
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .style(|s| {
                                s.absolute()
                                    .left(35)
                                    .top(65)
                                    .width(374)
                                    .height(158)
                                    .flex_wrap()
                                    .gap_row(2)
                                    .gap_column(2)
                            }),
                            button(img.at_path("WorldSelect/BtGoworld").unwrap())
                                .style(|s| s.absolute().right(22).bottom(2))
                                .on_click(move |_| {
                                    loading.set(true);
                                }),
                        ))
                        .style(|s| s.absolute().left(38).top(133).width(449).height(268))
                        .style(move |s| s.opacity(alpha.get())),))
                    } else {
                        fragment(())
                    }
                }
            }),
        ))
        .style(|s| s.absolute().left(158).top(79)),
        view(worlds).style(|s| s.absolute().left(151).top(104).gap_row(1)),
        dynamic(move || {
            if loading.get() {
                fragment(notice_loading(
                    move || {
                        loading.set(false);
                    },
                    move || {
                        loading.set(false);
                        on_enter.with(|f| f());
                    },
                ))
            } else {
                fragment(())
            }
        }),
    ))
    .style(|s| s.w_full().h_full())
    .on_click(move |event| {
        if event.current.map(|v| v == event.target).unwrap_or_default() {
            if selected_world.get_untracked().is_some() {
                selected_world.set(None);
                scroll_state.set(Some(1));
            }
        }
    })
}

pub fn world_select_sidebar(
    step: RwSignal<usize>,
    on_back: impl (Fn(&MouseEvent) -> ()) + 'static,
) -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();
    let step_1: Arc<::image::DynamicImage> =
        img.at_path("Common/step/1").unwrap().try_into().unwrap();
    fragment(
        view((
            Image::new(step_1).style(|s| s.absolute().left(0).top(33)),
            button(img.at_path("WorldSelect/BtViewChoice").unwrap())
                .style(|s| s.absolute().left(0).top(125)),
            button(img.at_path("ViewAllChar/BtVAC").unwrap())
                .style(|s| s.absolute().left(0).top(375)),
            button(img.at_path("Common/BtStart").unwrap())
                .style(|s| s.absolute().left(0).top(425))
                .on_click(on_back),
        ))
        .style(|s| s.absolute().left(0).top(0).width(0).height(0)),
    )
}

pub fn title_view(on_login: impl Fn() + 'static) -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();
    let title = img.get("Title");
    let checked = create_rw_signal(false);
    let check_image: Vec<Arc<::image::DynamicImage>> =
        title.at_path("check").unwrap().try_into().unwrap();

    view((
        view((
            TextInput::new().style(|s| s.height(23)),
            TextInput::new().style(|s| s.height(23)),
        ))
        .style(|s| {
            s.absolute()
                .left(45)
                .top(12)
                .width(147)
                .flex_col()
                .items_stretch()
                .gap_column(6)
                .color(Color::WHITE)
        }),
        button(title.get("BtLogin"))
            .style(|s| s.absolute().top(0).right(0))
            .on_click(move |e| {
                on_login();
            }),
        view((
            dynamic(move || {
                let image = if checked.get() {
                    Image::new(check_image[1].clone())
                } else {
                    Image::new(check_image[0].clone())
                };
                image.style(|s| s.margin_top(1))
            }),
            button(title.get("BtLoginIDSave"))
                .style(|s| s.absolute().top(1).left(19))
                .on_click(move |_| {
                    checked.update(|value| *value = !*value);
                }),
            button(title.get("BtLoginIDLost")).style(|s| s.absolute().top(0).left(126)),
            button(title.get("BtPasswdLost")).style(|s| s.absolute().top(1).left(208)),
        ))
        .style(|s| s.absolute().left(4).top(78).width(274).height(24)),
        view((
            button(title.get("BtNew")).style(|s| s.margin_top(1)),
            button(title.get("BtHomePage")),
            button(title.get("BtQuit")),
        ))
        .style(|s| {
            s.absolute()
                .width(284)
                .height(40)
                .left(0)
                .top(125)
                .gap_column(8)
        }),
    ))
    .style(move |s| s.absolute().left(396).top(223).width(288).height(165))
}

pub fn select_character_view() -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Login.img/CharSelect").unwrap();
    let ctx: LoginContext = use_context().unwrap();

    view((
        fragment(
            (0..3)
                .map(|i| {
                    view((
                        Image::new(
                            SpriteAnimation::try_from(node.at_path("character/1").unwrap())
                                .unwrap(),
                        )
                        .style(|s| s.absolute()),
                        Image::new(
                            SpriteAnimation::try_from(node.at_path("character/0").unwrap())
                                .unwrap(),
                        )
                        .style(|s| s.absolute()),
                    ))
                    .style(move |s| s.absolute().left(252 + i * 125).top(370))
                })
                .collect::<Vec<_>>(),
        ),
        view((
            button(node.get("BtSelect")),
            button(node.get("BtNew"))
                .style(|s| s.margin_top(8))
                .on_click(move |_| ctx.scroll_to(LoginStep::SelectRace)),
            button(node.get("BtDelete")).style(|s| s.margin_top(14)),
        ))
        .style(|s| s.block().absolute().left(576).top(147)),
    ))
    .style(|s| s.w_full().h_full())
}

pub fn select_race_view() -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Login.img/RaceSelect").unwrap();
    let selected = create_rw_signal("normal");
    let ctx: LoginContext = use_context().unwrap();

    let position = |race_type: &str| match race_type {
        "knight" => |s: StyleBuilder| s.absolute().left(104).top(59),
        "normal" => |s: StyleBuilder| s.absolute().left(293).top(59),
        "aran" => |s: StyleBuilder| s.absolute().left(511).top(60),
        _ => |s: StyleBuilder| s,
    };

    view((
        Image::new(SpriteAnimation::try_from(node.get("textGL")).unwrap())
            .style(|s| s.absolute().left(315 + 91).top(32 + 19)),
        button(node.at_path("knight/BtKnight").unwrap())
            .style(position("knight"))
            .on_click(move |_| selected.set("knight")),
        button(node.at_path("normal/BtNormal").unwrap())
            .style(position("normal"))
            .on_click(move |_| selected.set("normal")),
        button(node.at_path("aran/BtAran").unwrap())
            .style(position("aran"))
            .on_click(move |_| selected.set("aran")),
        dynamic({
            let node = node.clone();
            move || {
                let selected = selected.get();
                fragment((
                    Image::new(
                        SpriteAnimation::try_from(
                            node.at_path(&format!("{selected}/OnAnimation")).unwrap(),
                        )
                        .unwrap(),
                    )
                    .style(position(selected)),
                    Image::new(
                        SpriteAnimation::try_from(
                            node.at_path(&format!("{selected}/text")).unwrap(),
                        )
                        .unwrap(),
                    )
                    .style(|s| s.absolute().left(118).top(291)),
                ))
            }
        }),
        button(node.at_path("BtSelect").unwrap())
            .style(|s| s.absolute().left(530 + 36).top(402 + 14))
            .on_click(move |_| match selected.get() {
                "knight" => ctx.scroll_to(LoginStep::CreateKnight),
                "normal" => ctx.scroll_to(LoginStep::CreateAdventure),
                "aran" => ctx.scroll_to(LoginStep::CreateAran),
                _ => {}
            }),
    ))
    .style(|s| s.w_full().h_full())
}

pub fn create_normal() -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Login.img/NewChar").unwrap();
    let ctx: LoginContext = use_context().unwrap();

    let name_confirmed = create_rw_signal(false);

    let gender = create_rw_signal(0);

    let options = vec![
        Rc::new(vec![
            vec![
                (20000, "Motivated Look (Black)"),
                (20001, "Perplexed Stare (Black)"),
                (20002, "Leisure Look (Black)"),
            ],
            vec![
                (30000, "Toben Hair"),
                (30030, "Buzz Hair"),
                (30020, "Rebel Hair"),
            ],
            vec![(0, "Black"), (3, "Brown"), (7, "Blonde"), (2, "Orange")],
            vec![
                (2000, "Light"),
                (2001, "Tanned"),
                (2002, "Dark"),
                (2003, "Pale"),
            ],
            vec![
                (1040002, "White Undershirt"),
                (1040006, "Undershirt"),
                (1040010, "Grey T-Shirt"),
            ],
            vec![
                (1060006, "Brown Cotton Shorts"),
                (1060002, "Blue Jean Shorts"),
            ],
            vec![
                (1072001, "Red Rubber Boots"),
                (1072005, "Leather Sandals"),
                (1072037, "Yellow Rubber Boots"),
                (1072038, "Blue Rubber Boots"),
            ],
            vec![
                (1302000, "Sword"),
                (1322005, "Wooden Club"),
                (1312004, "Hand Axe"),
            ],
            vec![(0, "Male")],
        ]),
        Rc::new(vec![
            vec![
                (21000, "Motivated Look (Black)"),
                (21001, "Perplexed Stare (Black)"),
                (21002, "Leisure Look (Black)"),
            ],
            vec![
                (31000, "Sammy Hair"),
                (31040, "Edgy Hair"),
                (31050, "Connie Hair"),
            ],
            vec![(0, "Black"), (3, "Brown"), (7, "Blonde"), (2, "Orange")],
            vec![
                (2000, "Light"),
                (2001, "Tanned"),
                (2002, "Dark"),
                (2003, "Pale"),
            ],
            vec![
                (1041002, "White Tubetop"),
                (1041006, "Yellow T-Shirt"),
                (1041010, "Green T-Shirt"),
                (1041011, "Red-Striped Top"),
            ],
            vec![(1060006, "Red Miniskirt"), (1061008, "Indigo Miniskirt")],
            vec![
                (1072001, "Red Rubber Boots"),
                (1072005, "Leather Sandals"),
                (1072037, "Yellow Rubber Boots"),
                (1072038, "Blue Rubber Boots"),
            ],
            vec![
                (1302000, "Sword"),
                (1322005, "Wooden Club"),
                (1312004, "Hand Axe"),
            ],
            vec![(0, "Female")],
        ]),
    ];

    let value = create_rw_signal({
        options
            .iter()
            .map(|v| v.iter().map(|_| 0).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    });

    let z_map: Arc<ZMap> = Arc::new(base.at_path("zmap.img").unwrap().try_into().unwrap());

    let avatar = Rc::new(RefCell::new(Character::new(vec![], z_map)));

    create_effect({
        let options = options.clone();
        {
            let avatar = avatar.clone();
            move |_| {
                let binding = value.read();
                let value = binding.borrow();
                let gender = gender.get();
                let value = &value[gender];

                let value = value
                    .iter()
                    .enumerate()
                    .map(|(i, j)| options[gender][i][*j].0)
                    .collect::<Vec<_>>();

                let face = value[0];
                let mut parts = vec![];
                parts.push(format!("Face/000{}", value[0]));
                parts.push(format!("Hair/000{}", value[1] + value[2]));
                parts.push(format!("0000{}", value[3]));
                parts.push(format!("0001{}", value[3]));
                parts.push(format!("Coat/0{}", value[4]));
                parts.push(format!("Pants/0{}", value[5]));
                parts.push(format!("Shoes/0{}", value[6]));
                parts.push(format!("Weapon/0{}", value[7]));

                for part in parts {
                    dbg!(&part);
                    avatar
                        .borrow_mut()
                        .insert(base.at_path(&format!("Character/{part}.img")).unwrap())
                }

                avatar.borrow_mut().flip = true;
                avatar.borrow_mut().set_action("stand1");
            }
        }
    });

    // face
    // Base/Character/Face/00020000.img
    // Base/String/Eqp.img/Eqp/Face/20000/name

    // male:
    // "20000": {
    //     "name": "Motivated Look (Black)"
    // },
    // "20001": {
    //     "name": "Perplexed Stare (Black)"
    // },
    // "20002": {
    //     "name": "Leisure Look (Black)"
    // },

    // female
    // "21000": {
    //     "name": "Motivated Look (Black)"
    // },
    // "21001": {
    //     "name": "Fearful Stare (Black)"
    // },
    // "21002": {
    //     "name": "Leisure Look (Black)"
    // },

    // hair_style
    // male: Toben,Buzz,Rebel
    // female: Sammy,Edgy,Connie
    // hair_color
    // Black,Brown,Blonde,Orange

    // "30000": {
    //     "name": "Black Toben"
    // },
    // "30003": {
    //     "name": "Blonde Toben"
    // },
    // "30007": {
    //     "name": "Brown Toben"
    // },
    // "30002": {
    //     "name": "Orange Toben"
    // },

    // "30030": {
    //     "name": "Black Buzz"
    // },
    // "30033": {
    //     "name": "Blonde Buzz"
    // },
    // "30037": {
    //     "name": "Brown Buzz"
    // },
    // "30032": {
    //     "name": "Orange Buzz"
    // },

    // "30020": {
    //     "name": "Black Rebel"
    // },
    // "30023": {
    //     "name": "Blonde Rebel"
    // },
    // "30027": {
    //     "name": "Brown Rebel"
    // },
    // "30022": {
    //     "name": "Orange Rebel"
    // },

    // "31000": {
    //     "name": "Black Sammy"
    // },
    // "31003": {
    //     "name": "Blonde Sammy"
    // },
    // "31007": {
    //     "name": "Brown Sammy"
    // },
    // "31002": {
    //     "name": "Orange Sammy"
    // },

    // "31040": {
    //     "name": "Black Edgy"
    // },
    // "31043": {
    //     "name": "Blonde Edgy"
    // },
    // "31047": {
    //     "name": "Brown Edgy"
    // },
    // "31042": {
    //     "name": "Orange Edgy"
    // },

    // "31050": {
    //     "name": "Black Connie"
    // },
    // "31053": {
    //     "name": "Blonde Connie"
    // },
    // "31057": {
    //     "name": "Brown Connie"
    // },
    // "31052": {
    //     "name": "Orange Connie"
    // },

    // skin_color
    // Light,Tanned,Dark,Pale
    // Base/Character/00002000.img
    // Base/Character/00002001.img
    // Base/Character/00002002.img
    // Base/Character/00002003.img

    // top(male)
    // "1040002": {
    //     "name": "White Undershirt"
    // },
    // "1040006": {
    //     "name": "Undershirt"
    // },
    // "1040010": {
    //     "name": "Grey T-Shirt"
    // },

    // top(female)
    // "1041002": {
    //     "name": "White Tubetop"
    // },
    // "1041006": {
    //     "name": "Yellow T-Shirt"
    // },
    // "1041010": {
    //     "name": "Green T-Shirt"
    // },
    // "1041011": {
    //     "name": "Red-Striped Top"
    // },

    // bottom(male)
    // "1060006": {
    //     "name": "Brown Cotton Shorts"
    // },
    // "1060002": {
    //     "name": "Blue Jean Shorts"
    // },

    // bottom(female)
    // "1061002": {
    //     "name": "Red Miniskirt"
    // },
    // "1061008": {
    //     "name": "Indigo Miniskirt"
    // },

    // shoes
    // "1072001": {
    //     "name": "Red Rubber Boots"
    // },
    // "1072005": {
    //     "name": "Leather Sandals"
    // },
    // "1072037": {
    //     "name": "Yellow Rubber Boots"
    // },
    // "1072038": {
    //     "name": "Blue Rubber Boots"
    // },
    // weapon
    // "1302000": {
    //     "name": "Sword"
    // },
    // "1322005": {
    //     "name": "Wooden Club"
    // },
    // "1312004": {
    //     "name": "Hand Axe"
    // },
    // gender

    view((
        Image::new(avatar as Rc<RefCell<dyn Drawable>>).style(|s| s.absolute().left(395).top(340)),
        dynamic(move || {
            if !name_confirmed.get() {
                Image::new(node.get("charName"))
                    .style(|s| s.absolute().left(481).top(95).width(201).height(224))
                    .children((
                        TextInput::new()
                            .style(|s| s.absolute().left(29).top(104).width(147).height(23)),
                        button(node.get("BtYes"))
                            .style(|s| s.absolute().left(27).bottom(5))
                            .on_click(move |_| {
                                name_confirmed.set(true);
                            }),
                        button(node.get("BtNo"))
                            .style(|s| s.absolute().right(19).bottom(5))
                            .on_click(move |_| ctx.scroll_to(LoginStep::SelectRace)),
                    ))
            } else {
                Image::new(node.get("charSet"))
                    .style(|s| s.absolute().left(481).top(95).width(225).height(377))
                    .children((
                        view(
                            (0..9)
                                .map(|i| {
                                    let len = options[gender.get()][i].len();
                                    view((
                                        Image::new(
                                            node.at_path(&format!("avatarSel/{i}/normal")).unwrap(),
                                        ),
                                        view(
                                            text({
                                                let options = options.clone();
                                                move || {
                                                    let gender = gender.get();
                                                    options[gender][i][value.get()[gender][i]]
                                                        .1
                                                        .to_string()
                                                }
                                            })
                                            .style(|s| s.text_wrap(TextWrap::None)),
                                        )
                                        .style(|s| {
                                            s.absolute()
                                                .right(15)
                                                .bottom(0)
                                                .width(113)
                                                .height(15)
                                                .font_size(13.0)
                                                .line_height(15.0)
                                                .justify_center()
                                        }),
                                        button(node.get("BtLeft"))
                                            .style(|s| s.absolute().left(57).bottom(0))
                                            .on_click(move |_| {
                                                if i == 8 {
                                                    gender.update(|v| {
                                                        *v = if *v == 0 { 1 } else { 0 }
                                                    });
                                                    return;
                                                }
                                                let gender = gender.get_untracked();
                                                value.update(move |v| {
                                                    let v = &mut v[gender][i];
                                                    *v = if *v == 0 { len - 1 } else { *v - 1 };
                                                })
                                            }),
                                        button(node.get("BtRight"))
                                            .style(|s| s.absolute().right(0).bottom(0))
                                            .on_click(move |_| {
                                                if i == 8 {
                                                    gender.update(|v| {
                                                        *v = if *v == 0 { 1 } else { 0 }
                                                    });
                                                    return;
                                                }
                                                let gender = gender.get_untracked();
                                                value.update(move |v| {
                                                    let v = &mut v[gender][i];
                                                    *v = if *v == len - 1 { 0 } else { *v + 1 };
                                                })
                                            }),
                                    ))
                                })
                                .collect::<Vec<_>>(),
                        )
                        .style(|s| {
                            s.absolute()
                                .left(11)
                                .top(105)
                                .width(200)
                                .height(161)
                                .flex_col()
                                .gap_column(1)
                        }),
                        button(node.get("BtYes"))
                            .style(|s| s.absolute().left(37).bottom(5))
                            .on_click(move |_| {
                                ctx.scroll_to(LoginStep::SelectCharacter);
                            }),
                        button(node.get("BtNo"))
                            .style(|s| s.absolute().right(33).bottom(5))
                            .on_click(move |_| {
                                name_confirmed.set(false);
                            }),
                    ))
            }
        }),
    ))
    .style(|s| s.w_full().h_full())
}

enum LoginStep {
    Title,
    SelectWorld,
    SelectCharacter,
    SelectRace,
    CreateKnight,
    CreateAdventure,
    CreateAran,
}

impl Into<usize> for LoginStep {
    fn into(self) -> usize {
        match self {
            LoginStep::Title => 0,
            LoginStep::SelectWorld => 1,
            LoginStep::SelectCharacter => 2,
            LoginStep::SelectRace => 3,
            LoginStep::CreateKnight => 4,
            LoginStep::CreateAdventure => 5,
            LoginStep::CreateAran => 6,
        }
    }
}

#[derive(Copy, Clone)]
struct LoginContext {
    step: RwSignal<usize>,
}

impl LoginContext {
    pub fn scroll_to(&self, step: LoginStep) {
        self.step.set(step.into());
    }
}

pub fn login_scene(on_enter: impl Fn() + 'static) -> View {
    let on_enter = create_ref(on_enter);

    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();
    let frame: Arc<::image::DynamicImage> =
        img.at_path("Common/frame").unwrap().try_into().unwrap();

    let scroll_top = create_rw_signal(0.0);
    let step = create_rw_signal(LoginStep::Title.into());
    // let step = create_rw_signal(LoginStep::CreateAdventure.into());
    let ctx = LoginContext { step };

    provide_context(ctx);

    let steps = [
        // title
        fragment(title_view(move || ctx.scroll_to(LoginStep::SelectWorld))),
        // world & channel
        fragment(world_select_view(move || {
            ctx.scroll_to(LoginStep::SelectCharacter)
        })),
        // select character
        fragment(select_character_view()),
        // select race
        fragment(select_race_view()),
        // knight
        fragment(title_view(move || step.update(|step| *step += 1))),
        // adventure
        fragment(create_normal()),
        // aran
        fragment(title_view(move || step.update(|step| *step += 1))),
    ];

    let len = steps.len();
    let size = vec2(800.0, 600.0);

    create_effect(move |_| {
        scroll_top.set((len - step.get() - 1) as f32 * size.y);
    });

    let scene = Rc::new(RefCell::new(MainScene::new(
        map::Map::new(base, "login".to_string()).unwrap(),
    )));

    create_effect({
        let scene = scene.clone();
        move |_| {
            scene.borrow_mut().set_camera_position(
                vec2(28.0, -8.0) - size / 2.0
                    + vec2(0.0, scroll_top.get() - size.y * (len - 1) as f32),
            );
        }
    });

    view((
        scene,
        view(
            steps
                .into_iter()
                .rev()
                .map(move |step| view(step).style(move |s| s.width(size.x).height(size.y)))
                .collect::<Vec<_>>(),
        )
        .style(|s| s.flex_col())
        .style(move |s| s.translate_y(-scroll_top.get()).translate_x(0.0)),
        Image::new(frame).style(move |s| {
            s.pointer_events_none()
                .absolute()
                .left(0)
                .top(0)
                .width(size.x)
                .height(size.y)
        }),
        dynamic(move || {
            if step.get() >= 1 {
                fragment(world_select_sidebar(step, move |_| {
                    step.set(0);
                }))
            } else {
                fragment(())
            }
        }),
    ))
}
