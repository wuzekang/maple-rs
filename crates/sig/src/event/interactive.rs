use super::types::*;
use winit::event::ElementState;

/// EventHandler trait - Used for automatic event type inference
///
/// This trait allows the compiler to infer event types from closure signatures,
/// enabling users to write `.on(handler)` instead of `.on::<EventType, _>(handler)`
pub trait EventHandler<E: 'static> {
    fn call(&mut self, event: &mut E);
}

impl<E, F> EventHandler<E> for F
where
    E: 'static,
    F: FnMut(&mut E),
{
    fn call(&mut self, event: &mut E) {
        self(event)
    }
}

/// Interactive trait - Provides event handling capabilities for all Elements
///
/// Through blanket implementation, all components implementing the Element trait
/// automatically gain event handling capabilities without manually implementing this trait.
///
/// # Design Philosophy
///
/// Uses blanket impl pattern: `impl<T: Element> Interactive for T`
/// This way any type implementing Element automatically gains Interactive capabilities.
///
/// # Examples
///
/// ```rust
/// use sig::{Element, View, button, create_scope};
///
/// create_scope(|| {
///     // View 实现了 Element，自动获得 .on_click()
///     let v = view().on_click(|_| println!("View clicked"));
///
///     // Button 实现了 Element，也自动获得 .on_click()
///     let btn = button("Click").on_click(|_| println!("Button clicked"));
/// });
/// ```
pub trait Interactive: crate::Element {
    /// Add generic event listener (core method)
    ///
    /// This is the foundation of all event listener methods, supporting any event type.
    ///
    /// # Type Parameters
    /// - `E`: Event type (automatically inferred from closure signature)
    ///
    /// # Examples
    /// ```
    /// use sig::{create_scope, view, MouseEvent};
    ///
    /// create_scope(|| {
    ///     // ✅ 类型自动推断！
    ///     let v = view().on(|event: &mut MouseEvent| {
    ///         println!("Mouse at: {:?}", event.position);
    ///     });
    /// });
    /// ```
    fn on<E>(self, handler: impl EventHandler<E> + 'static) -> Self
    where
        E: 'static,
    {
        let view_id = self.id();
        crate::runtime::with_layout_mut(|runtime| {
            if let Some(state) = runtime.view_states.get_mut(&view_id) {
                state.event_handlers.add_handler(handler);
            }
        });
        self
    }

    /// Add click event listener
    ///
    /// # Parameters
    /// - `f`: Click event handler function
    ///
    /// # Examples
    /// ```
    /// use sig::{create_scope, button};
    ///
    /// create_scope(|| {
    ///     let btn = button("Click Me").on_click(|_| {
    ///         println!("Button clicked!");
    ///     });
    /// });
    /// ```
    fn on_click<F>(self, mut f: F) -> Self
    where
        F: FnMut(&MouseEvent) + 'static,
    {
        self.on(move |event: &mut MouseEvent| {
            if event.r#type == MouseEventType::Click {
                f(event);
            }
        })
    }
    
