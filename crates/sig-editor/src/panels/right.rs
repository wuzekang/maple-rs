use crate::constants::{
  BORDER_COLOR, ERROR_COLOR, PANEL_BG, PANEL_HEADER_BG, SUCCESS_COLOR, TEXT_PRIMARY, TEXT_SECONDARY,
};
use crate::state::AppState;
use crate::utils::node_to_json;
use clipboard_rs::{Clipboard, ClipboardContext};
use sig::prelude::*;
use sig::theme::{Border, FontSize, Radius, Spacing};
use sig::{button, dynamic, text, Image};
use std::rc::Rc;
use wz_parser::{
  property::{png, WzSubProperty, WzValue},
  WzNodeArc, WzObjectType,
};

/// Right panel with node details
pub fn right_panel(state: &Rc<AppState>) -> View {
  let selected_node = state.selected_node.clone();
  let location_success = state.location_success.clone();

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
    .child((
      // Location success message (if any)
      dynamic(move || {
        if let Some(success_msg) = location_success.read().as_ref() {
          view()
            .style(|s| {
              s.w_full()
                .padding(Spacing::SM)
                .background(SUCCESS_COLOR.with_alpha(0.1))
                .border_bottom(Border::THIN)
                .border_color(SUCCESS_COLOR)
                .flex_shrink(0.0)
            })
            .child(
              text(success_msg.clone()).style(|s| s.font_size(FontSize::SM).color(SUCCESS_COLOR)),
            )
        } else {
          view().style(|s| s.width(0.0).height(0.0))
        }
      }),
      // Existing content area
      dynamic(move || {
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
      }),
    ))
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
