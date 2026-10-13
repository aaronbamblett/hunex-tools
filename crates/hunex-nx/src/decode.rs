use alloc::vec;
use alloc::vec::Vec;

use crate::bntx::{Bntx, Texture};
use crate::deswizzle::{block_height, deswizzle, GOB_BYTES, GOB_WIDTH};
use crate::{Error, MAX_TEXTURE_PIXELS, MAX_TEXTURE_SIDE};

/// Decoded pixels: RGBA, row by row, four bytes per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba {
    /// Width of the decoded image in pixels.
    pub width: usize,
    /// Height of the decoded image in pixels.
    pub height: usize,
    /// The decoded pixel bytes.
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy)]
enum Codec {
    Bc7,
    Bc4,
    Bc3,
    Rgba8,
}

/// The codec, bytes per unit and pixels per unit side for a format code.
fn codec(code: u32) -> Result<(Codec, usize, usize), Error> {
    match code {
        0x2006 | 0x2001 => Ok((Codec::Bc7, 16, 4)),
        0x1D01 => Ok((Codec::Bc4, 8, 4)),
        0x1C06 => Ok((Codec::Bc3, 16, 4)),
        0x0B06 => Ok((Codec::Rgba8, 4, 1)),
        _ => Err(Error::UnsupportedFormat {
            code,
            reason: "unknown format",
        }),
    }
}

/// The stored size of a texture with these units, as the block-linear layout stores it, or
/// `None` when it does not fit in `usize`.
pub fn stored_size(row_bytes: usize, rows: usize) -> Option<usize> {
    let cols = row_bytes.div_ceil(GOB_WIDTH);
    let block = block_height(rows);
    rows.div_ceil(8 * block)
        .checked_mul(block)?
        .checked_mul(cols)?
        .checked_mul(GOB_BYTES)
}

/// Decodes texture `index` of a BNTX container to RGBA. BC3 textures use the standard channel
/// reading: color from the color block, alpha from the alpha block.
pub fn decode_texture(bntx: &Bntx<'_>, index: usize) -> Result<Rgba, Error> {
    let texture: &Texture = bntx.textures.get(index).ok_or(Error::BadIndex {
        index,
        count: bntx.textures.len(),
    })?;
    let (codec, bytes_per_unit, unit) = codec(texture.format)?;
    let (width, height) = (texture.width, texture.height);
    if width > MAX_TEXTURE_SIDE
        || height > MAX_TEXTURE_SIDE
        || width.saturating_mul(height) > MAX_TEXTURE_PIXELS
    {
        return Err(Error::Limit {
            what: "texture size",
        });
    }
    let width_units = width.div_ceil(unit);
    let rows = height.div_ceil(unit);
    let row_bytes = width_units * bytes_per_unit;
    let expected = stored_size(row_bytes, rows).ok_or(Error::Limit {
        what: "texture layout",
    })?;
    if expected != texture.image_size {
        return Err(Error::SizeMismatch {
            what: "texture layout",
            expected,
            found: texture.image_size,
        });
    }
    let data = bntx.data(index)?;
    let cols = row_bytes.div_ceil(GOB_WIDTH);
    let block = block_height(rows);
    let units = deswizzle(data, row_bytes, rows, cols, block);

    let mut pixels = vec![0u8; width * height * 4];
    match codec {
        Codec::Rgba8 => {
            for y in 0..height {
                for x in 0..width {
                    let at = y * row_bytes + x * 4;
                    let target = (y * width + x) * 4;
                    pixels[target..target + 4].copy_from_slice(&units[at..at + 4]);
                }
            }
        }
        Codec::Bc7 | Codec::Bc4 | Codec::Bc3 => {
            let block_bytes = if matches!(codec, Codec::Bc4) { 8 } else { 16 };
            let mut tile = [0u8; 64];
            for by in 0..rows {
                for bx in 0..width_units {
                    let start = by * row_bytes + bx * block_bytes;
                    let compressed = &units[start..start + block_bytes];
                    match codec {
                        Codec::Bc7 => bcdec_rs::bc7(compressed, &mut tile, 16),
                        Codec::Bc3 => bcdec_rs::bc3(compressed, &mut tile, 16),
                        _ => {
                            let gray = &mut tile[..16];
                            bcdec_rs::bc4(compressed, gray, 4, false);
                        }
                    }
                    for py in 0..4 {
                        for px in 0..4 {
                            let x = bx * 4 + px;
                            let y = by * 4 + py;
                            if x >= width || y >= height {
                                continue;
                            }
                            let target = (y * width + x) * 4;
                            match codec {
                                Codec::Bc7 => {
                                    let source = py * 16 + px * 4;
                                    pixels[target..target + 4]
                                        .copy_from_slice(&tile[source..source + 4]);
                                }
                                Codec::Bc3 => {
                                    let source = py * 16 + px * 4;
                                    pixels[target..target + 4]
                                        .copy_from_slice(&tile[source..source + 4]);
                                }
                                _ => {
                                    let value = tile[py * 4 + px];
                                    pixels[target..target + 4]
                                        .copy_from_slice(&[value, value, value, 255]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(Rgba {
        width,
        height,
        pixels,
    })
}
