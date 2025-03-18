use crate::cursor::CursorState;
use crate::login::login_scene;
use crate::map::world_map::WorldMap;
use crate::scene::MainScene;
use crate::sprite::{ASpriteAnimation, Sprite, SpriteAnimation};
use crate::timer::Repeat;
use crate::wz::Node;
use crate::WzBase;
use glam::{vec2, Vec2};
use image::DynamicImage;
use sdl3_sys::everything::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use ui::animation::use_raf;
use ui::event::{use_event, use_key, Interactive};
use ui::peniko::Color;
use ui::reactive::{
    create_effect, create_ref, create_rw_signal, use_context, RwSignal, SignalGet, SignalTrack,
    SignalUpdate, SignalWith,
};
use ui::style::dimension::{length, percent};
use ui::style::Styleable;
use ui::taffy::{AlignItems, Display, FlexDirection, JustifyContent, Position};
use ui::view_tuple::ViewTuple;
use ui::widget::debug::debug;
use ui::{
    dynamic, fragment, lazy, text, view, Bounds, Drawable, Element, Image, IntoElement,
    NineGridTexture, Renderer, Surface, TextInput, View,
};

#[derive(Clone)]
enum Stage {
    Logo,
    Login,
    Main,
}

pub fn app() -> impl IntoElement {
    // return fragment(text(|| "aa".to_string()).style(|s| s.block().width(100).height(100).bg_white()));

    let stage = RwSignal::new(Stage::Logo);
    let open = RwSignal::new(true);

    let current_map = RwSignal::new("910000000".to_string());

    fragment((
        view()
            .style(move |s| {
                s.cursor(CursorState::Idle)
                    .absolute()
                    .overflow_clip()
                    .width(800)
                    .height(600)
                    .bg_black()
            })
            .children(dynamic(move || match stage.get() {
                Stage::Logo => fragment(logo_scene(move || stage.set(Stage::Login))),
                Stage::Login => fragment((login_scene(move || stage.set(Stage::Main)))),
                Stage::Main => fragment((
                    map_scene(current_map),
                    status_bar().composite(),
                    world_map_window(open, current_map),
                )),
            })),
        debug().style(|s| {
            s.absolute()
                .top(0)
                .right(0)
                .width(800)
                .height(600)
                .background(Color::WHITE)
        }),
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
    fn draw(&self, painter: &mut Renderer) {
        let animation = &self.animations[self.index];
        animation.draw(painter);
    }

    fn size(&self) -> Vec2 {
        self.animations[self.index].size()
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn update(&mut self, delta: f32) -> bool {
        if self.complete {
            return false;
        }
        if self.animations.is_empty() {
            return false;
        }
        let animation = &mut self.animations[self.index];
        let updated = animation.update(delta);
        let index = self.index;
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

        self.index != index || updated
    }
}

pub fn logo_scene(on_finished: impl Fn() + 'static) -> impl IntoElement {
    let on_finished = create_ref(on_finished);

    lazy(
        move || {
            let WzBase { node: base } = use_context().unwrap();
            async move {
                let nexon: ASpriteAnimation = base
                    .at_path("UI/Logo.img/Nexon")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let wizet: ASpriteAnimation = base
                    .at_path("UI/Logo.img/Wizet")
                    .unwrap()
                    .try_into()
                    .unwrap();
                Some((nexon, wizet))
            }
        },
        move |resource| {
            let content = if let Some((nexon, wizet)) = resource {
                let mut animation = SequenceAnimation::new(vec![nexon, wizet]);
                animation.on_complete(move || {
                    on_finished.with(|f| f());
                });
                fragment(Image::new(animation).on_click(|_| {}))
            } else {
                fragment(view().style(|s| {
                    s.background(Color::new([1.0, 0.0, 0.0, 1.0]))
                        .width(20)
                        .height(20)
                }))
            };

            view()
                .style(|s| {
                    s.absolute()
                        .size_full()
                        .justify_center()
                        .items_center()
                        .bg_white()
                })
                .on_click(move |_| {
                    on_finished.with(|f| f());
                })
                .children(content)
        },
    )
}

pub fn button(btn_node: Node) -> View {
    let btn_state = create_rw_signal("normal".to_string());
    let animation = Rc::new(RefCell::new(None));
    let current = create_rw_signal(0);

    create_effect({
        let animation = animation.clone();
        move |_| {
            *animation.borrow_mut() = Some(
                SpriteAnimation::try_from(btn_node.at_path(&btn_state.get()).unwrap())
                    .unwrap()
                    .with_repeat(Repeat::Finite(1)),
            );
            current.set(0);
        }
    });

    use_raf({
        let animation = animation.clone();
        {
            move |delta| {
                let changed = {
                    let mut animation = animation.borrow_mut();
                    let animation = animation.as_mut().unwrap();
                    animation.update(delta);
                    animation.timer.index != current.get_untracked()
                };

                if changed {
                    let value = animation.borrow().as_ref().unwrap().timer.index;
                    current.set(value);
                }
            }
        }
    });

    view()
        .children(
            Image::dynamic({
                let animation = animation.clone();
                {
                    move || {
                        current.track();
                        let mut binding = animation.borrow_mut();
                        let animation = binding.as_mut().unwrap();
                        if animation.frames.len() == 0 {
                            None
                        } else {
                            Some(animation.current_frame().image.clone())
                        }
                    }
                }
            })
            .on_mouse_enter(move |_| {
                btn_state.set("mouseOver".to_string());
            })
            .on_mouse_leave(move |_| {
                btn_state.set("normal".to_string());
            })
            .style(move |s| {
                current.track();
                let binding = animation.borrow();
                let animation = binding.as_ref().unwrap();
                if animation.frames.len() == 0 {
                    return s;
                }
                let frame = animation.current_frame();
                let translation = frame.origin - animation.frames[0].origin;
                s.translate_x(translation.x)
                    .translate_y(translation.y)
                    .width(frame.size.x)
                    .height(frame.size.y)
            }),
        )
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
                let main_scene = Rc::new(RefCell::new(MainScene::new(map)));
                main_scene.borrow_mut().set_player(player);
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
    let images: Vec<Arc<DynamicImage>> = (0..9)
        .map(|i| node.get(&i.to_string()).try_into().unwrap())
        .collect();
    view()
        .style(|s| {
            s.justify_content(JustifyContent::FlexStart)
                .align_items(AlignItems::FlexStart)
                .gap_row(1.0)
                .gap_column(1.0)
        })
        .children((Image::new(images[1].clone()), Image::new(images[8].clone())))
}

pub fn bracket_wrap(children: impl ViewTuple) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/StatusBar.img/number").unwrap();
    let left: Arc<DynamicImage> = node.get("Lbracket").try_into().unwrap();
    let right: Arc<DynamicImage> = node.get("Rbracket").try_into().unwrap();
    view()
        .style(|s| {
            s.justify_content(JustifyContent::FlexStart)
                .align_items(AlignItems::Center)
                .gap_row(1.0)
                .gap_column(1.0)
        })
        .children(fragment((
            Image::new(left),
            fragment(children),
            Image::new(right),
        )))
}

pub fn status_bar_number(f: impl (Fn() -> String) + 'static) -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/StatusBar.img/number").unwrap();
    let images: Vec<Arc<DynamicImage>> = (0..10)
        .map(|i| node.get(&i.to_string()).try_into().unwrap())
        .collect();
    let slash: Arc<DynamicImage> = node.get("slash").try_into().unwrap();
    let percent: Arc<DynamicImage> = node.get("percent").try_into().unwrap();

    view()
        .style(|s| {
            s.justify_content(JustifyContent::FlexStart)
                .align_items(AlignItems::FlexStart)
        })
        .children(dynamic(move || {
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
}

pub fn status_bar() -> View {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/StatusBar.img").unwrap();
    let background: Arc<DynamicImage> = img
        .at_path("base")
        .unwrap()
        .get("backgrnd")
        .try_into()
        .unwrap();
    let background2: Arc<DynamicImage> = img
        .at_path("base")
        .unwrap()
        .get("backgrnd2")
        .try_into()
        .unwrap();

    let gauge = img.at_path("gauge").unwrap();
    let graduation: Arc<DynamicImage> = gauge.get("graduation").try_into().unwrap();
    let bar: Arc<DynamicImage> = gauge.get("bar").try_into().unwrap();

    let base_box: Arc<DynamicImage> = img.at_path("base/box").unwrap().try_into().unwrap();

    let icon_memo: Arc<DynamicImage> = img.at_path("base/iconMemo").unwrap().try_into().unwrap();
    let icon_blue: Arc<DynamicImage> = img.at_path("base/iconBlue").unwrap().try_into().unwrap();

    view()
        .style(|s| {
            s.position(Position::Absolute)
                .display(Display::Block)
                .left(length(0.0))
                .right(length(0.0))
                .width(percent(1.0))
                .bottom(length(0.0))
        })
        .children((
            Image::new(background),
            Image::new(background2).style(|s| {
                s.position(Position::Absolute)
                    .left(length(4.0))
                    .bottom(length(0.0))
            }),
            chat_box(),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .justify_content(JustifyContent::FlexEnd)
                        .align_items(AlignItems::FlexStart)
                        .top(length(8.0))
                        .right(length(4.0))
                })
                .children((
                    view()
                        .style(|s| {
                            s.margin_right(length(3.0))
                                .justify_content(JustifyContent::FlexStart)
                                .align_items(AlignItems::FlexStart)
                        })
                        .children((
                            Image::new(base_box),
                            view()
                                .style(|s| {
                                    s.position(Position::Absolute)
                                        .width(percent(1.0))
                                        .height(percent(1.0))
                                        .justify_content(JustifyContent::SpaceBetween)
                                        .align_items(AlignItems::Stretch)
                                })
                                .children((
                                    view()
                                        .style(|s| {
                                            s.width(length(20.0))
                                                .justify_content(JustifyContent::Center)
                                                .align_items(AlignItems::Center)
                                        })
                                        .children((Image::new(icon_blue))),
                                    view()
                                        .style(|s| {
                                            s.width(length(20.0))
                                                .justify_content(JustifyContent::Center)
                                                .align_items(AlignItems::Center)
                                        })
                                        .children((Image::new(icon_memo))),
                                )),
                        )),
                    view()
                        .style(|s| {
                            s.justify_content(JustifyContent::FlexStart)
                                .align_items(AlignItems::FlexStart)
                                .gap_column(2.0)
                        })
                        .children((
                            button(img.get("EquipKey")),
                            button(img.get("InvenKey")),
                            button(img.get("StatKey")),
                            button(img.get("SkillKey")),
                            button(img.get("KeySet")),
                            button(img.get("QuickSlot")),
                        )),
                )),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .left(length(2.0))
                        .right(length(4.0))
                        .bottom(length(1.0))
                        .height(length(34.0))
                        .justify_content(JustifyContent::FlexStart)
                        .align_items(AlignItems::Center)
                })
                .children((view()
                    .style(|s| {
                        s.width(length(208.0))
                            .justify_content(JustifyContent::FlexStart)
                            .align_items(AlignItems::Center)
                    })
                    .children((
                        // level card
                        view()
                            .style(|s| {
                                s.margin_left(length(3.0))
                                    .width(length(74.0))
                                    .height(length(30.0))
                            })
                            .children(
                                // level no
                                view()
                                    .style(|s| {
                                        s.position(Position::Absolute)
                                            .left(length(27.0))
                                            .bottom(length(8.0))
                                            .width(length(47.0))
                                            .height(length(11.0))
                                            .justify_content(JustifyContent::Center)
                                            .align_items(AlignItems::Center)
                                    })
                                    .children(level_no(|| 18)),
                            ),
                        // job name
                        view()
                            .style(|s| {
                                s.margin_left(length(8.0))
                                    .flex_grow(1.0)
                                    .height(length(30.0))
                                    .flex_direction(FlexDirection::Column)
                            })
                            .children((
                                view()
                                    .style(|s| {
                                        s.justify_content(JustifyContent::FlexStart)
                                            .align_items(AlignItems::FlexStart)
                                            .gap_column(length(2.0))
                                    })
                                    .children((
                                        (text(|| "魔法师").style(|s| {
                                            s.color(Color::WHITE).font_size(12.0).line_height(15.0)
                                        })),
                                        bracket_wrap(text(|| "魔法师").style(|s| {
                                            s.color(Color::WHITE).font_size(12.0).line_height(15.0)
                                        })),
                                    )),
                                text(|| "三个榔头").style(|s| {
                                    s.color(Color::WHITE).font_size(12.0).line_height(15.0)
                                }),
                            )),
                    )),)),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .display(Display::Block)
                        .left(length(218.0))
                        .bottom(length(1.0))
                })
                .children((
                    Image::new(bar),
                    Image::new(graduation)
                        .style(|s| s.position(Position::Absolute).bottom(length(0.0))),
                    view()
                        .style(|s| {
                            s.position(Position::Absolute)
                                .display(Display::Block)
                                .top(length(3.0))
                                .left(length(19.0))
                        })
                        .children(bracket_wrap(status_bar_number(|| "124/274".to_string()))),
                    view()
                        .style(|s| {
                            s.position(Position::Absolute)
                                .display(Display::Block)
                                .top(length(3.0))
                                .left(length(131.0))
                        })
                        .children(bracket_wrap(status_bar_number(|| "118/617".to_string()))),
                    view()
                        .style(|s| {
                            s.position(Position::Absolute)
                                .display(Display::Block)
                                .top(length(3.0))
                                .left(length(248.0))
                        })
                        .children(bracket_wrap(status_bar_number(|| "6839/13716".to_string()))),
                )),
            view()
                .style(|s| {
                    s.position(Position::Absolute)
                        .justify_content(JustifyContent::SpaceBetween)
                        .right(length(4.0))
                        .bottom(length(1.0))
                        .width(length(224.0))
                        .height(length(34.0))
                })
                .children((
                    button(img.get("BtShop")),
                    button(img.get("BtNPT")),
                    button(img.get("BtMenu")),
                    button(img.get("BtShort")),
                )),
        ))
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
                view()
                    .style(|s| {
                        s.position(Position::Absolute)
                            .justify_content(JustifyContent::SpaceBetween)
                            .align_items(AlignItems::Center)
                            .top(length(6.0))
                            .left(length(0.0))
                            .padding_left(length(8.0))
                            .width(length(568.0))
                            .height(length(24.0))
                            .background([0x88, 0x88, 0x88])
                    })
                    .children((
                        (text(|| "欢迎来到冒险岛，现在开始你的旅程吧～！".to_string())
                            .style(|s| s.font_size(11.0).line_height(11.0).color(Color::WHITE))),
                        view()
                            .style(|s| s.align_items(AlignItems::Center))
                            .children((
                                view().style(|s| s.margin_right(length(3.0))).children(
                                    button(basic.get("BtMax")).on_click(move |_| open.set(true)),
                                ),
                                scroll_vertical(),
                            )),
                    )),
            )
        } else {
            let mut input_ref = None;

            fragment(
                view()
                    .style(|s| {
                        s.position(Position::Absolute)
                            .align_items(AlignItems::Center)
                            .top(length(6.0))
                            .left(length(0.0))
                            .width(length(568.0))
                            .height(length(24.0))
                    })
                    .children((
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
                        view()
                            .style(|s| {
                                s.position(Position::Absolute)
                                    .align_items(AlignItems::Center)
                                    .height(percent(1.0))
                                    .right(length(18.0))
                            })
                            .children(
                                button(basic.get("BtMin")).on_click(move |_| open.set(false)),
                            ),
                    )),
            )
        }
    })
}

