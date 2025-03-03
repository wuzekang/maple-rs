use crate::map::world_map::WorldMap;
use crate::scene::MainScene;
use crate::sprite::{ASpriteAnimation, Sprite, SpriteAnimation};
use crate::wz::Node;
use crate::WzBase;
use glam::{vec2, Vec2};
use image::DynamicImage;
use sdl3_sys::everything::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use ui::event::{use_event, use_key, Interactive};
use ui::peniko::Color;
use ui::reactive::{create_rw_signal, use_context, RwSignal, SignalGet, SignalUpdate, SignalWith};
use ui::style::dimension::{length, percent};
use ui::style::{Cursor, Styleable};
use ui::taffy::{AlignItems, Display, FlexDirection, JustifyContent, Position};
use ui::view_tuple::ViewTuple;
use ui::widget::debug::debug;
use ui::{
    dynamic, fragment, input, lazy, text, view, Bounds, Drawable, Element, Image, ImageTexture,
    IntoElement, NineGridTexture, Renderer, Surface, TextInput, View,
};

enum CursorState {
    Idle,
    LClick,
    Game,
    House,
    Grab,
    Gift,
    VScroll,
    HScroll,
    VScrollIdle,
    HScrollIdle,
    Grabbing,
    Clicking,
    RClick,
}

impl CursorState {
    fn index(&self) -> u8 {
        match self {
            CursorState::Idle => 0,
            CursorState::LClick => 1,
            CursorState::Game => 2,
            CursorState::House => 3,
            CursorState::Grab => 5,
            CursorState::Gift => 6,
            CursorState::VScroll => 7,
            CursorState::HScroll => 8,
            CursorState::VScrollIdle => 9,
            CursorState::HScrollIdle => 10,
            CursorState::Grabbing => 11,
            CursorState::Clicking => 12,
            CursorState::RClick => 13,
        }
    }

    fn clicking_state(&self) -> CursorState {
        match self {
            CursorState::Idle => CursorState::Clicking,
            CursorState::LClick => CursorState::Clicking,
            CursorState::Game => CursorState::Clicking,
            CursorState::House => CursorState::Clicking,
            CursorState::Grab => CursorState::Grabbing,
            CursorState::Gift => CursorState::Clicking,
            CursorState::VScroll => CursorState::Clicking,
            CursorState::HScroll => CursorState::Clicking,
            CursorState::VScrollIdle => CursorState::Clicking,
            CursorState::HScrollIdle => CursorState::Clicking,
            CursorState::Grabbing => CursorState::Grabbing,
            CursorState::Clicking => CursorState::Clicking,
            CursorState::RClick => CursorState::Clicking,
        }
    }
}

impl Into<Cursor> for CursorState {
    fn into(self) -> Cursor {
        cursor(self)
    }
}

pub fn cursor_image(state: CursorState) -> SpriteAnimation {
    let WzBase { node: base } = use_context().unwrap();
    let basic = base.at_path("UI/Basic.img").unwrap();
    let cursor: SpriteAnimation = basic
        .at_path(&format!("Cursor/{}", state.index()))
        .unwrap()
        .into();
    cursor
}

pub fn cursor(state: CursorState) -> Cursor {
    Cursor::from_drawable({
        let cursor = DefaultCursor::new(state);
        move || Box::new(cursor.clone())
    })
}

#[derive(Clone)]
struct DefaultCursor {
    idle_image: SpriteAnimation,
    clicking_image: SpriteAnimation,
    clicking: bool,
}

impl DefaultCursor {
    pub fn new(state: CursorState) -> Self {
        Self {
            clicking: false,
            clicking_image: cursor_image(state.clicking_state()),
            idle_image: cursor_image(state),
        }
    }
}

impl Drawable for DefaultCursor {
    fn draw(&self, painter: &Renderer) {
        if self.clicking {
            self.clicking_image.draw(painter);
        } else {
            self.idle_image.draw(painter);
        }
    }

    fn size(&self) -> Vec2 {
        if self.clicking {
            self.clicking_image.size()
        } else {
            self.idle_image.size()
        }
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.idle_image.set_bounds(bounds.clone());
        self.clicking_image.set_bounds(bounds);
    }

