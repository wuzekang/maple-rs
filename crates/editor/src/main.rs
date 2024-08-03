use clipboard_rs::common::RustImage;
use clipboard_rs::Clipboard;
use eframe::egui;
use egui::Color32;
use egui::{collapsing_header::CollapsingState, RichText, Ui};
use image::DynamicImage;
use image::EncodableLayout;
use serde_json::{Map, Value};
use std::fs::read;
use wz_reader::util::resolve_base;
use wz_reader::{
    property::{WzSubProperty, WzValue},
    util::node_util::parse_node,
    WzNodeArc, WzObjectType,
};
use wz_reader::{util::node_util, WzNode};

use eframe::{
    egui::{Context, FontData, FontDefinitions},
    epaint::FontFamily,
};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};

fn load_system_font(ctx: &Context) {
    let mut fonts = FontDefinitions::empty();

    const FONT_NAME: &'static str = "PingFang SC";

    let handle = SystemSource::new()
        .select_best_match(
            &[FamilyName::Title(FONT_NAME.to_string())],
            &Properties::new(),
        )
        .unwrap();

    let buf: Vec<u8> = match handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => read(path).unwrap(),
    };

    fonts
        .font_data
        .insert(FONT_NAME.to_owned(), FontData::from_owned(buf));

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push(FONT_NAME.to_owned());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(FONT_NAME.to_owned());
    }

    ctx.set_fonts(fonts);
}

fn walk_node_and_to_json(node_arc: &WzNodeArc, json: &mut Map<String, Value>) {
    node_util::parse_node(node_arc).unwrap();
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
            if node.children.len() != 0 {
                for value in node.children.values() {
                    walk_node_and_to_json(value, &mut child_json);
                }
                json.insert(node.name.to_string(), Value::Object(child_json));
            }
        }
    }
}

fn to_json(node: &WzNode) -> String {
    let mut json = Map::new();

    for value in node.children.values() {
        walk_node_and_to_json(value, &mut json);
    }

    let json_string = serde_json::to_string_pretty(&Value::Object(json)).unwrap();
    json_string
}

#[derive(Default)]
struct Tree {
    selected: Option<WzNodeArc>,
    search: String,
}

impl Tree {
    pub fn ui(&mut self, ui: &mut Ui, node: &WzNodeArc) {
        self.children_ui(ui, node)
    }
}

impl Tree {
    fn ui_impl(&mut self, ui: &mut Ui, name: &str, node: &WzNodeArc) {
        if name.ends_with(".img") || node.read().unwrap().children.len() > 0 {
            let id = ui.make_persistent_id(
                "my_collapsing_header".to_string() + &node.read().unwrap().get_full_path(),
            );
            let (response, _, _) = CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    let mut text = RichText::new(name);
                    if self.search.len() > 0 && name.contains(&self.search) {
                        text = text.color(Color32::RED)
                    }
                    if ui.button(text).clicked() {
                        self.selected = Some(node.clone());
                    }
                })
                .body(|ui| self.children_ui(ui, node));
            if response.clicked() {
                parse_node(node).unwrap();
            }
        } else {
            if ui.button(name).clicked() {
                self.selected = Some(node.clone());
            };
        }
    }

    fn children_ui(&mut self, ui: &mut Ui, node: &WzNodeArc) {
        let binding = node.read().unwrap();
        let children = binding.children.iter().collect::<Vec<_>>();
        for (name, node) in children {
            self.ui_impl(ui, name.as_str(), &node);
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Maple RS",
        options,
        Box::new(|cc| {
            load_system_font(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    tree: Tree,
    node: WzNodeArc,
    clipboard: clipboard_rs::ClipboardContext,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            node: resolve_base("./Data/Base.wz", None).unwrap(),
            tree: Tree::default(),
            clipboard: clipboard_rs::ClipboardContext::new().unwrap(),
        }
    }
}

fn type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.tree.search);
                egui::ScrollArea::vertical().show(ui, |ui| self.tree.ui(ui, &self.node));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(node) = &self.tree.selected {
                    let node = node.read().unwrap();

                    ui.label(RichText::new(node.get_full_path()));
                    ui.separator();

                    match &node.object_type {
                        WzObjectType::File(v) => {}
                        WzObjectType::Image(_) => {}
                        WzObjectType::Directory(_) => {}
                        WzObjectType::Property(v) => match v {
                            WzSubProperty::Convex => {}
                            WzSubProperty::Sound(_) => {}
                            WzSubProperty::PNG(v) => {
                                let image = v.extract_png().unwrap();
                                let color_image = match image.clone() {
                                    DynamicImage::ImageRgb8(image) => {
                                        // common case optimization
                                        egui::ColorImage::from_rgb(
                                            [image.width() as usize, image.height() as usize],
                                            image.as_bytes(),
                                        )
                                    }
                                    other => {
                                        let image = other.to_rgba8();
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            [image.width() as usize, image.height() as usize],
                                            image.as_bytes(),
                                        )
                                    }
                                };
                                let ctx = ui.ctx();
                                let handle =
                                    ctx.load_texture("tile", color_image, Default::default());
                                ui.image(&handle);
                                ui.add_space(8.0);
                                if ui
                                    .button(&format!("Copy to clipboard [{}]", v.format()))
                                    .clicked()
                                {
                                    let _ = self.clipboard.set_image(
                                        clipboard_rs::RustImageData::from_dynamic_image(
                                            image.clone(),
                                        ),
                                    );
                                }
                            }
                            WzSubProperty::Property => {
                                ui.label(RichText::new(to_json(&node)));
                            }
                        },
                        WzObjectType::Value(v) => match v {
                            WzValue::RawData(v) => {
                                ui.label(RichText::new(format!("{:?} {}", v, type_of(v))));
                            }
                            WzValue::Lua(v) => {
                                ui.label(RichText::new(format!("{:?} {}", v, type_of(v))));
                            }
                            WzValue::Short(v) => {
                                ui.label(RichText::new(format!("{:?}{}", v, type_of(v))));
                            }
                            WzValue::Int(v) => {
                                ui.label(RichText::new(format!("{:?}{}", v, type_of(v))));
                            }
                            WzValue::Long(v) => {
                                ui.label(RichText::new(format!("{:?}{}", v, type_of(v))));
                            }
                            WzValue::Float(v) => {
                                ui.label(RichText::new(format!("{:?}{}", v, type_of(v))));
                            }
                            WzValue::Double(v) => {
                                ui.label(RichText::new(format!("{:?}{}", v, type_of(v))));
                            }
                            WzValue::Vector(v) => {
                                ui.label(RichText::new(format!("{:?}", v)));
                            }
                            WzValue::UOL(v) => {
                                ui.label(RichText::new(format!("{:?}", v)));
                            }
                            WzValue::String(v) => {
                                let v = &v.get_string().unwrap();
                                ui.label(RichText::new(format!("{:?}", v)));
                            }
                            WzValue::ParsedString(v) => {
                                ui.label(RichText::new(format!("{:?}", v)));
                            }
                            WzValue::Null => {
                                ui.label("null");
                            }
                        },
                    }
                }
            })
        });
    }
}
