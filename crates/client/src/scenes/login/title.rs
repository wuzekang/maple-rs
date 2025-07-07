use crate::sprite::SpriteAnimation;
use crate::ui::button;
use crate::WzBase;
use image::DynamicImage;
use std::sync::Arc;
use ::ui::event::Interactive;
use ::ui::peniko::Color;
use ::ui::reactive::{create_rw_signal, use_context, SignalGet, SignalUpdate};
use ::ui::style::Styleable;
use ::ui::{dynamic, fragment, view, Image, IntoElement, TextInput};

pub fn title_view(on_login: impl Fn() + 'static) -> impl IntoElement {
    let WzBase { node: base } = use_context().unwrap();
    let img = base.at_path("UI/Login.img").unwrap();
    let title = img.get("Title");
    let checked = create_rw_signal(false);
    let check_image: Vec<Arc<::image::DynamicImage>> =
        title.at_path("check").unwrap().try_into().unwrap();

    let position = [(562, 2), (561, 4), (565, 5), (558, 4), (552, 4), (555, 3)];

    let effect = Vec::<SpriteAnimation>::try_from(title.get("effect")).unwrap();
    let effect = view()
        .style(|s| s.pointer_events_none().absolute().left(0).top(0))
        .children(
            effect
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    Image::new(item)
                        .style(move |s| s.absolute().left(position[i].0).top(position[i].1))
                })
                .collect::<Vec<_>>(),
        );

    let board = view()
        .style(move |s| s.absolute().left(396).top(223).width(288).height(165))
        .children((
            view()
                .style(|s| {
                    s.absolute()
                        .left(45)
                        .top(12)
                        .width(147)
                        .flex_col()
                        .items_stretch()
                        .gap_column(6)
                        .color(Color::WHITE)
                })
                .children((
                    TextInput::new().style(|s| s.height(23)),
                    TextInput::new().style(|s| s.height(23)),
                )),
            button(title.get("BtLogin"))
                .style(|s| s.absolute().top(0).right(0))
                .on_click(move |_| {
                    on_login();
                }),
            view()
                .style(|s| s.absolute().left(4).top(78).width(274).height(24))
                .children((
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
                )),
            view()
                .style(|s| {
                    s.absolute()
                        .width(284)
                        .height(40)
                        .left(0)
                        .top(125)
                        .gap_column(8)
                })
                .children((
                    button(title.get("BtNew")).style(|s| s.margin_top(1)),
                    button(title.get("BtHomePage")),
                    button(title.get("BtQuit")),
                )),
        ));

    fragment((board, effect))
}