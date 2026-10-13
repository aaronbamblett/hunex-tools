//! Script text (`.ctd`) bodies of the HuneX engine.
//!
//! A decompressed body is a list of lines, each ended by CRLF. The line at index n is the text
//! for script id `$` followed by six decimal digits, with n written in those digits. A body
//! has no header, no index table and no padding. Lines hold UTF-8 text with no bare CR or LF.
//!
//! [`Text::from_bytes`] accepts a stored LenZu entry or a decompressed body, detected by the
//! LenZu magic. [`Text::to_stored`] writes a literal-only LenZu stream.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// The most lines an id can name: six digits.
pub const MAX_LINES: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidUtf8 {
        offset: usize,
    },
    BareLf {
        offset: usize,
    },
    BareCr {
        offset: usize,
    },
    /// The body does not end with CRLF; the bytes from `offset` form an unterminated line.
    Unterminated {
        offset: usize,
    },
    /// A line cannot hold CR or LF, because the body uses them as separators.
    LineBreakInText,
    IndexOutOfRange {
        index: usize,
    },
    LenZu(hunex_lenzu::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::InvalidUtf8 { offset } => write!(f, "body is not UTF-8 at 0x{offset:x}"),
            Error::BareLf { offset } => write!(f, "bare LF at 0x{offset:x}"),
            Error::BareCr { offset } => write!(f, "bare CR at 0x{offset:x}"),
            Error::Unterminated { offset } => write!(f, "unterminated line at 0x{offset:x}"),
            Error::LineBreakInText => write!(f, "a line cannot contain CR or LF"),
            Error::IndexOutOfRange { index } => write!(f, "line {index} is out of range"),
            Error::LenZu(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for Error {}

impl From<hunex_lenzu::Error> for Error {
    fn from(error: hunex_lenzu::Error) -> Self {
        Error::LenZu(error)
    }
}

/// The lines of a script text body, without their CRLF.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Text {
    lines: Vec<String>,
}

impl Text {
    pub fn new() -> Text {
        Text::default()
    }

    /// Parses a decompressed body.
    pub fn parse(body: &[u8]) -> Result<Text, Error> {
        let text = core::str::from_utf8(body).map_err(|error| Error::InvalidUtf8 {
            offset: error.valid_up_to(),
        })?;
        let mut lines = Vec::new();
        let mut start = 0;
        let mut index = 0;
        while index < body.len() {
            match body[index] {
                b'\r' => {
                    if body.get(index + 1) != Some(&b'\n') {
                        return Err(Error::BareCr { offset: index });
                    }
                    lines.push(String::from(&text[start..index]));
                    index += 2;
                    start = index;
                }
                b'\n' => return Err(Error::BareLf { offset: index }),
                _ => index += 1,
            }
        }
        if start != body.len() {
            return Err(Error::Unterminated { offset: start });
        }
        Ok(Text { lines })
    }

    /// Reads a stored LenZu entry or a decompressed body.
    pub fn from_bytes(bytes: &[u8]) -> Result<Text, Error> {
        if hunex_lenzu::is_lenzu(bytes) {
            Text::parse(&hunex_lenzu::decompress(bytes)?)
        } else {
            Text::parse(bytes)
        }
    }

    /// The decompressed body: every line followed by CRLF.
    pub fn write(&self) -> Vec<u8> {
        let size: usize = self.lines.iter().map(|line| line.len() + 2).sum();
        let mut body = Vec::with_capacity(size);
        for line in &self.lines {
            body.extend_from_slice(line.as_bytes());
            body.extend_from_slice(b"\r\n");
        }
        body
    }

    /// A stored entry: the body in a literal-only LenZu stream.
    pub fn to_stored(&self) -> Vec<u8> {
        hunex_lenzu::compress_literal(&self.write())
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(String::as_str)
    }

    /// The text for an id such as `$000079`.
    pub fn lookup(&self, id: &str) -> Option<&str> {
        self.line(parse_id(id)?)
    }

    pub fn set_line(&mut self, index: usize, text: &str) -> Result<(), Error> {
        check_line(text)?;
        let slot = self
            .lines
            .get_mut(index)
            .ok_or(Error::IndexOutOfRange { index })?;
        *slot = String::from(text);
        Ok(())
    }

    /// Appends a line and returns its index, which is also its id.
    pub fn push_line(&mut self, text: &str) -> Result<usize, Error> {
        check_line(text)?;
        self.lines.push(String::from(text));
        Ok(self.lines.len() - 1)
    }
}

fn check_line(text: &str) -> Result<(), Error> {
    if text.contains(['\r', '\n']) {
        Err(Error::LineBreakInText)
    } else {
        Ok(())
    }
}

/// The id for a line index, such as `$000079`. Returns `None` past the six-digit range.
pub fn format_id(index: usize) -> Option<String> {
    (index < MAX_LINES).then(|| alloc::format!("${index:06}"))
}

/// The index named by an id: `$` followed by exactly six decimal digits.
pub fn parse_id(id: &str) -> Option<usize> {
    let digits = id.strip_prefix('$')?;
    if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}
