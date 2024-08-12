use std::{borrow::Borrow, mem::MaybeUninit};

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

        let error = ft::FT_Set_Pixel_Sizes(face, 9, 17); /* set character size */
        if error > 0 {
            panic!("Error setting character size: {}", error);
        }

        println!(
            "{} {:?}",
            (*face).num_fixed_sizes,
            (*face).available_sizes.add(5).as_ref().unwrap()
        );
        let glyph_index = ft::FT_Get_Char_Index(face, '钅' as ft::FT_ULong);
        println!("glyph_index: {}", glyph_index);

        let error = ft::FT_Load_Glyph(face, glyph_index, ft::FT_LOAD_DEFAULT as i32);
        if error > 0 {
            panic!("Error loading glyph: {}", error);
        }

        let error = ft::FT_Render_Glyph((*face).glyph, ft::FT_Render_Mode_::FT_RENDER_MODE_LCD);
        if error > 0 {
            panic!("Error rendering glyph: {}", error);
        }

        let bitmap = (*(*face).glyph).bitmap;
        println!("{:?}", (*(*face).glyph));

        for i in 0..bitmap.rows {
            for j in 0..bitmap.width {
                let pixel = {
                    bitmap
                        .buffer
                        .add(i as usize * bitmap.pitch as usize)
                        .add(j as usize)
                        .read()
                };
                print!("{}", if pixel < 80 { ' ' } else { '*' });
            }
            println!();
        }
    }

    println!("Hello, world!");
}
