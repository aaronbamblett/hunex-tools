//! MRG sets: an index (`.hed`), a data file (`.mrg`) and names (`.nam`).
//!
//! An index row is eight little-endian bytes: the start sector (u32), the on-disk length in
//! sectors (u16), and the decompressed length in sectors (u16). Sectors are 2048 bytes. Rows of
//! zeros are empty slots, and a row whose start is 0xFFFFFFFF ends the index. A name row is 32
//! bytes: the name, zero padded, then CR LF. Row `i` names index row `i`.
//!
//! Entry data is read as whole sectors, so an entry's slice can end in padding; the formats
//! inside (NXX wrappers, MZP containers, JPG and PNG files) carry their own lengths.
//!
//! ```
//! // One index row (start sector 0, one sector on disk, one decompressed) and the end marker row.
//! let hed = [0, 0, 0, 0, 1, 0, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0];
//! let rows = hunex_mrg::parse_index(&hed).unwrap();
//! assert_eq!(rows.len(), 1);
//! assert_eq!(rows[0].start_sector, 0);
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Size of one sector in bytes. Entry data starts on sector boundaries.
pub const SECTOR: usize = 2048;
/// The value that ends the index: an index row whose start sector is this value.
pub const TERMINATOR: u32 = 0xFFFF_FFFF;
/// Size of one name row in the `.nam` file, in bytes.
pub const NAME_ROW: usize = 32;
/// Length of the name in a name row, not counting the line ending.
pub const NAME_LEN: usize = 30;
/// Name of the placeholder entry that stands for an empty slot.
pub const PLACEHOLDER: &str = "NOFILE.EXISTS";

/// Errors from reading an MRG set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The index or name table ends before the field at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// The index ends without a terminator row.
    NoTerminator,
    /// An entry's sectors run past the end of the data file.
    EntryPastEnd {
        /// Index of the entry that is not valid.
        index: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Truncated { offset } => write!(f, "MRG file truncated at 0x{offset:x}"),
            Error::NoTerminator => write!(f, "MRG index has no terminator row"),
            Error::EntryPastEnd { index } => {
                write!(f, "MRG entry {index} runs past the end of the data")
            }
        }
    }
}

impl core::error::Error for Error {}

/// One index row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// First sector of the entry's data in the `.mrg` file.
    pub start_sector: u32,
    /// Sectors the entry occupies on disk.
    pub on_disk_sectors: u16,
    /// Sectors the entry occupies after decompression, as recorded in the index.
    pub decompressed_sectors: u16,
}

impl Row {
    /// True for an empty slot: all three fields are zero.
    pub fn is_empty(&self) -> bool {
        self.start_sector == 0 && self.on_disk_sectors == 0 && self.decompressed_sectors == 0
    }

    /// The entry's first byte in the data file.
    pub fn byte_start(&self) -> u64 {
        u64::from(self.start_sector) * SECTOR as u64
    }

    /// The entry's sector span in bytes (it can run past the data file's end).
    pub fn byte_span(&self) -> u64 {
        u64::from(self.on_disk_sectors) * SECTOR as u64
    }
}

/// The kind of an entry, from its first bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Zero-length or empty slot.
    Empty,
    /// zlib-wrapped (`NXCX`) or gzip-wrapped (`NXGX`) texture or data.
    Nxx,
    /// An MZP container (`mrgd00`).
    Mzp,
    /// Anything else (JPG, PNG, BIN, DAT).
    Raw,
}

/// Classifies the data of an entry by its first bytes.
pub fn kind(data: &[u8]) -> Kind {
    if data.is_empty() {
        Kind::Empty
    } else if data.starts_with(b"NXCX") || data.starts_with(b"NXGX") {
        Kind::Nxx
    } else if data.starts_with(b"mrgd00") {
        Kind::Mzp
    } else {
        Kind::Raw
    }
}

/// One entry of a set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    /// Position of the entry in the index.
    pub index: usize,
    /// Name from the `.nam` file, or `None` when the name table is missing.
    pub name: Option<String>,
    /// The index row of the entry.
    pub row: Row,
    /// The entry's sectors, or an empty slice for an empty slot.
    pub data: &'a [u8],
}

