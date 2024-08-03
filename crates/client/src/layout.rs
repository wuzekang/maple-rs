use ab_glyph::{point, Font, Glyph, Point, PxScale, ScaleFont};
use image::{DynamicImage, Rgba, RgbaImage};

/// Simple paragraph layout for glyphs into `target`.
/// Starts at position `(0, ascent)`.
///
/// This is for testing and examples.
pub fn layout_paragraph<F, SF>(
    font: SF,
    position: Point,
    max_width: f32,
    text: &str,
    target: &mut Vec<Glyph>,
) where
    F: Font,
    SF: ScaleFont<F>,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        if !c.is_whitespace() && caret.x > position.x + max_width {
            caret = point(position.x, caret.y + v_advance);
            glyph.position = caret;
            last_glyph = None;
        }

        target.push(glyph);
    }
}

pub fn draw_image<F: Font>(font: F, font_size: f32, text: &str) -> RgbaImage {
    /// Dark red colour
    const COLOUR: (u8, u8, u8) = (0, 0, 0);

    // The font size to use
    let scale = PxScale::from(font_size);

    let scaled_font = font.as_scaled(scale);

    let mut glyphs = Vec::new();
    layout_paragraph(scaled_font, point(0.0, 0.0), 10000.0, text, &mut glyphs);

    // to work out the exact size needed for the drawn glyphs we need to outline
    // them and use their `px_bounds` which hold the coords of their render bounds.
    let outlined: Vec<_> = glyphs
        .into_iter()
        // Note: not all layout glyphs have outlines (e.g. " ")
        .filter_map(|g| font.outline_glyph(g))
        .collect();

    // combine px_bounds to get min bounding coords for the entire layout
    let Some(all_px_bounds) = outlined
        .iter()
        .map(|g| g.px_bounds())
        .reduce(|mut b, next| {
            b.min.x = b.min.x.min(next.min.x);
            b.max.x = b.max.x.max(next.max.x);
            b.min.y = b.min.y.min(next.min.y);
            b.max.y = b.max.y.max(next.max.y);
            b
        })
    else {
        panic!("No outlined glyphs?");
    };

    // create a new rgba image using the combined px bound width and height
    let mut image =
        DynamicImage::new_rgba8(all_px_bounds.width() as _, all_px_bounds.height() as _).to_rgba8();

    // Loop through the glyphs in the text, positing each one on a line
    for glyph in outlined {
        let bounds = glyph.px_bounds();
        // calc top/left ords in "image space"
        // image-x=0 means the *left most pixel*, equivalent to
        // px_bounds.min.x which *may be non-zero* (and similarly with y)
        // so `- px_bounds.min` converts the left-most/top-most to 0
        let img_left = bounds.min.x as u32 - all_px_bounds.min.x as u32;
        let img_top = bounds.min.y as u32 - all_px_bounds.min.y as u32;
        // Draw the glyph into the image per-pixel by using the draw closure
        glyph.draw(|x, y, v| {
            // Offset the position by the glyph bounding box
            let px = image.get_pixel_mut(img_left + x, img_top + y);
            // Turn the coverage into an alpha value (blended with any previous)
            *px = Rgba([
                COLOUR.0,
                COLOUR.1,
                COLOUR.2,
                px.0[3].saturating_add((v * 255.0) as u8),
            ]);
        });
    }

    image
}
