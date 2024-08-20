use std::mem::MaybeUninit;

use freetype_sys as ft;

fn main() {
    unsafe {
        let mut library = MaybeUninit::<ft::FT_Library>::uninit();
        let error = ft::FT_Init_FreeType(library.as_mut_ptr());
        if error > 0 {
            panic!("Error initializing FreeType: {}", error);
        }

        let mut face = MaybeUninit::<ft::FT_Face>::uninit();
        let error = ft::FT_New_Face(
            library.assume_init(),
            c"./Data/SimSun1.ttf".as_ptr(),
            0,
            face.as_mut_ptr(),
        );
        if error > 0 {
            panic!("Error loading font: {}", error);
        }
        let face = face.assume_init();

        let error = ft::FT_Set_Pixel_Sizes(face, 14, 0); /* set character size */
        if error > 0 {
            panic!("Error setting character size: {}", error);
        }

        println!(
            "{} {:?}",
            (*face).num_fixed_sizes,
            (*face).available_sizes.add(5).as_ref().unwrap()
        );

        let mut m_width = 0;
        let mut m_height = 0;
        let text = "Me MaxHP：789123456很多很多";
        for c in text.chars() {
            let glyph_index = ft::FT_Get_Char_Index(face, c as ft::FT_ULong);
            println!("glyph_index: {}", glyph_index);

            let error = ft::FT_Load_Glyph(face, glyph_index, ft::FT_LOAD_NO_BITMAP as i32);
            if error > 0 {
                panic!("Error loading glyph: {}", error);
            }

            let glyph = (*(*face).glyph);
            let metrics = (*(*face).size).metrics;
            let height = (metrics.ascender - metrics.descender) >> 6;
            let width = glyph.advance.x >> 6;

            m_width += width;
            m_height = m_height.max(height);
        }

        println!("m_width: {}, m_height: {}", m_width, m_height);

        let mut img = image::DynamicImage::new_luma8((m_width) as u32, m_height as u32);
        let img = img.as_mut_luma8().unwrap();

        let mut pen_x = 0;
        let mut pen_y = 0;

        for c in text.chars() {
            let glyph_index = ft::FT_Get_Char_Index(face, c as ft::FT_ULong);
            let error = ft::FT_Load_Glyph(face, glyph_index, ft::FT_LOAD_RENDER as i32);
            if error > 0 {
                panic!("Error loading glyph: {}", error);
            }

            let error = ft::FT_Render_Glyph((*face).glyph, ft::FT_Render_Mode_::FT_RENDER_MODE_LCD);
            if error > 0 {
                panic!("Error rendering glyph: {}", error);
            }

            let slot = (*(*face).glyph);
            let metrics = (*(*face).size).metrics;

            let height = (metrics.ascender - metrics.descender) >> 6;
            println!("metrics: {metrics:?}");
            let width = slot.advance.x >> 6;

            let bitmap = slot.bitmap;

            println!("{bitmap:?}");

            if bitmap.pixel_mode == ft::FT_Pixel_Mode_::FT_PIXEL_MODE_MONO as u8 {
                for i in 0..bitmap.rows {
                    for j in 0..bitmap.width {
                        let pixel = {
                            bitmap
                                .buffer
                                .add(i as usize * bitmap.pitch as usize)
                                .add(j as usize / 8)
                                .read()
                        };
                        println!(
                            "{} {i} {} {}",
                            (pen_y + i as i32 + bitmap.rows as i32 - slot.bitmap_top as i32),
                            bitmap.rows,
                            slot.bitmap_top
                        );
                        img.get_pixel_mut(
                            (pen_x + j as i32 + slot.bitmap_left) as u32,
                            (pen_y + i as i32 + (metrics.ascender >> 6) as i32
                                - slot.bitmap_top as i32) as u32,
                        )
                        .0[0] = if (pixel << (j % 8)) & 0x80 > 0 {
                            255
                        } else {
                            0
                        };
                    }
                }
            }

            if bitmap.pixel_mode == ft::FT_Pixel_Mode_::FT_PIXEL_MODE_GRAY as u8 {
                for i in 0..bitmap.rows {
                    for j in 0..bitmap.width {
                        let pixel = {
                            bitmap
                                .buffer
                                .add(i as usize * bitmap.pitch as usize)
                                .add(j as usize)
                                .read()
                        };
                        img.get_pixel_mut(
                            (pen_x + j as i32 + slot.bitmap_left) as u32,
                            (pen_y + i as i32 + (metrics.ascender >> 6) as i32
                                - slot.bitmap_top as i32) as u32,
                        )
                        .0[0] = pixel
                    }
                }
            }

            pen_x += (slot.advance.x >> 6) as i32;
            pen_y += (slot.advance.y >> 6) as i32;

            println!("pen_x: {}, pen_y: {}", pen_x, pen_y);
        }

        for i in img.iter_mut() {
            *i = 255 - *i;
        }

        img.save("./font.png").unwrap();
    }
}
