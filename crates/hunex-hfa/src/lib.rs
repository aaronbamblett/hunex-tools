//! HFA script archives of the HuneX engine.
//!
//! Layout: a 12-byte magic, a 32-bit entry count, then one 0x80-byte record per entry, then the
//! entry data packed back to back. A record holds a NUL-terminated name in a 0x60-byte field, a
//! 32-bit data offset and size (relative to the start of the data), and 24 reserved bytes.
//!
//! [`Archive::parse`] accepts only that packed layout: entries start at offset zero and each
//! begins where the previous one ends, the data ends at the end of the file, and the reserved
//! bytes are zero. [`Archive::write`] produces the same layout.
//!
//! ```
//! use std::borrow::Cow;
//! use hunex_hfa::{Archive, Entry};
//!
//! let archive = Archive::new(vec![Entry::new("a.bin".to_string(), Cow::Borrowed(b"xyz"))]);
//! let bytes = archive.write().unwrap();
//! let parsed = Archive::parse(&bytes).unwrap();
//! assert_eq!(parsed.entries()[0].name(), "a.bin");
//! assert_eq!(parsed.entries()[0].data(), b"xyz");
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// The magic that starts every archive: the ASCII text `HUNEXGGEFA10`.
pub const MAGIC: &[u8] = b"HUNEXGGEFA10";

const HEADER_SIZE: usize = 0x10;
const RECORD_SIZE: usize = 0x80;
const NAME_FIELD: usize = 0x60;
const RESERVED_AT: usize = 0x68;
const OFFSET_AT: usize = 0x60;
const SIZE_AT: usize = 0x64;

/// The longest name a writer accepts: the field must keep a NUL terminator.
pub const MAX_NAME_LEN: usize = NAME_FIELD - 1;

/// One entry of an archive: a name and its data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    name: String,
    data: Cow<'a, [u8]>,
}

impl<'a> Entry<'a> {
    /// Creates an entry from a name and its data. Borrowed data stays borrowed.
    pub fn new(name: String, data: Cow<'a, [u8]>) -> Entry<'a> {
        Entry { name, data }
    }

    /// The name, as stored in the archive.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The data of the entry.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The size of the data in bytes.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// An archive: its entries in the order they are stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive<'a> {
    entries: Vec<Entry<'a>>,
}

/// Errors from reading an HFA archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The file does not start with the magic.
    BadMagic,
    /// The file ends before the field at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// The entry count is larger than the file can hold.
    CountPastEnd {
        /// The entry count the header states.
        count: u32,
    },
    /// The name of entry `index` has no terminating zero byte.
    NameNotTerminated {
        /// Index of the entry.
        index: usize,
    },
    /// The name of entry `index` has bytes after its terminator.
    NameTrailingData {
        /// Index of the entry.
        index: usize,
    },
    /// The name of entry `index` is not valid UTF-8.
    NameNotUtf8 {
        /// Index of the entry.
        index: usize,
    },
    /// A reserved field of entry `index` is not zero.
    NonZeroReserved {
        /// Index of the entry.
        index: usize,
    },
    /// Entry `index` does not start where the previous entry ends.
    NotContiguous {
        /// Index of the entry.
        index: usize,
    },
    /// The data of entry `index` runs past the end of the file.
    DataPastEnd {
        /// Index of the entry.
        index: usize,
    },
    /// Bytes remain after the last entry.
    TrailingBytes {
        /// Byte offset of the first unused byte.
        offset: usize,
    },
}

/// Errors from writing an HFA archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    /// The name is empty, longer than [`MAX_NAME_LEN`], or contains a NUL.
    BadName {
        /// Index of the entry that cannot be written.
        index: usize,
    },
    /// The data or the entry count does not fit the 32-bit fields.
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
            WriteError::TooLarge => write!(f, "archive data does not fit the 32-bit offsets"),
        }
    }
}

impl core::error::Error for Error {}
impl core::error::Error for WriteError {}

impl<'a> Archive<'a> {
    /// Creates an archive from its entries, in the order given.
    pub fn new(entries: Vec<Entry<'a>>) -> Archive<'a> {
        Archive { entries }
    }

    /// The entries in order.
    pub fn entries(&self) -> &[Entry<'a>] {
        &self.entries
    }

    /// Takes the entries out of the archive.
    pub fn into_entries(self) -> Vec<Entry<'a>> {
        self.entries
    }

    /// Parses an archive. The entry data borrows from `bytes`.
    pub fn parse(bytes: &'a [u8]) -> Result<Archive<'a>, Error> {
        let table_len = table_len(bytes)?;
        let table = bytes.get(..table_len).ok_or(Error::CountPastEnd {
            count: u32_at(bytes, MAGIC.len())?,
        })?;
        let toc = read_toc(table, bytes.len() - table_len)?;
        let data = &bytes[table_len..];
        let entries = toc
            .into_iter()
            .map(|entry| Entry {
                data: Cow::Borrowed(&data[entry.offset..entry.offset + entry.size]),
                name: entry.name,
            })
            .collect();
        Ok(Archive { entries })
    }

    /// Writes the archive in entry order, packed with no padding.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let mut total = 0usize;
        for (index, entry) in self.entries.iter().enumerate() {
            let name = entry.name.as_bytes();
            if name.is_empty() || name.len() > MAX_NAME_LEN || name.contains(&0) {
                return Err(WriteError::BadName { index });
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

/// One table entry: its name, and its data's offset and size from the start of the data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    /// Name of the entry, as stored.
    pub name: String,
    /// Offset of the entry's data from the start of the data area.
    pub offset: usize,
    /// Size of the entry's data in bytes.
    pub size: usize,
}

/// The length of the header and table of an archive, from its first 16 bytes. The data follows.
pub fn table_len(head: &[u8]) -> Result<usize, Error> {
    if !head.starts_with(MAGIC) {
        return Err(Error::BadMagic);
    }
    let count = u32_at(head, MAGIC.len())?;
    usize::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(RECORD_SIZE))
        .and_then(|records| records.checked_add(HEADER_SIZE))
        .ok_or(Error::CountPastEnd { count })
}

/// Reads the table of an archive. `table` is the header and table (`table_len` bytes) and
/// `data_len` is the number of bytes of data after it. Entries must be packed back to back, and
/// the data must end where the last entry ends.
pub fn read_toc(table: &[u8], data_len: usize) -> Result<Vec<TocEntry>, Error> {
    let len = table_len(table)?;
    if table.len() != len {
        return Err(Error::CountPastEnd {
            count: u32_at(table, MAGIC.len())?,
        });
    }
    let count = u32_at(table, MAGIC.len())? as usize;
    let mut entries = Vec::with_capacity(count);
    let mut expected = 0usize;
    for index in 0..count {
        let record_at = HEADER_SIZE + index * RECORD_SIZE;
        let record = &table[record_at..record_at + RECORD_SIZE];
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
            .filter(|&end| end <= data_len)
            .ok_or(Error::DataPastEnd { index })?;
        entries.push(TocEntry { name, offset, size });
        expected = end;
    }
    if expected != data_len {
        return Err(Error::TrailingBytes {
            offset: len + expected,
        });
    }
    Ok(entries)
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    match bytes.get(offset..offset + 4) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(Error::Truncated { offset }),
    }
}
