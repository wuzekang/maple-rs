use image::{Rgb, Rgba};

pub trait SimpleColor
where
    Self: Sized,
{
    fn create(r: u8, g: u8, b: u8) -> Self;
    fn white() -> Self;
    fn black() -> Self;
    fn from_rgb565(color: u16) -> Self {
        let r = ((color & 0xF800) >> 11) as u8;
        let g = ((color & 0x07E0) >> 5) as u8;
        let b = (color & 0x001F) as u8;

        Self::create(r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)
    }
    fn r(&self) -> u8;
    fn g(&self) -> u8;
    fn b(&self) -> u8;
}

pub trait SimpleColorAlpha
where
    Self: Sized,
{
    fn create_alpha(r: u8, g: u8, b: u8, a: u8) -> Self;
    fn transparent() -> Self;
    fn from_argb1555(color: u16) -> Self {
        let a = if (color & 0x8000) != 0 { 255 } else { 0 };
        let r = ((color & 0x7C00) >> 10) as u8;
        let g = ((color & 0x03E0) >> 5) as u8;
        let b = (color & 0x001F) as u8;

        Self::create_alpha(r << 3 | r >> 2, g << 3 | g >> 2, b << 3 | b >> 2, a)
    }
    fn a(&self) -> u8;
}

impl SimpleColor for Rgba<u8> {
    fn create(r: u8, g: u8, b: u8) -> Self {
        Rgba([r, g, b, 255])
    }
    fn white() -> Self {
        Rgba([255, 255, 255, 255])
    }
    fn black() -> Self {
        Rgba([0, 0, 0, 255])
    }
    fn r(&self) -> u8 {
        self[0]
    }
    fn g(&self) -> u8 {
        self[1]
    }
    fn b(&self) -> u8 {
        self[2]
    }
}

impl SimpleColorAlpha for Rgba<u8> {
    fn create_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Rgba([r, g, b, a])
    }
    fn transparent() -> Self {
        Rgba([0, 0, 0, 0])
    }
    fn a(&self) -> u8 {
        self[3]
    }
}

impl SimpleColor for Rgb<u8> {
    fn create(r: u8, g: u8, b: u8) -> Self {
        Rgb([r, g, b])
    }
    fn white() -> Self {
        Rgb([255, 255, 255])
    }
    fn black() -> Self {
        Rgb([0, 0, 0])
    }
    fn r(&self) -> u8 {
        self[0]
    }
    fn g(&self) -> u8 {
        self[1]
    }
    fn b(&self) -> u8 {
        self[2]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use image::{Rgb, Rgba};

    const RED: u32 = 0xFF0000;
    const GREEN: u32 = 0x00FF00;
    const BLUE: u32 = 0x0000FF;
    const GRAY: u32 = 0x808080;

    const ARED: u32 = 0xFF000000;
    const AGREEN: u32 = 0x00FF0000;
    const ABLUE: u32 = 0x0000FF00;
    const AGRAY: u32 = 0x80808000;
    const TGRAY: u32 = 0x808080FF; // Transparent
    const HGRAY: u32 = 0x80808080; // Half Transparent

    fn create_rgb565_from_rgb(color: u32) -> u16 {
        (((color & 0xf80000) >> 8) + ((color & 0xfc00) >> 5) + ((color & 0xf8) >> 3)) as u16
    }

    fn create_argb1555_from_rgba(color: u32) -> u16 {
        let r = (((color & 0xff000000) >> 24) as u16 * 31 + 127) / 255;
        let g = (((color & 0x00ff0000) >> 16) as u16 * 31 + 127) / 255;
        let b = (((color & 0x0000ff00) >> 8) as u16 * 31 + 127) / 255;

        let a = if color & 0xff > 0 { 0 } else { 1 };

        dbg!((a << 15) | (r << 10) | (g << 5) | b)
    }

    #[test]
    fn rgb_from_rgb565() {
        let red = Rgb::from_rgb565(create_rgb565_from_rgb(RED));
        assert_eq!(red, Rgb([255, 0, 0]));

        let green = Rgb::from_rgb565(create_rgb565_from_rgb(GREEN));
        assert_eq!(green, Rgb([0, 255, 0]));

        let blue = Rgb::from_rgb565(create_rgb565_from_rgb(BLUE));
        assert_eq!(blue, Rgb([0, 0, 255]));

        let gray = Rgb::from_rgb565(create_rgb565_from_rgb(GRAY));
        assert_eq!(gray, Rgb([132, 130, 132]));
    }

    #[test]
    fn rgba_from_rgba1555() {
        let red = Rgba::from_argb1555(create_argb1555_from_rgba(ARED));
        assert_eq!(red, Rgba([255, 0, 0, 255]));

        let green = Rgba::from_argb1555(create_argb1555_from_rgba(AGREEN));
        assert_eq!(green, Rgba([0, 255, 0, 255]));

        let blue = Rgba::from_argb1555(create_argb1555_from_rgba(ABLUE));
        assert_eq!(blue, Rgba([0, 0, 255, 255]));

        let gray = Rgba::from_argb1555(create_argb1555_from_rgba(AGRAY));
        assert_eq!(gray, Rgba([132, 132, 132, 255]));

        let tgray = Rgba::from_argb1555(create_argb1555_from_rgba(TGRAY));
        assert_eq!(tgray, Rgba([132, 132, 132, 0]));

        // helf gray should turn to complete transparent
        let hgray = Rgba::from_argb1555(create_argb1555_from_rgba(HGRAY));
        assert_eq!(hgray, Rgba([132, 132, 132, 0]));
    }
}
