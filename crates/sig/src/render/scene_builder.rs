use vello::kurbo::{Affine, Circle, Ellipse, Line, RoundedRect, Stroke};
use vello::peniko::Color;
use vello::Scene;

pub struct SceneBuilder {
  scene: Scene,
}

impl SceneBuilder {
  pub fn new() -> Self {
    SceneBuilder {
      scene: Scene::new(),
    }
  }

  pub fn draw_rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: Color) {
    let stroke = Stroke::new(6.0);
    let rect = RoundedRect::new(x, y, x + width, y + height, 20.0);
    self
      .scene
      .stroke(&stroke, Affine::IDENTITY, color, None, &rect);
  }

  pub fn draw_filled_rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: Color) {
    let rect = RoundedRect::new(x, y, x + width, y + height, 0.0);
    self.scene.fill(
      vello::peniko::Fill::NonZero,
      Affine::IDENTITY,
      color,
      None,
      &rect,
    );
  }

  pub fn draw_circle(&mut self, center_x: f64, center_y: f64, radius: f64, color: Color) {
    let circle = Circle::new((center_x, center_y), radius);
    self.scene.fill(
      vello::peniko::Fill::NonZero,
      Affine::IDENTITY,
      color,
      None,
      &circle,
    );
  }

  pub fn draw_ellipse(
    &mut self,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    color: Color,
  ) {
    let ellipse = Ellipse::new((center_x, center_y), (radius_x, radius_y), rotation);
    self.scene.fill(
      vello::peniko::Fill::NonZero,
      Affine::IDENTITY,
      color,
      None,
      &ellipse,
    );
  }

  pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: Color) {
    let stroke = Stroke::new(6.0);
    let line = Line::new((x1, y1), (x2, y2));
    self
      .scene
      .stroke(&stroke, Affine::IDENTITY, color, None, &line);
  }

  pub fn build(self) -> Scene {
    self.scene
  }
}

impl Default for SceneBuilder {
  fn default() -> Self {
    Self::new()
  }
}
