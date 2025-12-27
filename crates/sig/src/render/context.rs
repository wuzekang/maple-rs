//! Render context module
//!
//! Provides context information during rendering, including DPR, window size, etc.

/// Render context
#[derive(Debug, Clone, Copy)]
pub struct RenderContext {
    /// Device pixel ratio
    pub scale_factor: f32,
    /// Window size (logical pixels)
    pub window_size: (f32, f32),
}

impl RenderContext {
    /// Create new render context
    pub fn new(scale_factor: f32, window_size: (f32, f32)) -> Self {
        Self {
            scale_factor,
            window_size,
        }
    }

    /// Convert logical pixels to physical pixels
    pub fn logical_to_physical(&self, logical: f32) -> f32 {
        logical * self.scale_factor
    }

    /// Convert physical pixels to logical pixels
    pub fn physical_to_logical(&self, physical: f32) -> f32 {
        physical / self.scale_factor
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            window_size: (800.0, 600.0),
        }
    }
}

/// Set current thread's render context
pub fn set_current_render_context(ctx: RenderContext) {
    crate::runtime::with_render_ctx_mut(|state| {
        state.current = ctx;
    });
}

/// Get current thread's render context
pub fn get_current_render_context() -> RenderContext {
    crate::runtime::with_render_ctx(|state| state.current)
}

/// Execute operations within specified render context (RAII pattern)
pub fn with_render_context<R>(ctx: RenderContext, f: impl FnOnce() -> R) -> R {
    let prev = get_current_render_context();
    set_current_render_context(ctx);
    let result = f();
    set_current_render_context(prev);
    result
}
