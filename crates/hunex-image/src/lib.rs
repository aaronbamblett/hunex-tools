//! Background images: MZP archives of MZX-compressed palette tiles, assembled into RGBA.
//!
//! Entry 0 is a header: six u16 values (canvas width and height, tile width and height, tiles
//! across and down), the bitmap type (0x0C), the depth (0), the crop byte, one flag byte, and
//! one transparency byte per tile. Entry i + 1 is tile i (row-major), an MZX0 stream of a HEP
//! tile: a 32-byte header, one palette index per pixel, and a 256-entry palette whose alpha is
//! stored in seven bits.
//!
//! Tiles abut at their full size: tile (column c, row r) covers canvas pixels from (c * tile
//! width, r * tile height). The image is the canvas, width x height. The crop byte is kept as
//! read; its meaning is unknown.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use hunex_mzp::Container;

pub const HEADER_BYTES: usize = 16;
pub const HEP_HEADER: usize = 32;
pub const PALETTE_ENTRIES: usize = 256;
pub const PALETTE_BYTES: usize = PALETTE_ENTRIES * 4;
pub const BITMAP_TYPE: u8 = 0x0C;
pub const ATLAS_TYPE: u8 = 0x01;
const HEP_MAGIC: &[u8; 4] = b"HEP\0";
const HEP_CONSTANTS: [u32; 3] = [0x2530_0000, 6, 16];
const HEP_TRANSPARENCY: u32 = 2;
const DEFAULT_TILE: (usize, usize) = (512, 256);
const DEFAULT_CROP: usize = 17;
/// The most pixels in one canvas (64 Mpx, 256 MiB of RGBA). A larger header or image is an error,
/// and it is checked before any canvas is allocated.
pub const MAX_CANVAS_PIXELS: usize = 1 << 26;

/// Maps a stored 7-bit alpha code to alpha: the code is doubled, with bit 6 copied into bit 0;
/// codes with bit 7 set mean opaque.
pub const fn decode_alpha(code: u8) -> u8 {
    if code & 0x80 != 0 {
        0xFF
    } else {
        (code << 1) | (code >> 6)
    }
}

const fn make_alpha_codes() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut alpha = 0usize;
    while alpha < 256 {
        let mut best = 0u8;
        let mut best_error = usize::MAX;
        let mut code = 0u8;
        while code < 128 {
            let decoded = decode_alpha(code) as usize;
            let error = decoded.abs_diff(alpha);
            if error < best_error {
                best_error = error;
                best = code;
            }
            code += 1;
        }
        table[alpha] = best;
        alpha += 1;
    }
    table
}

/// The stored code nearest to each alpha value. Exact for 0 and 255, and for every alpha the
/// mapping can produce.
pub const ALPHA_CODES: [u8; 256] = make_alpha_codes();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Container(hunex_mzp::Error),
    Mzx(hunex_mzx::Error),
    /// A header field is malformed or inconsistent.
    BadHeader {
        field: &'static str,
    },
    /// A header field has a value this crate does not decode.
    Unsupported {
        field: &'static str,
        value: u32,
    },
    /// A tile does not match the header; `index` is the tile number.
    BadTile {
        index: usize,
        field: &'static str,
    },
    /// The RGBA size does not match the layout.
    SizeMismatch {
        width: usize,
        height: usize,
        expected_width: usize,
        expected_height: usize,
    },
    NoEntries,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Container(error) => write!(f, "{error}"),
            Error::Mzx(error) => write!(f, "{error}"),
            Error::BadHeader { field } => write!(f, "image header: bad {field}"),
            Error::Unsupported { field, value } => {
                write!(f, "image header: unsupported {field} {value:#x}")
            }
            Error::BadTile { index, field } => write!(f, "tile {index}: bad {field}"),
            Error::SizeMismatch {
                width,
                height,
                expected_width,
                expected_height,
            } => write!(
                f,
                "image is {width}x{height}; this layout needs {expected_width}x{expected_height}"
            ),
            Error::NoEntries => write!(f, "the archive has no entries"),
        }
    }
}

impl core::error::Error for Error {}

impl From<hunex_mzp::Error> for Error {
    fn from(error: hunex_mzp::Error) -> Self {
        Error::Container(error)
    }
}

