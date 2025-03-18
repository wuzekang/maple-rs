pub struct Color {
    components: [f32; 4],
}

impl From<[f32; 4]> for Color {
    fn from(components: [f32; 4]) -> Self {
        Self { components }
    }
}

impl From<[u8; 3]> for Color {
    fn from([r, g, b]: [u8; 3]) -> Self {
        let components = [r, g, b, 255].map(|value| value as f32 / 255.0);
        Self { components }
    }
}

impl From<u32> for Color {
    fn from(color: u32) -> Self {
        Self {
            components: [
                color >> 24 & 0xff,
                color >> 16 & 0xff,
                color >> 8 & 0xff,
                color & 0xff,
            ]
            .map(|v| v as f32 / 255.0),
        }
    }
}

impl From<peniko::Color> for Color {
    fn from(color: peniko::Color) -> Self {
        Self {
            components: color.components,
        }
    }
}

impl Into<peniko::Color> for Color {
    fn into(self) -> peniko::Color {
        peniko::Color::new(self.components)
    }
}
