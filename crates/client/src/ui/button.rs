use crate::cursor::CursorState;
use crate::sound::play_sound;
use crate::sprite::SpriteAnimation;
use crate::timer::Repeat;
use crate::wz::Node;
use std::cell::RefCell;
use std::rc::Rc;
use ::ui::animation::use_raf;
use ::ui::event::Interactive;
use ::ui::reactive::{create_effect, create_rw_signal, SignalGet, SignalTrack, SignalUpdate};
use ::ui::style::Styleable;
use ::ui::{view, Drawable, Image, View};

pub fn button(btn_node: Node) -> View {
    let animation = Rc::new(RefCell::new(None));
    let current = create_rw_signal(0);
    let pressed = create_rw_signal(false);
    let hovered = create_rw_signal(false);

    create_effect({
        let animation = animation.clone();
        move |_| {
            let state = if pressed.get() {
                "pressed"
            } else if hovered.get() {
                "mouseOver"
            } else {
                "normal"
            };
            *animation.borrow_mut() = Some(
                SpriteAnimation::try_from(btn_node.at_path(state).unwrap())
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
                        if animation.frames.is_empty() {
                            None
                        } else {
                            Some(animation.current_frame().image.clone())
                        }
                    }
                }
            })
            .on_mouse_enter(move |_| {
                play_sound("Sound/UI.img/BtMouseOver");
                hovered.set(true);
            })
            .on_mouse_leave(move |_| {
                hovered.set(false);
                pressed.set(false);
            })
            .on_mouse_down(move |_| {
                pressed.set(true);
            })
            .on_mouse_up(move |_| {
                pressed.set(false);
            })
            .on_click(move |_| {
                play_sound("Sound/UI.img/BtMouseClick");
            })
            .style(move |s| {
                current.track();
                let binding = animation.borrow();
                let animation = binding.as_ref().unwrap();
                if animation.frames.is_empty() {
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