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
            c"./Data/simsun.ttc".as_ptr(),
            0,
            face.as_mut_ptr(),
        );
        if error > 0 {
            panic!("Error loading font: {}", error);
        }
        let face = face.assume_init();

        let error = ft::FT_Set_Pixel_Sizes(face, 17, 0); /* set character size */
        if error > 0 {
            panic!("Error setting character size: {}", error);
        }

        println!(
            "{} {:?}",
            (*face).num_fixed_sizes,
            (*face).available_sizes.add(5).as_ref().unwrap()
        );
        let glyph_index = ft::FT_Get_Char_Index(face, '我' as ft::FT_ULong);
        println!("glyph_index: {}", glyph_index);

        let error = ft::FT_Load_Glyph(face, glyph_index, ft::FT_LOAD_RENDER as i32);
        if error > 0 {
            panic!("Error loading glyph: {}", error);
        }

        let slot = (*(*face).glyph);

        let metrics = (*(*face).size).metrics;

        //
        println!("metrics: {:?}", metrics);
        let h = (metrics.ascender - metrics.descender) >> 6;

        // let h = 18;
        // let error = ft::FT_Render_Glyph((*face).glyph, ft::FT_Render_Mode_::FT_RENDER_MODE_NORMAL);
        // if error > 0 {
        //     panic!("Error rendering glyph: {}", error);
        // }
        println!("height: {:?} {:?}", h, slot);

        let bitmap = slot.bitmap;
        let mut img = image::DynamicImage::new_luma8((slot.advance.x / 64) as u32, h as u32);
        let img = img.as_mut_luma8().unwrap();

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
                    img.get_pixel_mut(
                        j + slot.bitmap_left as u32,
                        (i as i64 + bitmap.rows as i64 - slot.bitmap_top as i64) as u32,
                    )
                    .0[0] = if (pixel << (j % 8)) & 0x80 > 0 {
                        255
                    } else {
                        0
                    }
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
                    img.get_pixel_mut(j + slot.bitmap_left as u32, i).0[0] = pixel
                }
            }
        }

        img.save("./a.png").unwrap();
        println!("Hello, world!");
    }
}