impl From<hunex_mzx::Error> for Error {
    fn from(error: hunex_mzx::Error) -> Self {
        Error::Mzx(error)
    }
}

/// The header of an image archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub across: usize,
    pub down: usize,
    pub bitmap_type: u8,
    pub depth: u8,
    pub crop: usize,
    pub flags: u8,
    /// One value per tile, row-major: 0 all transparent, 2 all opaque, 1 mixed.
    pub transparency: Vec<u8>,
}

impl Layout {
    pub fn tiles(&self) -> usize {
        self.across * self.down
    }

    /// Image size: the canvas.
    pub fn image_size(&self) -> (usize, usize) {
        (self.canvas_width, self.canvas_height)
    }

    /// The layout a new image of this size gets: 512x256 tiles covering the image, with the
    /// crop byte of the samples. The canvas is the whole tile grid.
    pub fn default_for(width: usize, height: usize) -> Layout {
        let (tile_width, tile_height) = DEFAULT_TILE;
        let across = width.div_ceil(tile_width).max(1);
        let down = height.div_ceil(tile_height).max(1);
        Layout {
            canvas_width: across * tile_width,
            canvas_height: down * tile_height,
            tile_width,
            tile_height,
            across,
            down,
            bitmap_type: BITMAP_TYPE,
            depth: 0,
            crop: DEFAULT_CROP,
            flags: 1,
            transparency: vec![0; across * down],
        }
    }
}

/// Parses entry 0. Returns the layout and checks it against the entry length.
pub fn parse_header(entry: &[u8]) -> Result<Layout, Error> {
    if entry.len() < HEADER_BYTES {
        return Err(Error::BadHeader { field: "length" });
    }
    let word =
        |index: usize| usize::from(u16::from_le_bytes([entry[2 * index], entry[2 * index + 1]]));
    let trailer = if entry[12] == ATLAS_TYPE {
        HEADER_BYTES + PALETTE_BYTES
    } else {
        HEADER_BYTES
    };
    let Some(transparency) = entry.get(trailer..) else {
        return Err(Error::BadHeader { field: "length" });
    };
    let layout = Layout {
        canvas_width: word(0),
        canvas_height: word(1),
        tile_width: word(2),
        tile_height: word(3),
        across: word(4),
        down: word(5),
        bitmap_type: entry[12],
        depth: entry[13],
        crop: usize::from(entry[14]),
        flags: entry[15],
        transparency: transparency.to_vec(),
    };
    if layout.bitmap_type != BITMAP_TYPE && layout.bitmap_type != ATLAS_TYPE {
        return Err(Error::Unsupported {
            field: "bitmap type",
            value: u32::from(layout.bitmap_type),
        });
    }
    if layout.depth != 0 {
        return Err(Error::Unsupported {
            field: "depth",
            value: u32::from(layout.depth),
        });
    }
    if layout.across == 0 || layout.down == 0 || layout.tile_width == 0 || layout.tile_height == 0 {
        return Err(Error::BadHeader { field: "tile grid" });
    }
    if layout.transparency.len() != layout.tiles() {
        return Err(Error::BadHeader {
            field: "transparency count",
        });
    }
    Ok(layout)
}

/// A decoded image: RGBA, row-major, `width * height * 4` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub layout: Layout,
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

/// A HEP tile: palette indices and a palette with decoded alpha.
struct Tile {
    indices: Vec<u8>,
    palette: [[u8; 4]; PALETTE_ENTRIES],
}

fn parse_tile(index: usize, bytes: &[u8], layout: &Layout) -> Result<Tile, Error> {
    let bad = |field| Error::BadTile { index, field };
    if bytes.len() < HEP_HEADER || &bytes[..4] != HEP_MAGIC {
        return Err(bad("magic"));
    }
    let field = |slot: usize| {
        u32::from_le_bytes([
            bytes[4 * slot],
            bytes[4 * slot + 1],
            bytes[4 * slot + 2],
            bytes[4 * slot + 3],
        ])
    };
    let width = field(5) as usize;
    let height = field(6) as usize;
    if width != layout.tile_width || height != layout.tile_height {
        return Err(bad("size"));
    }
    let pixels = width * height;
    let expected = HEP_HEADER + pixels + PALETTE_BYTES;
    if field(1) as usize != expected || bytes.len() != expected {
        return Err(bad("length"));
    }
    let indices = bytes[HEP_HEADER..HEP_HEADER + pixels].to_vec();
    let mut palette = [[0u8; 4]; PALETTE_ENTRIES];
    for (entry, slot) in bytes[HEP_HEADER + pixels..]
        .chunks_exact(4)
        .zip(palette.iter_mut())
    {
        *slot = [entry[0], entry[1], entry[2], decode_alpha(entry[3] & 0x7F)];
    }
    Ok(Tile { indices, palette })
}

