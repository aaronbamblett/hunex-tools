//! CompressedBG images (`.cbg`) of the HuneX engine.
//!
//! A file is a 48-byte header (magic, width, height, stripe height, bits per pixel, zero
//! padding), a table of stripe offsets, and the stripes. Each stripe holds its rows after a
//! spatial predictor. The residuals are run-length coded, and the run-length stream is Huffman
//! coded with a table of 256 byte weights.
//!
//! [`decode`] returns RGBA pixels. [`encode`] writes the canonical form, which reproduces the
//! sample files byte for byte when their pixels are decoded and encoded again. [`encode_simple`]
//! writes a form with equal weights and literal runs only; it decodes to the same pixels.
//!
//! ```
//! use hunex_cbg::{Header, Image};
//!
//! let image = Image {
//!     header: Header { width: 2, height: 2, stripe_height: 60, bits_per_pixel: 32 },
//!     rgba: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 9, 9, 9, 128],
//! };
//! let file = hunex_cbg::encode(&image).unwrap();
//! assert_eq!(hunex_cbg::decode(&file).unwrap().rgba, image.rgba);
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

mod bits;
mod huffman;
#[cfg(test)]
mod tests;

use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use bits::{read_leb, write_leb, BitReader, BitWriter};
use huffman::{build, write_codes};

/// The magic that starts every file: the ASCII name and one zero byte.
pub const MAGIC: &[u8; 16] = b"CompressedBG_MT\0";
/// The stripe height of every sample file.
pub const STRIPE_HEIGHT: usize = 60;
/// The largest side of an image, in pixels.
pub const MAX_SIDE: usize = 16_384;
/// The most pixels in one image (64 Mpx). Larger headers are rejected before any allocation.
pub const MAX_PIXELS: usize = 1 << 26;
const HEADER_FIXED: usize = 48;

/// Errors from parsing, decoding and encoding a CBG file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The file does not start with the magic.
    BadMagic,
    /// The data ends before the field at `offset`.
    Truncated {
        /// Byte offset of the missing data.
        offset: usize,
    },
    /// A header field has a value this crate does not accept.
    BadHeader {
        /// Name of the field.
        field: &'static str,
    },
    /// A stripe is malformed. `what` names the problem.
    BadStream {
        /// Description of the problem.
        what: &'static str,
    },
    /// The image is larger than the limits this crate accepts.
    Limit {
        /// The image is larger than the limits this crate accepts.
        what: &'static str,
    },
    /// The pixels cannot be stored in the bit depth requested.
    Unsupported {
        /// The pixels cannot be stored in the bit depth requested.
        what: &'static str,
    },
    /// The RGBA buffer does not match the header.
    SizeMismatch {
        /// The RGBA buffer does not match the header.
        expected: usize,
        /// Bytes the header implies.
        found: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not a CompressedBG_MT file"),
            Error::Truncated { offset } => write!(f, "CBG data truncated at 0x{offset:x}"),
            Error::BadHeader { field } => write!(f, "bad CBG header: {field}"),
            Error::BadStream { what } => write!(f, "bad CBG stripe: {what}"),
            Error::Limit { what } => write!(f, "CBG {what} is beyond the size limit"),
            Error::Unsupported { what } => write!(f, "cannot encode: {what}"),
            Error::SizeMismatch { expected, found } => {
                write!(f, "RGBA buffer is {found} bytes, expected {expected}")
            }
        }
    }
}

impl core::error::Error for Error {}

/// The header fields of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Rows per stripe. Every sample uses 60.
    pub stripe_height: usize,
    /// 8 (gray), 24 (RGB, stored B G R) or 32 (RGBA, stored B G R A).
    pub bits_per_pixel: u8,
}

impl Header {
    /// Stored channels per pixel.
    fn channels(&self) -> usize {
        usize::from(self.bits_per_pixel) / 8
    }

    fn stripes(&self) -> usize {
        self.height.div_ceil(self.stripe_height)
    }

