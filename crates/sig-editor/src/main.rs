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

use clipboard_rs::{Clipboard, ClipboardContext};
use serde_json::{Map, Value};
use sig::prelude::*;
use sig::render::{run, AppConfig};
use sig::theme::{Border, FontSize, Radius, Size, Spacing};
use sig::{button, dynamic, Image, Signal, TreeNode, TreeView};
use std::rc::Rc;
use vello::peniko::Color;
use wz_parser::util::{node_util, resolve_base};
use wz_parser::{
  property::{png, WzSubProperty, WzValue},
  WzNode, WzNodeArc, WzObjectType,
};

// ============================================================================
// Constants (Using Design Tokens)
// ============================================================================

const DEFAULT_LEFT_WIDTH: f32 = 280.0;

// Colors (Consider moving to theme system in future)
const BG_COLOR: Color = Color::from_rgb8(250, 250, 250); // gray-50
const PANEL_BG: Color = Color::from_rgb8(255, 255, 255); // white
const PANEL_HEADER_BG: Color = Color::from_rgb8(248, 250, 252); // slate-50
const BORDER_COLOR: Color = Color::from_rgb8(226, 232, 240); // slate-200
const TEXT_PRIMARY: Color = Color::from_rgb8(30, 30, 30); // gray-900
const TEXT_SECONDARY: Color = Color::from_rgb8(100, 100, 100); // gray-500
const ERROR_COLOR: Color = Color::from_rgb8(220, 38, 38); // red-600
const SUCCESS_COLOR: Color = Color::from_rgb8(34, 197, 94); // green-500

// ============================================================================
// TreeNode Wrapper for WzNodeArc
// ============================================================================

/// Wrapper for WzNodeArc to implement TreeNode
#[derive(Clone)]
struct WzTreeNode(WzNodeArc);

impl PartialEq for WzTreeNode {
  fn eq(&self, other: &Self) -> bool {
    std::sync::Arc::ptr_eq(&self.0, &other.0)
  }
}

impl Eq for WzTreeNode {}

impl std::hash::Hash for WzTreeNode {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    // Hash using Arc pointer address
    (std::sync::Arc::as_ptr(&self.0) as usize).hash(state);
  }
}

impl TreeNode for WzTreeNode {
  fn label(&self) -> String {
    self.0.read().unwrap().name.to_string()
  }

  fn children(&self) -> Vec<Self> {
    self
      .0
      .read()
      .unwrap()
      .children
      .values()
      .map(|child| WzTreeNode(child.clone()))
      .collect()
  }

  fn has_children(&self) -> bool {
    let node = self.0.read().unwrap();
    !node.children.is_empty()
      || matches!(
        node.object_type,
        WzObjectType::Image(_) | WzObjectType::Directory(_)
      )
  }

  fn load_children(&self) -> Result<(), Box<dyn std::error::Error>> {
    node_util::parse_node(&self.0)?;
    Ok(())
  }
}

// ============================================================================
// Application State
// ============================================================================

struct AppState {
  root_node: Signal<Option<WzTreeNode>>,
  selected_node: Signal<Option<WzNodeArc>>, // Keep as WzNodeArc for compatibility with rendering
  left_width: Signal<f32>,
  error_message: Signal<Option<String>>,
  search_text: Signal<String>, // Search text for filtering
}

impl AppState {
  fn new() -> Rc<Self> {
    Rc::new(Self {
      root_node: Signal::new(None),
      selected_node: Signal::new(None),
      left_width: Signal::new(DEFAULT_LEFT_WIDTH),
      error_message: Signal::new(None),
      search_text: Signal::new(String::new()),
    })
  }
}

// ============================================================================
// UI Components
// ============================================================================