    fn update(&mut self, delta: f32) {
        self.clicking = input::mouse_button_pressed(SDL_BUTTON_LMASK);
        if self.clicking {
            self.clicking_image.update(delta);
        } else {
            self.idle_image.update(delta);
        }
    }
}

pub fn ui_view() -> impl IntoElement {
    let logo_visible = RwSignal::new(true);
    let open = RwSignal::new(true);
    let current_map = RwSignal::new("910000000".to_string());

    fragment((
        view(dynamic(move || {
            if logo_visible.get() {
                fragment(logo(logo_visible))
            } else {
                fragment((
                    map_scene(current_map),
                    status_bar(),
                    world_map_window(open, current_map),
                ))
            }
        }))
        .style(move |s| {
            s.cursor(CursorState::Idle)
                .position(Position::Absolute)
                .width(percent(1.0))
                .height(percent(1.0))
        }),
        debug(),
    ))
}

struct SequenceAnimation {
    index: usize,
    animations: Vec<ASpriteAnimation>,
    complete: bool,
    bounds: Bounds,
    listeners: Vec<Box<dyn Fn()>>,
}

impl SequenceAnimation {
    fn new(animations: Vec<ASpriteAnimation>) -> Self {
        Self {
            index: 0,
            animations,
            complete: false,
            bounds: Bounds::default(),
            listeners: Vec::new(),
        }
    }

    fn on_complete(&mut self, f: impl Fn() + 'static) {
        self.listeners.push(Box::new(f));
    }
}

impl Drawable for SequenceAnimation {
    fn draw(&self, painter: &Renderer) {
        let animation = &self.animations[self.index];
        animation.draw(painter);
    }

    fn size(&self) -> Vec2 {
        self.animations[self.index].size()
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, delta: f32) {
        if self.complete {
            return;
        }
        if self.animations.is_empty() {
            return;
        }
        let animation = &mut self.animations[self.index];
        animation.update(delta);
        if animation.complete {
            self.index += 1;
        }
        if self.index >= self.animations.len() {
            self.complete = true;
            self.index = self.animations.len() - 1;
            for f in self.listeners.iter() {
                f();
            }
        }
        self.animations[self.index].set_bounds(self.bounds.clone());
    }
}

pub fn logo(visible: RwSignal<bool>) -> impl IntoElement {
    lazy(
        move || {
            let WzBase { node: base } = use_context().unwrap();
            async move {
                let nexon: ASpriteAnimation = base.at_path("UI/Logo.img/Nexon").unwrap().into();
                let wizet: ASpriteAnimation = base.at_path("UI/Logo.img/Wizet").unwrap().into();
                Some((nexon, wizet))
            }
        },
        move |resource| {
            let content = if let Some((nexon, wizet)) = resource {
                let mut animation = SequenceAnimation::new(vec![nexon, wizet]);
                animation.on_complete(move || {
                    visible.set(false);
                });
                fragment(Image::new(animation).on_click(|_| {}))
            } else {
                fragment(view(()).style(|s| s.background(Color::RED).width(20).height(20)))
            };

            view(content)
                .style(|s| {
                    s.absolute()
                        .size_full()
                        .justify_center()
                        .items_center()
                        .bg_white()
                })
                .on_click(move |_| visible.set(false))
        },
    )
}

pub fn button(btn_node: Node) -> Image {
    let btn_state = create_rw_signal("normal".to_string());
    Image::dynamic(move || {
        let path = format!("{}/0", btn_state.get());
        let image: Arc<DynamicImage> = btn_node.at_path(&path).unwrap().into();
        ImageTexture::new(use_context().unwrap(), &image)
    })
    .on_mouse_enter(move |_| {
        btn_state.set("mouseOver".to_string());
    })
    .on_mouse_leave(move |_| {
        btn_state.set("normal".to_string());
    })
    .style(|s| s.cursor(CursorState::LClick))
}

pub fn map_scene(map_name: RwSignal<String>) -> impl IntoElement {
    lazy(
        {
            move || {
                let map_name = map_name.get();
                async move { Some(MainScene::resource(&map_name)) }
            }
        },
        move |map| match map {
            None => fragment(()),
            Some((player, map)) => {
                let main_scene = Rc::new(RefCell::new(MainScene::new(player, map)));
                use_event({
                    let main_scene = main_scene.clone();
                    move |event| {
                        main_scene.borrow_mut().event(event);
                    }
                });
                fragment(main_scene)
            }
        },
    )
}

