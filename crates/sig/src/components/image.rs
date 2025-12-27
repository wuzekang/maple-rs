//! Image component module
//!
//! Provides image rendering functionality with support for creating images from various formats.
//!
//! # Examples
//!
//! ```rust
//! use sig::{Image, create_scope};
//!
//! create_scope(|| {
//!     // Create from RGBA bytes
//!     let img = Image::from_rgba(rgba_bytes, 100, 100);
//!     
//!     // Custom display size
//!     let img = Image::from_rgba(rgba_bytes, 100, 100)
//!         .width(200.0)
//!         .height(150.0);
//! });
//! ```

use crate::{Element, ViewId, Widget, RenderContext};
use std::rc::Rc;
use std::sync::Arc;
use taffy::{AvailableSpace, Size};
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use vello::Scene;

/// Image component
///
/// Image implements the Element trait, automatically providing:
/// - Styleable: All styling methods
/// - Interactive: All event handling methods
///
/// # Examples
///
/// ```rust
/// use sig::{create_scope, Image};
///
/// create_scope(|| {
///     // Create from RGBA bytes
///     let rgba = vec![255u8; 100 * 100 * 4]; // White image
///     let img = Image::from_rgba(rgba, 100, 100);
///     
///     // Set display size
///     let img = Image::from_rgba(rgba, 100, 100)
///         .width(200.0)
///         .height(150.0);
/// });
/// ```
#[derive(Clone)]
pub struct Image {
    id: ViewId,
    image_data: ImageData,
    display_width: Option<f32>,
    display_height: Option<f32>,
}

impl Image {
    /// Create image from RGBA bytes
    ///
    /// # Parameters
    /// - `rgba`: RGBA format byte data (4 bytes per pixel)
    /// - `width`: Image width in pixels
    /// - `height`: Image height in pixels
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Image};
    ///
    /// create_scope(|| {
    ///     let rgba = vec![255u8; 100 * 100 * 4];
    ///     let img = Image::from_rgba(rgba, 100, 100);
    /// });
    /// ```
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        let image_data = ImageData {
            data: Blob::new(Arc::new(rgba)),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::AlphaPremultiplied,
            width,
            height,
        };
        
        Self::from_image_data(image_data)
    }
    
    /// Create image from RGB bytes (no alpha channel)
    ///
    /// # Parameters
    /// - `rgb`: RGB format byte data (3 bytes per pixel)
    /// - `width`: Image width in pixels
    /// - `height`: Image height in pixels
    pub fn from_rgb(rgb: Vec<u8>, width: u32, height: u32) -> Self {
        // Convert to RGBA by adding alpha channel
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for chunk in rgb.chunks(3) {
            rgba.push(chunk[0]);
            rgba.push(chunk[1]);
            rgba.push(chunk[2]);
            rgba.push(255); // Fully opaque
        }
        
        Self::from_rgba(rgba, width, height)
    }
    
    /// Create from image crate's DynamicImage
    ///
    /// # Parameters
    /// - `img`: image::DynamicImage instance
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Image};
    /// use image;
    ///
    /// create_scope(|| {
    ///     let img = image::load_from_memory(png_bytes).unwrap();
    ///     let sig_img = Image::from_dynamic_image(img);
    /// });
    /// ```
    #[cfg(feature = "image")]
    pub fn from_dynamic_image(img: image::DynamicImage) -> Self {
        let rgba = img.to_rgba8();
        let (width, height) = (img.width(), img.height());
        Self::from_rgba(rgba.into_raw(), width, height)
    }
    
    /// Create from peniko::ImageData
    ///
    /// This is the lowest-level creation method, directly using Vello's image data structure.
    pub fn from_image_data(image_data: ImageData) -> Self {
        let id = ViewId::new();
        
        // Set name in ViewState
        crate::runtime::with_layout_mut(|runtime| {
            if let Some(state) = runtime.view_states.get_mut(&id) {
                state.name = "Image".to_string();
            }
        });
        
        let image = Self {
            id,
            image_data,
            display_width: None,
            display_height: None,
        };
        
        // Create and register Widget
        let widget = Rc::new(ImageWidget {
            image_data: image.image_data.clone(),
            display_width: image.display_width,
            display_height: image.display_height,
        });
        
        id.widget(widget);
        
        image
    }
    
    /// Set display width
    ///
    /// # Parameters
    /// - `width`: Display width in logical pixels
    ///
    /// # Examples
    /// ```rust
    /// use sig::{create_scope, Image};
    ///
    /// create_scope(|| {
    ///     let img = Image::from_rgba(vec![], 100, 100)
    ///         .width(200.0); // Scale up to 200px
    /// });
    /// ```
    pub fn width(mut self, width: f32) -> Self {
        self.display_width = Some(width);
        self
    }
    
    /// Set display height
    ///
    /// # Parameters
    /// - `height`: Display height in logical pixels
    pub fn height(mut self, height: f32) -> Self {
        self.display_height = Some(height);
        self
    }
    
    /// Set display size (both width and height)
    ///
    /// # Parameters
    /// - `width`: Display width in logical pixels
    /// - `height`: Display height in logical pixels
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.display_width = Some(width);
        self.display_height = Some(height);
        self
    }
    
    /// Get original image width
    pub fn original_width(&self) -> u32 {
        self.image_data.width
    }
    
    /// Get original image height
    pub fn original_height(&self) -> u32 {
        self.image_data.height
    }
}

