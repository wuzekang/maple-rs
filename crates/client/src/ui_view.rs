use crate::scene::{Camera, MainScene, EventEmitter};
use crate::map::world_map::WorldMap;
use crate::sdl::{NineGridDrawable, Surface};
use crate::sprite::Sprite;
use crate::{map, sdl, WzBase};
use glam::vec2;
use image::DynamicImage;
use sdl3_sys::everything::SDL_Renderer;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use ui::event::Interactive;
use ui::peniko::Color;
use ui::reactive::{
    create_rw_signal, on_cleanup, use_context, RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use ui::taffy::prelude::{length, percent};
use ui::taffy::{AlignItems, FlexDirection, JustifyContent, Position};
use ui::{dynamic, fragment, text, view, Drawable, Image, ImageTexture, IntoElement, ViewId};

pub fn ui_view() -> impl IntoElement {
    let open = RwSignal::new(true);
    let current_map = RwSignal::new("002000000".to_string());

    fragment((
        dynamic(move || map_scene(&current_map.get())),
        world_map_window(open, current_map),
    ))
}

pub fn map_scene(map_name: &str) -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let map::Map { layers, .. } = map::Map::new(&base, map_name).unwrap();
    let window = use_context().unwrap();
    let renderer = use_context().unwrap();
    let camera_signal = create_rw_signal(Camera::default());
    let size = vec2(800.0, 600.0);

    let map_scene_drawable: Rc<RefCell<Box<dyn Drawable>>> = Rc::new(RefCell::new(Box::new(
        MainScene::new(window, renderer, size, map_name, camera_signal),
    )));

    let update_event: EventEmitter = use_context().unwrap();

    let key = update_event.on({
        let d = map_scene_drawable.clone();
        move || {
            d.borrow_mut().update();
        }
    });

    on_cleanup(move || {
        update_event.off(key);
    });

    let mut texts = vec![];
    for layer in &layers {
        for item in &layer.objects {
            for sprite in &item.sprites {
                let path = sprite.path.clone();
                let position = item.position;
                texts.push(text({ move || path.clone() }).style(move |s| {
                    s.position(Position::Absolute)
                        .left(length(position.x))
                        .top(length(position.y))
                        .font_size(12.0)
                        .line_height(14.0)
                        .background(Color::WHITE)
                        .color(Color::RED)
                }));
            }
        }
    }

    view((Image::new(map_scene_drawable), fragment(texts))).style(move |s| {
        let camera = camera_signal.get();
        s.position(Position::Absolute)
            .width(percent(1.0))
            .height(percent(1.0))
            .left(length(-camera.position.x))
            .top(length(-camera.position.y))
    })
}

pub fn world_map_window(open: RwSignal<bool>, current_map: RwSignal<String>) -> impl IntoElement {
    let root = use_context::<ViewId>().unwrap();
    let remove = root.add_event_listener(Box::new(move |event| {
        if unsafe { event.event.r#type } != sdl3_sys::events::SDL_EventType::KEY_DOWN.0 {
            return;
        }
        if unsafe { event.event.key.scancode } == sdl3_sys::scancode::SDL_SCANCODE_W {
            open.set(!open.get());
        }
    }));
    on_cleanup(move || {
        remove();
    });

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

        let bg = sdl::NineGridTexture::new(
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

        let btn_state = create_rw_signal("normal".to_string());
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
                        Image::new(ImageTexture::new(
                            renderer,
                            &map_image.get_untracked()[spot_type].image,
                        ))
                        .style(move |s| {
                            s.position(Position::Absolute)
                                .margin_left(length(position.x))
                                .margin_top(length(position.y))
                        })
                        .on_click(move |_| {
                            if let Some(map_no) = map_no.as_ref() {
                                if map_no.len() == 1 {
                                    current_map.set(format!("{:0>9}", map_no["0"]))
                                }
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
                    Image::new(ImageTexture::new(renderer, &image))
                        .style(move |s| {
                            s.position(Position::Absolute)
                                .left(length(position.x))
                                .top(length(position.y))
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

        return fragment(
            view(view((
                Image::new(NineGridDrawable { texture: bg }).style(|s| {
                    s.position(Position::Absolute)
                        .width(percent(1.0))
                        .height(percent(1.0))
                }),
                view(
                    view((
                        dynamic(move || {
                            world_map_signal.with(|world_map| {
                                Image::new(ImageTexture::new(renderer, &world_map.base_img.image))
                                    .style(move |s| {
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

                        if hovered_link.get() != key {
                            hovered_link.set(key);
                        }
                    })
                    .on_mouse_enter(|_| {})
                    .on_mouse_leave(move |_| {
                        if hovered_link.get().is_some() {
                            hovered_link.set(None);
                        }
                    }),
                )
                .style({
                    let padding = padding.clone();
                    move |s| s.position(Position::Relative).padding(padding)
                }),
                view((
                    Image::new(ImageTexture::new(renderer, &title.image)),
                    Image::new(Box::new(move || {
                        let path = format!("UI/Basic.img/BtClose/{}/0", btn_state.get());
                        let btn_close: Sprite = base.at_path(&path).unwrap().into();
                        ImageTexture::new(renderer, &btn_close.image)
                    }) as Box<dyn (Fn() -> ImageTexture)>)
                    .on_mouse_enter(move |_| {
                        btn_state.set("mouseOver".to_string());
                    })
                    .on_mouse_leave(move |_| {
                        btn_state.set("normal".to_string());
                    })
                    .on_click(move |_| {
                        open.set(false);
                    }),
                ))
                .style(move |s| {
                    s.position(Position::Absolute)
                        .width(percent(1.0))
                        .margin_top(length(5.0))
                        .padding_right(padding.right.into())
                        .padding_left(padding.left.into())
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