pub fn level_no<F>(value: F) -> View
where
    F: Fn() -> i32 + 'static,
{
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Basic.img/LevelNo").unwrap();
    let images: Vec<Arc<DynamicImage>> = (0..9).map(|i| node.get(&i.to_string()).into()).collect();
    view((Image::new(images[1].clone()), Image::new(images[8].clone()))).style(|s| {
        s.justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .gap_row(1.0)
            .gap_column(1.0)
    })
}

pub fn bracket_wrap(children: impl ViewTuple) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/StatusBar.img/number").unwrap();
    let left: Arc<DynamicImage> = node.get("Lbracket").into();
    let right: Arc<DynamicImage> = node.get("Rbracket").into();
    view(fragment((
        Image::new(left),
        fragment(children),
        Image::new(right),
    )))
    .style(|s| {
        s.justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::Center)
            .gap_row(1.0)
            .gap_column(1.0)
    })
}

pub fn status_bar_number(f: impl (Fn() -> String) + 'static) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/StatusBar.img/number").unwrap();
    let images: Vec<Arc<DynamicImage>> = (0..10).map(|i| node.get(&i.to_string()).into()).collect();
    let slash: Arc<DynamicImage> = node.get("slash").into();
    let percent: Arc<DynamicImage> = node.get("percent").into();

    view(dynamic(move || {
        f().chars()
            .filter_map(|ch| {
                if ch >= '0' && ch <= '9' {
                    Some(images[ch as usize - '0' as usize].clone())
                } else if ch == '/' {
                    Some(slash.clone())
                } else if ch == '%' {
                    Some(percent.clone())
                } else {
                    None
                }
            })
            .map(|item| Image::new(item))
            .collect::<Vec<_>>()
    }))
    .style(|s| {
        s.justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
    })
}

