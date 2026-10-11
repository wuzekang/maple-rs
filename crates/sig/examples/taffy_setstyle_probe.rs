//! Probe: does taffy's set_style mark layout dirty even when the style is
//! bit-identical? Isolates taffy semantics from sig's style pipeline.
//!
//! cargo run --release -p sig --example taffy_setstyle_probe

use taffy::prelude::*;

fn main() {
  let mut tree = taffy::TaffyTree::<()>::new();

  // root -> container -> 1000 fixed-height rows (mirrors style_bench's tree)
  let mut rows = Vec::new();
  for _ in 0..1000 {
    let id = tree.new_leaf(taffy::Style {
      size: taffy::Size { width: length(100.0), height: length(28.0) },
      ..Default::default()
    }).unwrap();
    rows.push(id);
  }
  let container = tree
    .new_with_children(
      taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        ..Default::default()
      },
      &rows,
    )
    .unwrap();
  let root = tree
    .new_with_children(
      taffy::Style::default(),
      &[container],
    )
    .unwrap();

  let space = taffy::Size {
    width: AvailableSpace::Definite(1200.0),
    height: AvailableSpace::Definite(800.0),
  };

  fn time_compute(tree: &mut taffy::TaffyTree<()>, root: taffy::NodeId, space: taffy::Size<AvailableSpace>, label: &str) {
    let t = std::time::Instant::now();
    tree.compute_layout(root, space).unwrap();
    eprintln!("{:<38} {:?}", label, t.elapsed());
  }

  time_compute(&mut tree, root, space, "1st compute (cold)");
  time_compute(&mut tree, root, space, "2nd compute (cached baseline)");

  // set_style with an IDENTICAL style object
  let s = tree.style(rows[500]).unwrap();
  tree.set_style(rows[500], s.clone()).unwrap();
  time_compute(&mut tree, root, space, "after set_style(SAME style)");

  // set_style with a genuinely different style
  tree
    .set_style(
      rows[500],
      taffy::Style {
        size: taffy::Size { width: length(100.0), height: length(34.0) },
        ..Default::default()
      },
    )
    .unwrap();
  time_compute(&mut tree, root, space, "after set_style(DIFFERENT style)");

  time_compute(&mut tree, root, space, "final compute (cached again)");
}
