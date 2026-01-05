//! TreeView component for rendering hierarchical tree structures
//!
//! Features:
//! - Lazy loading support
//! - Virtual scrolling for performance
//! - Search/filter functionality
//! - Customizable rendering
//!
//! # Example
//!
//! ```ignore
//! use sig::components::tree_view::{TreeView, TreeNode};
//!
//! // Simple usage with default styling
//! TreeView::new(root_node)
//!     .enable_search()
//!     .show_child_count(true)
//!     .build()
//!
//! // Custom rendering
//! TreeView::new(root_node)
//!     .build_with(|node, ctx| {
//!         view()
//!             .child(text(node.label()))
//!             .style(move |s| s.padding_left(ctx.depth as f32 * 20.0))
//!     })
//! ```

use crate::prelude::*;
use crate::{create_effect, dynamic, Signal, VirtualList};
use std::collections::HashMap;
use std::rc::Rc;
use vello::peniko::Color;

// ============================================================================
// Trait Definition
// ============================================================================

/// Trait for tree node data
///
/// Requirements:
/// - `Clone`: Must be cheaply cloneable (e.g., Arc, Rc)
/// - `PartialEq + Eq + Hash`: For node identification
/// - `'static`: Required for signal system
pub trait TreeNode: Clone + PartialEq + Eq + std::hash::Hash + 'static {
    /// Display label for the node
    fn label(&self) -> String;
    
    /// Get child nodes (may trigger parsing)
    fn children(&self) -> Vec<Self>;
    
    /// Check if node has children without loading them
    fn has_children(&self) -> bool {
        !self.children().is_empty()
    }
    
    /// Lazy load children (optional override)
    fn load_children(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// TreeView configuration
#[derive(Clone, Debug)]
pub struct TreeViewConfig {
    /// Height of each item in pixels
    pub item_height: f32,
    /// Indentation width per level
    pub indent_width: f32,
    /// Buffer size for virtual scrolling
    pub buffer_size: usize,
    /// Show expand/collapse icons
    pub show_expand_icon: bool,
    /// Show child count in labels
    pub show_child_count: bool,
    /// Enable search functionality
    pub enable_search: bool,
    /// Enable virtual scrolling
    pub enable_virtual_scroll: bool,
}

impl Default for TreeViewConfig {
    fn default() -> Self {
        Self {
            item_height: 28.0,
            indent_width: 16.0,
            buffer_size: 10,
            show_expand_icon: true,
            show_child_count: false,
            enable_search: false,
            enable_virtual_scroll: true,
        }
    }
}

/// Default colors (can be customized via TreeViewStyle)
#[derive(Clone, Debug)]
pub struct TreeViewStyle {
    pub text_color: Color,
    pub text_secondary: Color,
    pub selected_color: Color,
    pub hover_bg: Color,
    pub border_color: Color,
}

impl Default for TreeViewStyle {
    fn default() -> Self {
        Self {
            text_color: Color::from_rgb8(30, 30, 30),
            text_secondary: Color::from_rgb8(100, 100, 100),
            selected_color: Color::from_rgb8(59, 130, 246),
            hover_bg: Color::from_rgba8(0, 0, 0, 5),
            border_color: Color::from_rgb8(226, 232, 240),
        }
    }
}

// ============================================================================
// TreeView Component
// ============================================================================

/// Main TreeView component
pub struct TreeView<T: TreeNode> {
    root: T,
    config: TreeViewConfig,
    style: TreeViewStyle,
    flat_nodes: Signal<Vec<FlatNode<T>>>,
    open_states: Signal<HashMap<u64, bool>>,
    selected: Signal<Option<T>>,
    search: Signal<String>,
}

/// Flattened node for rendering
#[derive(Clone, PartialEq)]
struct FlatNode<T: TreeNode> {
    node: T,
    depth: usize,
    has_children: bool,
}

/// Context passed to custom render functions
pub struct TreeNodeContext {
    pub depth: usize,
    pub is_open: bool,
    pub is_selected: bool,
    pub has_children: bool,
    pub child_count: usize,
}

// ============================================================================
// Implementation
// ============================================================================

impl<T: TreeNode> TreeView<T> {
    // ------------------------------------------------------------------------
    // Constructor
    // ------------------------------------------------------------------------
    
    /// Create a new TreeView with the given root node
    pub fn new(root: T) -> Self {
        let config = TreeViewConfig::default();
        let style = TreeViewStyle::default();
        let flat_nodes = Self::flatten_tree_static(&root, 0, &HashMap::new(), "");
        
        let tree_view = Self {
            root,
            config,
            style,
            flat_nodes: Signal::new(flat_nodes),
            open_states: Signal::new(HashMap::new()),
            selected: Signal::new(None),
            search: Signal::new(String::new()),
        };
        
        // Setup search effect if search is enabled
        // Note: This will be called even if search is not enabled, but the effect
        // will be lightweight since search_text won't change unless enabled
        let tree_clone = tree_view.clone();
        let search = tree_view.search.clone();
        create_effect(move || {
            let _ = search.read();  // Subscribe to search changes
            tree_clone.rebuild_flat_list();
        });
        
        tree_view
    }
    
    // ------------------------------------------------------------------------
    // Builder Methods
    // ------------------------------------------------------------------------
    
    /// Set item height in pixels
    pub fn item_height(mut self, height: f32) -> Self {
        self.config.item_height = height;
        self
    }
    
    /// Set indentation width per level
    pub fn indent_width(mut self, width: f32) -> Self {
        self.config.indent_width = width;
        self
    }
    
    /// Show child count in node labels
    pub fn show_child_count(mut self, show: bool) -> Self {
        self.config.show_child_count = show;
        self
    }
    
    /// Enable search functionality
    pub fn enable_search(mut self) -> Self {
        self.config.enable_search = true;
        self
    }
    
    /// Disable virtual scrolling (use for small trees)
    pub fn disable_virtual_scroll(mut self) -> Self {
        self.config.enable_virtual_scroll = false;
        self
    }
    
    /// Set custom style
    pub fn style(mut self, style: TreeViewStyle) -> Self {
        self.style = style;
        self
    }
    
    // ------------------------------------------------------------------------
    // Public API (for external observation and control)
    // ------------------------------------------------------------------------
    
    /// Get the selected node signal (for external observation)
    pub fn selected(&self) -> Signal<Option<T>> {
        self.selected.clone()
    }
    
    /// Get the search text signal
    pub fn search_text(&self) -> Signal<String> {
        self.search.clone()
    }
    
    /// Programmatically expand/collapse a node
    pub fn toggle_node(&self, node: &T) {
        let key = Self::node_key(node);
        let is_open = self.open_states
            .read_untracked()
            .get(&key)
            .copied()
            .unwrap_or(false);
        
        // Load children if expanding
        if !is_open {
            if let Err(e) = node.load_children() {
                eprintln!("TreeView: Failed to load children: {}", e);
                return;
            }
        }
        
        // Toggle state
        self.open_states.write().insert(key, !is_open);
        
        // Rebuild flat list
        self.rebuild_flat_list();
    }
    
    /// Programmatically select a node
    pub fn select_node(&self, node: &T) {
        // Try to load the node
        if let Err(e) = node.load_children() {
            eprintln!("TreeView: Failed to load node: {}", e);
        }
        *self.selected.write() = Some(node.clone());
    }
    
    /// Check if a node is expanded
    pub fn is_open(&self, node: &T) -> bool {
        self.open_states
            .read_untracked()
            .get(&Self::node_key(node))
            .copied()
            .unwrap_or(false)
    }
    
    /// Get the number of visible (flattened) nodes
    pub fn visible_node_count(&self) -> usize {
        self.flat_nodes.read_untracked().len()
    }

    // ===== New Methods for Search Feature =====

    /// Get all open states (keyed by node hash)
    ///
    /// Returns a copy of the current open/closed states for all nodes.
    /// Useful for saving the current view state before performing operations
    /// that modify the tree structure.
    pub fn get_open_states(&self) -> HashMap<u64, bool> {
        self.open_states.read_untracked().clone()
    }

    /// Set open states (keyed by node hash) and rebuild flat list
    ///
    /// Allows bulk setting of node expand/collapse states.
    /// Automatically rebuilds the flattened node list to reflect changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut states = tree_view.get_open_states();
    /// states.insert(node_key, true);  // Expand a specific node
    /// tree_view.set_open_states(states);
    /// ```
    pub fn set_open_states(&self, states: HashMap<u64, bool>) {
        *self.open_states.write() = states;
        self.rebuild_flat_list();
    }

    /// Find index of a node in the flattened list
    ///
    /// Returns the position of the node in the currently flattened (visible) list.
    /// Returns `None` if the node is not currently visible (e.g., parent is collapsed).
    ///
    /// # Use Case
    ///
    /// Useful for calculating scroll positions when implementing `scroll_to_node()`.
    pub fn find_node_index(&self, node: &T) -> Option<usize> {
        self.flat_nodes.read_untracked()
            .iter()
            .position(|flat_node| {
                flat_node.node == *node
            })
    }

    /// Scroll to make a node visible in the viewport
    ///
    /// Programmatically scrolls the TreeView to ensure the specified node is visible.
    /// If the node is not currently in the flattened list (e.g., parent collapsed),
    /// this method does nothing.
    ///
    /// # Implementation Note
    ///
    /// Currently requires VirtualList to expose `scroll_offset` for full functionality.
    /// This is a placeholder implementation for future use.
    ///
    /// # Parameters
    ///
    /// - `node`: The node to scroll into view
    ///
    /// # Example
    ///
    /// ```ignore
    /// tree_view.expand_to_node(&target_node);
    /// tree_view.select_node(&target_node);
    /// tree_view.scroll_to_node(&target_node);  // Make it visible
    /// ```
    pub fn scroll_to_node(&self, node: &T) {
        // Placeholder: Requires VirtualList to expose scroll_offset
        // This will be implemented when VirtualList API is extended
        let _index = self.find_node_index(node);
        // TODO: Implement scrolling when VirtualList exposes scroll_offset
    }

    // ===== End New Methods =====

    // ------------------------------------------------------------------------
    // Internal Utilities
    // ------------------------------------------------------------------------
    
    /// Get unique key for a node (using hash)
    fn node_key(node: &T) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let mut hasher = DefaultHasher::new();
        node.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Rebuild the flattened node list
    fn rebuild_flat_list(&self) {
        let search = self.search.read_untracked().clone();
        let open_states = self.open_states.read_untracked().clone();
        let new_flat = Self::flatten_tree_static(&self.root, 0, &open_states, &search);
        *self.flat_nodes.write() = new_flat;
    }
    
    /// Flatten tree structure (static, for initialization and rebuilding)
    fn flatten_tree_static(
        node: &T,
        depth: usize,
        open_states: &HashMap<u64, bool>,  // ✅ 改为 u64
        search: &str,
    ) -> Vec<FlatNode<T>> {
        let label = node.label();
        let has_children = node.has_children();
        
        // Filter by search
        let matches = search.is_empty() || 
            label.to_lowercase().contains(&search.to_lowercase());
        
        let mut result = Vec::new();
        
        if matches {
            result.push(FlatNode {
                node: node.clone(),
                depth,
                has_children,
            });
        }
        
        // Recurse into children if open
        let key = Self::node_key(node);
        let is_open = open_states.get(&key).copied().unwrap_or(false);
        
        if is_open && has_children {
            let children = node.children();
            
            for child in children {
                result.extend(Self::flatten_tree_static(
                    &child,
                    depth + 1,
                    open_states,
                    search,
                ));
            }
        }
        
        result
    }
    
    // ------------------------------------------------------------------------
    // Rendering
    // ------------------------------------------------------------------------
    
    /// Build the view with default rendering
    pub fn build(self) -> View {
        self.build_with::<fn(&T, TreeNodeContext) -> View>(None)
    }
    
    /// Build the view with custom node rendering
    pub fn build_with<F>(self, custom_render: Option<F>) -> View
    where
        F: Fn(&T, TreeNodeContext) -> View + Clone + 'static,
    {
        let config = self.config.clone();
        let flat_nodes = self.flat_nodes.clone();
        
        // Wrap self in Rc for sharing across closures
        let tree_view = Rc::new(self);
        
        let root = view()
            .style(|s| s.w_full().h_full().flex().flex_col());
        
        // Note: Search box removed from TreeView component
        // Search functionality can be controlled externally via search_text() signal
        
        // Add tree content
        if config.enable_virtual_scroll {
            // ✅ 创建一次 VirtualList，使用已有的 flat_nodes Signal
            let tree_clone = tree_view.clone();
            let custom_clone = custom_render.clone();
            
            let virtual_list = VirtualList::new(flat_nodes)
                .item_height(config.item_height)
                .buffer_size(config.buffer_size)
                .build(move |flat_node, _index| {
                    tree_clone.render_node_with_custom(flat_node, &custom_clone)
                });
            
            root.child(virtual_list)
        } else {
            // Regular scrolling with dynamic
            let tree_clone = tree_view.clone();
            let custom_clone = custom_render.clone();
            
            let content = dynamic(move || {
                let nodes = flat_nodes.read().clone();
                let tree = tree_clone.clone();
                let custom = custom_clone.clone();
                
                view()
                    .style(|s| s.w_full().flex_grow(1.0).overflow_y_scroll())
                    .child(
                        nodes.into_iter().map(|flat_node| {
                            tree.render_node_with_custom(&flat_node, &custom)
                        }).collect::<Vec<_>>()
                    )
            });
            
            root.child(content)
        }
    }
    
    /// Render a single node with custom renderer
    fn render_node_with_custom<F>(
        &self,
        flat_node: &FlatNode<T>,
        custom_render: &Option<F>,
    ) -> View
    where
        F: Fn(&T, TreeNodeContext) -> View,
    {
        let node = &flat_node.node;
        let depth = flat_node.depth;
        let has_children = flat_node.has_children;
        
        // Get context
        let is_open = self.is_open(node);
        let is_selected = self.selected
            .read_untracked()
            .as_ref()
            .map(|sel| sel == node)
            .unwrap_or(false);
        let child_count = if has_children {
            node.children().len()
        } else {
            0
        };
        
        let ctx = TreeNodeContext {
            depth,
            is_open,
            is_selected,
            has_children,
            child_count,
        };
        
        // Use custom renderer if provided
        if let Some(renderer) = custom_render {
            return renderer(node, ctx);
        }
        
        // Default rendering
        self.render_node_default(node, ctx)
    }
    
    /// Default node rendering
    fn render_node_default(&self, node: &T, ctx: TreeNodeContext) -> View {
        let config = self.config.clone();
        let style = self.style.clone();
        let label = node.label();
        let node_clone = node.clone();
        let node_for_click = node.clone();
        
        let mut children = vec![];
        
        // Expand/collapse icon (responsive)
        if config.show_expand_icon {
            let style_clone = style.clone();
            let has_children = ctx.has_children;
            let open_states = self.open_states.clone();
            let node_for_icon = node.clone();
            
            let icon_view = view()
                .style(|s| s.width(16.0).justify_center())
                .child(
                    dynamic(move || {
                        let is_open = open_states
                            .read()
                            .get(&Self::node_key(&node_for_icon))
                            .copied()
                            .unwrap_or(false);
                        
                        text(if has_children {
                            if is_open { "▼" } else { "▶" }
                        } else {
                            " "
                        })
                        .style(move |s| {
                            s.font_size(10.0).color(style_clone.text_secondary)
                        })
                    })
                );
            children.push(icon_view);
        }
        
        // Node label (responsive)
        let style_clone = style.clone();
        let has_children = ctx.has_children;
        let selected_signal = self.selected.clone();
        let node_for_label = node.clone();
        
        let label_view = view()
            .style(|s| s.flex_grow(1.0))
            .child(
                dynamic(move || {
                    let is_selected = selected_signal
                        .read()
                        .as_ref()
                        .map(|sel| sel == &node_for_label)
                        .unwrap_or(false);
                    
                    let display_text = if config.show_child_count && ctx.child_count > 0 {
                        format!("{} ({})", label, ctx.child_count)
                    } else {
                        label.clone()
                    };
                    
                    text(display_text).style(move |s| {
                        s.font_size(13.0).color(if is_selected {
                            style_clone.selected_color
                        } else if has_children {
                            style_clone.text_color
                        } else {
                            style_clone.text_secondary
                        })
                    })
                })
            );
        children.push(label_view);
        
        let selected_for_bg = self.selected.clone();
        let node_for_bg = node.clone();
        
        view()
            .style(move |s| {
                let is_selected = selected_for_bg
                    .read()
                    .as_ref()
                    .map(|sel| sel == &node_for_bg)
                    .unwrap_or(false);
                
                let mut s = s.flex()
                    .height(config.item_height)
                    .w_full()
                    .padding_left(ctx.depth as f32 * config.indent_width + 8.0)
                    .padding_right(8.0)
                    .items_center()
                    .gap(8.0)
                    .border_bottom(1.0)
                    .border_color(style.border_color)
                    .cursor(crate::cursor::Cursor::Pointer);
                
                if is_selected {
                    s = s.background(style.selected_color.with_alpha(0.1));
                }
                s
            })
            .child(children)
            .on_click({
                let tree = Rc::new(self.clone());
                move |_| {
                    if ctx.has_children {
                        tree.toggle_node(&node_clone);
                    }
                    tree.select_node(&node_for_click);
                }
            })
    }
}

// Need to impl Clone for TreeView to use in closures
impl<T: TreeNode> Clone for TreeView<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            config: self.config.clone(),
            style: self.style.clone(),
            flat_nodes: self.flat_nodes.clone(),
            open_states: self.open_states.clone(),
            selected: self.selected.clone(),
            search: self.search.clone(),
        }
    }
}