    /// Add mouse down event listener
    fn on_mouse_down<F>(self, mut f: F) -> Self
    where
        F: FnMut(&MouseEvent) + 'static,
    {
        self.on(move |event: &mut MouseEvent| {
            if event.r#type == MouseEventType::MouseDown {
                f(event);
            }
        })
    }

    /// Add mouse up event listener
    fn on_mouse_up<F>(self, mut f: F) -> Self
    where 
        F: FnMut(&MouseEvent) + 'static,
    {
        self.on(move |event: &mut MouseEvent| {
            if event.r#type == MouseEventType::MouseUp {
                f(event);
            }
        })
    }

    /// Add mouse move event listener
    fn on_mouse_move<F>(self, mut f: F) -> Self
    where
        F: FnMut(&MouseEvent) + 'static,
    {
        self.on(move |event: &mut MouseEvent| {
            if event.r#type == MouseEventType::MouseMove {
                f(event);
            }
        })
    }

    /// Add mouse enter event listener
    fn on_mouse_enter<F>(self, mut f: F) -> Self
    where
        F: FnMut(&MouseEvent) + 'static,
    {
        self.on(move |event: &mut MouseEvent| {
            if event.r#type == MouseEventType::MouseEnter {
                f(event);
            }
        })
    }

    /// Add mouse leave event listener
    fn on_mouse_leave<F>(self, mut f: F) -> Self
    where
        F: FnMut(&MouseEvent) + 'static,
    {
        self.on(move |event: &mut MouseEvent| {
            if event.r#type == MouseEventType::MouseLeave {
                f(event);
            }
        })
    }

    /// Add wheel event listener
    fn on_wheel<F>(self, f: F) -> Self
    where
        F: FnMut(&mut WheelEvent) + 'static,
    {
        self.on(f)
    }

    /// Add scroll event listener
    ///
    /// Triggered when the element's scroll position changes.
    ///
    /// # Parameters
    /// - `f`: Scroll event handler function
    ///
    /// # Examples
    /// ```
    /// use sig::{create_scope, view};
    ///
    /// create_scope(|| {
    ///     let v = view()
    ///         .style(|s| s.overflow_y_scroll())
    ///         .on_scroll(|event| {
    ///             println!("scrollTop: {}", event.scroll_top);
    ///             println!("scrollHeight: {}", event.scroll_height);
    ///         });
    /// });
    /// ```
    fn on_scroll<F>(self, f: F) -> Self
    where
        F: FnMut(&mut ScrollEvent) + 'static,
    {
        self.on(f)
    }

    /// Add key down event listener
    fn on_key_down<F>(self, mut f: F) -> Self
    where
        F: FnMut(&KeyboardEvent) + 'static,
    {
        self.on(move |event: &mut KeyboardEvent| {
            if event.state == ElementState::Pressed {
                f(event);
            }
        })
    }

    /// Add key up event listener
    fn on_key_up<F>(self, mut f: F) -> Self
    where
        F: FnMut(&KeyboardEvent) + 'static,
    {
        self.on(move |event: &mut KeyboardEvent| {
            if event.state == ElementState::Released {
                f(event);
            }
        })
    }

    /// Add focus event listener
    fn on_focus<F>(self, mut f: F) -> Self
    where
        F: FnMut(&mut FocusEvent) + 'static,
    {
        self.on(move |event: &mut FocusEvent| {
            if event.r#type == FocusEventType::Focus {
                f(event);
            }
        })
    }

    /// Add blur event listener
    fn on_blur<F>(self, mut f: F) -> Self
    where
        F: FnMut(&mut FocusEvent) + 'static,
    {
        self.on(move |event: &mut FocusEvent| {
            if event.r#type == FocusEventType::Blur {
                f(event);
            }
        })
    }

    /// Make this view focusable (can receive keyboard focus)
    ///
    /// When a focusable view is clicked, it automatically receives focus and
    /// can handle keyboard and IME events.
    ///
    /// # Example
    /// ```ignore
    /// view()
    ///     .focusable()
    ///     .on_focus(|_| println!("Got focus"))
    ///     .on_keyboard(|e| println!("Key pressed"))
    /// ```
    fn focusable(self) -> Self {
        let view_id = self.id();
        crate::runtime::with_layout_mut(|runtime| {
            if let Some(state) = runtime.view_states.get_mut(&view_id) {
                state.focusable = true;
            }
        });
        self
    }

    /// Add keyboard event listener
    fn on_keyboard<F>(self, f: F) -> Self
    where
        F: FnMut(&mut KeyboardEvent) + 'static,
    {
        self.on(f)
    }

    /// Add IME event listener
    fn on_ime<F>(self, f: F) -> Self
    where
        F: FnMut(&mut ImeEvent) + 'static,
    {
        self.on(f)
    }

    /// Add drag start event listener
    ///
    /// Triggered when mouse moves beyond threshold (default 5px) after being pressed.
    ///
    /// # Parameters
    /// - `f`: Drag start event handler function
    ///
    /// # Examples
    /// ```
    /// use sig::{create_scope, view};
    ///
    /// create_scope(|| {
    ///     let v = view().on_drag_start(|event| {
    ///         println!("开始拖拽: {:?}", event.start_position);
    ///     });
    /// });
    /// ```
    fn on_drag_start<F>(self, mut f: F) -> Self
    where
        F: FnMut(&mut DragEvent) + 'static,
    {
        self.on(move |event: &mut DragEvent| {
            if event.r#type == DragEventType::DragStart {
                f(event);
            }
        })
    }

    /// Add drag event listener
    ///
    /// Triggered on every mouse movement after drag starts.
    ///
    /// # Parameters
    /// - `f`: Drag event handler function
    ///
    /// # Examples
    /// ```
    /// use sig::{create_scope, view};
    ///
    /// create_scope(|| {
    ///     let v = view().on_drag(|event| {
    ///         println!("拖拽中，总偏移: {:?}", event.total_delta);
    ///     });
    /// });
    /// ```
    fn on_drag<F>(self, mut f: F) -> Self
    where
        F: FnMut(&mut DragEvent) + 'static,
    {
        self.on(move |event: &mut DragEvent| {
            if event.r#type == DragEventType::Drag {
                f(event);
            }
        })
    }

    /// Add drag end event listener
    ///
    /// Triggered when mouse is released during drag.
    ///
    /// # Parameters
    /// - `f`: Drag end event handler function
    ///
    /// # Examples
    /// ```
    /// use sig::{create_scope, view};
    ///
    /// create_scope(|| {
    ///     let v = view().on_drag_end(|event| {
    ///         println!("拖拽结束，总偏移: {:?}", event.total_delta);
    ///     });
    /// });
    /// ```
    fn on_drag_end<F>(self, mut f: F) -> Self
    where
        F: FnMut(&mut DragEvent) + 'static,
    {
        self.on(move |event: &mut DragEvent| {
            if event.r#type == DragEventType::DragEnd {
                f(event);
            }
        })
    }
}

/// Blanket implementation - All Elements automatically gain Interactive
///
/// This is the core design of the component system: through a single blanket impl,
/// all components implementing the Element trait automatically gain 10+ event handling methods
impl<T: crate::Element> Interactive for T {}
