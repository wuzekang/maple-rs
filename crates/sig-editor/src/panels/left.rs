use crate::components::search_box::search_box_with_dropdown;
use crate::constants::{BORDER_COLOR, PANEL_BG, TEXT_SECONDARY};
use crate::state::AppState;
use sig::prelude::*;
use sig::theme::{Border, FontSize, Size};
use sig::{dynamic, text, TreeView};
use std::rc::Rc;

/// Left panel with tree view
pub fn left_panel(state: &Rc<AppState>) -> View {
  let root_signal = state.root_node.clone();
  let selected = state.selected_node.clone();
  let tree_view_ref = state.tree_view.clone();

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
      // Search box with popper dropdown
      search_box_with_dropdown(state.clone()),
      // Tree view
      dynamic(move || {
        if let Some(root) = root_signal.read().as_ref() {
          let root_clone = root.clone();
          let selected_clone = selected.clone();

          // Create TreeView
          let tree_view = Rc::new(
            TreeView::new(root_clone)
              .show_child_count(true)
              .item_height(Size::SM - 4.0), // 28px - slightly smaller than button height
          );

          // Store TreeView reference
          *tree_view_ref.write() = Some(tree_view.clone());

          let tree_selected = tree_view.selected();

          // Sync tree selection with app state (unwrap WzTreeNode to WzNodeArc)
          sig::create_effect(move || {
            if let Some(node) = tree_selected.read().as_ref() {
              *selected_clone.write() = Some(node.0.clone());
            }
          });

          // Clone the TreeView and call build (since build() takes self)
          (*tree_view).clone().build()
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
