use crate::ViewId;
use glam::Vec2;
use std::ops::{Deref, DerefMut};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::Key;

/// Generic event wrapper - Unified event metadata handling
///
/// # Design Philosophy
///
/// Use composition over inheritance: `Event<T>` wraps specific event data `T`.
/// Common fields (target, propagation, etc.) are managed by Event,
/// specific data is provided by T.
///
/// # Example
///
/// ```rust
/// use sig::{Event, ViewId};
///
/// // Custom event data
/// #[derive(Debug, Clone)]
/// pub struct GamepadData {
///     pub button: u8,
///     pub pressed: bool,
/// }
///
/// // Wrap with Event
/// pub type GamepadEvent = Event<GamepadData>;
///
/// // Use in handler
/// view.on(|e: &mut GamepadEvent| {
///     println!("Button: {}", e.button);  // Direct access via Deref
///     e.stop_propagation();              // Event-provided method
/// });
/// ```
#[derive(Debug, Clone)]
pub struct Event<T> {
    /// Event target (View that initially triggered the event)
    pub target: ViewId,
    /// Current View handling event (during bubbling)
    pub current: Option<ViewId>,
    /// Whether to continue propagation
    pub propagation: bool,
    /// Specific event data
    pub data: T,
}

impl<T> Event<T> {
    /// Create new event
    pub fn new(target: ViewId, data: T) -> Self {
        Self {
            target,
            current: None,
            propagation: true,
            data,
        }
    }

    /// Stop event propagation
    pub fn stop_propagation(&mut self) {
        self.propagation = false;
    }

    /// Set current View handling event
    pub fn set_current(&mut self, view_id: ViewId) {
        self.current = Some(view_id);
    }
}

/// Implement Deref to allow direct access to event data
impl<T> Deref for Event<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Event<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

// ============ Mouse Events ============

/// Mouse event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    MouseDown,
    MouseUp,
    MouseMove,
    MouseEnter,
    MouseLeave,
    Click,
}

/// Mouse event data
#[derive(Debug, Clone, PartialEq)]
pub struct MouseData {
    pub position: Vec2,
    pub button: Option<MouseButton>,
    pub state: ElementState,
    pub r#type: MouseEventType,
    pub modifiers: winit::keyboard::ModifiersState,
}

/// Mouse event type alias
pub type MouseEvent = Event<MouseData>;

impl MouseEvent {
    /// Get client coordinates (screen absolute coordinates)
    pub fn client(&self) -> Vec2 {
        self.position
    }

    /// Get offset coordinates relative to target element
    pub fn offset(&self) -> Vec2 {
        if let Some(current) = self.current {
            crate::runtime::with_layout(|runtime| {
                if let Ok(layout) = runtime.taffy.layout(current.node_id()) {
                    let location = layout.location;
                    let scroll_offset = runtime.view_states
                        .get(&current)
                        .map(|state| state.scroll_offset)
                        .unwrap_or((0.0, 0.0));

                    return self.position
                        - Vec2::new(location.x, location.y)
                        - Vec2::new(scroll_offset.0, scroll_offset.1);
                }
                self.position
            })
        } else {
            self.position
        }
    }
}

// ============ Wheel Events ============

/// Wheel event data
#[derive(Debug, Clone, PartialEq)]
pub struct WheelData {
    pub position: Vec2,
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Wheel event type alias
pub type WheelEvent = Event<WheelData>;

// ============ Keyboard Events ============

/// Keyboard event data
#[derive(Debug, Clone, PartialEq)]
pub struct KeyboardData {
    pub logical_key: Key,
    pub state: ElementState,
    pub modifiers: winit::keyboard::ModifiersState,
}

/// Keyboard event type alias
pub type KeyboardEvent = Event<KeyboardData>;

// ============ Scroll Events ============

/// Scroll event data
///
/// Reference DOM scroll event, directly carries scroll-related properties
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollData {
    /// scrollLeft - horizontal scroll position
    pub scroll_left: f32,
    /// scrollTop - vertical scroll position
    pub scroll_top: f32,

    /// scrollWidth - total content width
    pub scroll_width: f32,
    /// scrollHeight - total content height
    pub scroll_height: f32,

    /// clientWidth - visible area width
    pub client_width: f32,
    /// clientHeight - visible area height
    pub client_height: f32,
}

/// Scroll event type alias
pub type ScrollEvent = Event<ScrollData>;

// ============ Drag Events ============

/// Drag event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragEventType {
    /// Drag start (after mouse down and moved beyond threshold)
    DragStart,
    /// Drag in progress (mouse moving)
    Drag,
    /// Drag end (mouse released)
    DragEnd,
}

/// Drag event data
///
/// Drag events are automatically generated by the framework by combining MouseDown + MouseMove + MouseUp.
///
/// # Field Descriptions
///
/// - `start_position`: Drag start position (coordinates when mouse pressed)
/// - `current_position`: Current mouse position
/// - `delta`: Offset of current frame relative to previous frame
/// - `total_delta`: Total offset from start position to current position
#[derive(Debug, Clone, PartialEq)]
pub struct DragData {
    pub r#type: DragEventType,
    /// Drag start position (screen coordinates)
    pub start_position: Vec2,
    /// Current mouse position (screen coordinates)
    pub current_position: Vec2,
    /// Offset of current frame relative to previous frame
    pub delta: Vec2,
    /// Total offset: current position relative to start position
    pub total_delta: Vec2,
}

/// Drag event type alias
pub type DragEvent = Event<DragData>;

impl DragEvent {
    /// Get client coordinates (screen absolute coordinates)
    pub fn client(&self) -> Vec2 {
        self.current_position
    }

    /// Get offset coordinates relative to target element
    pub fn offset(&self) -> Vec2 {
        if let Some(current) = self.current {
            crate::runtime::with_layout(|runtime| {
                if let Ok(layout) = runtime.taffy.layout(current.node_id()) {
                    let location = layout.location;

                    let scroll_offset = runtime.view_states
                        .get(&current)
                        .map(|state| state.scroll_offset)
                        .unwrap_or((0.0, 0.0));

                    return self.current_position
                        - Vec2::new(location.x, location.y)
                        - Vec2::new(scroll_offset.0, scroll_offset.1);
                }

                self.current_position
            })
        } else {
            self.current_position
        }
    }
}

// ============ Focus Events ============

/// Focus event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEventType {
    /// View gains focus
    Focus,
    /// View loses focus
    Blur,
}

/// Focus event data
#[derive(Debug, Clone, PartialEq)]
pub struct FocusData {
    pub r#type: FocusEventType,
    /// The view that previously had focus (None if no previous focus)
    pub related_target: Option<ViewId>,
}

/// Focus event type alias
pub type FocusEvent = Event<FocusData>;

// ============ IME Events ============

/// IME event data (Input Method Editor)
#[derive(Debug, Clone)]
pub struct ImeData {
    /// IME event from winit
    pub ime: winit::event::Ime,
}

/// IME event type alias
pub type ImeEvent = Event<ImeData>;
