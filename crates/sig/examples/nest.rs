use sig::render::{AppConfig, run};
use sig::{dynamic, fragment, prelude::*};
use std::time::Instant;
use wz_parser::util::{node_util, resolve_base};
use wz_parser::WzNodeArc;

fn node_view(node: &WzNodeArc, depth: usize) -> View {
  let open = Signal::new(false);
  let node = Signal::new(node.clone());
  view()
    .style(move |s| s.flex_col().margin_left(depth * 8))
    .child(
      view()
        // .style(|s| s.bg_black())
        .child(dynamic(move || {
          text(if *open.read() { "[+]" } else { "[-]" })
        }))
        .child(text(format!(
          "{} ({})",
          node.read_untracked().read().unwrap().name.to_string(),
          node.read_untracked().read().unwrap().children.len()
        )))
        .on_click(move |_| {
          let start = Instant::now();

          let value = { !*open.read_untracked() };
          if value {
            let parse_start = Instant::now();
            node_util::parse_node(&node.read_untracked()).unwrap();
            println!("parse_node time: {:?}", parse_start.elapsed());
          }

          *open.write() = value;

          println!(
            "click {:?}, total time: {:?}",
            node.read_untracked().read().unwrap().name,
            start.elapsed()
          );
        }),
    )
    .child(dynamic(move || {
      if !*open.read() {
        fragment(())
      } else {
        fragment(
          node
            .read_untracked()
            .read()
            .unwrap()
            .children
            .iter()
            .map(|(_, node)| node_view(node, depth + 1))
            .collect::<Vec<_>>(),
        )
      }
    }))
}

fn main() -> anyhow::Result<()> {
  run(AppConfig::default(), || {
    let node = resolve_base("./data/Base.wz", None).unwrap();

    view()
      .style(|s| s.w_full().h_full().overflow_y_scroll())
      .child(node_view(&node, 0))
  })
}
