use crate::character::{Character, ZMap};
use crate::scenes::login::{LoginContext, LoginStep};
use crate::ui::button;
use crate::WzBase;
use image::DynamicImage;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use ::ui::event::Interactive;
use ::ui::peniko::Color;
use ::ui::reactive::{
    create_effect, create_rw_signal, use_context, SignalGet, SignalRead, SignalUpdate,
};
use ::ui::style::{Styleable, TextWrap};
use ::ui::{dynamic, text, view, Drawable, Image, IntoElement, TextInput};

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

                // let face = value[0];
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
                    avatar
                        .borrow_mut()
                        .insert(base.at_path(&format!("Character/{part}.img")).unwrap())
                }

                avatar.borrow_mut().flip = true;
                avatar.borrow_mut().set_action("stand1");
            }
        }
    });

    view().style(|s| s.w_full().h_full()).children((
        Image::new(avatar as Rc<RefCell<dyn Drawable>>).style(|s| s.absolute().left(395).top(340)),
        dynamic(move || {
            if !name_confirmed.get() {
                view()
                    .style(|s| s.absolute().left(481).top(95).width(201).height(224))
                    .children((
                        Image::new({
                            let char_name: Arc<DynamicImage> = node.get("charName").try_into().unwrap();
                            char_name
                        }),
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
                        Image::new({
                            let char_set: Arc<DynamicImage> = node.get("charSet").try_into().unwrap();
                            char_set
                        }),
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
                                            Image::new({
                                                let avatar: Arc<DynamicImage> = node.at_path(&format!("avatarSel/{i}/normal"))
                                                    .unwrap()
                                                    .try_into()
                                                    .unwrap();
                                                avatar
                                            }),
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
                                                            options[gender][i]
                                                                [value.get()[gender][i]]
                                                                .1
                                                                .to_string()
                                                        }
                                                    })
                                                    .style(|s| s.text_wrap(TextWrap::None)),
                                                ),
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
    ))
}