use dioxus_core::{prelude::*, ElementId, WriteMutations};
use dioxus_core_macro::rsx;
use std::rc::Rc;
use tokio::select;

mod dioxus_elements {
    pub trait DioxusElement {
        const TAG_NAME: &'static str;
        const NAME_SPACE: Option<&'static str>;
    }
    pub struct div;
    impl DioxusElement for div {
        const TAG_NAME: &'static str = "div";
        const NAME_SPACE: Option<&'static str> = None;
    }

    pub mod events {

        use dioxus_core::{prelude::EventHandler, Attribute, AttributeValue, Event};

        pub fn on_click(mut f: impl FnMut(i32) + 'static) -> Attribute {
            Attribute::new(
                "on_click",
                AttributeValue::Listener(EventHandler::new(move |e| {
                    f(0);
                })),
                None,
                false,
            )
        }
    }
}

use dioxus_elements::*;
use dioxus_hooks::use_signal;

fn app() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        div {
            on_click: move |_| {
                println!("clicked");
                count += 1;
            },
            "count: {count}"
        }
    }
}

#[tokio::main]
async fn main() {
    let mut dom = VirtualDom::new(app);
    let mutations = dom.rebuild_to_vec();
    println!("initial: {mutations:?}");

    dom.handle_event("_click", Rc::new(0), ElementId(1), true);
    dom.process_events();

    let mutations = dom.render_immediate_to_vec();
    println!("update: {mutations:?}");
}