impl Entry<'_> {
    /// True when the entry is the placeholder for an empty slot.
    pub fn is_placeholder(&self) -> bool {
        self.name.as_deref() == Some(PLACEHOLDER)
    }
}

/// An opened set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set<'a> {
    /// The entries in index order.
    pub entries: Vec<Entry<'a>>,
}

/// Parses an index. Rows stop at the first terminator; a missing terminator is an error.
pub fn parse_index(hed: &[u8]) -> Result<Vec<Row>, Error> {
    let mut rows = Vec::new();
    for (index, chunk) in hed.chunks(8).enumerate() {
        let offset = index * 8;
        if chunk.len() < 8 {
            return Err(Error::Truncated { offset });
        }
        let start = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if start == TERMINATOR {
            return Ok(rows);
        }
        rows.push(Row {
            start_sector: start,
            on_disk_sectors: u16::from_le_bytes([chunk[4], chunk[5]]),
            decompressed_sectors: u16::from_le_bytes([chunk[6], chunk[7]]),
        });
    }
    Err(Error::NoTerminator)
}

/// Parses names. Rows stop at an all-zero row; an empty name is kept as an empty string.
pub fn parse_names(nam: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for row in nam.chunks(NAME_ROW) {
        if row.len() < NAME_ROW || row.iter().all(|&byte| byte == 0) {
            break;
        }
        let field = &row[..NAME_LEN];
        let end = field.iter().position(|&byte| byte == 0).unwrap_or(NAME_LEN);
        names.push(String::from_utf8_lossy(&field[..end]).into_owned());
    }
    names
}

/// Opens a set from its index, data and (optional) names.
pub fn open<'a>(hed: &[u8], mrg: &'a [u8], nam: Option<&[u8]>) -> Result<Set<'a>, Error> {
    let rows = parse_index(hed)?;
    let names = nam.map(parse_names).unwrap_or_default();
    let mut entries = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        let data = if row.is_empty() {
            &mrg[..0]
        } else {
            let begin = (row.start_sector as usize)
                .checked_mul(SECTOR)
                .filter(|&begin| begin <= mrg.len())
                .ok_or(Error::EntryPastEnd { index })?;
            let end = begin
                .saturating_add(usize::from(row.on_disk_sectors) * SECTOR)
                .min(mrg.len());
            &mrg[begin..end]
        };
        entries.push(Entry {
            index,
            name: names.get(index).cloned(),
            row,
            data,
        });
    }
    Ok(Set { entries })
}

/// One index row with its name, without any entry data. Used to read large data files
/// entry by entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta {
    /// Position of the entry in the index.
    pub index: usize,
    /// Name from the `.nam` file, or `None` when the name table is missing.
    pub name: Option<String>,
    /// The index row of the entry.
    pub row: Row,
}

/// Reads the index and names of a set.
pub fn index(hed: &[u8], nam: Option<&[u8]>) -> Result<alloc::vec::Vec<Meta>, Error> {
    let rows = parse_index(hed)?;
    let names = nam.map(parse_names).unwrap_or_default();
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| Meta {
            index,
            name: names.get(index).cloned(),
            row,
        })
        .collect())
}

/// Errors from writing an MRG set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    /// An entry is longer than its index row can count, in sectors.
    TooLarge {
        /// Index of the entry.
        index: usize,
    },
    /// The data would start past the 32-bit sector field of the index.
    TooManySectors,
    /// A name does not fit the 30-byte name field, or contains a NUL or a line break.
    BadName {
        /// Index of the slot.
        index: usize,
    },
    /// The number of names does not match the number of slots.
    NameCount {
        /// Number of slots.
        expected: usize,
        /// Number of names given.
        found: usize,
    },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            WriteError::TooLarge { index } => {
                write!(f, "MRG entry {index} is too large for its index row")
            }
            WriteError::TooManySectors => write!(f, "MRG data is too large for 32-bit sectors"),
            WriteError::BadName { index } => {
                write!(f, "MRG name of slot {index} does not fit the 30-byte field")
            }
            WriteError::NameCount { expected, found } => {
                write!(f, "MRG set has {expected} slots but {found} names")
            }
        }
    }
}

