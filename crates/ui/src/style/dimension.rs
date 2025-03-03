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

impl Into<taffy::Dimension> for Dimension {
    fn into(self) -> taffy::Dimension {
        self.0
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

impl Into<taffy::LengthPercentageAuto> for LengthPercentageAuto {
    fn into(self) -> taffy::LengthPercentageAuto {
        self.0
    }
}

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

impl From<taffy::LengthPercentage> for LengthPercentage {
    fn from(value: taffy::LengthPercentage) -> Self {
        Self(value)
    }
}

impl Into<taffy::LengthPercentage> for LengthPercentage {
    fn into(self) -> taffy::LengthPercentage {
        self.0
    }
}