/// Left panel with tree view
fn left_panel(state: &Rc<AppState>) -> View {
  let root_signal = state.root_node.clone();
  let selected = state.selected_node.clone();
  let search_text = state.search_text.clone();

  view()
    .style(|s| {
      s.w_full()
        .h_full()
        .flex()
        .flex_col()
        .background(PANEL_BG)
        .border_right(Border::THIN) // Using design token
        .border_color(BORDER_COLOR)
    })
    .child((
      // Search box at the top (no linking logic with tree)
      view()
        .style(|s| {
          s.w_full()
            .padding(Spacing::MD) // 12px padding
            .border_bottom(Border::THIN)
            .border_color(BORDER_COLOR)
            .background(PANEL_HEADER_BG)
            .flex_shrink(0.0)
        })
        .child(
          sig::text_input()
            .placeholder("Search...")
            .value(search_text)
            .style(|s| {
              s.w_full()
                .height(Size::SM) // 32px
                .padding_left(Spacing::SM)
                .padding_right(Spacing::SM)
                .border_radius(Radius::MD) // 6px
                .border_all(Border::THIN, BORDER_COLOR)
                .background(PANEL_BG)
                .font_size(FontSize::SM)
                .color(TEXT_PRIMARY)
            }),
        ),
      // Tree view
      dynamic(move || {
        if let Some(root) = root_signal.read().as_ref() {
          let root_clone = root.clone();
          let selected_clone = selected.clone();

          // Create TreeView without search (removed .enable_search())
          let tree_view = TreeView::new(root_clone)
            .show_child_count(true)
            .item_height(Size::SM - 4.0); // 28px - slightly smaller than button height

          let tree_selected = tree_view.selected();

          // Sync tree selection with app state (unwrap WzTreeNode to WzNodeArc)
          sig::create_effect(move || {
            if let Some(node) = tree_selected.read().as_ref() {
              *selected_clone.write() = Some(node.0.clone());
            }
          });

          tree_view.build()
        } else {
          view()
            .style(|s| s.w_full().h_full().flex().items_center().justify_center())
            .child(
              text("Loading WZ file...").style(|s| s.font_size(FontSize::MD).color(TEXT_SECONDARY)),
            )
        }
      }),
    ))
}

/// Right panel with node details
fn right_panel(state: &Rc<AppState>) -> View {
  let selected_node = state.selected_node.clone();

  view()
    .style(|s| {
      s.flex_grow(1.0)
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .background(PANEL_BG)
        .min_width(0.0) // Prevent content from expanding beyond container
    })
    .child(dynamic(move || {
      if let Some(node) = selected_node.read().as_ref() {
        let node_data = node.read().unwrap();
        let full_path = node_data.get_full_path();
        let object_type = node_data.object_type.clone();
        drop(node_data);

        view()
          .style(|s| s.flex().flex_col().w_full().h_full())
          .child((
            // Header with path (fixed at top) - using design tokens
            view()
              .style(|s| {
                s.w_full()
                  .padding(Spacing::LG) // 16px - consistent padding
                  .border_bottom(Border::THIN) // 1px border
                  .border_color(BORDER_COLOR)
                  .background(PANEL_HEADER_BG)
                  .flex_shrink(0.0)
              })
              .child(
                text(full_path).style(|s| s.font_size(FontSize::MD).color(TEXT_PRIMARY)), // 14px
              ),
            // Content (scrollable) - using design tokens
            view()
              .style(|s| {
                s.w_full()
                  .flex_grow(1.0)
                  .min_height(0.0)
                  .min_width(0.0)
                  .overflow_y_scroll()
                  .padding(Spacing::LG) // 16px - consistent padding
              })
              .child(render_node_content(node.clone(), object_type)),
          ))
      } else {
        view()
          .style(|s| s.w_full().h_full().flex().items_center().justify_center())
          .child(
            text("Select a node to view details")
              .style(|s| s.font_size(FontSize::MD).color(TEXT_SECONDARY)), // 14px
          )
      }
    }))
}

// ============================================================================
// JSON Serialization Helpers
// ============================================================================

/// Recursively walk node tree and convert to JSON
fn walk_node_and_to_json(node_arc: &WzNodeArc, json: &mut Map<String, Value>) {
  // Parse node first
  let _ = node_util::parse_node(node_arc);

  let node = node_arc.read().unwrap();
  match &node.object_type {
    WzObjectType::Value(value_type) => {
      json.insert(node.name.to_string(), value_type.clone().into());
    }
    WzObjectType::Directory(_)
    | WzObjectType::Image(_)
    | WzObjectType::File(_)
    | WzObjectType::Property(_) => {
      let mut child_json = Map::new();
      if !node.children.is_empty() {
        for value in node.children.values() {
          walk_node_and_to_json(value, &mut child_json);
        }
        json.insert(node.name.to_string(), Value::Object(child_json));
      }
    }
  }
}