impl core::error::Error for WriteError {}

/// The sector count of an entry of `len` bytes: whole sectors, with the last one padded.
pub fn sectors_for(len: usize) -> Result<u16, WriteError> {
    u16::try_from(len.div_ceil(SECTOR)).map_err(|_| WriteError::TooLarge { index: 0 })
}

/// The decompressed sector count to record for an entry.
///
/// `head` holds the first eight bytes of the entry. A `NXCX` or `NXGX` wrapper records its
/// decompressed size at byte 4; the count is that size in sectors. Every other entry records
/// the same count as its on-disk sectors.
pub fn decompressed_sectors(head: &[u8], on_disk: u16) -> u16 {
    if head.len() >= 8 && (head.starts_with(b"NXCX") || head.starts_with(b"NXGX")) {
        let size = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        u16::try_from(size.div_ceil(SECTOR)).unwrap_or(u16::MAX)
    } else {
        on_disk
    }
}

/// Plans the index rows of a set. Each slot is `Some((sectors, decompressed_sectors))` for an
/// entry or `None` for an empty slot. Entries are placed back to back in slot order, from
/// sector zero.
pub fn plan(slots: &[Option<(u16, u16)>]) -> Result<Vec<Row>, WriteError> {
    let mut next: u64 = 0;
    let mut rows = Vec::with_capacity(slots.len());
    for slot in slots {
        match *slot {
            None => rows.push(Row {
                start_sector: 0,
                on_disk_sectors: 0,
                decompressed_sectors: 0,
            }),
            Some((on_disk, decompressed)) => {
                let start = u32::try_from(next).map_err(|_| WriteError::TooManySectors)?;
                rows.push(Row {
                    start_sector: start,
                    on_disk_sectors: on_disk,
                    decompressed_sectors: decompressed,
                });
                next += u64::from(on_disk);
            }
        }
    }
    if next > u64::from(u32::MAX) {
        return Err(WriteError::TooManySectors);
    }
    Ok(rows)
}

/// Serializes an index: one eight-byte row per slot, then the two terminator rows.
pub fn index_bytes(rows: &[Row]) -> Vec<u8> {
    let mut out = Vec::with_capacity((rows.len() + 2) * 8);
    for row in rows {
        out.extend_from_slice(&row.start_sector.to_le_bytes());
        out.extend_from_slice(&row.on_disk_sectors.to_le_bytes());
        out.extend_from_slice(&row.decompressed_sectors.to_le_bytes());
    }
    out.extend_from_slice(&[0xFF; 8]);
    out.extend_from_slice(&[0xFF; 8]);
    out
}

/// Serializes names: one 32-byte row per slot (the name, zero-padded to 30 bytes, then CR LF),
/// then an all-zero row. `names` has one entry per slot. Some sets carry one more row, with an
/// empty name, after the last slot; pass that row as an extra name to keep the set byte-identical.
pub fn name_bytes(names: &[&str], slots: usize) -> Result<Vec<u8>, WriteError> {
    if names.len() != slots && names.len() != slots + 1 {
        return Err(WriteError::NameCount {
            expected: slots,
            found: names.len(),
        });
    }
    let mut out = Vec::with_capacity((slots + 1) * NAME_ROW);
    for (index, name) in names.iter().enumerate() {
        let bytes = name.as_bytes();
        if bytes.len() > NAME_LEN
            || bytes.contains(&0)
            || bytes.contains(&b'\n')
            || bytes.contains(&b'\r')
        {
            return Err(WriteError::BadName { index });
        }
        out.extend_from_slice(bytes);
        out.resize(out.len() + (NAME_LEN - bytes.len()), 0);
        out.extend_from_slice(b"\r\n");
    }
    out.resize(out.len() + NAME_ROW, 0);
    Ok(out)
}
