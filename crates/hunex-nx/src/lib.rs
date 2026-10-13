//! Switch textures: NXX wrappers around BNTX containers, block-linear layout and BC decoding.

#![no_std]

extern crate alloc;

mod bntx;
mod decode;
mod deswizzle;
mod nxx;
#[cfg(test)]
mod tests;

pub use bntx::{Bntx, Texture};
pub use decode::{decode_texture, Rgba};
pub use deswizzle::{block_height, deswizzle, gob_offset};
pub use nxx::{is_nxx, unwrap};

/// The largest texture side decoded, in pixels. Larger textures are an error.
pub const MAX_TEXTURE_SIDE: usize = 16_384;
/// The most pixels in one decoded texture (64 Mpx, 256 MiB of RGBA). Larger textures are an error.
pub const MAX_TEXTURE_PIXELS: usize = 1 << 26;
/// The most bytes an NXX wrapper may unwrap to (256 MiB). A larger declared size is an error.
pub const MAX_UNWRAPPED_BYTES: usize = 1 << 28;

use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    BadMagic {
        what: &'static str,
    },
    Truncated {
        offset: usize,
    },
    Inflate {
        what: &'static str,
    },
    /// The decompressed size in the wrapper does not match the body.
    SizeMismatch {
        what: &'static str,
        expected: usize,
        found: usize,
    },
    /// A texture format this crate does not decode.
    UnsupportedFormat {
        code: u32,
        reason: &'static str,
    },
    BadTexture {
        index: usize,
        field: &'static str,
    },
    BadIndex {
        index: usize,
        count: usize,
    },
    /// A size beyond the limits this crate accepts, or one that does not fit in `usize`.
    Limit {
        what: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadMagic { what } => write!(f, "{what}: bad magic"),
            Error::Truncated { offset } => write!(f, "texture data truncated at 0x{offset:x}"),
            Error::Inflate { what } => write!(f, "{what}: inflate failed"),
            Error::SizeMismatch {
                what,
                expected,
                found,
            } => {
                write!(f, "{what}: expected {expected} bytes, found {found}")
            }
            Error::UnsupportedFormat { code, reason } => {
                write!(f, "texture format 0x{code:04x} is not decoded: {reason}")
            }
            Error::BadTexture { index, field } => write!(f, "texture {index}: bad {field}"),
            Error::BadIndex { index, count } => write!(f, "texture index {index} out of {count}"),
            Error::Limit { what } => write!(f, "{what}: beyond the size limit"),
        }
    }
}

impl core::error::Error for Error {}