/// Convert node to formatted JSON string
fn node_to_json(node: &WzNode) -> String {
  let mut json = Map::new();

  for value in node.children.values() {
    walk_node_and_to_json(value, &mut json);
  }

  serde_json::to_string_pretty(&Value::Object(json)).unwrap_or_else(|_| "{}".to_string())
}

/// Render node content based on type
fn render_node_content(node: WzNodeArc, object_type: WzObjectType) -> View {
  match object_type {
    WzObjectType::Value(value) => {
      let text_content = match value {
        WzValue::Short(v) => format!("Short: {}", v),
        WzValue::Int(v) => format!("Int: {}", v),
        WzValue::Long(v) => format!("Long: {}", v),
        WzValue::Float(v) => format!("Float: {}", v),
        WzValue::Double(v) => format!("Double: {}", v),
        WzValue::String(v) => format!("String: {:?}", v.get_string().unwrap_or_default()),
        WzValue::ParsedString(v) => format!("String: {}", v),
        WzValue::Vector(v) => format!("Vector: x={}, y={}", v.0, v.1),
        WzValue::UOL(v) => format!("UOL: {:?}", v),
        WzValue::Null => "Null".to_string(),
        WzValue::RawData(_) => "Raw Data".to_string(),
        WzValue::Lua(_) => "Lua Script".to_string(),
      };

      view().child(
        text(text_content).style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px - slightly smaller for data
      )
    }
    WzObjectType::Property(prop) => match prop {
      WzSubProperty::PNG(png_prop) => {
        // Try to extract the image
        let format = png_prop.format();
        let width = png_prop.width;
        let height = png_prop.height;

        view()
          .style(|s| s.flex().flex_col().gap(Spacing::MD)) // 12px gap
          .child((
            // Info text
            view()
              .style(|s| s.flex().flex_col().gap(Spacing::XS)) // 4px gap
              .child((
                text(format!("PNG Image (Format: {})", format)).style(|s| {
                  s.font_size(FontSize::MD)
                    .font_weight(600)
                    .color(TEXT_PRIMARY)
                }), // 14px
                text(format!("Size: {}x{} pixels", width, height))
                  .style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)), // 12px
              )),
            // Try to render the image
            view().child(dynamic(move || {
              // Extract the PNG
              match png::get_image(&node) {
                Ok(dynamic_img) => {
                  // Convert to Image component
                  let img = Image::from_dynamic_image(dynamic_img.clone());

                  // Limit display size to reasonable dimensions
                  let display_width = width.min(600) as f32;
                  let display_height = height.min(600) as f32;

                  // Clone variables for the button handler
                  let node_for_button = node.clone();
                  let dynamic_img_for_button = dynamic_img.clone();

                  view()
                    .style(|s| s.flex().flex_col().gap(Spacing::MD)) // 12px gap
                    .child((
                      // Image display
                      view()
                        .style(|s| {
                          s.padding(Spacing::MD) // 12px padding
                            .background(PANEL_HEADER_BG)
                            .border_radius(Radius::LG) // 8px - container radius
                            .border_all(Border::THIN, BORDER_COLOR) // 1px border
                        })
                        .child(
                          img
                            .size(display_width, display_height)
                            .style(|s| s.border_radius(Radius::SM)), // 4px - inner radius
                        ),
                      // Copy button - using component defaults
                      button(format!("Copy to clipboard [{}]", format))
                        .style(|s| s.width(250.0))
                        .on_click(move |_| {
                          let node_data = node_for_button.read().unwrap();
                          let full_path = node_data.get_full_path();
                          drop(node_data);

                          // 生成文件名：将路径中的 "/" 替换为 "_"
                          let filename = std::env::temp_dir()
                            .join(format!("{}.png", full_path.replace("/", "_")));

                          // 保存 PNG 文件到 tmp 目录
                          let image_rgba = dynamic_img_for_button.clone().into_rgba8();
                          if let Ok(_) = image_rgba.save(&filename) {
                            // 使用 set_files 设置文件路径，这样粘贴时可以带上路径信息
                            if let Ok(clipboard) = ClipboardContext::new() {
                              let _ =
                                clipboard.set_files(vec![filename.to_string_lossy().to_string()]);
                              println!("Image copied to clipboard: {:?}", filename);
                            } else {
                              eprintln!("Failed to create clipboard context");
                            }
                          } else {
                            eprintln!("Failed to save image to temp file");
                          }
                        }),
                    ))
                }
                Err(e) => {
                  view().child(
                    text(format!("Failed to load image: {}", e))
                      .style(|s| s.font_size(FontSize::SM).color(ERROR_COLOR)), // 12px red
                  )
                }
              }
            })),
          ))
      }
      WzSubProperty::Sound(sound_prop) => {
        view()
          .style(|s| s.flex().flex_col().gap(Spacing::SM)) // 8px gap
          .child((
            text(format!("Sound: {:?}", sound_prop.sound_type))
              .style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
            text(format!("Duration: {} ms", sound_prop.duration))
              .style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)), // 12px
            text("Audio playback not yet implemented")
              .style(|s| s.font_size(FontSize::XS).color(TEXT_SECONDARY)), // 11px
          ))
      }
      WzSubProperty::Property => {
        // For Property type, show JSON representation
        let node_data = node.read().unwrap();

        // Only show JSON if it has children
        if node_data.children.is_empty() {
          view().child(
            text("Property Container (Empty)")
              .style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)), // 12px
          )
        } else {
          let json_str = node_to_json(&node_data);
          drop(node_data);

          view()
            .style(|s| {
              s.flex()
                .flex_col()
                .gap(Spacing::SM)
                .w_full() // 确保容器填充父元素宽度
                .min_width(0.0)
            }) // 8px gap
            .child((
              text("Property Container (JSON)").style(|s| {
                s.font_size(FontSize::MD)
                  .font_weight(600)
                  .color(TEXT_PRIMARY)
              }), // 14px
              view()
                .style(|s| {
                  s.padding(Spacing::MD) // 12px padding
                    .background(PANEL_HEADER_BG)
                    .border_radius(Radius::LG) // 8px
                    .border_all(Border::THIN, BORDER_COLOR) // 1px
                    .overflow_y_scroll()
                    .overflow_x_scroll() // Allow horizontal scrolling for long lines
                    .max_height(400.0)
                    .flex() // Use flex layout
                    .w_full()
                    .min_width(0.0) // Allow container to shrink
                })
                .child(text(json_str).style(|s| {
                  s.font_size(FontSize::XS) // 11px for code
                    .color(TEXT_PRIMARY)
                    .line_height(1.5)
                    .flex_grow(1.0) // Make text fill available space
                    .min_width(0.0) // Allow text to shrink
                })),
            ))
        }
      }
      WzSubProperty::Convex => {
        view().child(
          text("Convex Property").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
        )
      }
    },
    WzObjectType::Directory(_) => {
      view().child(
        text("Directory").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
      )
    }
    WzObjectType::Image(_) => {
      view().child(
        text("Image Container").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
      )
    }
    WzObjectType::File(_) => {
      view().child(
        text("File").style(|s| s.font_size(FontSize::SM).color(TEXT_PRIMARY)), // 12px
      )
    }
  }
}

