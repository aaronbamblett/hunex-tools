//! Switch script text: pairs of an offset table and a text entry.
//!
//! The offset table is big-endian u32 values. The first N values are the start of each slot,
//! value N is the text length, and value N + 1 is 0xFFFFFFFF. Slot N - 1 is a zero-length
//! terminator, so there are N - 1 lines. Each line in the text ends with CR LF, which is not
//! part of the line when it is read.
//!
//! ```
//! // Two lines, "ab" and "cd": starts 0 and 4, the text length 8, then the 0xFFFFFFFF end marker.
//! let table = [0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 8, 0xFF, 0xFF, 0xFF, 0xFF];
//! let pair = hunex_switch_text::Pair::parse(&table, b"ab\r\ncd\r\n").unwrap();
//! assert_eq!(pair.lookup("$000001").unwrap(), Some("cd"));
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// The value that ends the offset table. Read as the first offset of a slot that has no text.
pub const SENTINEL: u32 = 0xFFFF_FFFF;

/// Errors from reading a script text file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The table is not a whole number of u32 values.
    /// The table is not a whole number of 4-byte offsets.
    BadTableLength {
        /// The length of the table in bytes.
        length: usize,
    },
    /// The table does not end with the sentinel, or is too short.
    BadTable,
    /// Slot starts go backwards, or the length value does not match the text.
    /// The offsets of a slot are not in order or do not fit the text.
    BadOffsets {
        /// The slot whose offsets are wrong.
        slot: usize,
    },
    /// A line is not UTF-8.
    /// A line is not valid UTF-8.
    NotUtf8 {
        /// The line number of the text.
        line: usize,
    },
    /// The language pair is missing.
    /// The pair index is past the end of the file.
    NoSuchPair {
        /// The pair index that was asked for.
        pair: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadTableLength { length } => {
                write!(f, "offset table of {length} bytes is not whole u32 values")
            }
            Error::BadTable => write!(f, "offset table has no sentinel"),
            Error::BadOffsets { slot } => write!(f, "offset table is inconsistent at slot {slot}"),
            Error::NotUtf8 { line } => write!(f, "line {line} is not UTF-8"),
            Error::NoSuchPair { pair } => write!(f, "no language pair {pair}"),
        }
    }
}

impl core::error::Error for Error {}

/// One language: the lines of an offset table and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair<'a> {
    text: &'a [u8],
    starts: Vec<usize>,
}

impl<'a> Pair<'a> {
    /// Parses a pair from its offset table and text entry.
    pub fn parse(table: &[u8], text: &'a [u8]) -> Result<Pair<'a>, Error> {
        if !table.len().is_multiple_of(4) {
            return Err(Error::BadTableLength {
                length: table.len(),
            });
        }
        let values: Vec<u32> = table
            .chunks_exact(4)
            .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        if values.len() < 3 || values.last() != Some(&SENTINEL) {
            return Err(Error::BadTable);
        }
        let slots = values.len() - 2;
        let length = values[slots] as usize;
        if length != text.len() {
            return Err(Error::BadOffsets { slot: slots });
        }
        let mut starts = Vec::with_capacity(slots);
        for (slot, &value) in values[..slots].iter().enumerate() {
            let start = value as usize;
            if start > length || starts.last().is_some_and(|&previous| previous > start) {
                return Err(Error::BadOffsets { slot });
            }
            starts.push(start);
        }
        Ok(Pair { text, starts })
    }

    /// The number of lines (slots minus the terminator).
    pub fn len(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    /// True when the table holds no pairs.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The raw bytes of line `index`, with its CR LF.
    pub fn raw_line(&self, index: usize) -> Option<&'a [u8]> {
        if index >= self.len() {
            return None;
        }
        Some(&self.text[self.starts[index]..self.starts[index + 1]])
    }

    /// The text of line `index` without its CR LF.
    pub fn line(&self, index: usize) -> Result<Option<&'a str>, Error> {
        let Some(raw) = self.raw_line(index) else {
            return Ok(None);
        };
        let trimmed = raw.strip_suffix(b"\r\n").unwrap_or(raw);
        core::str::from_utf8(trimmed)
            .map(Some)
            .map_err(|_| Error::NotUtf8 { line: index })
    }

    /// The text for a `$` reference such as `$000079`.
    pub fn lookup(&self, id: &str) -> Result<Option<&'a str>, Error> {
        match ref_index(id) {
            Some(index) => self.line(index),
            None => Ok(None),
        }
    }

    /// Every line, in order, with CR LF removed.
    pub fn lines(&self) -> Result<Vec<&'a str>, Error> {
        (0..self.len())
            .map(|index| self.line(index).map(|text| text.unwrap_or("")))
            .collect()
    }
}

/// The index named by `$` followed by six decimal digits.
pub fn ref_index(id: &str) -> Option<usize> {
    let digits = id.strip_prefix('$')?;
    if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// The `$` reference for a line index.
pub fn format_ref(index: usize) -> String {
    alloc::format!("${index:06}")
}

/// The pairs of a `script_text` archive, given its ten entries in order.
pub fn pairs<'a>(entries: &[&'a [u8]]) -> Result<Vec<Pair<'a>>, Error> {
    entries
        .chunks_exact(2)
        .map(|pair| Pair::parse(pair[0], pair[1]))
        .collect()
}