pub fn status_bar() -> View {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/StatusBar.img").unwrap();
    let background: Arc<DynamicImage> = img.at_path("base").unwrap().get("backgrnd").into();
    let background2: Arc<DynamicImage> = img.at_path("base").unwrap().get("backgrnd2").into();

    let gauge = img.at_path("gauge").unwrap();
    let graduation: Arc<DynamicImage> = gauge.get("graduation").into();
    let bar: Arc<DynamicImage> = gauge.get("bar").into();

    let base_box: Arc<DynamicImage> = img.at_path("base/box").unwrap().into();

    let icon_memo: Arc<DynamicImage> = img.at_path("base/iconMemo").unwrap().into();
    let icon_blue: Arc<DynamicImage> = img.at_path("base/iconBlue").unwrap().into();

    view((
        Image::new(background),
        Image::new(background2).style(|s| {
            s.position(Position::Absolute)
                .left(length(4.0))
                .bottom(length(0.0))
        }),
        chat_box(),
        view((
            view((
                Image::new(base_box),
                view((
                    view((Image::new(icon_blue))).style(|s| {
                        s.width(length(20.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center)
                    }),
                    view((Image::new(icon_memo))).style(|s| {
                        s.width(length(20.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center)
                    }),
                ))
                    .style(|s| {
                        s.position(Position::Absolute)
                            .width(percent(1.0))
                            .height(percent(1.0))
                            .justify_content(JustifyContent::SpaceBetween)
                            .align_items(AlignItems::Stretch)
                    }),
            ))
                .style(|s| {
                    s.margin_right(length(3.0))
                        .justify_content(JustifyContent::FlexStart)
                        .align_items(AlignItems::FlexStart)
                }),
            view((
                button(img.get("EquipKey")),
                button(img.get("InvenKey")),
                button(img.get("StatKey")),
                button(img.get("SkillKey")),
                button(img.get("KeySet")),
                button(img.get("QuickSlot")),
            ))
                .style(|s| {
                    s.justify_content(JustifyContent::FlexStart)
                        .align_items(AlignItems::FlexStart)
                        .gap_column(2.0)
                }),
        ))
            .style(|s| {
                s.position(Position::Absolute)
                    .justify_content(JustifyContent::FlexEnd)
                    .align_items(AlignItems::FlexStart)
                    .top(length(8.0))
                    .right(length(4.0))
            }),
        view(
            //
            (view((
                // level card
                view(
                    // level no
                    view(level_no(|| 18)).style(|s| {
                        s.position(Position::Absolute)
                            .left(length(27.0))
                            .bottom(length(8.0))
                            .width(length(47.0))
                            .height(length(11.0))
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center)
                    }),
                )
                    .style(|s| {
                        s.margin_left(length(3.0))
                            .width(length(74.0))
                            .height(length(30.0))
                    }),
                // job name
                view((
                    view((
                        (text(|| "魔法师")
                            .style(|s| s.color(Color::WHITE).font_size(12.0).line_height(15.0))),
                        bracket_wrap(
                            text(|| "魔法师")
                                .style(|s| s.color(Color::WHITE).font_size(12.0).line_height(15.0)),
                        ),
                    ))
                        .style(|s| {
                            s.justify_content(JustifyContent::FlexStart)
                                .align_items(AlignItems::FlexStart)
                                .gap_column(length(2.0))
                        }),
                    text(|| "三个榔头")
                        .style(|s| s.color(Color::WHITE).font_size(12.0).line_height(15.0)),
                ))
                    .style(|s| {
                        s.margin_left(length(8.0))
                            .flex_grow(1.0)
                            .height(length(30.0))
                            .flex_direction(FlexDirection::Column)
                    }),
            ))
                 .style(|s| {
                     s.width(length(208.0))
                         .justify_content(JustifyContent::FlexStart)
                         .align_items(AlignItems::Center)
                 }),),
        )
            .style(|s| {
                s.position(Position::Absolute)
                    .left(length(2.0))
                    .right(length(4.0))
                    .bottom(length(1.0))
                    .height(length(34.0))
                    .justify_content(JustifyContent::FlexStart)
                    .align_items(AlignItems::Center)
            }),
        view((
            Image::new(bar),
            Image::new(graduation).style(|s| s.position(Position::Absolute).bottom(length(0.0))),
            view(bracket_wrap(status_bar_number(|| "124/274".to_string()))).style(|s| {
                s.position(Position::Absolute)
                    .display(Display::Block)
                    .top(length(3.0))
                    .left(length(19.0))
            }),
            view(bracket_wrap(status_bar_number(|| "118/617".to_string()))).style(|s| {
                s.position(Position::Absolute)
                    .display(Display::Block)
                    .top(length(3.0))
                    .left(length(131.0))
            }),
            view(bracket_wrap(status_bar_number(|| "6839/13716".to_string()))).style(|s| {
                s.position(Position::Absolute)
                    .display(Display::Block)
                    .top(length(3.0))
                    .left(length(248.0))
            }),
        ))
            .style(|s| {
                s.position(Position::Absolute)
                    .display(Display::Block)
                    .left(length(218.0))
                    .bottom(length(1.0))
            }),
        view((
            button(img.get("BtShop")),
            button(img.get("BtNPT")),
            button(img.get("BtMenu")),
            button(img.get("BtShort")),
        ))
            .style(|s| {
                s.position(Position::Absolute)
                    .justify_content(JustifyContent::SpaceBetween)
                    .right(length(4.0))
                    .bottom(length(1.0))
                    .width(length(224.0))
                    .height(length(34.0))
            }),
    ))
    .style(|s| {
        s.position(Position::Absolute)
            .display(Display::Block)
            .left(length(0.0))
            .right(length(0.0))
            .width(percent(1.0))
            .bottom(length(0.0))
    })
}

pub fn chat_box() -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let basic = base.at_path("UI/Basic.img").unwrap();
    let open = create_rw_signal(false);

    dynamic(move || {
        if !open {
            use_key(13, move || {
                open.set(true);
            });

            fragment(
                view((
                    (text(|| "欢迎来到冒险岛，现在开始你的旅程吧～！".to_string())
                        .style(|s| s.font_size(11.0).line_height(11.0).color(Color::WHITE))),
                    view((
                        view(button(basic.get("BtMax")).on_click(move |_| open.set(true)))
                            .style(|s| s.margin_right(length(3.0))),
                        scroll_vertical(),
                    ))
                    .style(|s| s.align_items(AlignItems::Center)),
                ))
                .style(|s| {
                    s.position(Position::Absolute)
                        .justify_content(JustifyContent::SpaceBetween)
                        .align_items(AlignItems::Center)
                        .top(length(6.0))
                        .left(length(0.0))
                        .padding_left(length(8.0))
                        .width(length(568.0))
                        .height(length(24.0))
                        .background([0x88, 0x88, 0x88].into())
                }),
            )
        } else {
            let mut input_ref = None;

            fragment(
                view((
                    TextInput::new()
                        ._ref(&mut input_ref)
                        .style(|s| {
                            s.position(Position::Absolute)
                                .left(length(4.0))
                                .width(length(563.0))
                                .height(length(22.0))
                        })
                        .on_attach(move || {
                            input_ref.as_ref().unwrap().with(|input| {
                                input.focus();
                            });
                        })
                        .on_key_down(move |event| {
                            if event.key == 13 {
                                open.set(false);
                            }
                            event.propagation = false;
                        })
                        .on_key_up(move |event| event.propagation = false),
                    view(button(basic.get("BtMin")).on_click(move |_| open.set(false))).style(
                        |s| {
                            s.position(Position::Absolute)
                                .align_items(AlignItems::Center)
                                .height(percent(1.0))
                                .right(length(18.0))
                        },
                    ),
                ))
                .style(|s| {
                    s.position(Position::Absolute)
                        .align_items(AlignItems::Center)
                        .top(length(6.0))
                        .left(length(0.0))
                        .width(length(568.0))
                        .height(length(24.0))
                }),
            )
        }
    })
}

