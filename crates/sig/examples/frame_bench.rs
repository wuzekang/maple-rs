//! Frame pipeline benchmark.
//!
//! Builds a text-heavy view tree and drives continuous damage-driven frames by
//! flipping a signal on a ~16ms cadence. Exits after the requested frame count
//! and prints per-segment pipeline timings to stderr.
//!
//! Run:
//!   SIG_FRAME_STATS=1 cargo run --release -p sig --example frame_bench -- [rows] [frames]

use sig::prelude::*;
use vello::peniko::Color;

fn main() -> anyhow::Result<()> {
  let rows: usize = std::env::args()
    .nth(1)
    .and_then(|s| s.parse().ok())
    .unwrap_or(300);
  let flips: usize = std::env::args()
    .nth(2)
    .and_then(|s| s.parse().ok())
    .unwrap_or(300);

  eprintln!("frame_bench: rows={rows} flips={flips}");

  run(AppConfig::default(), move || {
    let tick = Signal::new(0usize);

    // Drive damage every ~16ms; each flip re-styles the rows (color/badge
    // alternate with parity) so every frame pays style+layout+scene+raster.
    let driver = tick.clone();
    sig::reactive::spawn(async move {
      for i in 1..=flips {
        *driver.write() = i;
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        if i % 100 == 0 {
          eprintln!("flip {i}: frames={}", sig::frame_stats::frame_count());
        }
      }
      // Let the last frame land, then report and exit.
      tokio::time::sleep(std::time::Duration::from_millis(120)).await;
      eprint!("{}{}redraw requests: {}\n", sig::frame_stats::summary(), sig::frame_stats::gap_summary(), sig::runtime::redraw_request_count());
      std::process::exit(0);
    });

    let rows_view: Vec<_> = (0..rows)
      .map(|i| {
        let tick = tick.clone();
        view()
          .style(move |s| {
            // Tracked read: every flip marks every row dirty → re-style,
            // re-layout, full scene rebuild each frame.
            let t = *tick.read();
            let parity = t % 2 == 0;
            s.w_full()
              .height(28.0)
              .flex()
              .items_center()
              .padding_left(12.0).padding_right(12.0)
              .background(if (i % 2 == 0) == parity {
                Color::from_rgb8(248, 250, 252)
              } else {
                Color::from_rgb8(241, 245, 249)
              })
          })
          .child(
            text(format!("row {i:04} — every frame restyles this list"))
              .style(|s| s.font_size(13.0).color(Color::from_rgb8(51, 65, 85))),
          )
      })
      .collect();

    view()
      .style(|s| s.size_full().flex().flex_col().background(Color::WHITE))
      .child((
        view()
          .style(|s| {
            s.height(48.0)
              .background(Color::from_rgb8(15, 23, 42))
              .flex()
              .items_center()
              .padding_left(16.0).padding_right(16.0)
              .w_full()
          })
          .child(
            text("frame bench").style(|s| s.font_size(18.0).color(Color::WHITE)),
          ),
        // Scrollable body so the tree is bigger than the viewport.
        view()
          .style(|s| {
            s.flex_grow(1.0)
              .w_full()
              .flex_col()
              .overflow_y_scroll()
          })
          .child(rows_view),
        // Live counter proves reactive updates land every flip.
        {
          let tick = tick.clone();
          view().style(|s| s.height(32.0).flex().items_center().w_full()).child(
            dynamic_text(move || format!("ticks: {}", *tick.read()))
              .style(|s| s.font_size(12.0).color(Color::from_rgb8(100, 116, 139))),
          )
        },
      ))
  })
}
