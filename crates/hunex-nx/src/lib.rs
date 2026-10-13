//! Switch textures: NXX wrappers around BNTX containers, block-linear layout and BC decoding.
//!
//! ```
//! // An NXX wrapper: the magic, the decompressed and compressed sizes, four zero bytes, then zlib data.
//! const ZLIB: &[u8] = &[0x78, 0x9c, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x07, 0x00, 0x06, 0x2c, 0x02, 0x15];
//! let mut wrapped = b"NXCX".to_vec();
//! wrapped.extend_from_slice(&5u32.to_le_bytes());
//! wrapped.extend_from_slice(&(ZLIB.len() as u32).to_le_bytes());
//! wrapped.extend_from_slice(&[0; 4]);
//! wrapped.extend_from_slice(ZLIB);
//! assert!(hunex_nx::is_nxx(&wrapped));
//! assert_eq!(hunex_nx::unwrap(&wrapped).unwrap(), b"hello");
//! ```

#![warn(missing_docs)]
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

/// Errors from reading a Switch NX file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A magic value does not match. `what` names it.
    BadMagic {
        /// The name of the magic that did not match.
        what: &'static str,
    },
    /// The data ends before the field at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// A compressed block did not inflate. `what` names the block.
    Inflate {
        /// The name of the block that did not inflate.
        what: &'static str,
    },
    /// The decompressed size in the wrapper does not match the body.
    SizeMismatch {
        /// The name of the value whose size does not match.
        what: &'static str,
        /// The size the header states.
        expected: usize,
        /// The size found in the file.
        found: usize,
    },
    /// A texture format this crate does not decode.
    UnsupportedFormat {
        /// The texture format code that is not supported.
        code: u32,
        /// Why the format is refused.
        reason: &'static str,
    },
    /// A texture has a value that this crate does not accept.
    BadTexture {
        /// Index of the texture.
        index: usize,
        /// The name of the field that is not valid.
        field: &'static str,
    },
    /// A texture index is past the end of the list.
    BadIndex {
        /// The index that was asked for.
        index: usize,
        /// The number of textures.
        count: usize,
    },
    /// A size beyond the limits this crate accepts, or one that does not fit in `usize`.
    Limit {
        /// The name of the value that is not valid.
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