/// The pixel count of a canvas, checked against `MAX_CANVAS_PIXELS`.
fn canvas_pixels(width: usize, height: usize) -> Result<usize, Error> {
    width
        .checked_mul(height)
        .filter(|&pixels| pixels <= MAX_CANVAS_PIXELS)
        .ok_or(Error::BadHeader {
            field: "canvas size",
        })
}

/// The most bytes a tile stream may decompress to: the HEP header, one index per pixel of the
/// tile, and the palette.
fn tile_bytes(layout: &Layout) -> Result<usize, Error> {
    layout
        .tile_width
        .checked_mul(layout.tile_height)
        .and_then(|pixels| pixels.checked_add(HEP_HEADER + PALETTE_BYTES))
        .ok_or(Error::BadHeader { field: "tile size" })
}

/// Decodes one tile stream of a layout, with the tile's size as the limit.
fn decode_tile(index: usize, stream: &[u8], layout: &Layout) -> Result<Tile, Error> {
    let decoded = hunex_mzx::decompress_within(stream, tile_bytes(layout)?)?;
    parse_tile(index, &decoded, layout)
}

/// The palette slot an atlas index reads. Atlas palettes are stored with bits 3 and 4 of the
/// index swapped; the swap is its own inverse, so the same map writes and reads.
fn atlas_slot(index: u8) -> usize {
    let i = usize::from(index);
    (i & 0xE7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)
}

/// Decodes a font atlas: one palette-indexed image, with the palette in the header entry.
fn decode_atlas(header: &[u8], layout: &Layout, pixels: &[&[u8]]) -> Result<Image, Error> {
    let (width, height) = layout.image_size();
    let count = canvas_pixels(width, height)?;
    let palette = &header[HEADER_BYTES..HEADER_BYTES + PALETTE_BYTES];
    let indices = hunex_mzx::decompress_within(pixels[0], count)?;
    if indices.len() != count {
        return Err(Error::BadHeader {
            field: "atlas pixel count",
        });
    }
    let mut rgba = Vec::with_capacity(indices.len() * 4);
    for &index in &indices {
        let at = atlas_slot(index) * 4;
        rgba.extend_from_slice(&[
            palette[at],
            palette[at + 1],
            palette[at + 2],
            decode_alpha(palette[at + 3]),
        ]);
    }
    Ok(Image {
        layout: layout.clone(),
        width,
        height,
        rgba,
    })
}