    fn stripe_rows(&self, index: usize) -> usize {
        let start = index * self.stripe_height;
        self.stripe_height.min(self.height - start)
    }
}

/// A decoded image: its header and RGBA pixels, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The header of the image.
    pub header: Header,
    /// Pixels as RGBA, four bytes per pixel, row by row.
    pub rgba: Vec<u8>,
}

fn check_header(header: &Header) -> Result<(), Error> {
    if header.width == 0 || header.height == 0 {
        return Err(Error::BadHeader { field: "size" });
    }
    if header.width > MAX_SIDE || header.height > MAX_SIDE {
        return Err(Error::Limit {
            what: "width or height",
        });
    }
    if header.width * header.height > MAX_PIXELS {
        return Err(Error::Limit {
            what: "pixel count",
        });
    }
    if header.stripe_height == 0 || header.stripe_height > MAX_SIDE {
        return Err(Error::BadHeader {
            field: "stripe height",
        });
    }
    if !matches!(header.bits_per_pixel, 8 | 24 | 32) {
        return Err(Error::BadHeader {
            field: "bits per pixel",
        });
    }
    Ok(())
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    match bytes.get(offset..offset + 4) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(Error::Truncated { offset }),
    }
}

/// Reads the header of a file.
pub fn parse_header(bytes: &[u8]) -> Result<Header, Error> {
    if !bytes.starts_with(MAGIC) {
        return Err(Error::BadMagic);
    }
    let header = Header {
        width: u32_at(bytes, 16)? as usize,
        height: u32_at(bytes, 20)? as usize,
        stripe_height: u32_at(bytes, 24)? as usize,
        bits_per_pixel: u32_at(bytes, 28)?
            .try_into()
            .map_err(|_| Error::BadHeader {
                field: "bits per pixel",
            })?,
    };
    check_header(&header)?;
    Ok(header)
}

