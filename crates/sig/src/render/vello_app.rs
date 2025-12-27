use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions, Scene};
use vello::peniko::Color;
use vello::wgpu;
use winit::window::Window;
use crate::ViewId;
use super::view_render::render_view;

pub(super) struct RenderState {
  surface: RenderSurface<'static>,
  #[allow(dead_code)]
  window: Arc<Window>,
  scale_factor: f64, // 🎯 Store DPR
}

pub struct VelloApp {
  context: RenderContext,
  renderers: Vec<Option<Renderer>>,
  pub(super) state: Option<RenderState>,
  scene: Scene,
}

impl VelloApp {
  pub fn new() -> Self {
    Self {
      context: RenderContext::new(),
      renderers: vec![],
      state: None,
      scene: Scene::new(),
    }
  }

  pub fn resume(&mut self, window: Arc<Window>) {
    let size = window.inner_size();
    let scale_factor = window.scale_factor(); // 🎯 Get DPR

    let surface_future = self.context.create_surface(
      window.clone(),
      size.width,
      size.height,
      wgpu::PresentMode::AutoVsync,
    );
    let surface = pollster::block_on(surface_future).expect("Error creating surface");

    self
      .renderers
      .resize_with(self.context.devices.len(), || None);
    self.renderers[surface.dev_id]
      .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

    // 🎯 Save scale_factor
    self.state = Some(RenderState {
      surface,
      window,
      scale_factor,
    });
  }

  pub fn suspend(&mut self) {
    self.state = None;
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    if let Some(state) = &mut self.state {
      // 🎯 Update scale_factor (may change during window resize)
      state.scale_factor = state.window.scale_factor();

      // 🎯 Clamp dimensions to GPU texture size limits (typically 8192 or 16384)
      // wgpu enforces these limits during texture creation
      const MAX_TEXTURE_DIMENSION: u32 = 8192;
      let clamped_width = width.min(MAX_TEXTURE_DIMENSION);
      let clamped_height = height.min(MAX_TEXTURE_DIMENSION);

      if clamped_width != width || clamped_height != height {
        eprintln!(
          "⚠️  GPU texture size limit exceeded: requested {}x{}, clamped to {}x{}",
          width, height, clamped_width, clamped_height
        );
      }

      self
        .context
        .resize_surface(&mut state.surface, clamped_width, clamped_height);
    }
  }

  pub fn render(&mut self, view: &ViewId, hover_top_view: Option<ViewId>) {
    if let Some(state) = &mut self.state {
      let width = state.surface.config.width;
      let height = state.surface.config.height;
      let dev_id = state.surface.dev_id;
      let scale_factor = state.scale_factor as f32;

      // 🎯 Convert physical pixels to logical pixels
      let logical_width = width as f32 / scale_factor;
      let logical_height = height as f32 / scale_factor;

      // 🎯 Create render context (using logical pixels)
      let render_ctx = crate::RenderContext::new(scale_factor, (logical_width, logical_height));

      // 🎯 Add layout computation
      let available_space = taffy::Size {
        width: taffy::AvailableSpace::Definite(logical_width),
        height: taffy::AvailableSpace::Definite(logical_height),
      };

      // 🎯 Compute styles before layout computation (ensure inheritance is correct)
      //
      // 1. Batch bubbling: Mark upwards from collected dirty nodes (parent-child relationships are stable)
      // 2. Compute styles: Use Dirty Bubbling pruning traversal
      // 3. Style computation internally triggers necessary repaints via request_repaint
      crate::style::compute::batch_bubble_dirty_marks();

      let mut style_ctx = crate::style::compute::StyleComputeContext::new();
      crate::style::compute::compute_style_recursive(view, &mut style_ctx);

      view.compute_layout(available_space, &render_ctx);

      // Reset scene
      self.scene.reset();

      // 🎯 Render view, passing render context
      render_view(&mut self.scene, *view, 0.0, 0.0, &render_ctx, hover_top_view);

      let device_handle = &self.context.devices[dev_id];

      // Render to texture
      self.renderers[dev_id]
        .as_mut()
        .unwrap()
        .render_to_texture(
          &device_handle.device,
          &device_handle.queue,
          &self.scene,
          &state.surface.target_view,
          &vello::RenderParams {
            base_color: Color::new([0.95, 0.95, 0.97, 1.0]), // Light background
            width,
            height,
            antialiasing_method: AaConfig::Msaa16,
          },
        )
        .expect("failed to render to surface");

      // Blit to surface
      let surface_texture = state
        .surface
        .surface
        .get_current_texture()
        .expect("failed to get surface texture");

      let mut encoder =
        device_handle
          .device
          .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Blit"),
          });

      state.surface.blitter.copy(
        &device_handle.device,
        &mut encoder,
        &state.surface.target_view,
        &surface_texture
          .texture
          .create_view(&wgpu::TextureViewDescriptor::default()),
      );

      device_handle.queue.submit([encoder.finish()]);
      surface_texture.present();

      // Poll device to keep UI responsive
      device_handle.device.poll(wgpu::PollType::Poll).unwrap();
    }
  }

  pub fn get_scale_factor(&self) -> Option<f64> {
    self.state.as_ref().map(|s| s.scale_factor)
  }

  pub fn has_state(&self) -> bool {
    self.state.is_some()
  }

  pub fn update_scale_factor(&mut self, scale_factor: f64) {
    if let Some(state) = &mut self.state {
      state.scale_factor = scale_factor;
    }
  }
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface<'_>) -> Renderer {
  Renderer::new(
    &render_cx.devices[surface.dev_id].device,
    RendererOptions::default(),
  )
  .expect("Couldn't create renderer")
}
