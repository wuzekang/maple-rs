//! Dimension module - Provides convenient dimension type conversions
//!
//! Reference implementation: crates/ui/src/style/dimension.rs

/// Dimension wrapper - Supports conversion from f32 and i32
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Dimension(taffy::Dimension);

impl From<f32> for Dimension {
    fn from(value: f32) -> Self {
        Self(taffy::Dimension::Length(value))
    }
}

impl From<i32> for Dimension {
    fn from(value: i32) -> Self {
        Self(taffy::Dimension::Length(value as f32))
    }
}

impl From<Length> for Dimension {
    fn from(Length(value): Length) -> Self {
        Self(taffy::Dimension::Length(value))
    }
}

impl From<Percent> for Dimension {
    fn from(Percent(value): Percent) -> Self {
        Self(taffy::Dimension::Percent(value))
    }
}

impl From<taffy::Dimension> for Dimension {
    fn from(dimension: taffy::Dimension) -> Self {
        Self(dimension)
    }
}

impl From<Dimension> for taffy::Dimension {
    fn from(val: Dimension) -> Self {
        val.0
    }
}

/// Length type - Represents a fixed length
pub struct Length(f32);

/// Create a fixed length value
pub fn length(value: f32) -> Length {
    Length(value)
}

/// Percent type - Represents a percentage
pub struct Percent(f32);

/// Create a percentage value
/// 
/// # Arguments
/// * `value` - Percentage as a decimal (0.5 = 50%, 1.0 = 100%)
/// 
/// # Examples
/// ```
/// percent(0.5)  // 50%
/// percent(1.0)  // 100%
/// percent(-0.5) // -50%
/// ```
pub fn percent(value: f32) -> Percent {
    Percent(value)
}

/// LengthPercentageAuto wrapper
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LengthPercentageAuto(taffy::LengthPercentageAuto);

impl From<Length> for LengthPercentageAuto {
    fn from(Length(value): Length) -> Self {
        Self(taffy::LengthPercentageAuto::Length(value))
    }
}

impl From<Percent> for LengthPercentageAuto {
    fn from(Percent(value): Percent) -> Self {
        Self(taffy::LengthPercentageAuto::Percent(value))
    }
}

impl From<f32> for LengthPercentageAuto {
    fn from(value: f32) -> Self {
        Self(taffy::LengthPercentageAuto::Length(value))
    }
}

impl From<i32> for LengthPercentageAuto {
    fn from(value: i32) -> Self {
        Self(taffy::LengthPercentageAuto::Length(value as f32))
    }
}

impl From<usize> for LengthPercentageAuto {
    fn from(value: usize) -> Self {
        Self(taffy::LengthPercentageAuto::Length(value as f32))
    }
}

impl From<LengthPercentageAuto> for taffy::LengthPercentageAuto {
    fn from(val: LengthPercentageAuto) -> Self {
        val.0
    }
}

impl From<taffy::LengthPercentageAuto> for LengthPercentageAuto {
    fn from(value: taffy::LengthPercentageAuto) -> Self {
        Self(value)
    }
}

/// LengthPercentage wrapper
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LengthPercentage(taffy::LengthPercentage);

impl From<Length> for LengthPercentage {
    fn from(Length(value): Length) -> Self {
        Self(taffy::LengthPercentage::Length(value))
    }
}

impl From<Percent> for LengthPercentage {
    fn from(Percent(value): Percent) -> Self {
        Self(taffy::LengthPercentage::Percent(value))
    }
}

impl From<f32> for LengthPercentage {
    fn from(value: f32) -> Self {
        Self(taffy::LengthPercentage::Length(value))
    }
}

impl From<i32> for LengthPercentage {
    fn from(value: i32) -> Self {
        Self(taffy::LengthPercentage::Length(value as f32))
    }
}

impl From<usize> for LengthPercentage {
    fn from(value: usize) -> Self {
        Self(taffy::LengthPercentage::Length(value as f32))
    }
}

