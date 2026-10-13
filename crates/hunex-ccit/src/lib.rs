//! Glyph tables (`.ccit`) of the HuneX engine.
//!
//! A decompressed table starts with the glyph count N and the record size 8. N records of
//! (code, index) follow. The coordinate block is either N, the record size 20 and N records of
//! (page, x, y, width, height), or a single zero word when the table has no coordinates.
//! A code is the glyph's UTF-8 bytes read as one big-endian integer. Records are in code
//! order, and the indices are 0 to N-1 with no repeats.
//!
//! [`Table::parse`] reads a table exactly, and [`Table::to_bytes`] writes it back byte for byte.
//! [`Table::insert`] and [`Table::replace`] change a parsed table and keep it in that order.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;

const RECORD_SIZE: u32 = 8;
const COORD_SIZE: u32 = 20;

/// Glyphs per atlas page: an atlas holds 14 columns of 7 rows of 73 px cells.
pub const GLYPHS_PER_PAGE: u32 = 98;
/// Columns in one atlas page.
pub const ATLAS_COLUMNS: u32 = 14;
/// Side of one atlas cell, in pixels.
pub const CELL_SIZE: u32 = 73;

/// One character record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub code: u32,
    pub index: u32,
}

/// The atlas position of one glyph: page, and the pixel rectangle of its cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub page: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A record and its coordinates, when the table has them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub record: Record,
    pub coord: Option<Coord>,
}

/// Where a glyph index sits in the atlas grid: the page and the cell origin in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub page: u32,
    pub x: u32,
    pub y: u32,
}

/// The atlas cell of a glyph index, from its index alone (spec section 4).
pub fn cell_of(index: u32) -> Cell {
    let within = index % GLYPHS_PER_PAGE;
    Cell {
        page: index / GLYPHS_PER_PAGE,
        x: (within % ATLAS_COLUMNS) * CELL_SIZE,
        y: (within / ATLAS_COLUMNS) * CELL_SIZE,
    }
}

/// The code of a character: its UTF-8 bytes as one big-endian integer.
pub fn code_of(c: char) -> u32 {
    let mut buffer = [0u8; 4];
    c.encode_utf8(&mut buffer)
        .bytes()
        .fold(0u32, |code, byte| (code << 8) | u32::from(byte))
}

/// The character with this code, or `None` when the code is not one UTF-8 character.
pub fn char_of(code: u32) -> Option<char> {
    let bytes = code.to_be_bytes();
    let start = bytes.iter().position(|&byte| byte != 0)?;
    let mut chars = core::str::from_utf8(&bytes[start..]).ok()?.chars();
    let c = chars.next()?;
    if chars.next().is_some() || code_of(c) != code {
        return None;
    }
    Some(c)
}

/// A table whose contents could not be read, or could not be changed as asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The bytes end before the field at `offset`.
    Truncated { offset: usize },
    /// The record size in the header is not 8.
    RecordSize { found: u32 },
    /// The coordinate record size is not 20.
    CoordSize { found: u32 },
    /// The coordinate block declares a different glyph count from the header.
    CoordCount { glyphs: u32, coords: u32 },
    /// The coordinate block is not the zero word, or the table is not exactly its length.
    Layout { expected: usize, found: usize },
    /// Two records share a code.
    DuplicateCode { code: u32 },
    /// The records are not in code order.
    Unsorted,
    /// The indices are not 0 to N-1 with no repeats.
    IndexNotDense,
    /// Coordinates were given for a table without them, or left out for one with them.
    CoordsMismatch,
    /// No record has this code.
    NoSuchCode { code: u32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Truncated { offset } => write!(f, "table ends early at 0x{offset:x}"),
            Error::RecordSize { found } => write!(f, "record size {found}, expected 8"),
            Error::CoordSize { found } => write!(f, "coordinate size {found}, expected 20"),
            Error::CoordCount { glyphs, coords } => {
                write!(f, "{coords} coordinates for {glyphs} glyphs")
            }
            Error::Layout { expected, found } => {
                write!(f, "table is {found} bytes, expected {expected}")
            }
            Error::DuplicateCode { code } => write!(f, "code 0x{code:x} appears twice"),
            Error::Unsorted => write!(f, "records are not in code order"),
            Error::IndexNotDense => write!(f, "indices are not 0 to N-1 with no repeats"),
            Error::CoordsMismatch => write!(f, "coordinates do not match the table"),
            Error::NoSuchCode { code } => write!(f, "no record has code 0x{code:x}"),
        }
    }
}

