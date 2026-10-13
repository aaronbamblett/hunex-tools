//! MZP containers: `mrgd00`, an entry count, 8-byte records, then the entries.
//!
//! A record holds four u16 values: the sector index and the byte within it (together, the
//! entry offset from the start of the data), the sector span of the entry, and the low 16 bits
//! of its size. The size is the one value that matches both the low bits and the span.
//! Entries follow each other, and each is padded with 0xFF to the next 8-byte boundary.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::fmt;

pub const MAGIC: &[u8; 6] = b"mrgd00";
pub const SECTOR: usize = 0x800;
const HEADER: usize = 8;
const RECORD: usize = 8;
const PAD_BYTE: u8 = 0xFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BadMagic,
    Truncated {
        offset: usize,
    },
    /// No entry size fits the record's low bits and span.
    BadSize {
        index: usize,
    },
    /// More than one entry size fits the record.
    AmbiguousSize {
        index: usize,
    },
    TooLarge,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not an MZP container"),
            Error::Truncated { offset } => write!(f, "container truncated at 0x{offset:x}"),
            Error::BadSize { index } => write!(f, "entry {index}: no size matches its record"),
            Error::AmbiguousSize { index } => {
                write!(f, "entry {index}: more than one size matches its record")
            }
            Error::TooLarge => write!(f, "container too large for 16-bit sector fields"),
        }
    }
}

impl core::error::Error for Error {}

/// Entries of a container, borrowed from the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container<'a> {
    pub entries: Vec<&'a [u8]>,
}

/// The sector span of `[offset, offset + size)`.
fn span(offset: usize, size: usize) -> usize {
    (offset + size).div_ceil(SECTOR) - offset / SECTOR
}

/// Parses a container with the sector-span size rule of the PC files.
pub fn parse(bytes: &[u8]) -> Result<Container<'_>, Error> {
    parse_with(bytes, false)
}

/// Parses a container with the Switch size rule: the span times 0x800, with its low 16 bits
/// replaced by the stored low 16 bits. Layout and magic are the same as the PC files.
pub fn parse_switch(bytes: &[u8]) -> Result<Container<'_>, Error> {
    parse_with(bytes, true)
}

fn parse_with(bytes: &[u8], switch_rule: bool) -> Result<Container<'_>, Error> {
    if !bytes.starts_with(MAGIC) {
        return Err(Error::BadMagic);
    }
    let count = usize::from(read_u16(bytes, 6)?);
    let table_end = HEADER + RECORD * count;
    if table_end > bytes.len() {
        return Err(Error::Truncated {
            offset: bytes.len(),
        });
    }
    let data_start = table_end;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let record = HEADER + RECORD * index;
        let sector = usize::from(read_u16(bytes, record)?);
        let within = usize::from(read_u16(bytes, record + 2)?);
        let stored_span = usize::from(read_u16(bytes, record + 4)?);
        let low = usize::from(read_u16(bytes, record + 6)?);
        let offset = sector * SECTOR + within;
        let start = data_start
            .checked_add(offset)
            .ok_or(Error::BadSize { index })?;
        let available = bytes.len().saturating_sub(start);
        if switch_rule {
            let size = ((stored_span * SECTOR) & !0xFFFF) | low;
            if size > available {
                return Err(Error::BadSize { index });
            }
            let entry = bytes
                .get(start..start + size)
                .ok_or(Error::BadSize { index })?;
            entries.push(entry);
            continue;
        }
        let candidates: Vec<usize> = (0..)
            .map(|step| low + step * 0x1_0000)
            .take_while(|&size| size <= available)
            .filter(|&size| span(offset, size) == stored_span)
            .collect();
        let size = match candidates.as_slice() {
            [] => return Err(Error::BadSize { index }),
            [size] => *size,
            _ => return Err(Error::AmbiguousSize { index }),
        };
        let entry = bytes
            .get(start..start + size)
            .ok_or(Error::BadSize { index })?;
        entries.push(entry);
    }
    Ok(Container { entries })
}

/// One entry's place in a container: its offset from the start of the data and its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub offset: u64,
    pub size: usize,
}

/// The header length of a Switch container from its first eight bytes.
pub fn switch_header_len(first: &[u8]) -> Result<usize, Error> {
    if !first.starts_with(MAGIC) {
        return Err(Error::BadMagic);
    }
    Ok(HEADER + RECORD * usize::from(read_u16(first, 6)?))
}

/// The entry spans of a Switch container, from its header and table (`switch_header_len` bytes).
/// Offsets are relative to the start of the data, which follows the table.
pub fn switch_spans(table: &[u8]) -> Result<Vec<Span>, Error> {
    let count = usize::from(read_u16(table, 6)?);
    let mut spans = Vec::with_capacity(count);
    for index in 0..count {
        let record = HEADER + RECORD * index;
        let sector = usize::from(read_u16(table, record)?);
        let within = usize::from(read_u16(table, record + 2)?);
        let stored_span = usize::from(read_u16(table, record + 4)?);
        let low = usize::from(read_u16(table, record + 6)?);
        let size = ((stored_span * SECTOR) & !0xFFFF) | low;
        spans.push(Span {
            offset: (sector * SECTOR + within) as u64,
            size,
        });
    }
    Ok(spans)
}

/// Writes a container. Entries are padded with 0xFF to the next 8-byte boundary, the last one too.
pub fn write(entries: &[&[u8]]) -> Result<Vec<u8>, Error> {
    let count = u16::try_from(entries.len()).map_err(|_| Error::TooLarge)?;
    let mut table = Vec::with_capacity(RECORD * entries.len());
    let mut data = Vec::new();
    let mut cursor = 0usize;
    for entry in entries {
        let sector = cursor / SECTOR;
        let within = cursor % SECTOR;
        let sector = u16::try_from(sector).map_err(|_| Error::TooLarge)?;
        table.extend_from_slice(&sector.to_le_bytes());
        table.extend_from_slice(&(within as u16).to_le_bytes());
        table.extend_from_slice(&(span(cursor, entry.len()) as u16).to_le_bytes());
        table.extend_from_slice(&((entry.len() & 0xFFFF) as u16).to_le_bytes());
        data.extend_from_slice(entry);
        let pad = 8 - entry.len() % 8;
        data.extend(core::iter::repeat_n(PAD_BYTE, pad));
        cursor += entry.len() + pad;
    }
    let mut out = Vec::with_capacity(HEADER + table.len() + data.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&table);
    out.extend_from_slice(&data);
    Ok(out)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, Error> {
    match bytes.get(offset..offset + 2) {
        Some(&[a, b]) => Ok(u16::from_le_bytes([a, b])),
        _ => Err(Error::Truncated { offset }),
    }
}
