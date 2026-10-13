//! HFA script archives of the HuneX engine.
//!
//! Layout: a 12-byte magic, a 32-bit entry count, then one 0x80-byte record per entry, then the
//! entry data packed back to back. A record holds a NUL-terminated name in a 0x60-byte field, a
//! 32-bit data offset and size (relative to the start of the data), and 24 reserved bytes.
//!
//! [`Archive::parse`] accepts only that packed layout: entries start at offset zero and each
//! begins where the previous one ends, the data ends at the end of the file, and the reserved
//! bytes are zero. [`Archive::write`] produces the same layout.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::borrow::Cow;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

pub const MAGIC: &[u8] = b"HUNEXGGEFA10";

const HEADER_SIZE: usize = 0x10;
const RECORD_SIZE: usize = 0x80;
const NAME_FIELD: usize = 0x60;
const RESERVED_AT: usize = 0x68;
const OFFSET_AT: usize = 0x60;
const SIZE_AT: usize = 0x64;

/// The longest name a writer accepts: the field must keep a NUL terminator.
pub const MAX_NAME_LEN: usize = NAME_FIELD - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    name: String,
    data: Cow<'a, [u8]>,
}

impl<'a> Entry<'a> {
    pub fn new(name: String, data: Cow<'a, [u8]>) -> Entry<'a> {
        Entry { name, data }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive<'a> {
    entries: Vec<Entry<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BadMagic,
    Truncated { offset: usize },
    CountPastEnd { count: u32 },
    NameNotTerminated { index: usize },
    NameTrailingData { index: usize },
    NameNotUtf8 { index: usize },
    NonZeroReserved { index: usize },
    NotContiguous { index: usize },
    DataPastEnd { index: usize },
    TrailingBytes { offset: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    /// The name is empty, longer than [`MAX_NAME_LEN`], or contains a NUL.
    BadName {
        index: usize,
    },
    DuplicateName {
        index: usize,
    },
    TooLarge,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not an HFA archive"),
            Error::Truncated { offset } => write!(f, "archive truncated at 0x{offset:x}"),
            Error::CountPastEnd { count } => {
                write!(f, "entry count {count} runs past the end of the file")
            }
            Error::NameNotTerminated { index } => write!(
                f,
                "entry {index}: name is not NUL-terminated within 0x60 bytes"
            ),
            Error::NameTrailingData { index } => {
                write!(f, "entry {index}: name field has data after its NUL")
            }
            Error::NameNotUtf8 { index } => write!(f, "entry {index}: name is not UTF-8"),
            Error::NonZeroReserved { index } => {
                write!(f, "entry {index}: reserved record bytes are not zero")
            }
            Error::NotContiguous { index } => write!(
                f,
                "entry {index}: data does not start where the previous entry ends"
            ),
            Error::DataPastEnd { index } => {
                write!(f, "entry {index}: data runs past the end of the file")
            }
            Error::TrailingBytes { offset } => {
                write!(f, "bytes after the last entry at 0x{offset:x}")
            }
        }
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            WriteError::BadName { index } => {
                write!(f, "entry {index}: name is empty, too long, or has a NUL")
            }
            WriteError::DuplicateName { index } => {
                write!(f, "entry {index}: name is used more than once")
            }
            WriteError::TooLarge => write!(f, "archive data does not fit the 32-bit offsets"),
        }
    }
}

impl core::error::Error for Error {}
impl core::error::Error for WriteError {}

impl<'a> Archive<'a> {
    pub fn new(entries: Vec<Entry<'a>>) -> Archive<'a> {
        Archive { entries }
    }

    pub fn entries(&self) -> &[Entry<'a>] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<Entry<'a>> {
        self.entries
    }

    /// Parses an archive. The entry data borrows from `bytes`.
    pub fn parse(bytes: &'a [u8]) -> Result<Archive<'a>, Error> {
        if !bytes.starts_with(MAGIC) {
            return Err(Error::BadMagic);
        }
        let count = u32_at(bytes, MAGIC.len())?;
        let data_start = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(RECORD_SIZE))
            .and_then(|records| records.checked_add(HEADER_SIZE))
            .filter(|&start| start <= bytes.len())
            .ok_or(Error::CountPastEnd { count })?;
        let data = &bytes[data_start..];

        let mut entries = Vec::with_capacity(count as usize);
        let mut expected = 0usize;
        for index in 0..count as usize {
            let record_at = HEADER_SIZE + index * RECORD_SIZE;
            let record = &bytes[record_at..record_at + RECORD_SIZE];
            let name = parse_name(index, &record[..NAME_FIELD])?;
            if record[RESERVED_AT..].iter().any(|&byte| byte != 0) {
                return Err(Error::NonZeroReserved { index });
            }
            let offset = u32_at(record, OFFSET_AT)? as usize;
            let size = u32_at(record, SIZE_AT)? as usize;
            if offset != expected {
                return Err(Error::NotContiguous { index });
            }
            let end = offset
                .checked_add(size)
                .filter(|&end| end <= data.len())
                .ok_or(Error::DataPastEnd { index })?;
            entries.push(Entry {
                name,
                data: Cow::Borrowed(&data[offset..end]),
            });
            expected = end;
        }
        if expected != data.len() {
            return Err(Error::TrailingBytes {
                offset: data_start + expected,
            });
        }
        Ok(Archive { entries })
    }

    /// Writes the archive in entry order, packed with no padding.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let mut names = BTreeSet::new();
        let mut total = 0usize;
        for (index, entry) in self.entries.iter().enumerate() {
            let name = entry.name.as_bytes();
            if name.is_empty() || name.len() > MAX_NAME_LEN || name.contains(&0) {
                return Err(WriteError::BadName { index });
            }
            if !names.insert(entry.name.as_str()) {
                return Err(WriteError::DuplicateName { index });
            }
            total = total
                .checked_add(entry.data.len())
                .ok_or(WriteError::TooLarge)?;
        }
        if u32::try_from(total).is_err() || u32::try_from(self.entries.len()).is_err() {
            return Err(WriteError::TooLarge);
        }

        let mut out = Vec::with_capacity(HEADER_SIZE + self.entries.len() * RECORD_SIZE + total);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        let mut offset = 0u32;
        for entry in &self.entries {
            let mut name_field = [0u8; NAME_FIELD];
            name_field[..entry.name.len()].copy_from_slice(entry.name.as_bytes());
            out.extend_from_slice(&name_field);
            out.extend_from_slice(&offset.to_le_bytes());
            let size = entry.data.len() as u32;
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&[0u8; RECORD_SIZE - RESERVED_AT]);
            offset += size;
        }
        for entry in &self.entries {
            out.extend_from_slice(&entry.data);
        }
        Ok(out)
    }
}

fn parse_name(index: usize, field: &[u8]) -> Result<String, Error> {
    let end = field
        .iter()
        .position(|&byte| byte == 0)
        .ok_or(Error::NameNotTerminated { index })?;
    if field[end..].iter().any(|&byte| byte != 0) {
        return Err(Error::NameTrailingData { index });
    }
    let name = core::str::from_utf8(&field[..end]).map_err(|_| Error::NameNotUtf8 { index })?;
    Ok(String::from(name))
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    match bytes.get(offset..offset + 4) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(Error::Truncated { offset }),
    }
}
