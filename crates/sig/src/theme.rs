//! Design System Foundation Tokens
//!
//! This module defines the foundation layer of the Sig design system,
//! providing standardized values for spacing, sizing, border radius, and borders.
//!
//! All tokens follow the design guidelines in `docs/DESIGN_GUIDELINES.md`.
//!
//! # Usage
//!
//! ```rust
//! use sig::theme::{Spacing, Size, Radius, Border};
//!
//! // Use tokens instead of hardcoded values
//! let padding = Spacing::MD;  // 12.0px
//! let height = Size::MD;       // 40.0px
//! let radius = Radius::MD;     // 6.0px
//! ```

/// Spacing scale - Based on 8px grid system
///
/// All spacing values are multiples of 4px for high-DPI compatibility.
/// Optimized for compact desktop UIs.
pub struct Spacing;

impl Spacing {
    /// Extra small spacing: 2px - Minimal gaps (very tight elements)
    pub const XXS: f32 = 2.0;
    
    /// Extra small spacing: 4px - For tightly related elements
    pub const XS: f32 = 4.0;
    
    /// Small spacing: 8px - For related elements
    pub const SM: f32 = 8.0;
    
    /// Medium spacing: 12px - Default inner padding
    pub const MD: f32 = 12.0;
    
    /// Large spacing: 16px - Default outer margin
    pub const LG: f32 = 16.0;
    
    /// Extra large spacing: 20px - Block separation
    pub const XL: f32 = 20.0;
    
    /// 2X large spacing: 24px - Major section separation
    pub const XXL: f32 = 24.0;
    
    /// 3X large spacing: 32px - Page-level separation
    pub const XXXL: f32 = 32.0;
}

/// Size scale - Standard component heights
///
/// Optimized for desktop applications, code editors, and embedded UIs.
/// These values are more compact than typical web UI frameworks.
pub struct Size;

impl Size {
    /// Small size: 24px - Very compact UI (tree items, status bar)
    pub const SM: f32 = 24.0;
    
    /// Medium size: 32px - Default size (code editor standard)
    pub const MD: f32 = 32.0;
    
    /// Large size: 40px - Emphasis scenarios (primary actions)
    pub const LG: f32 = 40.0;
    
    /// Extra large size: 48px - Hero buttons (rare use)
    pub const XL: f32 = 48.0;
}

/// Radius scale - Border radius values
///
/// Defines standard border radius values for different component types.
/// More subtle radii for professional desktop applications.
pub struct Radius;

impl Radius {
    /// No radius: 0px - For special cases
    pub const NONE: f32 = 0.0;
    
    /// Small radius: 2px - For compact components (subtle rounding)
    pub const SM: f32 = 2.0;
    
    /// Medium radius: 4px - Default for buttons and inputs
    pub const MD: f32 = 4.0;
    
    /// Large radius: 6px - For cards and containers
    pub const LG: f32 = 6.0;
    
    /// Extra large radius: 8px - For emphasis scenarios
    pub const XL: f32 = 8.0;
    
    /// Full radius: 9999px - For circular shapes (badges, avatars)
    pub const FULL: f32 = 9999.0;
}

/// Border scale - Border width values
///
/// Defines standard border widths for different states.
pub struct Border;

impl Border {
    /// Thin border: 1px - Standard border (default)
    pub const THIN: f32 = 1.0;
    
    /// Medium border: 2px - Emphasis border (focus state)
    pub const MEDIUM: f32 = 2.0;
    
    /// Thick border: 3px - Heavy emphasis
    pub const THICK: f32 = 3.0;
}

/// Font size scale - Typography sizes
///
/// Defines standard font sizes that align with component size variants.
pub struct FontSize;

impl FontSize {
    /// Extra small font: 11px - For captions and helper text
    pub const XS: f32 = 11.0;
    
    /// Small font: 12px - For small components
    pub const SM: f32 = 12.0;
    
    /// Medium font: 14px - Default body text
    pub const MD: f32 = 14.0;
    
    /// Large font: 16px - For large components
    pub const LG: f32 = 16.0;
    
    /// Extra large font: 18px - For headings
    pub const XL: f32 = 18.0;
    
    /// 2X large font: 20px - For large headings
    pub const XXL: f32 = 20.0;
}

/// Opacity scale - Standard opacity values
///
/// Defines standard opacity values for different states.
pub struct Opacity;

impl Opacity {
    /// Disabled state: 50-60% opacity
    pub const DISABLED: f32 = 0.5;
    
    /// Hover overlay: 10% opacity
    pub const HOVER_OVERLAY: f32 = 0.1;
    
    /// Press overlay: 20% opacity
    pub const PRESS_OVERLAY: f32 = 0.2;
    
    /// Subtle overlay: 5% opacity
    pub const SUBTLE: f32 = 0.05;
}

/// Animation duration scale - Standard transition durations
///
/// Defines standard animation durations for smooth transitions.
pub struct Duration;

impl Duration {
    /// Fast transition: 100ms - For press effects
    pub const FAST: u64 = 100;
    
    /// Normal transition: 150ms - For hover effects
    pub const NORMAL: u64 = 150;
    
    /// Slow transition: 300ms - For complex animations
    pub const SLOW: u64 = 300;
}
