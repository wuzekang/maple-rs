use crate::cursor::CursorState;
use crate::map::world_map::{load_world_map, WorldMap};
use crate::sprite::Sprite;
use crate::ui::async_image::AsyncImage;
use crate::ui::button;
use crate::wz::{Node, WzSplitReaderExt};
use crate::WzSplitReaderContext;
use ::ui::event::{use_key, Interactive};
use ::ui::reactive::{
  create_rw_signal, use_context, RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use ::ui::style::dimension::{length, percent};
use ::ui::style::Styleable;
use ::ui::taffy::Position;
use ::ui::widget::draggable::use_draggable;
use ::ui::{dynamic, fragment, lazy, view, Image, IntoElement, NineGridTexture, Surface};
use glam::vec2;
use image::DynamicImage;
use sdl3_sys::everything::{SDL_Renderer, SDLK_W};
use std::rc::Rc;
use std::sync::Arc;

// 只存储可以跨线程传递的数据
struct WorldMapData {
  border_images: Vec<Arc<DynamicImage>>,
  map_images: Vec<Sprite>,
  world_map: WorldMap,
  title: Sprite,
  close_button_node: Option<Node>,
}

// 异步加载所有 WorldMap 数据
async fn load_world_map_data(
  reader: Arc<wz_splitter::reader::SplitWzReader>,
  world_map_path: String,
) -> Result<WorldMapData, Box<dyn std::error::Error>> {
  // 并发加载多个资源
  let (border_node, map_images_node, ui_node, close_button_node) = futures::try_join!(
    reader.get_node("UI/UIWindow.img/WorldMap/Border"),
    reader.get_node("Map/MapHelper.img/worldMap/mapImage"),
    reader.get_node("UI/UIWindow.img/WorldMap"),
    reader.get_node("UI/Basic.img/BtClose")
  )?;

  // 加载世界地图
  let world_map = load_world_map(reader, &world_map_path).await?;

  // 转换数据
  let border_images: Vec<Arc<DynamicImage>> = border_node
    .try_into()
    .map_err(|_| "Failed to parse border images")?;

  let map_images: Vec<Sprite> = map_images_node
    .try_into()
    .map_err(|_| "Failed to parse map images")?;

  let title: Sprite = ui_node
    .get("title")
    .try_into()
    .map_err(|_| "Failed to parse title")?;

  Ok(WorldMapData {
    border_images,
    map_images,
    world_map,
    title,
    close_button_node: Some(close_button_node),
  })
}

pub fn world_map_window(
  open: RwSignal<bool>,
  current_map: RwSignal<(String, Option<String>)>,
) -> impl IntoElement {
  use_key(SDLK_W, move || {
    open.set(true);
  });

  dynamic(move || {
    if !open.get() {
      return fragment(());
    }

    let reader = use_context::<WzSplitReaderContext>().map(|ctx| ctx.reader.clone());
    let renderer: *mut SDL_Renderer = use_context().unwrap();

    // 用于跟踪当前世界地图路径
    let world_map_path = create_rw_signal("Map/WorldMap/WorldMap.img".to_string());
    let hovered_link = create_rw_signal(None);
    let content_size = vec2(640.0, 470.0);
    // 窗口偏移量
    let window_offset = create_rw_signal(vec2(0.0, 0.0));

    fragment(lazy(
      move || {
        let reader = reader.clone();
        let path = world_map_path.get();

        async move {
          match reader {
            Some(reader) => match load_world_map_data(reader, path).await {
              Ok(data) => Some(data),
              Err(e) => {
                log::error!("Failed to load world map data: {}", e);
                None
              }
            },
            None => {
              log::warn!("No WzSplitReaderContext available");
              None
            }
          }
        }
      },
      move |data: Option<WorldMapData>| {
        match data {
          Some(data) => {
            // 在渲染时创建 SDL 资源
            let border_surfaces: Vec<Surface> = data
              .border_images
              .into_iter()
              .map(|item| item.into())
              .collect();

            let bg = NineGridTexture::new(
              (
                &border_surfaces[0],
                &border_surfaces[1],
                &border_surfaces[2],
                &border_surfaces[3],
                &border_surfaces[4],
                &border_surfaces[5],
                &border_surfaces[6],
                &border_surfaces[7],
              ),
              renderer,
            );

            let padding = bg.border();
            let world_map_signal = create_rw_signal(data.world_map);
            let map_image = create_rw_signal(Rc::new(data.map_images));

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
                    view()
                      .children(AsyncImage::new(
                        map_image.get_untracked()[spot_type].path.clone(),
                      ))
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
              let Some(key) = hovered_link.get() else {
                return fragment(());
              };

              let Some((path, position, link_map)) = world_map_signal.with(move |world_map| {
                let map_link = world_map.map_link.as_ref()?;
                let link = &map_link[&key];
                let link_img = &link.link_img;
                let link_map = link.link_map.clone();
                let path = link_img.path.clone();
                let position = content_size / 2.0 - link_img.origin;
                Some((path, position, link_map))
              }) else {
                return fragment(());
              };

              fragment(
                view()
                  .children(AsyncImage::new(path))
                  .style(move |s| {
                    s.position(Position::Absolute)
                      .left(length(position.x))
                      .top(length(position.y))
                      .cursor(CursorState::LClick)
                  })
                  .on_click(move |_| {
                    hovered_link.set(None);
                    world_map_path.set(format!("Map/WorldMap/{}.img", link_map));
                  }),
              )
            });

            let close_button = match data.close_button_node {
              Some(node) => fragment(button(node).on_click(move |_| open.set(false))),
              None => fragment(()),
            };

            fragment(
              view()
                .style(move |s| {
                  let offset = window_offset.get();
                  s.absolute()
                    .left(0)
                    .top(0)
                    .w_full()
                    .h_full()
                    .justify_center()
                    .items_center()
                    .translate_x(offset.x)
                    .translate_y(offset.y)
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
                                view()
                                  .children(AsyncImage::new(world_map.base_img.path.clone()))
                                  .style(move |s| {
                                    s.width(length(content_size.x))
                                      .height(length(content_size.y))
                                  })
                              })
                            }),
                            active_link_view,
                            spots,
                          )),
                      ),
                    use_draggable(
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
                        .children((AsyncImage::new(data.title.path), close_button)),
                      move |offset| {
                        window_offset.set(offset);
                      },
                    ),
                  )),
                ),
            )
          }
          None => fragment(()),
        }
      },
    ))
  })
}
