//! Ogg pages: reading with CRC checks, writing with CRC computed, and first-packet access.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::fmt;

pub const CAPTURE: &[u8; 4] = b"OggS";
pub const FLAG_CONTINUED: u8 = 0x01;
pub const FLAG_BOS: u8 = 0x02;
pub const FLAG_EOS: u8 = 0x04;
/// Granule position of a page that completes no packet.
pub const NO_GRANULE: u64 = u64::MAX;

const PAGE_HEADER: usize = 27;
const CRC_OFFSET: usize = 22;
const MAX_SEGMENTS: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NotOgg {
        offset: usize,
    },
    BadVersion {
        offset: usize,
    },
    BadChecksum {
        offset: usize,
    },
    Truncated {
        offset: usize,
    },
    TrailingBytes {
        offset: usize,
    },
    /// The packets do not fit in one page (at most 255 lacing segments).
    TooManySegments,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::NotOgg { offset } => write!(f, "no Ogg page at 0x{offset:x}"),
            Error::BadVersion { offset } => {
                write!(f, "Ogg page at 0x{offset:x} has an unknown version")
            }
            Error::BadChecksum { offset } => write!(f, "Ogg page at 0x{offset:x} has a bad CRC"),
            Error::Truncated { offset } => write!(f, "Ogg data truncated at 0x{offset:x}"),
            Error::TrailingBytes { offset } => {
                write!(f, "bytes after the last Ogg page at 0x{offset:x}")
            }
            Error::TooManySegments => {
                write!(f, "packets need more than 255 lacing segments in one page")
            }
        }
    }
}

impl core::error::Error for Error {}

/// A page of a file, borrowing its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page<'a> {
    pub header_type: u8,
    pub granule: u64,
    pub serial: u32,
    pub sequence: u32,
    pub segments: &'a [u8],
    pub data: &'a [u8],
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut value = (index as u32) << 24;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 0x8000_0000 != 0 {
                (value << 1) ^ 0x04C1_1DB7
            } else {
                value << 1
            };
            bit += 1;
        }
        table[index] = value;
        index += 1;
    }
    table
}

const CRC_TABLE: [u32; 256] = crc_table();

fn crc_update(mut crc: u32, data: &[u8]) -> u32 {
    for &byte in data {
        crc = (crc << 8) ^ CRC_TABLE[(((crc >> 24) as u8) ^ byte) as usize];
    }
    crc
}

/// The Ogg CRC-32 of a byte string (polynomial 0x04C11DB7, no reflection, no final XOR).
pub fn crc32(data: &[u8]) -> u32 {
    crc_update(0, data)
}

/// Reads the page that starts at `at`. Returns it with the offset just past it.
pub fn read_page(bytes: &[u8], at: usize) -> Result<(Page<'_>, usize), Error> {
    let header = bytes
        .get(at..at + PAGE_HEADER)
        .ok_or(Error::Truncated { offset: at })?;
    if &header[..4] != CAPTURE {
        return Err(Error::NotOgg { offset: at });
    }
    if header[4] != 0 {
        return Err(Error::BadVersion { offset: at });
    }
    let segment_count = usize::from(header[26]);
    let table_end = at + PAGE_HEADER + segment_count;
    let segments = bytes
        .get(at + PAGE_HEADER..table_end)
        .ok_or(Error::Truncated { offset: at })?;
    let data_len: usize = segments.iter().map(|&length| usize::from(length)).sum();
    let end = table_end + data_len;
    let data = bytes
        .get(table_end..end)
        .ok_or(Error::Truncated { offset: at })?;

    let stored = u32::from_le_bytes([header[22], header[23], header[24], header[25]]);
    let computed = crc_update(
        crc_update(crc_update(0, &bytes[at..at + CRC_OFFSET]), &[0; 4]),
        &bytes[at + CRC_OFFSET + 4..end],
    );
    if stored != computed {
        return Err(Error::BadChecksum { offset: at });
    }

    let page = Page {
        header_type: header[5],
        granule: u64::from_le_bytes([
            header[6], header[7], header[8], header[9], header[10], header[11], header[12],
            header[13],
        ]),
        serial: u32::from_le_bytes([header[14], header[15], header[16], header[17]]),
        sequence: u32::from_le_bytes([header[18], header[19], header[20], header[21]]),
        segments,
        data,
    };
    Ok((page, end))
}

/// Reads every page of a file. The pages must fill the file exactly.
pub fn pages(bytes: &[u8]) -> Result<Vec<Page<'_>>, Error> {
    let mut pages = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let (page, next) = read_page(bytes, at)?;
        pages.push(page);
        at = next;
    }
    Ok(pages)
}

/// The first packet of a file, if it completes on the first page.
pub fn first_packet(bytes: &[u8]) -> Result<Option<Vec<u8>>, Error> {
    let (page, _) = read_page(bytes, 0)?;
    let mut packet = Vec::new();
    let mut offset = 0;
    for &length in page.segments {
        packet.extend_from_slice(&page.data[offset..offset + usize::from(length)]);
        offset += usize::from(length);
        if length < 255 {
            return Ok(Some(packet));
        }
    }
    Ok(None)
}

/// Appends one page holding the given packets, which must be whole packets.
pub fn write_page(
    out: &mut Vec<u8>,
    header_type: u8,
    granule: u64,
    serial: u32,
    sequence: u32,
    packets: &[&[u8]],
) -> Result<(), Error> {
    let mut segments = Vec::new();
    for packet in packets {
        let mut rest = packet.len();
        while rest >= MAX_SEGMENTS {
            segments.push(MAX_SEGMENTS as u8);
            rest -= MAX_SEGMENTS;
        }
        segments.push(rest as u8);
    }
    if segments.len() > MAX_SEGMENTS {
        return Err(Error::TooManySegments);
    }
    let start = out.len();
    out.extend_from_slice(CAPTURE);
    out.push(0);
    out.push(header_type);
    out.extend_from_slice(&granule.to_le_bytes());
    out.extend_from_slice(&serial.to_le_bytes());
    out.extend_from_slice(&sequence.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.push(segments.len() as u8);
    out.extend_from_slice(&segments);
    for packet in packets {
        out.extend_from_slice(packet);
    }
    let crc = crc_update(0, &out[start..]);
    out[start + CRC_OFFSET..start + CRC_OFFSET + 4].copy_from_slice(&crc.to_le_bytes());
    Ok(())
}