/// Decodes a file to RGBA pixels.
pub fn decode(bytes: &[u8]) -> Result<Image, Error> {
    let header = parse_header(bytes)?;
    let stripes = header.stripes();
    let table_end = HEADER_FIXED + 4 * stripes;
    if table_end > bytes.len() {
        return Err(Error::Truncated {
            offset: bytes.len(),
        });
    }
    let mut offsets = Vec::with_capacity(stripes + 1);
    for index in 0..stripes {
        offsets.push(u32_at(bytes, HEADER_FIXED + 4 * index)? as usize);
    }
    offsets.push(bytes.len());
    if offsets[0] != table_end {
        return Err(Error::BadHeader {
            field: "first stripe offset",
        });
    }
    if offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(Error::BadHeader {
            field: "stripe offsets",
        });
    }

    let channels = header.channels();
    let mut rgba = vec![0u8; header.width * header.height * 4];
    for index in 0..stripes {
        let rows = header.stripe_rows(index);
        let stored = decode_stripe(
            &bytes[offsets[index]..offsets[index + 1]],
            header.width,
            rows,
            channels,
        )?;
        let first_row = index * header.stripe_height;
        for (pixel, values) in stored.chunks_exact(channels).enumerate() {
            let at = (first_row * header.width + pixel) * 4;
            let (r, g, b, a) = match channels {
                1 => (values[0], values[0], values[0], 255),
                3 => (values[2], values[1], values[0], 255),
                _ => (values[2], values[1], values[0], values[3]),
            };
            rgba[at..at + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    Ok(Image { header, rgba })
}

/// Decodes one stripe to its stored channel values (B G R [A], or gray).
fn decode_stripe(
    stripe: &[u8],
    width: usize,
    rows: usize,
    channels: usize,
) -> Result<Vec<u8>, Error> {
    let expected = width * rows * channels;
    let run_length = u32_at(stripe, 0)? as usize;
    if run_length == 0 || run_length > 2 * expected + 16 {
        return Err(Error::BadStream {
            what: "run-length stream length",
        });
    }
    let mut position = 4usize;
    let mut weights = [0u64; 256];
    for weight in weights.iter_mut() {
        let (value, next) = read_leb(stripe, position)?;
        *weight = value;
        position = next;
    }
    let tree = build(&weights)?;
    let mut bits = BitReader::new(&stripe[position..]);
    let mut run = Vec::with_capacity(run_length);
    for _ in 0..run_length {
        run.push(tree.read(&mut bits)?);
    }
    if bits.used().div_ceil(8) != stripe.len() - position || !bits.rest_is_zero() {
        return Err(Error::BadStream {
            what: "bitstream length or padding",
        });
    }
    let residuals = expand_runs(&run, expected)?;
    Ok(reconstruct(&residuals, width, rows, channels))
}

/// Expands the alternating literal and zero runs to `expected` residual bytes.
fn expand_runs(run: &[u8], expected: usize) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(expected);
    let mut position = 0usize;
    let mut literal = true;
    while position < run.len() {
        let (count, next) = read_leb(run, position)?;
        position = next;
        let count = usize::try_from(count).map_err(|_| Error::BadStream { what: "run length" })?;
        if out
            .len()
            .checked_add(count)
            .is_none_or(|total| total > expected)
        {
            return Err(Error::BadStream {
                what: "run overflows the stripe",
            });
        }
        if literal {
            let end = position
                .checked_add(count)
                .filter(|&end| end <= run.len())
                .ok_or(Error::Truncated { offset: position })?;
            out.extend_from_slice(&run[position..end]);
            position = end;
        } else {
            out.resize(out.len() + count, 0);
        }
        literal = !literal;
    }
    if out.len() != expected {
        return Err(Error::BadStream {
            what: "expanded size",
        });
    }
    Ok(out)
}

/// The prediction for one channel of a pixel, from pixels already known in the same stripe.
fn predict(
    values: &[u8],
    width: usize,
    channels: usize,
    row: usize,
    column: usize,
    channel: usize,
) -> u8 {
    let at = |r: usize, c: usize| values[(r * width + c) * channels + channel];
    match (row, column) {
        (0, 0) => 0,
        (0, _) => at(0, column - 1),
        (_, 0) => at(row - 1, 0),
        _ => ((u16::from(at(row - 1, column)) + u16::from(at(row, column - 1))) / 2) as u8,
    }
}

/// Adds the predictions back to the residuals.
fn reconstruct(residuals: &[u8], width: usize, rows: usize, channels: usize) -> Vec<u8> {
    let mut values = vec![0u8; residuals.len()];
    for row in 0..rows {
        for column in 0..width {
            for channel in 0..channels {
                let i = (row * width + column) * channels + channel;
                let prediction = predict(&values, width, channels, row, column, channel);
                values[i] = residuals[i].wrapping_add(prediction);
            }
        }
    }
    values
}

/// The residuals of stored channel values.
fn residuals(values: &[u8], width: usize, rows: usize, channels: usize) -> Vec<u8> {
    let mut out = vec![0u8; values.len()];
    for row in 0..rows {
        for column in 0..width {
            for channel in 0..channels {
                let i = (row * width + column) * channels + channel;
                let prediction = predict(values, width, channels, row, column, channel);
                out[i] = values[i].wrapping_sub(prediction);
            }
        }
    }
    out
}

/// The stored channel values of RGBA pixels for a bit depth, checking that they fit it.
fn stored_channels(image: &Image) -> Result<Vec<u8>, Error> {
    let header = &image.header;
    check_header(header)?;
    let expected = header.width * header.height * 4;
    if image.rgba.len() != expected {
        return Err(Error::SizeMismatch {
            expected,
            found: image.rgba.len(),
        });
    }
    let mut out = Vec::with_capacity(header.width * header.height * header.channels());
    for pixel in image.rgba.chunks_exact(4) {
        match header.bits_per_pixel {
            8 => {
                if pixel[0] != pixel[1] || pixel[1] != pixel[2] {
                    return Err(Error::Unsupported {
                        what: "8 bits per pixel needs gray pixels",
                    });
                }
                out.push(pixel[0]);
            }
            24 => {
                if pixel[3] != 255 {
                    return Err(Error::Unsupported {
                        what: "24 bits per pixel needs opaque pixels",
                    });
                }
                out.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
            }
            _ => out.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]),
        }
    }
    Ok(out)
}

