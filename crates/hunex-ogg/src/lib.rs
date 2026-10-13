//! Ogg pages: reading with CRC checks, writing with CRC computed, and first-packet access.
//!
//! ```
//! assert_eq!(hunex_ogg::crc32(b""), 0);
//! assert_eq!(hunex_ogg::crc32(b"123456789"), 0x89A1_897F);
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::fmt;

/// The capture pattern that starts every Ogg page: the ASCII text `OggS`.
pub const CAPTURE: &[u8; 4] = b"OggS";
/// Header flag: the page continues a packet from the previous page.
pub const FLAG_CONTINUED: u8 = 0x01;
/// Header flag: the first page of a logical stream.
pub const FLAG_BOS: u8 = 0x02;
/// Header flag: the last page of a logical stream.
pub const FLAG_EOS: u8 = 0x04;
/// Granule position of a page that completes no packet.
pub const NO_GRANULE: u64 = u64::MAX;

const PAGE_HEADER: usize = 27;
const CRC_OFFSET: usize = 22;
const MAX_SEGMENTS: usize = 255;

/// Errors from reading an Ogg file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The page at `offset` does not start with `OggS`.
    NotOgg {
        /// Byte offset of the page.
        offset: usize,
    },
    /// The stream structure version is not 0.
    BadVersion {
        /// Byte offset of the page.
        offset: usize,
    },
    /// The page checksum does not match.
    BadChecksum {
        /// Byte offset of the page.
        offset: usize,
    },
    /// The data ends inside the page at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// Bytes remain after the last page.
    TrailingBytes {
        /// Byte offset of the first unused byte.
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
    /// Header type flags: `FLAG_CONTINUED`, `FLAG_BOS` and `FLAG_EOS`.
    pub header_type: u8,
    /// The granule position: for audio, the number of samples up to the end of the page.
    pub granule: u64,
    /// Serial number of the logical stream the page belongs to.
    pub serial: u32,
    /// Page sequence number within the logical stream.
    pub sequence: u32,
    /// The segment table: the length of each packet segment on the page.
    pub segments: &'a [u8],
    /// The page's data, the segments back to back.
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
    let data: Vec<u8> = packets.concat();
    write_raw_page(
        out,
        header_type,
        granule,
        serial,
        sequence,
        &segments,
        &data,
    );
    Ok(())
}

/// Writes one page from its segment table and data.
fn write_raw_page(
    out: &mut Vec<u8>,
    header_type: u8,
    granule: u64,
    serial: u32,
    sequence: u32,
    segments: &[u8],
    data: &[u8],
) {
    let start = out.len();
    out.extend_from_slice(CAPTURE);
    out.push(0);
    out.push(header_type);
    out.extend_from_slice(&granule.to_le_bytes());
    out.extend_from_slice(&serial.to_le_bytes());
    out.extend_from_slice(&sequence.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.push(segments.len() as u8);
    out.extend_from_slice(segments);
    out.extend_from_slice(data);
    let crc = crc_update(0, &out[start..]);
    out[start + CRC_OFFSET..start + CRC_OFFSET + 4].copy_from_slice(&crc.to_le_bytes());
}

/// Writes one packet, split over as many pages as its lacing needs. Pages after the first carry
/// `FLAG_CONTINUED`; every page but the last has the granule position -1. Returns the sequence
/// number the next page should use.
pub fn write_packet(
    out: &mut Vec<u8>,
    flags: u8,
    granule: u64,
    serial: u32,
    first_sequence: u32,
    packet: &[u8],
) -> u32 {
    let mut segments = Vec::new();
    let mut rest = packet.len();
    while rest >= MAX_SEGMENTS {
        segments.push(MAX_SEGMENTS as u8);
        rest -= MAX_SEGMENTS;
    }
    segments.push(rest as u8);
    let mut sequence = first_sequence;
    let mut offset = 0usize;
    let groups: Vec<&[u8]> = segments.chunks(MAX_SEGMENTS).collect();
    for (index, group) in groups.iter().enumerate() {
        let bytes: usize = group.iter().map(|&len| usize::from(len)).sum();
        let last = index + 1 == groups.len();
        let page_flags = match (index, last) {
            (0, true) => flags,
            (0, false) => flags & !FLAG_EOS,
            (_, true) => FLAG_CONTINUED | (flags & FLAG_EOS),
            (_, false) => FLAG_CONTINUED,
        };
        let page_granule = if last { granule } else { u64::MAX };
        write_raw_page(
            out,
            page_flags,
            page_granule,
            serial,
            sequence,
            group,
            &packet[offset..offset + bytes],
        );
        offset += bytes;
        sequence += 1;
    }
    sequence
}

/// Reads every packet of the file's logical stream, joining packets that span pages.
pub fn packets(bytes: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
    let mut packets = Vec::new();
    let mut current = Vec::new();
    for page in pages(bytes)? {
        let mut data = page.data;
        for &len in page.segments {
            let (chunk, rest) = data.split_at(usize::from(len));
            current.extend_from_slice(chunk);
            data = rest;
            if len < 255 {
                packets.push(core::mem::take(&mut current));
            }
        }
    }
    Ok(packets)
}