pub fn scroll_vertical() -> View {
    let WzBase { node: base } = use_context().unwrap();
    let node = base.at_path("UI/Basic.img/VScr5/enabled").unwrap();
    let prev: Arc<DynamicImage> = node.get("prev0").try_into().unwrap();
    let next: Arc<DynamicImage> = node.get("next0").try_into().unwrap();
    view()
        .style(|s| s.width(length(15.0)).height(length(25.0)))
        .children((
            Image::new(prev).style(|s| s.position(Position::Absolute).top(length(0.0))),
            Image::new(next).style(|s| s.position(Position::Absolute).bottom(length(0.0))),
        ))
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
            .try_into()
            .unwrap();

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
            .try_into()
            .unwrap();

        let map_image = create_rw_signal(Rc::new(map_image));

        let world_map_signal = create_rw_signal(
            WorldMap::try_from(base.at_path("Map/WorldMap/WorldMap.img").unwrap()).unwrap(),
        );

        let hovered_link = create_rw_signal(None);

        let content_size = vec2(640.0, 470.0);

        let world_map_node = base.at_path("UI/UIWindow.img/WorldMap").unwrap();
        let title: Sprite = world_map_node.get("title").try_into().unwrap();

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
                            world_map_signal.set(
                                WorldMap::try_from(
                                    base.at_path(&format!("Map/WorldMap/{}.img", &link_map))
                                        .unwrap(),
                                )
                                .unwrap(),
                            );
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
            view()
                .style(|s| {
                    s.absolute()
                        .left(0)
                        .top(0)
                        .w_full()
                        .h_full()
                        .justify_center()
                        .items_center()
                })
                .children(
                    view().children((
                        Image::new(bg).style(|s| {
                            s.position(Position::Absolute)
                                .width(percent(1.0))
                                .height(percent(1.0))
                        }),
                        view()
                            .composite()
                            .style({
                                let padding = padding.clone();
                                move |s| s.position(Position::Relative).padding(padding)
                            })
                            .children(
                                view()
                                    .on_mouse_move(move |event| {
                                        let mouse = event.offset();
                                        let key =
                                            world_map_signal.with_untracked(move |world_map| {
                                                if let Some(map_link) = &world_map.map_link {
                                                    for (key, item) in map_link {
                                                        let lt = content_size / 2.0
                                                            - item.link_img.origin;
                                                        let rb = lt + item.link_img.size;
                                                        if (mouse.cmpge(lt).all())
                                                            && mouse.cmplt(rb).all()
                                                        {
                                                            let pt = mouse - lt;

                                                            let pixel = item
                                                                .link_img
                                                                .image
                                                                .as_rgba8()
                                                                .unwrap()
                                                                .get_pixel(
                                                                    pt.x as u32,
                                                                    pt.y as u32,
                                                                );

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
                                    })
                                    .children((
                                        dynamic(move || {
                                            world_map_signal.with(|world_map| {
                                                Image::new(world_map.base_img.image.clone()).style(
                                                    move |s| {
                                                        s.width(length(content_size.x))
                                                            .height(length(content_size.y))
                                                    },
                                                )
                                            })
                                        }),
                                        active_link_view,
                                        spots,
                                    )),
                            ),
                        view()
                            .style(move |s| {
                                s.position(Position::Absolute)
                                    .w_full()
                                    .height(14)
                                    .margin_top(5)
                                    .padding_right(padding.right)
                                    .padding_left(padding.left)
                                    .flex_row()
                                    .justify_between()
                                    .items_center()
                            })
                            .children((Image::new(title.image), close_button)),
                    )),
                ),
        );
    })
}
