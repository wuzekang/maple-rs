use crate::character::{Character, ZMap};
use crate::scenes::login::{LoginContext, LoginStep};
use crate::ui::async_image::AsyncImage;
use crate::ui::button;
use crate::{wz::WzSplitReaderExt, WzSplitReaderContext};
use ::ui::event::Interactive;
use ::ui::peniko::Color;
use ::ui::reactive::{create_rw_signal, use_context, SignalGet, SignalRead, SignalUpdate};
use ::ui::style::{Styleable, TextWrap};
use ::ui::{
  dynamic, fragment, lazy, text, use_resource, view, Drawable, Image, IntoElement, TextInput,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub fn create_normal() -> impl IntoElement {
  let WzSplitReaderContext { reader } = use_context().unwrap();
  let ctx: LoginContext = use_context().unwrap();

  lazy(
    move || {
      let reader = reader.clone();
      async move {
        let newchar_node = reader.get_node("UI/Login.img/NewChar").await.ok()?;
        let zmap_node = reader.get_node("zmap.img").await.ok()?;
        
        // 加载默认 avatar 配置
        let default_parts = vec![
          "Face/00020000",     // 默认脸部
          "Hair/00030020",     // 默认发型 + 颜色
          "00002000",          // 默认皮肤
          "00012000",          // 默认皮肤
          "Coat/01040002",     // 默认上衣
          "Pants/01060002",    // 默认裤子
          "Shoes/01072005",    // 默认鞋子
          "Weapon/01302000",   // 默认武器
        ];
        
        let mut default_nodes = Vec::new();
        for part in default_parts {
          if let Ok(node) = reader.get_node(&format!("Character/{part}.img")).await {
            default_nodes.push(node);
          }
        }
        
        Some((newchar_node, zmap_node, default_nodes))
      }
    },
    move |data_opt: Option<(crate::wz::Node, crate::wz::Node, Vec<crate::wz::Node>)>| {
      let Some((node, zmap_node, default_nodes)) = data_opt else {
        return fragment(());
      };
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

      let z_map: Arc<ZMap> = Arc::new(zmap_node.try_into().unwrap());

      let avatar = Rc::new(RefCell::new(Character::new(default_nodes, z_map)));

      let WzSplitReaderContext { reader } = use_context().unwrap();

      // 使用 use_resource 来动态加载角色部件
      use_resource(
        {
          let options = options.clone();
          move || {
            let binding = value.read();
            let value = binding.borrow();
            let gender = gender.get();
            let value = &value[gender];

            let value = value
              .iter()
              .enumerate()
              .map(|(i, j)| options[gender][i][*j].0)
              .collect::<Vec<_>>();

            let mut parts = vec![];
            parts.push(format!("Face/000{}", value[0]));
            parts.push(format!("Hair/000{}", value[1] + value[2]));
            parts.push(format!("0000{}", value[3]));
            parts.push(format!("0001{}", value[3]));
            parts.push(format!("Coat/0{}", value[4]));
            parts.push(format!("Pants/0{}", value[5]));
            parts.push(format!("Shoes/0{}", value[6]));
            parts.push(format!("Weapon/0{}", value[7]));

            let reader = reader.clone();
            async move {
              let mut nodes = Vec::new();
              for part in parts {
                if let Ok(node) = reader.get_node(&format!("Character/{part}.img")).await {
                  nodes.push(node);
                }
              }
              nodes
            }
          }
        },
        {
          let avatar = avatar.clone();
          move |nodes: Vec<crate::wz::Node>| {
            // 清理之前的部件
            avatar.borrow_mut().slots.clear();
            for node in nodes {
              avatar.borrow_mut().insert(node);
            }
            avatar.borrow_mut().flip = true;
            avatar.borrow_mut().set_action("stand1");
          }
        },
      );

      fragment(view().style(|s| s.w_full().h_full()).children((
        Image::new(avatar as Rc<RefCell<dyn Drawable>>).style(|s| s.absolute().left(395).top(340)),
        dynamic(move || {
          if !name_confirmed.get() {
            view()
              .style(|s| s.absolute().left(481).top(95).width(201).height(224))
              .children((
                AsyncImage::new("UI/Login.img/NewChar/charName"),
                TextInput::new().style(|s| {
                  s.absolute()
                    .left(29)
                    .top(104)
                    .width(147)
                    .height(23)
                    .font_size(13.0)
                    .line_height(15.0)
                    .color(Color::WHITE)
                }),
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
            view()
              .style(|s| s.absolute().left(481).top(95).width(225).height(377))
              .children((
                AsyncImage::new("UI/Login.img/NewChar/charSet"),
                view()
                  .style(|s| {
                    s.absolute()
                      .left(11)
                      .top(105)
                      .width(200)
                      .height(161)
                      .flex_col()
                      .gap_column(1)
                  })
                  .children(
                    (0..9)
                      .map(|i| {
                        let len = options[gender.get()][i].len();
                        view().children((
                          AsyncImage::new(format!("UI/Login.img/NewChar/avatarSel/{i}/normal")),
                          view()
                            .style(|s| {
                              s.absolute()
                                .right(15)
                                .bottom(0)
                                .width(113)
                                .height(15)
                                .font_size(13.0)
                                .line_height(15.0)
                                .justify_center()
                            })
                            .children(
                              text({
                                let options = options.clone();
                                move || {
                                  let gender = gender.get();
                                  options[gender][i][value.get()[gender][i]].1.to_string()
                                }
                              })
                              .style(|s| s.text_wrap(TextWrap::None)),
                            ),
                          button(node.get("BtLeft"))
                            .style(|s| s.absolute().left(57).bottom(0))
                            .on_click(move |_| {
                              if i == 8 {
                                gender.update(|v| *v = if *v == 0 { 1 } else { 0 });
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
                                gender.update(|v| *v = if *v == 0 { 1 } else { 0 });
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
                  ),
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
      )))
    },
  )
}
