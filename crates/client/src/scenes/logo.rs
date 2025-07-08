use crate::animation::SequenceAnimation;
use crate::sound::play_sound;
use crate::sprite::ASpriteAnimation;
use crate::{wz::WzSplitReaderExt, WzSplitReaderContext};
use ::ui::event::Interactive;
use ::ui::reactive::{create_ref, create_rw_signal, use_context, SignalGet, SignalUpdate};
use ::ui::style::Styleable;
use ::ui::{dynamic, fragment, lazy, view, Image, IntoElement};

pub fn logo_scene(on_finished: impl Fn() + 'static) -> impl IntoElement {
    let on_finished = create_ref(on_finished);

    lazy(
        move || {
            let WzSplitReaderContext { reader } = use_context().unwrap();
            async move {
                let logo_node = reader.get_node("UI/Logo.img").await.ok()?;
                let nexon: ASpriteAnimation = logo_node
                    .get("Nexon")
                    .try_into()
                    .unwrap();
                let wizet: ASpriteAnimation = logo_node
                    .get("Wizet")
                    .try_into()
                    .unwrap();
                Some((nexon, wizet))
            }
        },
        move |resource| {
            #[derive(Copy, Clone)]
            enum Step {
                NxLogo,
                WzLogo,
            }

            impl Step {
                fn next(self) -> Option<Self> {
                    match self {
                        Step::NxLogo => Some(Step::WzLogo),
                        Step::WzLogo => None,
                    }
                }
            }

            let step = create_rw_signal(Step::NxLogo);

            let content = if let Some((nexon, wizet)) = resource {
                fragment(dynamic(move || {
                    Image::new({
                        let mut animation = SequenceAnimation::new(vec![match step.get() {
                            Step::NxLogo => {
                                play_sound("Sound/BgmUI.img/NxLogo");
                                nexon.clone()
                            }
                            Step::WzLogo => {
                                play_sound("Sound/BgmUI.img/WzLogo");
                                wizet.clone()
                            }
                        }]);
                        animation.on_complete(move || {
                            if let Some(next) = step.get_untracked().next() {
                                step.set(next);
                            } else {
                                on_finished.with(|f| f());
                            }
                        });
                        animation
                    })
                }))
            } else {
                fragment(())
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
                    if let Some(next) = step.get_untracked().next() {
                        step.set(next);
                    } else {
                        on_finished.with(|f| f());
                    }
                })
                .children(content)
        },
    )
}