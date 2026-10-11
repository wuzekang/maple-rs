//! Headless CPU benchmark for the style pipeline (no window, no GPU).
//!
//! Times the three CPU segments of a frame — reactive effect re-runs,
//! style computation, layout, scene encoding — across scenarios chosen
//! to bracket the known optimization targets:
//!
//!   clean          no change at all              → the O(tree) floor of the walk
//!   one_paint      one row recolors (paint-only)  → pruning quality
//!   all_paint      every row recolors (paint-only)
//!   one_layout     one row height changes
//!   all_layout     every row height changes
//!
//! Run:
//!   cargo run --release -p sig --example style_bench -- [rows] [iterations]

use sig::prelude::*;
use std::time::{Duration, Instant};
use vello::peniko::Color;

#[derive(Clone, Copy, PartialEq)]
enum Mutation {
  Clean,
  OnePaint,
  AllPaint,
  OneLayout,
  AllLayout,
}

const SEGMENTS: [&str; 4] = ["effects", "style", "layout", "scene"];

struct Timings {
  segments: [Vec<Duration>; 4],
}

impl Timings {
  fn new() -> Self {
    Self {
      segments: std::array::from_fn(|_| Vec::new()),
    }
  }

  fn report(&self, name: &str) {
    eprint!("  {:<12}", name);
    for seg in &self.segments {
      let mut samples: Vec<u128> = seg.iter().map(|d| d.as_micros()).collect();
      samples.sort_unstable();
      let p50 = samples[samples.len() / 2];
      let p95 = samples[(samples.len() * 95) / 100];
      eprint!("  {:>5}/{:>5}µs", p50, p95);
    }
    eprintln!();
  }
}

fn build_tree(rows: usize, layout_mode: bool) -> (sig::ViewId, Vec<sig::Signal<bool>>) {
  let signals: Vec<sig::Signal<bool>> = (0..rows)
    .map(|_| sig::Signal::new(false))
    .collect();

  let row_views: Vec<_> = (0..rows)
    .map(|i| {
      let s = signals[i];
      view()
        .style(move |st| {
          let on = *s.read();
          if layout_mode {
            st.height(if on { 34.0 } else { 28.0 }).w_full()
          } else {
            st.height(28.0).w_full().background(if on {
              Color::from_rgb8(241, 245, 249)
            } else {
              Color::from_rgb8(248, 250, 252)
            })
          }
        })
        .child(text(format!("row {i:04}")).style(|s| s.font_size(13.0)))
    })
    .collect();

  let root = view()
    .style(|s| s.width(1200.0).height(800.0).flex().flex_col())
    .child((
      view().style(|s| s.height(48.0).w_full().flex_shrink(0.0)),
      view()
        .style(|s| s.flex_grow(1.0).w_full().flex_col().overflow_y_scroll())
        .child(row_views),
    ));

  (root.id, signals)
}

fn run_scenario(name: &str, rows: usize, iters: usize, mutation: Mutation) {
  let layout_mode = matches!(mutation, Mutation::OneLayout | Mutation::AllLayout);
  let (root, signals) = build_tree(rows, layout_mode);

  let render_ctx = sig::RenderContext::new(1.0, (1200.0, 800.0));
  let available_space = taffy::Size {
    width: taffy::AvailableSpace::Definite(1200.0),
    height: taffy::AvailableSpace::Definite(800.0),
  };
  let mut scene = vello::Scene::new();

  let mut timings = Timings::new();

  for it in 0..(20 + iters) {
    let warmup = it < 20;
    let toggle = it % 2 == 0;

    // Segment 1: reactive effect re-runs caused by the signal writes.
    let t0 = Instant::now();
    match mutation {
      Mutation::Clean => {}
      Mutation::OnePaint | Mutation::OneLayout => *signals[0].write() = toggle,
      Mutation::AllPaint | Mutation::AllLayout => {
        for s in &signals {
          *s.write() = toggle;
        }
      }
    }
    let effects = t0.elapsed();

    // Segment 2: style pass (exactly what the render loop does pre-layout).
    let t1 = Instant::now();
    sig::style::compute::batch_bubble_dirty_marks();
    let mut style_ctx = sig::style::compute::StyleComputeContext::new();
    sig::style::compute::compute_style_recursive(&root, &mut style_ctx);
    let style = t1.elapsed();

    // Segment 3: layout.
    let t2 = Instant::now();
    root.compute_layout(available_space, &render_ctx);
    let layout = t2.elapsed();

    // Segment 4: scene encoding (CPU side of rendering).
    let t3 = Instant::now();
    scene.reset();
    sig::render::render_view(&mut scene, root, 0.0, 0.0, &render_ctx, None);
    let scene_t = t3.elapsed();

    if !warmup {
      timings.segments[0].push(effects);
      timings.segments[1].push(style);
      timings.segments[2].push(layout);
      timings.segments[3].push(scene_t);
    }
  }

  timings.report(name);
}

fn main() {
  let rows: usize = std::env::args()
    .nth(1)
    .and_then(|s| s.parse().ok())
    .unwrap_or(1000);
  let iters: usize = std::env::args()
    .nth(2)
    .and_then(|s| s.parse().ok())
    .unwrap_or(200);

  eprintln!("style_bench: rows={rows} iterations={iters} (p50/p95 per segment)");
  eprint!("  {:<12}", "scenario");
  for seg in SEGMENTS {
    eprint!("  {:>13}", seg);
  }
  eprintln!();

  run_scenario("clean", rows, iters, Mutation::Clean);
  run_scenario("one_paint", rows, iters, Mutation::OnePaint);
  run_scenario("all_paint", rows, iters, Mutation::AllPaint);
  run_scenario("one_layout", rows, iters, Mutation::OneLayout);
  run_scenario("all_layout", rows, iters, Mutation::AllLayout);
}
