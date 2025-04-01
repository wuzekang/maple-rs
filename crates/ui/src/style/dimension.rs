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

pub struct Length(f32);

pub fn length(value: f32) -> Length {
    Length(value)
}

pub struct Percent(f32);
pub fn percent(value: f32) -> Percent {
    Percent(value)
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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