/// Run-length codes the residuals: a literal run holds the nonzero bytes, a zero run the zero
/// bytes, and the runs alternate from a literal run.
fn run_length_canonical(residuals: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut position = 0usize;
    loop {
        let literal_end = position
            + residuals[position..]
                .iter()
                .take_while(|&&b| b != 0)
                .count();
        write_leb(&mut out, (literal_end - position) as u64);
        out.extend_from_slice(&residuals[position..literal_end]);
        position = literal_end;
        if position == residuals.len() {
            return out;
        }
        let zero_end = position
            + residuals[position..]
                .iter()
                .take_while(|&&b| b == 0)
                .count();
        write_leb(&mut out, (zero_end - position) as u64);
        position = zero_end;
        if position == residuals.len() {
            return out;
        }
    }
}

/// One literal run holding every residual byte.
fn run_length_literal(residuals: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(residuals.len() + 5);
    write_leb(&mut out, residuals.len() as u64);
    out.extend_from_slice(residuals);
    out
}

/// Encodes one stripe with the given run-length form and weights.
fn encode_stripe(
    values: &[u8],
    width: usize,
    rows: usize,
    channels: usize,
    canonical: bool,
) -> Result<Vec<u8>, Error> {
    let residuals = residuals(values, width, rows, channels);
    let run = if canonical {
        run_length_canonical(&residuals)
    } else {
        run_length_literal(&residuals)
    };
    let mut weights = [0u64; 256];
    if canonical {
        for &byte in &run {
            weights[usize::from(byte)] += 1;
        }
    } else {
        weights = [1u64; 256];
    }
    let tree = build(&weights)?;
    let codes = tree.codes();
    let mut writer = BitWriter::default();
    write_codes(&mut writer, &codes, &run)?;
    let bitstream = writer.finish();

    let mut out = Vec::with_capacity(4 + 256 + bitstream.len());
    out.extend_from_slice(&(run.len() as u32).to_le_bytes());
    for &weight in &weights {
        write_leb(&mut out, weight);
    }
    out.extend_from_slice(&bitstream);
    Ok(out)
}

fn encode_with(image: &Image, canonical: bool) -> Result<Vec<u8>, Error> {
    let header = image.header;
    let channels = header.channels();
    let stored = stored_channels(image)?;
    let stripes = header.stripes();
    let mut blobs = Vec::with_capacity(stripes);
    for index in 0..stripes {
        let rows = header.stripe_rows(index);
        let start = index * header.stripe_height * header.width * channels;
        let values = &stored[start..start + rows * header.width * channels];
        blobs.push(encode_stripe(
            values,
            header.width,
            rows,
            channels,
            canonical,
        )?);
    }

    let table_end = HEADER_FIXED + 4 * stripes;
    let mut out = Vec::with_capacity(table_end + blobs.iter().map(Vec::len).sum::<usize>());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(header.width as u32).to_le_bytes());
    out.extend_from_slice(&(header.height as u32).to_le_bytes());
    out.extend_from_slice(&(header.stripe_height as u32).to_le_bytes());
    out.extend_from_slice(&u32::from(header.bits_per_pixel).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);
    let mut offset = table_end;
    for blob in &blobs {
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += blob.len();
    }
    for blob in &blobs {
        out.extend_from_slice(blob);
    }
    Ok(out)
}

/// Encodes an image in the canonical form: weights are the counts of the run-length bytes.
pub fn encode(image: &Image) -> Result<Vec<u8>, Error> {
    encode_with(image, true)
}

/// Encodes an image with literal runs only and equal weights. Larger than `encode`, and decodes
/// to the same pixels.
pub fn encode_simple(image: &Image) -> Result<Vec<u8>, Error> {
    encode_with(image, false)
}