/// Main application view
fn app_view() -> View {
  let state = AppState::new();

  // Use use_resource to load WZ file asynchronously
  let wz_file_path = "./Data/Base.wz";
  let root_node_signal = state.root_node.clone();
  let error_signal = state.error_message.clone();

  let wz_resource = use_resource(move || {
    let path = wz_file_path.to_string();
    async move {
      // This runs in a background task, not blocking the UI
      match resolve_base(&path, None) {
        Ok(node) => Ok(node),
        Err(e) => Err(format!("Failed to load WZ file: {}", e)),
      }
    }
  });

  // Update state when resource is ready
  create_effect(move || {
    if wz_resource.ready() {
      if let Some(result) = wz_resource.value() {
        match result {
          Ok(node) => {
            *root_node_signal.write() = Some(WzTreeNode(node));
            *error_signal.write() = None;
          }
          Err(err) => {
            *error_signal.write() = Some(err);
          }
        }
      }
    }
  });

  let error_display = state.error_message.clone();

  view()
    .style(|s| s.w_full().h_full().flex().flex_col().background(BG_COLOR))
    .child(dynamic(move || {
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
                text(error.clone()).style(|s| s.font_size(FontSize::SM).color(TEXT_SECONDARY)),
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
    }))
}

fn main() -> anyhow::Result<()> {
  run(
    AppConfig {
      title: "WZ Editor (Sig Framework)".to_string(),
      width: 1200.0,
      height: 800.0,
    },
    app_view,
  )
}
