use crate::property::string::resolve_string_from_node;
use crate::util::color::{SimpleColor, SimpleColorAlpha};
use crate::{
    property::WzSubProperty,
    reader::{self, Reader},
    util::node_util,
    WzNodeArc, WzObjectType,
};
use flate2::{Decompress, FlushDecompress};
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum WzPngParseError {
    #[error("inflate raw data failed")]
    InflateError(#[from] flate2::DecompressError),

    #[error("Unknown format: {0}")]
    UnknownFormat(u32),

    #[error("Unsupported header: {0}")]
    UnsupportedHeader(i32),

    #[error("Error reading color: {0}")]
    ReadColorError(#[from] reader::Error),

    #[error(transparent)]
    SaveError(#[from] image::ImageError),

    #[error("Can't not resolve _inlink or _outlink")]
    LinkError,

    #[error("Not a PNG property")]
    NotPngProperty,
}

type ImageBufferRgbaChunk = ImageBuffer<Rgba<u8>, Vec<u8>>;
type ImageBufferRgbChunk = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// A helper get image from `WzNodeArc`, will also resolve `_inlink` or `_outlink`
pub fn get_image(node: &WzNodeArc) -> Result<DynamicImage, WzPngParseError> {
    let node_read = node.read().unwrap();
    match &node_read.object_type {
        WzObjectType::Property(WzSubProperty::PNG(png)) => {
            let inlink_target = node_read
                .at("_inlink")
                .and_then(|node| resolve_string_from_node(&node).ok())
                .and_then(|inlink| node_util::resolve_inlink(&inlink, node));

            if let Some(target) = inlink_target {
                return get_image(&target);
            }

            let outlink_target = node_read
                .at("_outlink")
                .and_then(|node| resolve_string_from_node(&node).ok())
                .and_then(|outlink| node_util::resolve_outlink(&outlink, node, true));

            if let Some(target) = outlink_target {
                return get_image(&target);
            }

            png.extract_png()
        }
        _ => Err(WzPngParseError::NotPngProperty),
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct WzPng {
    #[cfg_attr(feature = "serde", serde(skip))]
    reader: Arc<reader::WzBytesReader>,
    #[cfg_attr(feature = "serde", serde(skip))]
    offset: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    block_size: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    format1: u32,
    #[cfg_attr(feature = "serde", serde(skip))]
    format2: u32,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub header: i32,

    pub width: u32,
    pub height: u32,
}

impl WzPng {
    pub fn new(
        reader: &Arc<reader::WzBytesReader>,
        size: (u32, u32),
        format: (u32, u32),
        data_range: (usize, usize),
        header: i32,
    ) -> WzPng {
        WzPng {
            reader: Arc::clone(reader),
            offset: data_range.0,
            block_size: data_range.1,
            width: size.0,
            height: size.1,
            format1: format.0,
            format2: format.1,
            header,
        }
    }
    pub fn format(&self) -> u32 {
        self.format1 + self.format2
    }
    fn has_zlib_header(&self) -> bool {
        self.header == 0x9C78
            || self.header == 0xDA78
            || self.header == 0x0178
            || self.header == 0x5E78
    }
    pub fn extract_png(&self) -> Result<DynamicImage, WzPngParseError> {
        let data = self
            .reader
            .get_slice(self.offset..(self.offset + self.block_size));
        /* decompress */
        let pixels = self.get_raw_data(data)?;

        match self.format() {
            1 => get_image_from_bgra4444(pixels, self.width, self.height),
            2 => get_image_from_bgra8888(pixels, self.width, self.height),
            3 | 1026 => get_image_from_dxt3(&pixels, self.width, self.height),
            257 => get_image_from_argb1555(&pixels, self.width, self.height),
            513 => get_image_from_rgb565(&pixels, self.width, self.height),
            517 => {
                let decoded = get_pixel_data_form_517(&pixels, self.width, self.height);
                get_image_from_rgb565(&decoded, self.width, self.height)
            }
            2050 => get_image_from_dxt5(&pixels, self.width, self.height),
            _ => Err(WzPngParseError::UnknownFormat(self.format())),
        }
    }
    fn get_buff_size(&self) -> Result<usize, WzPngParseError> {
        match self.format() {
            1 | 257 | 513 => Ok((self.width * self.height * 2) as usize),
            2 => Ok((self.width * self.height * 4) as usize),
            3 => Ok((self.width * self.height * 4) as usize),
            1026 | 2050 => Ok((self.width * self.height) as usize),
            517 => {
                /* 128 = 16 * 16 / 2 */
                Ok((self.width * self.height / 128) as usize)
            }
            _ => Err(WzPngParseError::UnknownFormat(self.format())),
        }
    }
    fn get_raw_data(&self, data: &[u8]) -> Result<Vec<u8>, WzPngParseError> {
        let capacity = self.get_buff_size()?;

        if self.has_zlib_header() {
            inflate(true, data, capacity)
        } else {
            let mut keys = self.reader.keys.write().unwrap();

            let total_end = self.offset + self.block_size;

            let mut offset = self.offset;
            let mut end = 0;

            let mut decrypted = Vec::with_capacity(self.block_size);

            while offset < total_end {
                let block_size = self.reader.read_i32_at(offset).unwrap() as usize;
                offset += 4;

                let data = self.reader.get_slice(offset..(offset + block_size));
                offset += block_size;

                decrypted.extend_from_slice(data);

                keys.ensure_key_size(data.len()).unwrap();

                keys.decrypt_slice(&mut decrypted[end..(end + block_size)]);

                end += block_size;
            }

            /* the total chunk shoud start decryption at index 2 */
            inflate(false, &decrypted[2..], capacity)
        }
    }
}

fn inflate(with_header: bool, data: &[u8], capacity: usize) -> Result<Vec<u8>, WzPngParseError> {
    let mut deflater = Decompress::new(with_header);
    let mut result = Vec::with_capacity(capacity);

    if let Err(e) = deflater.decompress_vec(data, &mut result, FlushDecompress::Sync) {
        return Err(WzPngParseError::from(e));
    };

    Ok(result)
}

fn get_image_from_bgra4444(
    raw_data: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<DynamicImage, WzPngParseError> {
    let imgbuffer = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            image::ImageBuffer::from_par_fn(width, height, |x, y| {
                let i = (x + y * width) as usize * 2;
                let pixel = raw_data[i];

                let b = pixel & 0x0F;
                let b = b | (b << 4);

                let g = pixel & 0xF0;
                let g = g | (g >> 4);

                let pixel = raw_data[i + 1];

                let r = pixel & 0x0F;
                let r = r | (r << 4);

                let a = pixel & 0xF0;
                let a = a | (a >> 4);

                image::Rgba([r, g, b, a])
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            image::ImageBuffer::from_fn(width, height, |x, y| {
                let i = (x + y * width) as usize * 2;
                let pixel = raw_data[i];

                let b = pixel & 0x0F;
                let b = b | (b << 4);

                let g = pixel & 0xF0;
                let g = g | (g >> 4);

                let pixel = raw_data[i + 1];

                let r = pixel & 0x0F;
                let r = r | (r << 4);

                let a = pixel & 0xF0;
                let a = a | (a >> 4);

                image::Rgba([r, g, b, a])
            })
        }
    };

    Ok(imgbuffer.into())
}

fn get_image_from_dxt3(
    raw_data: &[u8],
    width: u32,
    height: u32,
) -> Result<DynamicImage, WzPngParseError> {
    #[cfg(not(target_arch = "wasm32"))]
    let image_buffer_chunks = raw_data
        .par_chunks(16)
        .try_fold_with::<_, Vec<ImageBufferRgbaChunk>, Result<_, WzPngParseError>>(
            Vec::new(),
            |mut v, chunk| {
                let alpha_table = create_alpha_table_dxt3(chunk, 0);

                let u0: u16 = reader::read_u16_at(chunk, 8)?;
                let u1: u16 = reader::read_u16_at(chunk, 10)?;

                let color_table = create_color_table(u0, u1);
                let color_idx_table = create_color_index_table(chunk, 12);

                let mut img_buffer = image::ImageBuffer::new(4, 4);
                for y in 0..4 {
                    for x in 0..4 {
                        let idx = (y * 4 + x) as usize;
                        let color_idx = color_idx_table[idx] as usize;
                        let color = color_table[color_idx];
                        let alpha = alpha_table[idx];

                        img_buffer.put_pixel(
                            x,
                            y,
                            image::Rgba([color.r(), color.g(), color.b(), alpha]),
                        );
                    }
                }
                v.push(img_buffer);
                Ok(v)
            },
        )
        .try_reduce(Vec::new, |mut acc, v| {
            acc.extend(v);
            Ok(acc)
        })?;

    #[cfg(target_arch = "wasm32")]
    let image_buffer_chunks: Result<Vec<ImageBufferRgbaChunk>, WzPngParseError> = {
        let mut chunks = Vec::new();
        for chunk in raw_data.chunks(16) {
            let alpha_table = create_alpha_table_dxt3(chunk, 0);
            let u0: u16 = reader::read_u16_at(chunk, 8)?;
            let u1: u16 = reader::read_u16_at(chunk, 10)?;
            let color_table = create_color_table(u0, u1);
            let color_idx_table = create_color_index_table(chunk, 12);
            
            let mut img_buffer = image::ImageBuffer::new(4, 4);
            for y in 0..4 {
                for x in 0..4 {
                    let idx = (y * 4 + x) as usize;
                    let color_idx = color_idx_table[idx] as usize;
                    let color = color_table[color_idx];
                    let alpha = alpha_table[idx];
                    
                    img_buffer.put_pixel(
                        x,
                        y,
                        image::Rgba([color.r(), color.g(), color.b(), alpha]),
                    );
                }
            }
            chunks.push(img_buffer);
        }
        Ok(chunks)
    };
    
    #[cfg(target_arch = "wasm32")]
    let image_buffer_chunks = image_buffer_chunks?;

    // combine image buffer
    let grid_row_count = width / 4;
    #[cfg(not(target_arch = "wasm32"))]
    let img_buffer = image::ImageBuffer::from_par_fn(width, height, |x, y| {
        *image_buffer_chunks[(x / 4 + y / 4 * grid_row_count) as usize].get_pixel(x % 4, y % 4)
    });
    #[cfg(target_arch = "wasm32")]
    let img_buffer = image::ImageBuffer::from_fn(width, height, |x, y| {
        *image_buffer_chunks[(x / 4 + y / 4 * grid_row_count) as usize].get_pixel(x % 4, y % 4)
    });

    Ok(img_buffer.into())
}

fn get_image_from_dxt5(
    raw_data: &[u8],
    width: u32,
    height: u32,
) -> Result<DynamicImage, WzPngParseError> {
    #[cfg(not(target_arch = "wasm32"))]
    let image_buffer_chunks = raw_data
        .par_chunks(16)
        .try_fold_with::<_, Vec<ImageBufferRgbaChunk>, Result<_, WzPngParseError>>(
            Vec::new(),
            |mut v, chunk| {
                let alpha_table = create_alpha_table_dxt5(chunk[0], chunk[1]);
                let alpha_idx_table = create_alpha_index_table_dxt5(chunk, 2);

                let u0: u16 = reader::read_u16_at(chunk, 8)?;
                let u1: u16 = reader::read_u16_at(chunk, 10)?;

                let color_table = create_color_table(u0, u1);
                let color_idx_table = create_color_index_table(chunk, 12);

                let mut img_buffer = image::ImageBuffer::new(4, 4);
                for y in 0..4 {
                    for x in 0..4 {
                        let idx = (y * 4 + x) as usize;
                        let color_idx = color_idx_table[idx] as usize;
                        let color = color_table[color_idx];
                        let alpha_idx = alpha_idx_table[idx] as usize;
                        let alpha = alpha_table[alpha_idx];

                        img_buffer.put_pixel(
                            x,
                            y,
                            image::Rgba([color.r(), color.g(), color.b(), alpha]),
                        );
                    }
                }
                v.push(img_buffer);
                Ok(v)
            },
        )
        .try_reduce(Vec::new, |mut acc, v| {
            acc.extend(v);
            Ok(acc)
        })?;

    #[cfg(target_arch = "wasm32")]
    let image_buffer_chunks: Result<Vec<ImageBufferRgbaChunk>, WzPngParseError> = {
        let mut chunks = Vec::new();
        for chunk in raw_data.chunks(16) {
            let alpha_table = create_alpha_table_dxt5(chunk[0], chunk[1]);
            let alpha_idx_table = create_alpha_index_table_dxt5(chunk, 2);
            let u0: u16 = reader::read_u16_at(chunk, 8)?;
            let u1: u16 = reader::read_u16_at(chunk, 10)?;
            let color_table = create_color_table(u0, u1);
            let color_idx_table = create_color_index_table(chunk, 12);
            
            let mut img_buffer = image::ImageBuffer::new(4, 4);
            for y in 0..4 {
                for x in 0..4 {
                    let idx = (y * 4 + x) as usize;
                    let color_idx = color_idx_table[idx] as usize;
                    let color = color_table[color_idx];
                    let alpha_idx = alpha_idx_table[idx] as usize;
                    let alpha = alpha_table[alpha_idx];
                    
                    img_buffer.put_pixel(
                        x,
                        y,
                        image::Rgba([color.r(), color.g(), color.b(), alpha]),
                    );
                }
            }
            chunks.push(img_buffer);
        }
        Ok(chunks)
    };
    
    #[cfg(target_arch = "wasm32")]
    let image_buffer_chunks = image_buffer_chunks?;

    let grid_row_count = width / 4;
    #[cfg(not(target_arch = "wasm32"))]
    let img_buffer = image::ImageBuffer::from_par_fn(width, height, |x, y| {
        *image_buffer_chunks[(x / 4 + y / 4 * grid_row_count) as usize].get_pixel(x % 4, y % 4)
    });
    #[cfg(target_arch = "wasm32")]
    let img_buffer = image::ImageBuffer::from_fn(width, height, |x, y| {
        *image_buffer_chunks[(x / 4 + y / 4 * grid_row_count) as usize].get_pixel(x % 4, y % 4)
    });

    Ok(img_buffer.into())
}

fn get_pixel_data_form_517(raw_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    /* pixels is 256 times of raw_data */
    let mut pixels: Vec<u8> = vec![0; (width * height * 2) as usize];
    let mut line_index: usize = 0;
    // 16 probably means rgb565, it store 2 bytes for each pixel.
    let j_steps = (height / 16) as usize;
    let i_steps = (width / 16) as usize;
    for j in 0..j_steps {
        let mut dst_idx = line_index;
        for i in 0..i_steps {
            let idx = (i + j * i_steps) * 2;

            for _ in 0..16 {
                pixels[dst_idx] = raw_data[idx];
                dst_idx += 1;
                pixels[dst_idx] = raw_data[idx + 1];
                dst_idx += 1;
            }
        }
        // dst_idx totally add (width / 16) * 2 * 16 => width * 2 here

        // copy data from previous loop to next 16 chunks
        // don't know why it start from 1, and plus 32(16*2) later.
        for _ in 1..16 {
            let copy_len = (width * 2) as usize;
            let source_range = line_index..(line_index + copy_len);
            // copy source_range to dst_idx..(dst_idx + copy_len), and dst_idx would add copy_len in this loop.
            for suroce_idx in source_range {
                pixels[dst_idx] = pixels[suroce_idx];
                dst_idx += 1;
            }
        }

        line_index += (width * 32) as usize;
    }

    pixels
}

fn create_color_table(c0: u16, c1: u16) -> [Rgb<u8>; 4] {
    let color1 = Rgb::<u8>::from_rgb565(c0);
    let color2 = Rgb::<u8>::from_rgb565(c1);
    let color3: Rgb<u8>;
    let color4: Rgb<u8>;

    let r = color1.r() as i32;
    let g = color1.g() as i32;
    let b = color1.b() as i32;

    let r1 = color2.r() as i32;
    let g1 = color2.g() as i32;
    let b1 = color2.b() as i32;

    if c0 > c1 {
        color3 = Rgb([
            ((r * 2 + r1 + 1) / 3) as u8,
            ((g * 2 + g1 + 1) / 3) as u8,
            ((b * 2 + b1 + 1) / 3) as u8,
        ]);
        color4 = Rgb([
            ((r + r1 * 2 + 1) / 3) as u8,
            ((g + g1 * 2 + 1) / 3) as u8,
            ((b + b1 * 2 + 1) / 3) as u8,
        ]);
    } else {
        color3 = Rgb([
            ((r + r1) / 2) as u8,
            ((g + g1) / 2) as u8,
            ((b + b1) / 2) as u8,
        ]);
        color4 = Rgb::<u8>::black();
    }

    [color1, color2, color3, color4]
}

#[allow(dead_code)]
fn expand_color_table(color_table: &mut [Rgb<u8>; 4], c0: u16, c1: u16) {
    color_table[0] = Rgb::from_rgb565(c0);
    color_table[1] = Rgb::from_rgb565(c1);

    let r = color_table[0].r() as i32;
    let g = color_table[0].g() as i32;
    let b = color_table[0].b() as i32;

    let r1 = color_table[1].r() as i32;
    let g1 = color_table[1].g() as i32;
    let b1 = color_table[1].b() as i32;

    if c0 > c1 {
        color_table[2] = Rgb([
            ((r * 2 + r1 + 1) / 3) as u8,
            ((g * 2 + g1 + 1) / 3) as u8,
            ((b * 2 + b1 + 1) / 3) as u8,
        ]);
        color_table[3] = Rgb([
            ((r + r1 * 2 + 1) / 3) as u8,
            ((g + g1 * 2 + 1) / 3) as u8,
            ((b + b1 * 2 + 1) / 3) as u8,
        ]);
    } else {
        color_table[2] = Rgb([
            ((r + r1) / 2) as u8,
            ((g + g1) / 2) as u8,
            ((b + b1) / 2) as u8,
        ]);
        color_table[3] = Rgb::black();
    }
}

fn create_color_index_table(raw_data: &[u8], offset: usize) -> [u8; 16] {
    let mut color_index_table = [0u8; 16];

    exnand_color_index_table(&mut color_index_table, raw_data, offset);

    color_index_table
}

fn exnand_color_index_table(color_index_table: &mut [u8; 16], raw_data: &[u8], offset: usize) {
    for i in 0..4 {
        let local_offset = offset + i;
        let color = raw_data[local_offset];
        color_index_table[i * 4] = color & 0x03;
        color_index_table[i * 4 + 1] = color & 0x0C >> 2;
        color_index_table[i * 4 + 2] = color & 0x30 >> 4;
        color_index_table[i * 4 + 3] = color & 0xC0 >> 6;
    }
}

fn create_alpha_table_dxt3(raw_data: &[u8], offset: usize) -> [u8; 16] {
    let mut alpha_table = [0u8; 16];

    expand_alpha_table_dxt3(&mut alpha_table, raw_data, offset);

    alpha_table
}

fn expand_alpha_table_dxt3(alpha_table: &mut [u8; 16], raw_data: &[u8], offset: usize) {
    let mut local_offset = offset;
    for i in 0..8 {
        let alpha = raw_data[local_offset];
        alpha_table[i * 2] = alpha & 0x0F;
        alpha_table[i * 2 + 1] = (alpha & 0xf0) >> 4;

        local_offset += 1;
    }
    for item in alpha_table.iter_mut().take(16) {
        *item = *item | (*item << 4);
    }
}

fn create_alpha_table_dxt5(a0: u8, a1: u8) -> [u8; 16] {
    let mut alpha_table = [0u8; 16];

    expand_alpha_table_dxt5(&mut alpha_table, a0, a1);

    alpha_table
}

fn expand_alpha_table_dxt5(alpha_table: &mut [u8; 16], a0: u8, a1: u8) {
    alpha_table[0] = a0;
    alpha_table[1] = a1;
    if a0 > a1 {
        for i in 2i32..8i32 {
            alpha_table[i as usize] = (((8 - i) * a0 as i32 + (i - 1) * a1 as i32 + 3) / 7) as u8;
        }
    } else {
        for i in 2i32..6i32 {
            alpha_table[i as usize] = (((6 - i) * a0 as i32 + (i - 1) * a1 as i32 + 2) / 5) as u8;
        }
        alpha_table[6] = 0;
        alpha_table[7] = 255;
    }
}

fn create_alpha_index_table_dxt5(raw_data: &[u8], offset: usize) -> [u8; 16] {
    let mut alpha_index_table = [0u8; 16];

    expand_alpha_index_table_dxt5(&mut alpha_index_table, raw_data, offset);

    alpha_index_table
}

fn expand_alpha_index_table_dxt5(alpha_index_table: &mut [u8; 16], raw_data: &[u8], offset: usize) {
    for i in 0..2 {
        let local_offset = offset + i * 3;
        let flags = (raw_data[local_offset] as u32)
            | ((raw_data[local_offset + 1] as u32) << 8)
            | ((raw_data[local_offset + 2] as u32) << 16);
        for j in 0..8 {
            let mask = (7 << (3 * j)) as u32;
            alpha_index_table[(i * 8) + j] = ((flags & mask) >> (3 * j)) as u8;
        }
    }
}

fn get_image_from_bgra8888(
    raw_data: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<DynamicImage, WzPngParseError> {
    let img_buffer = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            image::ImageBuffer::from_par_fn(width, height, |x, y| {
                let i = (x + y * width) as usize * 4;
                image::Rgba([
                    raw_data[i + 2],
                    raw_data[i + 1],
                    raw_data[i],
                    raw_data[i + 3],
                ])
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            image::ImageBuffer::from_fn(width, height, |x, y| {
                let i = (x + y * width) as usize * 4;
                image::Rgba([
                    raw_data[i + 2],
                    raw_data[i + 1],
                    raw_data[i],
                    raw_data[i + 3],
                ])
            })
        }
    };

    Ok(img_buffer.into())
}

fn get_image_from_rgb565(
    raw_data: &[u8],
    width: u32,
    height: u32,
) -> Result<DynamicImage, WzPngParseError> {
    let mut img_buffer: ImageBufferRgbChunk = image::ImageBuffer::new(width, height);
    img_buffer
        .enumerate_pixels_mut()
        .try_for_each::<_, Result<(), WzPngParseError>>(|(x, y, pixel)| {
            let i = (x + y * width) as usize * 2;
            let color = reader::read_u16_at(raw_data, i)?;
            *pixel = Rgb::<u8>::from_rgb565(color);
            Ok(())
        })?;

    Ok(img_buffer.into())
}

fn get_image_from_argb1555(
    raw_data: &[u8],
    width: u32,
    height: u32,
) -> Result<DynamicImage, WzPngParseError> {
    let mut img_buffer: ImageBufferRgbaChunk = image::ImageBuffer::new(width, height);
    img_buffer
        .enumerate_pixels_mut()
        .try_for_each::<_, Result<(), WzPngParseError>>(|(x, y, pixel)| {
            let i = (x + y * width) as usize * 2;
            let color = reader::read_u16_at(raw_data, i)?;
            *pixel = Rgba::<u8>::from_argb1555(color);
            Ok(())
        })?;

    Ok(img_buffer.into())
}
