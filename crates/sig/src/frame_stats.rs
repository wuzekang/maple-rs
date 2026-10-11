//! Per-frame segment timing for the render pipeline.
//!
//! Enable with `SIG_FRAME_STATS=1`; every rendered frame records how long
//! each pipeline segment took (style compute, layout, scene build, GPU
//! raster, present). `summary()` returns aggregate percentiles.

use std::cell::RefCell;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameSegments {
  pub style: Duration,
  pub layout: Duration,
  pub scene: Duration,
  pub raster: Duration,
  pub present: Duration,
}

thread_local! {
  static FRAMES: RefCell<Vec<FrameSegments>> = const { RefCell::new(Vec::new()) };
  static LAST_FRAME_AT: RefCell<Option<std::time::Instant>> = const { RefCell::new(None) };
  static ENABLED: bool = {
    matches!(
      std::env::var("SIG_FRAME_STATS").as_deref(),
      Ok("1") | Ok("true") | Ok("yes")
    )
  };
}

pub fn enabled() -> bool {
  ENABLED.with(|e| *e)
}

pub fn record(segments: FrameSegments) {
  let now = std::time::Instant::now();
  let gap = LAST_FRAME_AT.with(|l| l.borrow().map(|t| now - t));
  if let Some(g) = gap {
    GAPS.with(|gs| gs.borrow_mut().push(g));
  }
  LAST_FRAME_AT.with(|l| *l.borrow_mut() = Some(now));
  FRAMES.with(|f| f.borrow_mut().push(segments));
}

thread_local! {
  static GAPS: RefCell<Vec<Duration>> = const { RefCell::new(Vec::new()) };
}

/// Inter-frame gap histogram (frame pacing diagnostics).
pub fn gap_summary() -> String {
  let gaps: Vec<Duration> = GAPS.with(|g| g.borrow().clone());
  if gaps.is_empty() {
    return String::from("gaps: none\n");
  }
  let mut out = format!(
    "gaps: {} frames, {:<10} {:>10}\n",
    gaps.len(),
    "bucket",
    "count"
  );
  let mut buckets = std::collections::BTreeMap::new();
  for g in &gaps {
    let ms = g.as_millis();
    let bucket = match ms {
      0..=2 => "<2ms (burst)",
      3..=6 => "3-6ms",
      7..=11 => "7-11ms",
      12..=20 => "12-20ms (60Hz)",
      21..=34 => "21-34ms",
      _ => "35ms+",
    };
    *buckets.entry(bucket).or_insert(0usize) += 1;
  }
  for (k, v) in buckets {
    out.push_str(&format!("  {:<15} {:>5}\n", k, v));
  }
  out
}

pub fn frame_count() -> usize {
  FRAMES.with(|f| f.borrow().len())
}

fn percentiles(mut samples: Vec<Duration>) -> (Duration, Duration, Duration, Duration) {
  if samples.is_empty() {
    return (Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO);
  }
  samples.sort();
  let pick = |q: f64| -> Duration {
    let idx = ((samples.len() as f64 - 1.0) * q).round() as usize;
    samples[idx.min(samples.len() - 1)]
  };
  let avg = samples.iter().sum::<Duration>() / samples.len() as u32;
  (pick(0.5), pick(0.95), *samples.last().unwrap(), avg)
}

/// Human-readable aggregate over all recorded frames.
pub fn summary() -> String {
  let frames: Vec<FrameSegments> = FRAMES.with(|f| f.borrow().clone());
  if frames.is_empty() {
    return String::from("frame_stats: no frames recorded\n");
  }

  let mut out = format!(
    "frame_stats: {} frames\n  {:>10}  {:>10}  {:>10}  {:>10}\n",
    frames.len(), "p50", "p95", "max", "avg"
  );

  let segments: [(&str, Box<dyn Fn(&FrameSegments) -> Duration>); 5] = [
    ("style", Box::new(|f| f.style)),
    ("layout", Box::new(|f| f.layout)),
    ("scene", Box::new(|f| f.scene)),
    ("raster", Box::new(|f| f.raster)),
    ("present", Box::new(|f| f.present)),
  ];

  for (name, get) in segments {
    let (p50, p95, max, avg) = percentiles(frames.iter().map(|f| get(f)).collect());
    out.push_str(&format!(
      "  {:<10} {:>9.1?}  {:>9.1?}  {:>9.1?}  {:>9.1?}\n",
      name, p50, p95, max, avg
    ));
  }

  let totals: Vec<Duration> = frames
    .iter()
    .map(|f| f.style + f.layout + f.scene + f.raster + f.present)
    .collect();
  let (p50, p95, max, avg) = percentiles(totals);
  out.push_str(&format!(
    "  {:<10} {:>9.1?}  {:>9.1?}  {:>9.1?}  {:>9.1?}\n",
    "total", p50, p95, max, avg
  ));
  out
}
