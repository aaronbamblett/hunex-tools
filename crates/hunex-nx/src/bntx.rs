use alloc::string::String;
use alloc::vec::Vec;

use crate::Error;

/// One texture record of a BNTX container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    pub name: Option<String>,
    pub format: u32,
    pub width: usize,
    pub height: usize,
    pub image_size: usize,
    pub alignment: usize,
    pub layout: u32,
    /// Offset of the texture's data in the container.
    pub data_offset: usize,
}

/// A parsed BNTX container, borrowing the decompressed bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bntx<'a> {
    bytes: &'a [u8],
    pub textures: Vec<Texture>,
}

/// The `len` bytes at `offset`, or `None` when they do not lie inside `bytes`.
fn slice_at(bytes: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    bytes.get(offset..offset.checked_add(len)?)
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, Error> {
    match slice_at(bytes, offset, 2) {
        Some(&[a, b]) => Ok(u16::from_le_bytes([a, b])),
        _ => Err(Error::Truncated { offset }),
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    match slice_at(bytes, offset, 4) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(Error::Truncated { offset }),
    }
}

fn u64_at(bytes: &[u8], offset: usize) -> Result<usize, Error> {
    let low = u64::from(u32_at(bytes, offset)?);
    let high = u64::from(u32_at(
        bytes,
        offset.checked_add(4).ok_or(Error::Truncated { offset })?,
    )?);
    usize::try_from(low | (high << 32)).map_err(|_| Error::Truncated { offset })
}

/// Rounds `value` up to a multiple of `alignment`, or `None` when that does not fit.
fn align_up(value: usize, alignment: usize) -> Option<usize> {
    if alignment <= 1 {
        Some(value)
    } else {
        value.div_ceil(alignment).checked_mul(alignment)
    }
}

impl<'a> Bntx<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Bntx<'a>, Error> {
        if !bytes.starts_with(b"BNTX") {
            return Err(Error::BadMagic { what: "BNTX" });
        }
        if bytes.get(0x0C..0x0E) != Some(&[0xFF, 0xFE]) {
            return Err(Error::BadMagic {
                what: "BNTX byte order",
            });
        }
        if bytes.get(0x20..0x24) != Some(b"NX  ") {
            return Err(Error::BadMagic {
                what: "BNTX block tag",
            });
        }
        let count = u32_at(bytes, 0x24)? as usize;
        if count > bytes.len() / 8 {
            return Err(Error::Truncated { offset: 0x24 });
        }
        let pointers = u64_at(bytes, 0x28)?;
        let data_block = u64_at(bytes, 0x30)?;
        let dictionary = u64_at(bytes, 0x38)?;
        let names = read_names(bytes, pointers, dictionary, count);
        if slice_at(bytes, data_block, 4) != Some(b"BRTD") {
            return Err(Error::BadMagic { what: "BRTD" });
        }
        let first = data_block
            .checked_add(0x10)
            .ok_or(Error::Truncated { offset: data_block })?;

        let mut textures = Vec::with_capacity(count);
        let mut next = first;
        for index in 0..count {
            let record_pointer = pointers
                .checked_add(8 * index)
                .ok_or(Error::Truncated { offset: pointers })?;
            let record = u64_at(bytes, record_pointer)?;
            let field = |at: usize| {
                record
                    .checked_add(at)
                    .ok_or(Error::Truncated { offset: record })
            };
            if slice_at(bytes, record, 4) != Some(b"BRTI") {
                return Err(Error::BadTexture {
                    index,
                    field: "BRTI magic",
                });
            }
            let format = u32_at(bytes, field(0x1C)?)?;
            let width = u32_at(bytes, field(0x24)?)? as usize;
            let height = u32_at(bytes, field(0x28)?)? as usize;
            let layout = u32_at(bytes, field(0x38)?)?;
            let image_size = u32_at(bytes, field(0x50)?)? as usize;
            let alignment = (u32_at(bytes, field(0x54)?)? as usize).max(1);
            if width == 0 || height == 0 {
                return Err(Error::BadTexture {
                    index,
                    field: "size",
                });
            }
            let data_offset = if index == 0 {
                first
            } else {
                align_up(next, alignment).ok_or(Error::Limit {
                    what: "texture offset",
                })?
            };
            next = data_offset.checked_add(image_size).ok_or(Error::Limit {
                what: "texture offset",
            })?;
            textures.push(Texture {
                name: names.get(index).cloned(),
                format,
                width,
                height,
                image_size,
                alignment,
                layout,
                data_offset,
            });
        }
        Ok(Bntx { bytes, textures })
    }

    /// The stored bytes of texture `index`.
    pub fn data(&self, index: usize) -> Result<&'a [u8], Error> {
        let texture = self.textures.get(index).ok_or(Error::BadIndex {
            index,
            count: self.textures.len(),
        })?;
        slice_at(self.bytes, texture.data_offset, texture.image_size).ok_or(Error::Truncated {
            offset: texture.data_offset,
        })
    }
}

/// Reads the `_STRP` name table, if present. Returns no names when the table is missing or
/// does not match the texture count.
fn read_names(bytes: &[u8], pointers: usize, dictionary: usize, count: usize) -> Vec<String> {
    let end = dictionary.min(bytes.len());
    let Some(start) = bytes.get(pointers.min(end)..end).and_then(|window| {
        window
            .windows(5)
            .position(|candidate| candidate == b"_STRP")
            .map(|position| pointers.saturating_add(position))
    }) else {
        return Vec::new();
    };
    let Ok(table_count) = u64_at(bytes, start.saturating_add(0x10)) else {
        return Vec::new();
    };
    if table_count != count {
        return Vec::new();
    }
    let mut names = Vec::with_capacity(count);
    let mut position = start.saturating_add(0x18);
    for _ in 0..count {
        let Ok(length) = u16_at(bytes, position) else {
            return Vec::new();
        };
        let length = usize::from(length);
        let Some(text) = slice_at(bytes, position.saturating_add(2), length) else {
            return Vec::new();
        };
        names.push(String::from_utf8_lossy(text).into_owned());
        position = position.saturating_add(2 + length);
    }
    names
}