impl From<taffy::LengthPercentage> for LengthPercentage {
    fn from(value: taffy::LengthPercentage) -> Self {
        Self(value)
    }
}

impl From<LengthPercentage> for taffy::LengthPercentage {
    fn from(val: LengthPercentage) -> Self {
        val.0
    }
}

/// Rect wrapper - Supports convenient unified value conversion
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect<T: Clone + Copy> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T: Into<Dimension> + Clone> From<T> for Rect<Dimension> {
    fn from(value: T) -> Self {
        Self {
            left: value.clone().into(),
            top: value.clone().into(),
            right: value.clone().into(),
            bottom: value.clone().into(),
        }
    }
}

impl<T: Into<LengthPercentage> + Clone> From<T> for Rect<LengthPercentage> {
    fn from(value: T) -> Self {
        Self {
            left: value.clone().into(),
            top: value.clone().into(),
            right: value.clone().into(),
            bottom: value.clone().into(),
        }
    }
}

impl<T: Into<LengthPercentage> + Clone> From<[T; 2]> for Rect<LengthPercentage> {
    fn from([v, h]: [T; 2]) -> Self {
        Self {
            left: h.clone().into(),
            top: v.clone().into(),
            right: h.clone().into(),
            bottom: v.clone().into(),
        }
    }
}

impl<T: Into<LengthPercentageAuto> + Clone> From<T> for Rect<LengthPercentageAuto> {
    fn from(value: T) -> Self {
        Self {
            left: value.clone().into(),
            top: value.clone().into(),
            right: value.clone().into(),
            bottom: value.clone().into(),
        }
    }
}

impl<T: Into<LengthPercentageAuto> + Clone> From<[T; 2]> for Rect<LengthPercentageAuto> {
    fn from([v, h]: [T; 2]) -> Self {
        Self {
            left: h.clone().into(),
            top: v.clone().into(),
            right: h.clone().into(),
            bottom: v.clone().into(),
        }
    }
}

impl<U, T: Into<U> + Clone + Copy> From<Rect<T>> for taffy::Rect<U> {
    fn from(val: Rect<T>) -> Self {
        taffy::Rect {
            left: val.left.into(),
            top: val.top.into(),
            right: val.right.into(),
            bottom: val.bottom.into(),
        }
    }
}

/// Size wrapper - Supports convenient size setting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size<T: Clone + Copy> {
    pub width: T,
    pub height: T,
}

// Convenient conversion for Size<LengthPercentage>
impl From<f32> for Size<LengthPercentage> {
    fn from(value: f32) -> Self {
        Size {
            width: LengthPercentage::from(value),
            height: LengthPercentage::from(value),
        }
    }
}

impl From<i32> for Size<LengthPercentage> {
    fn from(value: i32) -> Self {
        Size {
            width: LengthPercentage::from(value),
            height: LengthPercentage::from(value),
        }
    }
}

impl From<[f32; 2]> for Size<LengthPercentage> {
    fn from([width, height]: [f32; 2]) -> Self {
        Size {
            width: LengthPercentage::from(width),
            height: LengthPercentage::from(height),
        }
    }
}

impl From<[i32; 2]> for Size<LengthPercentage> {
    fn from([width, height]: [i32; 2]) -> Self {
        Size {
            width: LengthPercentage::from(width),
            height: LengthPercentage::from(height),
        }
    }
}

// Convert to taffy::Size - concrete implementation
impl From<Size<LengthPercentage>> for taffy::Size<taffy::LengthPercentage> {
    fn from(val: Size<LengthPercentage>) -> Self {
        taffy::Size {
            width: val.width.into(),
            height: val.height.into(),
        }
    }
}

/// Helper function: Create Size from a single value (used for gap, etc.)
pub fn size_all<T: Into<taffy::LengthPercentage>>(value: T) -> taffy::Size<taffy::LengthPercentage> {
    let val = value.into();
    taffy::Size {
        width: val,
        height: val,
    }
}