/// ImageWidget - Implements image rendering logic
struct ImageWidget {
    image_data: ImageData,
    display_width: Option<f32>,
    display_height: Option<f32>,
}

impl Widget for ImageWidget {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _style: &crate::style::Style,
        _ctx: &RenderContext,
        _font_ctx: &mut parley::FontContext,
        _layout_ctx: &mut parley::LayoutContext<()>,
    ) -> Size<f32> {
        // Get display size (use custom size if set, otherwise use original size)
        let width = if let Some(w) = known_dimensions.width {
            w
        } else if let Some(w) = self.display_width {
            w
        } else {
            self.image_data.width as f32
        };
        
        let height = if let Some(h) = known_dimensions.height {
            h
        } else if let Some(h) = self.display_height {
            h
        } else {
            self.image_data.height as f32
        };
        
        Size { width, height }
    }
    
    fn paint(
        &self,
        scene: &mut Scene,
        width: f32,
        height: f32,
        abs_x: f64,
        abs_y: f64,
        _style: &crate::style::Style,
        ctx: &RenderContext,
    ) {
        // Convert logical pixel dimensions to physical pixels
        let physical_width = ctx.logical_to_physical(width);
        let physical_height = ctx.logical_to_physical(height);
        
        // Calculate scale factors (physical display size / original image size)
        let scale_x = physical_width / self.image_data.width as f32;
        let scale_y = physical_height / self.image_data.height as f32;
        
        // Convert to physical pixel coordinates
        let physical_x = ctx.logical_to_physical(abs_x as f32) as f64;
        let physical_y = ctx.logical_to_physical(abs_y as f32) as f64;
        
        // Build transform matrix: translate to position, then scale
        let transform = Affine::translate((physical_x, physical_y))
            * Affine::scale_non_uniform(scale_x as f64, scale_y as f64);
        
        // Render image using Vello
        let brush = ImageBrush::new(self.image_data.clone());
        scene.draw_image(&brush, transform);
    }
}

/// Implement Element trait for Image
///
/// This provides Image with:
/// - Styleable: All styling methods (40+)
/// - Interactive: All event handling methods (10+)
/// - ViewTuple: Can be added as a child element to View (via blanket impl)
impl Element for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn name(&self) -> String {
        "Image".to_string()
    }
}

/// Convenience function for creating an image
///
/// # Examples
/// ```rust
/// use sig::{create_scope, image};
///
/// create_scope(|| {
///     let rgba = vec![255u8; 100 * 100 * 4];
///     let img = image(rgba, 100, 100);
/// });
/// ```
pub fn image(rgba: Vec<u8>, width: u32, height: u32) -> Image {
    Image::from_rgba(rgba, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_scope;

    #[test]
    fn test_image_from_rgba() {
        create_scope(|| {
            let rgba = vec![255u8; 100 * 100 * 4];
            let img = Image::from_rgba(rgba, 100, 100);
            assert_eq!(img.name(), "Image");
            assert_eq!(img.original_width(), 100);
            assert_eq!(img.original_height(), 100);
        });
    }

    #[test]
    fn test_image_from_rgb() {
        create_scope(|| {
            let rgb = vec![255u8; 100 * 100 * 3];
            let img = Image::from_rgb(rgb, 100, 100);
            assert_eq!(img.original_width(), 100);
            assert_eq!(img.original_height(), 100);
        });
    }

    #[test]
    fn test_image_with_custom_size() {
        create_scope(|| {
            let rgba = vec![255u8; 100 * 100 * 4];
            let img = Image::from_rgba(rgba, 100, 100)
                .width(200.0)
                .height(150.0);
            
            assert_eq!(img.display_width, Some(200.0));
            assert_eq!(img.display_height, Some(150.0));
        });
    }

    #[test]
    fn test_image_size_method() {
        create_scope(|| {
            let rgba = vec![255u8; 50 * 50 * 4];
            let img = Image::from_rgba(rgba, 50, 50)
                .size(100.0, 100.0);
            
            assert_eq!(img.display_width, Some(100.0));
            assert_eq!(img.display_height, Some(100.0));
        });
    }

    #[test]
    fn test_convenience_function() {
        create_scope(|| {
            let rgba = vec![255u8; 10 * 10 * 4];
            let img = image(rgba, 10, 10);
            assert_eq!(img.name(), "Image");
        });
    }
}
