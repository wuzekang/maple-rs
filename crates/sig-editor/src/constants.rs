use vello::peniko::Color;

pub const DEFAULT_LEFT_WIDTH: f32 = 280.0;

// Colors (Consider moving to theme system in future)
pub const BG_COLOR: Color = Color::from_rgb8(250, 250, 250); // gray-50
pub const PANEL_BG: Color = Color::from_rgb8(255, 255, 255); // white
pub const PANEL_HEADER_BG: Color = Color::from_rgb8(248, 250, 252); // slate-50
pub const BORDER_COLOR: Color = Color::from_rgb8(226, 232, 240); // slate-200
pub const TEXT_PRIMARY: Color = Color::from_rgb8(30, 30, 30); // gray-900
pub const TEXT_SECONDARY: Color = Color::from_rgb8(100, 100, 100); // gray-500
pub const ERROR_COLOR: Color = Color::from_rgb8(220, 38, 38); // red-600
pub const SUCCESS_COLOR: Color = Color::from_rgb8(34, 197, 94); // green-500