/// Decodes an image archive from its container entries.
pub fn decode(entries: &[&[u8]]) -> Result<Image, Error> {
    let (header, tiles) = entries.split_first().ok_or(Error::NoEntries)?;
    let layout = parse_header(header)?;
    if tiles.len() != layout.tiles() {
        return Err(Error::BadHeader {
            field: "entry count",
        });
    }
    let (width, height) = layout.image_size();
    let pixels = canvas_pixels(width, height)?;
    if layout.bitmap_type == ATLAS_TYPE {
        return decode_atlas(header, &layout, tiles);
    }
    if layout.across * layout.tile_width < width || layout.down * layout.tile_height < height {
        return Err(Error::BadHeader { field: "tile grid" });
    }
    let mut rgba = vec![0u8; pixels * 4];
    for row in 0..layout.down {
        for column in 0..layout.across {
            let index = row * layout.across + column;
            let tile = decode_tile(index, tiles[index], &layout)?;
            for y in 0..layout.tile_height {
                for x in 0..layout.tile_width {
                    let (canvas_x, canvas_y) =
                        (column * layout.tile_width + x, row * layout.tile_height + y);
                    if canvas_x < width && canvas_y < height {
                        let color =
                            tile.palette[usize::from(tile.indices[y * layout.tile_width + x])];
                        let at = (canvas_y * width + canvas_x) * 4;
                        rgba[at..at + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }
    Ok(Image {
        layout,
        width,
        height,
        rgba,
    })
}

/// Decodes an MZP image archive.
pub fn decode_mzp(bytes: &[u8]) -> Result<Image, Error> {
    let Container { entries } = hunex_mzp::parse(bytes)?;
    decode(&entries)
}

/// Encodes RGBA pixels as an image archive (its container entries). With a template, the header
/// is copied from it and the image must be the template's canvas size. Without one, the layout is
/// 512x256 tiles covering the image. The image is padded with transparent pixels to whole tiles.
pub fn encode(
    rgba: &[u8],
    width: usize,
    height: usize,
    template: Option<&Layout>,
) -> Result<Vec<Vec<u8>>, Error> {
    if rgba.len() != width * height * 4 {
        return Err(Error::SizeMismatch {
            width,
            height,
            expected_width: width,
            expected_height: height,
        });
    }
    let mut layout = match template {
        Some(template) => {
            let (expected_width, expected_height) = template.image_size();
            if (expected_width, expected_height) != (width, height) {
                return Err(Error::SizeMismatch {
                    width,
                    height,
                    expected_width,
                    expected_height,
                });
            }
            if template.bitmap_type == ATLAS_TYPE {
                return encode_atlas(rgba, template);
            }
            template.clone()
        }
        None => Layout::default_for(width, height),
    };
    let (padded_width, padded_height) = (
        layout.across * layout.tile_width,
        layout.down * layout.tile_height,
    );
    canvas_pixels(padded_width, padded_height)?;
    if padded_width < width || padded_height < height {
        return Err(Error::BadHeader { field: "tile grid" });
    }
    let canvas = pad(rgba, width, height, padded_width, padded_height);
    let mut entries: Vec<Vec<u8>> = Vec::with_capacity(1 + layout.tiles());
    let mut transparency = Vec::with_capacity(layout.tiles());
    let mut tiles = Vec::with_capacity(layout.tiles());
    for row in 0..layout.down {
        for column in 0..layout.across {
            let (hep, byte) = encode_tile(&layout, &canvas, padded_width, column, row);
            transparency.push(byte);
            tiles.push(hunex_mzx::compress(&hep));
        }
    }
    layout.transparency = transparency;
    let mut header = header_fields(&layout)?;
    header.extend_from_slice(&layout.transparency);
    entries.push(header);
    entries.extend(tiles);
    Ok(entries)
}

/// Encodes and writes an MZP image archive.
pub fn encode_mzp(
    rgba: &[u8],
    width: usize,
    height: usize,
    template: Option<&Layout>,
) -> Result<Vec<u8>, Error> {
    let entries = encode(rgba, width, height, template)?;
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    Ok(hunex_mzp::write(&refs)?)
}

/// What `encode_like` re-encoded: a tile that has at least one changed pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct TileReport {
    pub index: usize,
    pub changed_pixels: usize,
    pub colours: usize,
    pub quantised: bool,
    /// The largest per-channel difference between the intended tile and the stored one.
    pub max_error: u8,
    /// The mean per-channel difference over the whole tile.
    pub mean_error: f64,
}

/// Encodes an image against the entries of its original archive. A pixel that equals the original
/// decoded image keeps the original tile's own pixel, so the original tile bytes are kept. Tiles
/// with no changed pixel are copied byte for byte. A changed tile is encoded from its pixels: with
/// an exact palette when it has 256 colours or fewer, and quantised otherwise.
pub fn encode_like(
    rgba: &[u8],
    width: usize,
    height: usize,
    original: &[&[u8]],
) -> Result<(Vec<Vec<u8>>, Vec<TileReport>), Error> {
    let (header, tiles) = original.split_first().ok_or(Error::NoEntries)?;
    let layout = parse_header(header)?;
    if layout.bitmap_type == ATLAS_TYPE {
        let unchanged = (width, height) == layout.image_size() && decode(original)?.rgba == rgba;
        if unchanged {
            return Ok((
                original.iter().map(|entry| entry.to_vec()).collect(),
                Vec::new(),
            ));
        }
        return Ok((encode(rgba, width, height, Some(&layout))?, Vec::new()));
    }
    let (expected_width, expected_height) = layout.image_size();
    if rgba.len() != width * height * 4 || (expected_width, expected_height) != (width, height) {
        return Err(Error::SizeMismatch {
            width,
            height,
            expected_width,
            expected_height,
        });
    }
    let before = decode(original)?;
    let (tile_width, tile_height) = (layout.tile_width, layout.tile_height);
    let mut transparency = Vec::with_capacity(layout.tiles());
    let mut entries = Vec::with_capacity(1 + layout.tiles());
    let mut reports = Vec::new();
    for row in 0..layout.down {
        for column in 0..layout.across {
            let index = row * layout.across + column;
            let original_tile = decode_tile(index, tiles[index], &layout)?;
            let mut intended = Vec::with_capacity(tile_width * tile_height);
            let mut changed = 0usize;
            for y in 0..tile_height {
                for x in 0..tile_width {
                    let (canvas_x, canvas_y) = (column * tile_width + x, row * tile_height + y);
                    let inside = canvas_x < width && canvas_y < height;
                    let at = (canvas_y * width + canvas_x) * 4;
                    if !inside || rgba[at..at + 4] == before.rgba[at..at + 4] {
                        let k = y * tile_width + x;
                        intended.push(original_tile.palette[usize::from(original_tile.indices[k])]);
                    } else {
                        changed += 1;
                        let p = &rgba[at..at + 4];
                        intended.push([
                            p[0],
                            p[1],
                            p[2],
                            decode_alpha(ALPHA_CODES[usize::from(p[3])]),
                        ]);
                    }
                }
            }
            if changed == 0 {
                transparency.push(layout.transparency[index]);
                entries.push(tiles[index].to_vec());
                continue;
            }
            let (hep, byte) = encode_tile_pixels(&layout, &intended);
            let stored = parse_tile(index, &hep, &layout)?;
            let (mut max_error, mut error_sum) = (0u8, 0u64);
            for (k, want) in intended.iter().enumerate() {
                let got = stored.palette[usize::from(stored.indices[k])];
                for channel in 0..4 {
                    let d = want[channel].abs_diff(got[channel]);
                    max_error = max_error.max(d);
                    error_sum += u64::from(d);
                }
            }
            let mut distinct = intended.clone();
            distinct.sort_unstable();
            distinct.dedup();
            transparency.push(byte);
            entries.push(hunex_mzx::compress(&hep));
            reports.push(TileReport {
                index,
                changed_pixels: changed,
                colours: distinct.len(),
                quantised: distinct.len() > PALETTE_ENTRIES,
                max_error,
                mean_error: error_sum as f64 / (intended.len() * 4) as f64,
            });
        }
    }
    let mut header = header_fields(&layout)?;
    header.extend_from_slice(&transparency);
    let mut all = vec![header];
    all.extend(entries);
    Ok((all, reports))
}

/// `encode_like` for a whole MZP archive: the original file's bytes in, the new file's bytes out.
pub fn encode_like_mzp(
    rgba: &[u8],
    width: usize,
    height: usize,
    original: &[u8],
) -> Result<(Vec<u8>, Vec<TileReport>), Error> {
    let Container { entries } = hunex_mzp::parse(original)?;
    let (packed, reports) = encode_like(rgba, width, height, &entries)?;
    let refs: Vec<&[u8]> = packed.iter().map(Vec::as_slice).collect();
    Ok((hunex_mzp::write(&refs)?, reports))
}

fn pad(
    rgba: &[u8],
    width: usize,
    height: usize,
    padded_width: usize,
    padded_height: usize,
) -> Vec<u8> {
    if (width, height) == (padded_width, padded_height) {
        return rgba.to_vec();
    }
    let mut out = vec![0u8; padded_width * padded_height * 4];
    for y in 0..height {
        let source = &rgba[y * width * 4..(y + 1) * width * 4];
        out[y * padded_width * 4..y * padded_width * 4 + width * 4].copy_from_slice(source);
    }
    out
}

/// Builds the HEP tile for one grid cell from the padded canvas. Returns the HEP bytes and the
/// tile's transparency byte.
fn encode_tile(
    layout: &Layout,
    canvas: &[u8],
    canvas_width: usize,
    column: usize,
    row: usize,
) -> (Vec<u8>, u8) {
    let (tile_width, tile_height) = (layout.tile_width, layout.tile_height);
    let mut pixels: Vec<[u8; 4]> = Vec::with_capacity(tile_width * tile_height);
    for y in 0..tile_height {
        for x in 0..tile_width {
            let at = ((row * tile_height + y) * canvas_width + column * tile_width + x) * 4;
            let [r, g, b, a] = [canvas[at], canvas[at + 1], canvas[at + 2], canvas[at + 3]];
            pixels.push([r, g, b, decode_alpha(ALPHA_CODES[usize::from(a)])]);
        }
    }
    encode_tile_pixels(layout, &pixels)
}

/// Encodes one tile from all of its pixels, with alpha already in the decodable set. The palette
/// is exact when the tile has 256 colours or fewer; the transparency byte covers the whole tile.
fn encode_tile_pixels(layout: &Layout, pixels: &[[u8; 4]]) -> (Vec<u8>, u8) {
    let (tile_width, tile_height) = (layout.tile_width, layout.tile_height);
    let (palette, indices) = build_palette(pixels);

    let mut alphas = (false, false);
    for &index in &indices {
        let alpha = palette[usize::from(index)][3];
        alphas = (alphas.0 || alpha != 0, alphas.1 || alpha != 255);
    }
    let transparency = match alphas {
        (false, _) => 0,
        (true, false) => 2,
        _ => 1,
    };

    let mut hep = Vec::with_capacity(HEP_HEADER + indices.len() + PALETTE_BYTES);
    let size = (HEP_HEADER + indices.len() + PALETTE_BYTES) as u32;
    hep.extend_from_slice(HEP_MAGIC);
    hep.extend_from_slice(&size.to_le_bytes());
    for constant in HEP_CONSTANTS {
        hep.extend_from_slice(&constant.to_le_bytes());
    }
    hep.extend_from_slice(&(tile_width as u32).to_le_bytes());
    hep.extend_from_slice(&(tile_height as u32).to_le_bytes());
    hep.extend_from_slice(&HEP_TRANSPARENCY.to_le_bytes());
    hep.extend_from_slice(&indices);
    write_palette(&mut hep, &palette);
    (hep, transparency)
}

/// The 16 header bytes: the six u16 fields, bitmap type, depth, crop and flags.
fn header_fields(layout: &Layout) -> Result<Vec<u8>, Error> {
    let mut header = Vec::with_capacity(HEADER_BYTES);
    for value in [
        layout.canvas_width,
        layout.canvas_height,
        layout.tile_width,
        layout.tile_height,
        layout.across,
        layout.down,
    ] {
        let value = u16::try_from(value).map_err(|_| Error::BadHeader {
            field: "header field beyond 65535",
        })?;
        header.extend_from_slice(&value.to_le_bytes());
    }
    header.extend_from_slice(&[
        layout.bitmap_type,
        layout.depth,
        layout.crop as u8,
        layout.flags,
    ]);
    Ok(header)
}

/// Writes 256 palette entries as R, G, B and the 7-bit alpha code; unused entries are zero.
fn write_palette(out: &mut Vec<u8>, palette: &[[u8; 4]]) {
    for slot in 0..PALETTE_ENTRIES {
        match palette.get(slot) {
            Some(color) => out.extend_from_slice(&[
                color[0],
                color[1],
                color[2],
                ALPHA_CODES[usize::from(color[3])],
            ]),
            None => out.extend_from_slice(&[0, 0, 0, 0]),
        }
    }
}

/// Encodes a font atlas: the whole image as one palette-indexed MZX stream.
fn encode_atlas(rgba: &[u8], template: &Layout) -> Result<Vec<Vec<u8>>, Error> {
    let pixels: Vec<[u8; 4]> = rgba
        .chunks_exact(4)
        .map(|p| {
            [
                p[0],
                p[1],
                p[2],
                decode_alpha(ALPHA_CODES[usize::from(p[3])]),
            ]
        })
        .collect();
    let (palette, indices) = build_palette(&pixels);
    let mut alphas = (false, false);
    for &index in &indices {
        let alpha = palette[usize::from(index)][3];
        alphas = (alphas.0 || alpha != 0, alphas.1 || alpha != 255);
    }
    let transparency = match alphas {
        (false, _) => 0,
        (true, false) => 2,
        _ => 1,
    };
    let mut slots = vec![[0u8; 4]; PALETTE_ENTRIES];
    for (index, color) in palette.iter().enumerate() {
        slots[atlas_slot(index as u8)] = *color;
    }
    let mut header = header_fields(template)?;
    write_palette(&mut header, &slots);
    header.push(transparency);
    Ok(vec![header, hunex_mzx::compress(&indices)])
}

/// Builds a palette of at most 256 colours for the pixels, and maps each pixel to it. Exact when
/// there are at most 256 distinct colours; otherwise a median cut, which is lossy.
fn build_palette(pixels: &[[u8; 4]]) -> (Vec<[u8; 4]>, Vec<u8>) {
    let mut first_seen: Vec<[u8; 4]> = Vec::new();
    let mut counts: BTreeMap<[u8; 4], usize> = BTreeMap::new();
    for &pixel in pixels {
        let entry = counts.entry(pixel).or_insert(0);
        if *entry == 0 {
            first_seen.push(pixel);
        }
        *entry += 1;
    }
    if first_seen.len() <= PALETTE_ENTRIES {
        let index: BTreeMap<[u8; 4], u8> = first_seen
            .iter()
            .enumerate()
            .map(|(i, &c)| (c, i as u8))
            .collect();
        let mapped = pixels.iter().map(|pixel| index[pixel]).collect();
        return (first_seen, mapped);
    }
    let palette = median_cut(&first_seen, &counts);
    let mapped_colours: BTreeMap<[u8; 4], u8> = first_seen
        .iter()
        .map(|colour| (*colour, nearest(&palette, colour)))
        .collect();
    let mapped = pixels.iter().map(|pixel| mapped_colours[pixel]).collect();
    (palette, mapped)
}

fn nearest(palette: &[[u8; 4]], colour: &[u8; 4]) -> u8 {
    let mut best = 0usize;
    let mut best_distance = u64::MAX;
    for (index, entry) in palette.iter().enumerate() {
        let distance: u64 = (0..4)
            .map(|channel| {
                let difference = i64::from(entry[channel]) - i64::from(colour[channel]);
                (difference * difference) as u64
            })
            .sum();
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    best as u8
}

fn median_cut(colours: &[[u8; 4]], counts: &BTreeMap<[u8; 4], usize>) -> Vec<[u8; 4]> {
    let mut boxes: Vec<Vec<[u8; 4]>> = vec![colours.to_vec()];
    while boxes.len() < PALETTE_ENTRIES {
        let mut choice = None;
        let mut widest = 0u32;
        for (index, cube) in boxes.iter().enumerate() {
            if cube.len() < 2 {
                continue;
            }
            for channel in 0..4 {
                let low = cube.iter().map(|c| c[channel]).min().unwrap_or(0);
                let high = cube.iter().map(|c| c[channel]).max().unwrap_or(0);
                let range = u32::from(high - low);
                if range > widest {
                    widest = range;
                    choice = Some((index, channel));
                }
            }
        }
        let Some((index, channel)) = choice else {
            break;
        };
        let mut cube = boxes.swap_remove(index);
        cube.sort_by_key(|c| (c[channel], *c));
        let total: usize = cube.iter().map(|c| counts[c]).sum();
        let mut running = 0usize;
        let mut split = cube.len() - 1;
        for (position, colour) in cube.iter().enumerate() {
            running += counts[colour];
            if running * 2 >= total {
                split = (position + 1).min(cube.len() - 1);
                break;
            }
        }
        let upper = cube.split_off(split);
        boxes.push(cube);
        boxes.push(upper);
    }
    boxes
        .iter()
        .map(|cube| {
            let total: u64 = cube.iter().map(|c| counts[c] as u64).sum();
            let mut mean = [0u8; 4];
            for (channel, slot) in mean.iter_mut().enumerate() {
                let sum: u64 = cube
                    .iter()
                    .map(|c| u64::from(c[channel]) * counts[c] as u64)
                    .sum();
                *slot = ((sum + total / 2) / total) as u8;
            }
            mean
        })
        .collect()
}