impl core::error::Error for Error {}

/// A glyph table: records in code order, and coordinates for each record when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    records: Vec<Record>,
    coords: Option<Vec<Coord>>,
}

fn word(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let field = bytes
        .get(offset..offset + 4)
        .ok_or(Error::Truncated { offset })?;
    Ok(u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
}

impl Table {
    /// Reads a decompressed table. Every byte must belong to a field.
    pub fn parse(bytes: &[u8]) -> Result<Table, Error> {
        let glyphs = word(bytes, 0)?;
        let record_size = word(bytes, 4)?;
        if record_size != RECORD_SIZE {
            return Err(Error::RecordSize { found: record_size });
        }
        let records_end = 8
            + (glyphs as usize)
                .checked_mul(RECORD_SIZE as usize)
                .ok_or(Error::Layout {
                    expected: usize::MAX,
                    found: bytes.len(),
                })?;
        let mut records = Vec::new();
        for at in (8..records_end).step_by(RECORD_SIZE as usize) {
            records.push(Record {
                code: word(bytes, at)?,
                index: word(bytes, at + 4)?,
            });
        }
        let block = word(bytes, records_end)?;
        if bytes.len() == records_end + 4 {
            if block != 0 {
                return Err(Error::CoordCount {
                    glyphs,
                    coords: block,
                });
            }
            return Ok(Table {
                records,
                coords: None,
            });
        }
        if block != glyphs {
            return Err(Error::CoordCount {
                glyphs,
                coords: block,
            });
        }
        let coord_size = word(bytes, records_end + 4)?;
        if coord_size != COORD_SIZE {
            return Err(Error::CoordSize { found: coord_size });
        }
        let coords_start = records_end + 8;
        let expected = (glyphs as usize)
            .checked_mul(COORD_SIZE as usize)
            .and_then(|block| block.checked_add(coords_start))
            .ok_or(Error::Layout {
                expected: usize::MAX,
                found: bytes.len(),
            })?;
        if bytes.len() != expected {
            return Err(Error::Layout {
                expected,
                found: bytes.len(),
            });
        }
        let mut coords = Vec::with_capacity(records.len());
        for at in (coords_start..expected).step_by(COORD_SIZE as usize) {
            coords.push(Coord {
                page: word(bytes, at)?,
                x: word(bytes, at + 4)?,
                y: word(bytes, at + 8)?,
                width: word(bytes, at + 12)?,
                height: word(bytes, at + 16)?,
            });
        }
        Ok(Table {
            records,
            coords: Some(coords),
        })
    }

    /// Builds a table from parts, checking the invariants that [`Table::insert`] keeps: codes
    /// strictly increasing, indices 0 to N-1 with no repeats, and one coordinate per record.
    pub fn from_parts(records: Vec<Record>, coords: Option<Vec<Coord>>) -> Result<Table, Error> {
        if let Some(coords) = &coords {
            if coords.len() != records.len() {
                return Err(Error::CoordsMismatch);
            }
        }
        if records.windows(2).any(|pair| pair[0].code >= pair[1].code) {
            let duplicate = records
                .windows(2)
                .find(|pair| pair[0].code == pair[1].code)
                .map(|pair| pair[0].code);
            return Err(match duplicate {
                Some(code) => Error::DuplicateCode { code },
                None => Error::Unsorted,
            });
        }
        let mut seen = Vec::new();
        seen.resize(records.len(), false);
        for record in &records {
            let slot = seen
                .get_mut(record.index as usize)
                .ok_or(Error::IndexNotDense)?;
            if *slot {
                return Err(Error::IndexNotDense);
            }
            *slot = true;
        }
        Ok(Table { records, coords })
    }

    /// Writes the table in the layout [`Table::parse`] reads.
    pub fn to_bytes(&self) -> Vec<u8> {
        let glyphs = self.records.len() as u32;
        let mut out = Vec::new();
        out.extend_from_slice(&glyphs.to_le_bytes());
        out.extend_from_slice(&RECORD_SIZE.to_le_bytes());
        for record in &self.records {
            out.extend_from_slice(&record.code.to_le_bytes());
            out.extend_from_slice(&record.index.to_le_bytes());
        }
        match &self.coords {
            None => out.extend_from_slice(&0u32.to_le_bytes()),
            Some(coords) => {
                out.extend_from_slice(&glyphs.to_le_bytes());
                out.extend_from_slice(&COORD_SIZE.to_le_bytes());
                for coord in coords {
                    for value in [coord.page, coord.x, coord.y, coord.width, coord.height] {
                        out.extend_from_slice(&value.to_le_bytes());
                    }
                }
            }
        }
        out
    }

    /// The records, in code order.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// The coordinates, one per record in record order, or `None` for a table without them.
    pub fn coords(&self) -> Option<&[Coord]> {
        self.coords.as_deref()
    }

    /// The glyphs in record order.
    pub fn glyphs(&self) -> impl Iterator<Item = Glyph> + '_ {
        self.records.iter().enumerate().map(|(i, &record)| Glyph {
            record,
            coord: self.coords.as_ref().map(|coords| coords[i]),
        })
    }

    /// The glyph for a character, if the table has one.
    pub fn find(&self, c: char) -> Option<Glyph> {
        self.find_code(code_of(c))
    }

    /// The glyph with this code, if the table has one.
    pub fn find_code(&self, code: u32) -> Option<Glyph> {
        let at = self.records.iter().position(|record| record.code == code)?;
        Some(Glyph {
            record: self.records[at],
            coord: self.coords.as_ref().map(|coords| coords[at]),
        })
    }

    /// Adds a glyph for `code` at the next free index, which is the glyph count, and returns
    /// that index. The record goes in code order. `coord` must be given exactly when the table
    /// has coordinates.
    pub fn insert(&mut self, code: u32, coord: Option<Coord>) -> Result<u32, Error> {
        self.check_coord_presence(coord)?;
        let at = self.position_for(code)?;
        let index = self.records.len() as u32;
        self.records.insert(at, Record { code, index });
        if let (Some(coords), Some(coord)) = (self.coords.as_mut(), coord) {
            coords.insert(at, coord);
        }
        Ok(index)
    }

    /// Replaces the coordinates of the glyph with `code`. Its record and index stay the same.
    pub fn replace(&mut self, code: u32, coord: Option<Coord>) -> Result<(), Error> {
        self.check_coord_presence(coord)?;
        let at = self
            .records
            .iter()
            .position(|record| record.code == code)
            .ok_or(Error::NoSuchCode { code })?;
        if let (Some(coords), Some(coord)) = (self.coords.as_mut(), coord) {
            coords[at] = coord;
        }
        Ok(())
    }

    fn check_coord_presence(&self, coord: Option<Coord>) -> Result<(), Error> {
        if self.coords.is_some() == coord.is_some() {
            Ok(())
        } else {
            Err(Error::CoordsMismatch)
        }
    }

    fn position_for(&self, code: u32) -> Result<usize, Error> {
        if self
            .records
            .windows(2)
            .any(|pair| pair[0].code >= pair[1].code)
        {
            return Err(Error::Unsorted);
        }
        let at = self.records.partition_point(|record| record.code < code);
        match self.records.get(at) {
            Some(record) if record.code == code => Err(Error::DuplicateCode { code }),
            _ => Ok(at),
        }
    }
}

#[cfg(test)]
mod tests;
