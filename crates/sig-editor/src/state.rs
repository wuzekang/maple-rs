use crate::constants::DEFAULT_LEFT_WIDTH;
use crate::types::{SearchResult, WzTreeNode};
use sig::{Signal, TreeView};
use std::rc::Rc;
use wz_parser::WzNodeArc;

// Application State
pub struct AppState {
  pub root_node: Signal<Option<WzTreeNode>>,
  pub selected_node: Signal<Option<WzNodeArc>>, // Keep as WzNodeArc for compatibility with rendering
  pub left_width: Signal<f32>,
  pub error_message: Signal<Option<String>>,
  pub search_text: Signal<String>, // Search text for filtering

  // New fields for search feature
  pub tree_view: Signal<Option<Rc<TreeView<WzTreeNode>>>>,
  pub search_results: Signal<Vec<SearchResult>>,
  pub show_search_dropdown: Signal<bool>,
  pub selected_result_index: Signal<Option<usize>>,
  pub saved_open_states: Signal<std::collections::HashMap<u64, bool>>,
  pub location_success: Signal<Option<String>>,
}

impl AppState {
  pub fn new() -> Rc<Self> {
    Rc::new(Self {
      root_node: Signal::new(None),
      selected_node: Signal::new(None),
      left_width: Signal::new(DEFAULT_LEFT_WIDTH),
      error_message: Signal::new(None),
      search_text: Signal::new(String::new()),

      // Initialize new fields
      tree_view: Signal::new(None),
      search_results: Signal::new(Vec::new()),
      show_search_dropdown: Signal::new(false),
      selected_result_index: Signal::new(None),
      saved_open_states: Signal::new(std::collections::HashMap::new()),
      location_success: Signal::new(None),
    })
  }
}