pub fn scroll_vertical() -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Basic.img/VScr5/enabled").unwrap();
    let prev: Arc<DynamicImage> = node.get("prev0").into();
    let next: Arc<DynamicImage> = node.get("next0").into();
    view((
        Image::new(prev).style(|s| s.position(Position::Absolute).top(length(0.0))),
        Image::new(next).style(|s| s.position(Position::Absolute).bottom(length(0.0))),
    ))
    .style(|s| s.width(length(15.0)).height(length(25.0)))
}
pub fn world_map_window(open: RwSignal<bool>, current_map: RwSignal<String>) -> impl IntoElement {
    use_key(SDLK_W, move || open.set(!open.get()));

    dynamic(move || {
        if !open.get() {
            return fragment(());
        }

        let WzBase { node: base } = use_context().unwrap();
        let renderer: *mut SDL_Renderer = use_context().unwrap();
        let border: Vec<Arc<DynamicImage>> = base
            .at_path("UI/UIWindow.img/WorldMap/Border")
            .unwrap()
            .into();

        let surfaces = border
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<Surface>>();

        let bg = NineGridTexture::new(
            (
                &surfaces[0],
                &surfaces[1],
                &surfaces[2],
                &surfaces[3],
                &surfaces[4],
                &surfaces[5],
                &surfaces[6],
                &surfaces[7],
            ),
            renderer,
        );

        let padding = bg.border();

        let map_image: Vec<Sprite> = base
            .at_path("Map/MapHelper.img/worldMap/mapImage")
            .unwrap()
            .into();

        let map_image = create_rw_signal(Rc::new(map_image));

        let world_map_signal = create_rw_signal(WorldMap::from(
            base.at_path("Map/WorldMap/WorldMap.img").unwrap(),
        ));

        let hovered_link = create_rw_signal(None);

        let content_size = vec2(640.0, 470.0);

        let world_map_node = base.at_path("UI/UIWindow.img/WorldMap").unwrap();
        let title: Sprite = world_map_node.get("title").into();

        let spots = dynamic(move || {
            fragment(world_map_signal.with(|world_map| {
                world_map
                    .map_list
                    .iter()
                    .map(|(_, item)| {
                        let spot_type = item.r#type as usize;
                        let origin = map_image.get_untracked()[spot_type].origin;
                        let position = vec2(item.spot.x, item.spot.y) - origin + content_size / 2.0;
                        let map_no = item.map_no.clone();
                        let clickable = map_no.and_then(|map_no| {
                            if map_no.len() == 1 {
                                Some(map_no["0"])
                            } else {
                                None
                            }
                        });
                        Image::new(map_image.get_untracked()[spot_type].image.clone())
                            .style(move |s| {
                                s.position(Position::Absolute)
                                    .margin_left(length(position.x))
                                    .margin_top(length(position.y))
                                    .cursor(if clickable.is_some() {
                                        CursorState::LClick
                                    } else {
                                        CursorState::Idle
                                    })
                            })
                            .on_click(move |_| {
                                if let Some(map_no) = clickable {
                                    current_map.set(format!("{:0>9}", map_no))
                                }
                            })
                    })
                    .collect::<Vec<_>>()
            }))
        });

        let active_link_view = dynamic(move || {
            if let Some(key) = hovered_link.get() {
                let (image, position, link_map) = (world_map_signal.with(move |world_map| {
                    let map_link = world_map.map_link.as_ref().unwrap();
                    let link = &map_link[&key];
                    let link_img = &link.link_img;
                    let link_map = link.link_map.clone();
                    let image = link_img.image.clone();
                    let text = map_link[&key].tool_tip.clone();
                    let position = content_size / 2.0 - link_img.origin;
                    (image, position, link_map)
                }));
                fragment(
                    Image::new(image)
                        .style(move |s| {
                            s.position(Position::Absolute)
                                .left(length(position.x))
                                .top(length(position.y))
                                .cursor(CursorState::LClick)
                        })
                        .on_click(move |_| {
                            let WzBase { node: base } = use_context().unwrap();
                            hovered_link.set(None);
                            world_map_signal.set(WorldMap::from(
                                base.at_path(&format!("Map/WorldMap/{}.img", &link_map))
                                    .unwrap(),
                            ));
                        }),
                )
            } else {
                fragment(())
            }
        });

        let close_button =
            button(base.at_path("UI/Basic.img/BtClose").unwrap()).on_click(move |_| {
                open.set(false);
            });

        return fragment(
            view(view((
                Image::new(bg).style(|s| {
                    s.position(Position::Absolute)
                        .width(percent(1.0))
                        .height(percent(1.0))
                }),
                view(
                    view((
                        dynamic(move || {
                            world_map_signal.with(|world_map| {
                                Image::new(world_map.base_img.image.clone()).style(move |s| {
                                    s.width(length(content_size.x))
                                        .height(length(content_size.y))
                                })
                            })
                        }),
                        active_link_view,
                        spots,
                    ))
                    .on_mouse_move(move |event| {
                        let mouse = event.offset();
                        let key = world_map_signal.with_untracked(move |world_map| {
                            if let Some(map_link) = &world_map.map_link {
                                for (key, item) in map_link {
                                    let lt = content_size / 2.0 - item.link_img.origin;
                                    let rb = lt + item.link_img.size;
                                    if (mouse.cmpge(lt).all()) && mouse.cmplt(rb).all() {
                                        let pt = mouse - lt;

                                        let pixel = item
                                            .link_img
                                            .image
                                            .as_rgba8()
                                            .unwrap()
                                            .get_pixel(pt.x as u32, pt.y as u32);

                                        if pixel.0[3] > 0 {
                                            return Some(key.clone());
                                        }
                                    }
                                }
                            }
                            return None;
                        });

                        if hovered_link.get_untracked() != key {
                            hovered_link.set(key);
                        }
                    })
                    .on_mouse_enter(|_| {})
                    .on_mouse_leave(move |_| {
                        if hovered_link.get_untracked().is_some() {
                            hovered_link.set(None);
                        }
                    }),
                )
                .style({
                    let padding = padding.clone();
                    move |s| s.position(Position::Relative).padding(padding)
                }),
                view((Image::new(title.image), close_button)).style(move |s| {
                    s.position(Position::Absolute)
                        .width(percent(1.0))
                        .margin_top(length(5.0))
                        .padding_right(padding.right)
                        .padding_left(padding.left)
                        .height(length(14.0))
                        .flex_direction(FlexDirection::Row)
                        .justify_content(JustifyContent::SpaceBetween)
                        .align_items(AlignItems::Center)
                }),
            )))
            .style(|s| {
                s.position(Position::Absolute)
                    .width(percent(1.0))
                    .height(percent(1.0))
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
            }),
        );
    })
}
