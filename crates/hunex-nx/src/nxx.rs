use alloc::vec::Vec;

use crate::{Error, MAX_UNWRAPPED_BYTES};

const HEADER: usize = 16;

/// True if the bytes start with an NXX magic.
pub fn is_nxx(bytes: &[u8]) -> bool {
    bytes.starts_with(b"NXCX") || bytes.starts_with(b"NXGX")
}

/// Unwraps an NXX entry: the 16-byte header, then the zlib (`NXCX`) or gzip (`NXGX`) body.
pub fn unwrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let zlib = bytes.starts_with(b"NXCX");
    if !zlib && !bytes.starts_with(b"NXGX") {
        return Err(Error::BadMagic { what: "NXX" });
    }
    let field = |offset: usize| -> Result<usize, Error> {
        match bytes.get(offset..offset + 4) {
            Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d]) as usize),
            _ => Err(Error::Truncated { offset }),
        }
    };
    let decompressed = field(4)?;
    let compressed = field(8)?;
    if decompressed > MAX_UNWRAPPED_BYTES {
        return Err(Error::Limit {
            what: "NXX unwrapped size",
        });
    }
    let end = HEADER.checked_add(compressed).ok_or(Error::Truncated {
        offset: bytes.len(),
    })?;
    let body = bytes.get(HEADER..end).ok_or(Error::Truncated {
        offset: bytes.len(),
    })?;
    // The declared size is the limit: inflation stops as soon as it would be exceeded.
    let out = if zlib {
        miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(body, decompressed)
            .map_err(|_| Error::Inflate { what: "NXCX" })?
    } else {
        let deflate = gzip_deflate_stream(body)?;
        miniz_oxide::inflate::decompress_to_vec_with_limit(deflate, decompressed)
            .map_err(|_| Error::Inflate { what: "NXGX" })?
    };
    if out.len() != decompressed {
        return Err(Error::SizeMismatch {
            what: "NXX",
            expected: decompressed,
            found: out.len(),
        });
    }
    Ok(out)
}

/// Skips a gzip member header and returns the raw deflate stream that follows it.
fn gzip_deflate_stream(body: &[u8]) -> Result<&[u8], Error> {
    let bad = Error::BadMagic { what: "NXGX gzip" };
    if body.len() < 10 || body[0] != 0x1F || body[1] != 0x8B || body[2] != 8 {
        return Err(bad);
    }
    let flags = body[3];
    let mut position = 10;
    if flags & 0x04 != 0 {
        let length = usize::from(
            *body
                .get(position)
                .ok_or(Error::Truncated { offset: position })?,
        ) | usize::from(
            *body
                .get(position + 1)
                .ok_or(Error::Truncated { offset: position })?,
        ) << 8;
        position += 2 + length;
    }
    for flag in [0x08, 0x10] {
        if flags & flag != 0 {
            let end = body
                .get(position..)
                .and_then(|rest| rest.iter().position(|&byte| byte == 0))
                .ok_or(Error::Truncated { offset: position })?;
            position += end + 1;
        }
    }
    if flags & 0x02 != 0 {
        position += 2;
    }
    body.get(position..)
        .ok_or(Error::Truncated { offset: position })
}
