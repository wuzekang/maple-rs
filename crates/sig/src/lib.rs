// Core
pub mod core;
pub use core::*;

// Reactive
pub mod reactive;
pub use reactive::*;

// Components
pub mod components;
pub use components::*;

// Events
pub mod event;
pub use event::*;

// Style
pub mod style;
pub use style::*;

// Theme (Design System Foundation Tokens)
// Note: Theme tokens are accessible via `theme::` to avoid naming conflicts
pub mod theme;

// Layout
pub mod layout;
pub use layout::{LayoutState, compute_layout};

// Runtime (state management)
pub mod runtime;
pub use runtime::{
  AppRuntime,
  InputState,
  RenderCtxState,
  WindowState,
  clear_focus,
  current_owner,
  current_scope_id,
  get_focused,
  is_focused,
  // Reactive runtime functions (from sig-reactive)
  on_cleanup,
  remove_scope,
  // Focus management (from app/runtime)
  set_focus,
  untrack,
  with_input,
  with_input_mut,
  with_layout,
  with_layout_mut,
  with_render_ctx,
  with_render_ctx_mut,
  with_window,
  with_window_mut,
};

// App (application layer - event loop, runner)
pub mod app;
pub use app::{AppConfig, Runner, run};

// Render
pub mod render;
pub use render::*;

// Portal (simplified API)
pub mod portal;

/// A node in the view tree
#[derive(Clone, Debug)]
pub enum Node {
  /// A collection of child nodes
  Fragment(Vec<Node>),
  /// Dynamic content stored in a signal, each element carries a ScopeId for lifecycle management
  Dynamic(Signal<Vec<(Node, reactive::ScopeId)>>),
  /// View component
  View(ViewId),
}

impl Default for Node {
  fn default() -> Self {
    Node::Fragment(Vec::new())
  }
}

pub mod prelude {
  pub use crate::components::button::{
    Button, ButtonSize, ButtonVariant, button, danger_button, outline_button, primary_button,
    secondary_button, text_button,
  };
  pub use crate::components::collapsible::{Collapsible, collapsible};
  pub use crate::components::image::{Image, image};
  pub use crate::components::resizable::{
    HandlePosition, Resizable, ResizeDirection, resizable_horizontal, resizable_vertical,
  };
  pub use crate::components::separator::{
    Orientation, Separator, horizontal_separator, separator, vertical_separator,
  };
  pub use crate::components::text::{TextInput, TextArea, dynamic_text, text, text_area, text_input};
  pub use crate::layout::taffy;
  pub use crate::runtime::{on_cleanup, untrack};
  pub use crate::style::{StyleBuilder, Styleable};
  pub use crate::{AppConfig, AppRuntime, Runner, run};
  pub use crate::{
    Interactive, IntoElement, Signal, View, ViewTuple, create_effect, create_scope, fragment, view,
  };

  // Task API
  pub use crate::reactive::{Task, current_task, parent_task, poll_tasks, spawn};

  // Resource API
  pub use crate::reactive::{Resource, ResourceState, use_resource};
}
