use crate::cursor::CursorState;
use crate::map::world_map::WorldMap;
use crate::sprite::Sprite;
use crate::ui::button;
use crate::WzBase;
use glam::{vec2, Vec2};
use image::DynamicImage;
use sdl3_sys::everything::{SDL_Renderer, SDLK_W};
use std::rc::Rc;
use std::sync::Arc;
use ::ui::event::{use_key, Interactive};
use ::ui::reactive::{
    create_rw_signal, use_context, RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use ::ui::style::dimension::{length, percent};
use ::ui::style::Styleable;
use ::ui::taffy::Position;
use ::ui::{dynamic, fragment, view, Image, IntoElement, NineGridTexture, Surface};

pub fn world_map_window(
    open: RwSignal<bool>,
    current_map: RwSignal<(String, Option<String>)>,
) -> impl IntoElement {
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
                                    current_map.set((format!("{:0>9}", map_no), None))
                                }
                            })
                    })
                    .collect::<Vec<_>>()
            }))
        });

        let active_link_view = dynamic(move || {
            if let Some(key) = hovered_link.get() {
                let (image, position, link_map) = world_map_signal.with(move |world_map| {
                    let map_link = world_map.map_link.as_ref().unwrap();
                    let link = &map_link[&key];
                    let link_img = &link.link_img;
                    let link_map = link.link_map.clone();
                    let image = link_img.image.clone();
                    // let text = map_link[&key].tool_tip.clone();
                    let position = content_size / 2.0 - link_img.origin;
                    (image, position, link_map)
                });
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

        fragment(
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
                                let padding = padding;
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
                                                None
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
        )
    })
}