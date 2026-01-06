//! WZ Editor implemented with Sig framework
//!
//! Features:
//! - Left panel: Tree node browser with virtual scrolling
//! - Right panel: Node detail viewer (images, JSON, properties)
//! - Resizable panels
//! - Search functionality
//! - Async WZ file loading with use_resource to prevent UI blocking
//!
//! # Design Compliance
//!
//! This application uses the Sig design system tokens for consistent spacing,
//! sizing, and styling. See `sig/docs/DESIGN_GUIDELINES.md` for details.

mod components;
mod constants;
mod panels;
mod search;
mod state;
mod types;
mod utils;

use crate::constants::{BG_COLOR, ERROR_COLOR, PANEL_BG};
use crate::panels::{left::left_panel, right::right_panel};
use crate::search::setup_search_effect;
use crate::state::AppState;
use crate::types::WzTreeNode;
use sig::prelude::*;
use sig::render::{run, AppConfig};
use sig::theme::{Border, FontSize, Radius, Spacing};
use sig::{dynamic, text, Resizable};
use wz_parser::util::resolve_base;

/// Main application view
fn app_view() -> View {
  // Create portal container (must be at root level)
  let portal_root = sig::portal::provide();

  let state = AppState::new();

  // Use use_resource to load WZ file asynchronously
  let wz_file_path = "./Data/Base.wz";
  let root_node_signal = state.root_node.clone();
  let error_signal = state.error_message.clone();

  println!("📂 [Startup] Loading WZ file: {}", wz_file_path);

  let wz_resource = use_resource(move || {
    let path = wz_file_path.to_string();
    async move {
      println!("🔄 [Resource] Starting WZ file load...");
      let start = std::time::Instant::now();

      // This runs in a background task, not blocking the UI
      let result = match resolve_base(&path, None) {
        Ok(node) => {
          let elapsed = start.elapsed();
          println!("✅ [Resource] WZ file loaded successfully in {:?}", elapsed);
          Ok(node)
        }
        Err(e) => {
          let elapsed = start.elapsed();
          eprintln!(
            "❌ [Resource] Failed to load WZ file after {:?}: {}",
            elapsed, e
          );
          Err(format!("Failed to load WZ file: {}", e))
        }
      };

      result
    }
  });

  // Update state when resource is ready
  create_effect(move || {
    if wz_resource.ready() {
      println!("📝 [State] Resource ready, updating state...");
      if let Some(result) = wz_resource.value() {
        match result {
          Ok(node) => {
            println!("✅ [State] Root node set successfully");
            *root_node_signal.write() = Some(WzTreeNode(node));
            *error_signal.write() = None;
          }
          Err(err) => {
            eprintln!("❌ [State] Setting error: {}", err);
            *error_signal.write() = Some(err);
          }
        }
      }
    }
  });

  // Setup search effect
  println!("🔧 [Startup] Setting up search effect...");
  setup_search_effect(&state);

  let error_display = state.error_message.clone();

  view()
    .style(|s| s.w_full().h_full().flex().flex_col().background(BG_COLOR))
    .child((
      dynamic(move || {
        // Show error if any
        if let Some(error) = error_display.read().as_ref() {
          view()
            .style(|s| s.w_full().h_full().flex().items_center().justify_center())
            .child(
              view()
                .style(|s| {
                  s.flex()
                    .flex_col()
                    .gap(Spacing::MD)
                    .padding(Spacing::XL)
                    .background(PANEL_BG)
                    .border_radius(Radius::LG)
                    .border_all(Border::THIN, ERROR_COLOR)
                })
                .child((
                  text("Error Loading WZ File").style(|s| {
                    s.font_size(FontSize::LG)
                      .font_weight(600)
                      .color(ERROR_COLOR)
                  }),
                  text(error.clone()).style(|s| s.font_size(FontSize::SM).color(crate::constants::TEXT_SECONDARY)),
                )),
            )
        } else {
          view()
            .style(|s| s.w_full().h_full().flex().flex_row())
            .child((
              // Left resizable panel
              Resizable::horizontal(state.left_width.clone())
                .min_size(200.0)
                .max_size(600.0)
                .child(left_panel(&state)),
              // Right panel
              right_panel(&state),
            ))
        }
      }),
      portal_root,
    ))
}

fn main() -> anyhow::Result<()> {
  println!("🚀 ============================================");
  println!("🚀 WZ Editor Starting...");
  println!("🚀 ============================================");

  let result = run(
    AppConfig {
      title: "WZ Editor (Sig Framework)".to_string(),
      width: 1200.0,
      height: 800.0,
    },
    app_view,
  );

  println!("👋 WZ Editor shutting down");
  result
}
