//! Nintendo OPUS audio (the Switch `stream` files) and their remux to Ogg Opus.
//!
//! Layout (little-endian): a 0x20-byte basic chunk (id 0x80000001, size 0x18, version 0,
//! channel count, frame size 0, sample rate, data chunk offset 0x20, a zero field, context
//! offset 0, pre-skip, padding), then a data chunk (id 0x80000004, size counted to the end of
//! the file), holding packets. Each packet is a big-endian u32 length, a 4-byte field this
//! crate ignores, and the Opus packet itself.
//!
//! ```
//! // The first packet's TOC byte gives its frame size: configuration 0 is 10 ms, or 480 samples at 48 kHz.
//! assert_eq!(hunex_nopus::toc_samples(&[0x00, 0x00]), Some(480));
//! assert!(hunex_nopus::Stream::parse(&[0u8; 4]).is_err());
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use hunex_ogg::{
    packets as ogg_packets, write_packet, write_page, Error as OggError, FLAG_BOS, FLAG_EOS,
};

/// Chunk tag of the Opus configuration chunk.
pub const BASIC_CHUNK: u32 = 0x8000_0001;
/// Chunk tag of the audio data chunk.
pub const DATA_CHUNK: u32 = 0x8000_0004;
/// Size of the file header in bytes.
pub const HEADER_SIZE: usize = 0x20;
/// Sample rate of the remuxed Ogg Opus stream. Opus always decodes at this rate.
pub const SAMPLE_RATE_OUT: u32 = 48_000;
/// Serial number of the remuxed Ogg stream.
pub const SERIAL: u32 = 0x4E50;
/// Vendor string written into the Ogg Opus header.
pub const VENDOR: &str = "hunex";

/// The comment that carries the 4-byte field of each packet in an Ogg Opus file made by
/// `to_ogg`. Ogg Opus has no place for those fields, so the comment holds them as hex.
pub const FIELDS_COMMENT: &str = "HUNEX_NINTENDO_FIELDS";

const PACKET_PREFIX: usize = 8;
const MAX_FRAMES: u32 = 48;
const MAX_PACKET_SAMPLES: u32 = 5_760;

/// Samples per frame at 48 kHz for each Opus configuration (RFC 6716, table 2).
const FRAME_SAMPLES: [u32; 32] = [
    480, 960, 1920, 2880, 480, 960, 1920, 2880, 480, 960, 1920, 2880, 480, 960, 480, 960, 120, 240,
    480, 960, 120, 240, 480, 960, 120, 240, 480, 960, 120, 240, 480, 960,
];

/// Errors from reading a Nintendo OPUS file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The file ends before the field or packet at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// A chunk has an unexpected tag.
    BadChunk {
        /// Byte offset of the chunk.
        offset: usize,
        /// The tag the chunk has.
        found: u32,
    },
    /// A layout field this crate does not understand; `field` names it.
    UnsupportedLayout {
        /// A header field has a value that cannot be written.
        field: &'static str,
    },
    /// The Opus version in the configuration chunk is not supported.
    UnsupportedVersion(u8),
    /// The channel count is not 1 or 2.
    UnsupportedChannels(u8),
    /// The file declares no channels.
    NoChannels,
    /// The data size in the header does not match the file.
    BadDataSize {
        /// The data size the header states.
        stored: u32,
        /// The data size in the file.
        actual: u32,
    },
    /// A packet runs past the end of the data.
    TruncatedPacket {
        /// Byte offset of the packet.
        offset: usize,
    },
    /// A packet has no bytes.
    EmptyPacket {
        /// Byte offset of the packet.
        offset: usize,
    },
    /// The first byte of an Opus packet (its TOC) is not valid.
    BadToc {
        /// Byte offset of the failing packet or chunk.
        offset: usize,
    },
    /// The file is not Ogg Opus: its first packet is not an OpusHead.
    NotOpus,
    /// The Ogg layer reported an error.
    Ogg(OggError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Truncated { offset } => write!(f, "file truncated at 0x{offset:x}"),
            Error::BadChunk { offset, found } => {
                write!(f, "chunk at 0x{offset:x} has id 0x{found:08x}")
            }
            Error::UnsupportedLayout { field } => write!(f, "unsupported layout: {field}"),
            Error::UnsupportedVersion(version) => write!(f, "unsupported version {version}"),
            Error::UnsupportedChannels(channels) => {
                write!(f, "{channels} channels; only mono and stereo are supported")
            }
            Error::NoChannels => write!(f, "channel count is zero"),
            Error::BadDataSize { stored, actual } => {
                write!(f, "data size field is {stored}, the data is {actual} bytes")
            }
            Error::TruncatedPacket { offset } => {
                write!(f, "packet at 0x{offset:x} runs past the end of the file")
            }
            Error::EmptyPacket { offset } => write!(f, "packet at 0x{offset:x} is empty"),
            Error::BadToc { offset } => write!(f, "packet at 0x{offset:x} has an invalid TOC byte"),
            Error::NotOpus => write!(f, "the file is not Ogg Opus"),
            Error::Ogg(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for Error {}

impl From<OggError> for Error {
    fn from(error: OggError) -> Self {
        Error::Ogg(error)
    }
}

/// One Opus packet and the number of 48 kHz samples it decodes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packet<'a> {
    /// The packet's bytes, borrowed from the input.
    pub data: &'a [u8],
    /// Number of 48 kHz samples the packet decodes to.
    pub samples: u32,
    /// The little-endian field stored before the packet's bytes. Its meaning is not known.
    pub field: u32,
}

/// A parsed Nintendo OPUS file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream<'a> {
    /// Number of channels, 1 or 2.
    pub channels: u8,
    /// Sample rate of the source stream in hertz.
    pub sample_rate: u32,
    /// Samples to drop from the start of the decoded output, from the configuration chunk.
    pub pre_skip: u16,
    /// The packets in order.
    pub packets: Vec<Packet<'a>>,
}

impl<'a> Stream<'a> {
    /// Reads a Nintendo OPUS file: its configuration chunk, then its audio data chunk and packets.
    pub fn parse(bytes: &'a [u8]) -> Result<Stream<'a>, Error> {
        let data_at = HEADER_SIZE;
        let prefix_end = data_at + PACKET_PREFIX;
        if bytes.len() < prefix_end {
            return Err(Error::Truncated {
                offset: bytes.len(),
            });
        }
        let basic = read_u32(bytes, 0);
        if basic != BASIC_CHUNK {
            return Err(Error::BadChunk {
                offset: 0,
                found: basic,
            });
        }
        if read_u32(bytes, 4) != 0x18 {
            return Err(Error::UnsupportedLayout {
                field: "basic chunk size",
            });
        }
        let version = bytes[8];
        if version != 0 {
            return Err(Error::UnsupportedVersion(version));
        }
        let channels = bytes[9];
        if channels == 0 {
            return Err(Error::NoChannels);
        }
        if channels > 2 {
            return Err(Error::UnsupportedChannels(channels));
        }
        if read_u16(bytes, 0x0A) != 0 {
            return Err(Error::UnsupportedLayout {
                field: "fixed frame size",
            });
        }
        let sample_rate = read_u32(bytes, 0x0C);
        if read_u32(bytes, 0x10) != HEADER_SIZE as u32 {
            return Err(Error::UnsupportedLayout {
                field: "data chunk offset",
            });
        }
        if read_u32(bytes, 0x14) != 0 {
            return Err(Error::UnsupportedLayout {
                field: "field 0x14",
            });
        }
        if read_u32(bytes, 0x18) != 0 {
            return Err(Error::UnsupportedLayout {
                field: "context chunk",
            });
        }
        let pre_skip = read_u16(bytes, 0x1C);

        let chunk = read_u32(bytes, data_at);
        if chunk != DATA_CHUNK {
            return Err(Error::BadChunk {
                offset: data_at,
                found: chunk,
            });
        }
        let stored = read_u32(bytes, data_at + 4);
        let actual = (bytes.len() - prefix_end) as u32;
        if stored != actual {
            return Err(Error::BadDataSize { stored, actual });
        }

        let mut packets = Vec::new();
        let mut position = prefix_end;
        while position < bytes.len() {
            let header = bytes
                .get(position..position + PACKET_PREFIX)
                .ok_or(Error::Truncated { offset: position })?;
            let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
            let start = position + PACKET_PREFIX;
            let end = start
                .checked_add(length)
                .filter(|&end| end <= bytes.len())
                .ok_or(Error::TruncatedPacket { offset: position })?;
            if length == 0 {
                return Err(Error::EmptyPacket { offset: position });
            }
            let data = &bytes[start..end];
            let samples = toc_samples(data).ok_or(Error::BadToc { offset: position })?;
            packets.push(Packet {
                data,
                samples,
                field: u32::from_le_bytes([header[4], header[5], header[6], header[7]]),
            });
            position = end;
        }

        Ok(Stream {
            channels,
            sample_rate,
            pre_skip,
            packets,
        })
    }

    /// Total samples at 48 kHz, including pre-skip.
    pub fn total_samples(&self) -> u64 {
        self.packets
            .iter()
            .map(|packet| u64::from(packet.samples))
            .sum()
    }

    /// Decoded samples at 48 kHz: the total minus pre-skip.
    pub fn decoded_samples(&self) -> u64 {
        self.total_samples()
            .saturating_sub(u64::from(self.pre_skip))
    }

    /// Remuxes to Ogg Opus. The packets are copied unchanged. The packet fields go into the
    /// tags packet as the `FIELDS_COMMENT` comment, so `from_ogg` can restore the file.
    pub fn to_ogg(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        let head = opus_head(self.channels, self.pre_skip, self.sample_rate);
        write_page(&mut out, FLAG_BOS, 0, SERIAL, 0, &[&head])?;
        let fields: Vec<u8> = self
            .packets
            .iter()
            .flat_map(|packet| packet.field.to_le_bytes())
            .collect();
        let tags = opus_tags(&[format!("{FIELDS_COMMENT}={}", hex(&fields))]);
        if self.packets.is_empty() {
            write_packet(&mut out, FLAG_EOS, 0, SERIAL, 1, &tags);
            return Ok(out);
        }
        let mut sequence = write_packet(&mut out, 0, 0, SERIAL, 1, &tags);
        let mut granule = 0u64;
        let last = self.packets.len() - 1;
        for (index, packet) in self.packets.iter().enumerate() {
            granule += u64::from(packet.samples);
            let flags = if index == last { FLAG_EOS } else { 0 };
            sequence = write_packet(&mut out, flags, granule, SERIAL, sequence, packet.data);
        }
        Ok(out)
    }

    /// The Nintendo OPUS file for these packets: the basic chunk, the data chunk, and each packet
    /// with its length and field.
    pub fn to_bytes(&self) -> Vec<u8> {
        let parts: Vec<(&[u8], u32)> = self
            .packets
            .iter()
            .map(|packet| (packet.data, packet.field))
            .collect();
        nintendo_bytes(self.channels, self.sample_rate, self.pre_skip, &parts)
    }
}

/// The outcome of converting an Ogg Opus file back to a Nintendo OPUS file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromOgg {
    /// The Nintendo OPUS file.
    pub file: Vec<u8>,
    /// True when the packet fields were read from the file's comment. Otherwise they are zero.
    pub fields_recorded: bool,
}

/// Converts an Ogg Opus file to a Nintendo OPUS file. The packets are copied unchanged. The
/// packet fields come from the `FIELDS_COMMENT` comment written by `Stream::to_ogg`; a file
/// without it gets zero fields, and `fields_recorded` is false.
pub fn from_ogg(bytes: &[u8]) -> Result<FromOgg, Error> {
    let packets = ogg_packets(bytes)?;
    let head = packets.first().ok_or(Error::NotOpus)?;
    if head.len() < 19 || !head.starts_with(b"OpusHead") || head[8] != 1 {
        return Err(Error::NotOpus);
    }
    let channels = head[9];
    if channels == 0 || channels > 2 {
        return Err(Error::UnsupportedChannels(channels));
    }
    let pre_skip = u16::from_le_bytes([head[10], head[11]]);
    let rate = u32::from_le_bytes([head[12], head[13], head[14], head[15]]);
    let audio = &packets[2.min(packets.len())..];
    let recorded = packets.get(1).and_then(|tags| fields_from_tags(tags));
    let fields_recorded = recorded
        .as_ref()
        .is_some_and(|fields| fields.len() == 4 * audio.len());
    let fields = match recorded {
        Some(fields) if fields_recorded => fields,
        _ => alloc::vec![0; 4 * audio.len()],
    };
    let parts: Vec<(&[u8], u32)> = audio
        .iter()
        .enumerate()
        .map(|(index, data)| {
            let field = u32::from_le_bytes([
                fields[4 * index],
                fields[4 * index + 1],
                fields[4 * index + 2],
                fields[4 * index + 3],
            ]);
            (data.as_slice(), field)
        })
        .collect();
    Ok(FromOgg {
        file: nintendo_bytes(channels, rate, pre_skip, &parts),
        fields_recorded,
    })
}

fn nintendo_bytes(
    channels: u8,
    sample_rate: u32,
    pre_skip: u16,
    packets: &[(&[u8], u32)],
) -> Vec<u8> {
    let data_len: usize = packets
        .iter()
        .map(|(data, _)| PACKET_PREFIX + data.len())
        .sum();
    let mut out = Vec::with_capacity(HEADER_SIZE + data_len);
    out.extend_from_slice(&BASIC_CHUNK.to_le_bytes());
    out.extend_from_slice(&0x18u32.to_le_bytes());
    out.push(0);
    out.push(channels);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&pre_skip.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&DATA_CHUNK.to_le_bytes());
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    for (data, field) in packets {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(&field.to_le_bytes());
        out.extend_from_slice(data);
    }
    out
}

/// The packet fields stored in an OpusTags packet, if it has the comment.
fn fields_from_tags(tags: &[u8]) -> Option<Vec<u8>> {
    if !tags.starts_with(b"OpusTags") {
        return None;
    }
    let mut at = 8;
    let vendor = read_u32_checked(tags, at)? as usize;
    at += 4 + vendor;
    let count = read_u32_checked(tags, at)?;
    at += 4;
    let prefix = format!("{FIELDS_COMMENT}=");
    for _ in 0..count {
        let length = read_u32_checked(tags, at)? as usize;
        at += 4;
        let comment = tags.get(at..at + length)?;
        at += length;
        if let Some(value) = comment.strip_prefix(prefix.as_bytes()) {
            return unhex(value);
        }
    }
    None
}

fn read_u32_checked(bytes: &[u8], offset: usize) -> Option<u32> {
    let value = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn hex(bytes: &[u8]) -> String {
    use core::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn unhex(text: &[u8]) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    text.chunks(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some((high * 16 + low) as u8)
        })
        .collect()
}

/// Samples at 48 kHz that an Opus packet decodes to, from its TOC byte (RFC 6716, section 3.1).
pub fn toc_samples(packet: &[u8]) -> Option<u32> {
    let toc = *packet.first()?;
    let frame = FRAME_SAMPLES[usize::from(toc >> 3)];
    let frames = match toc & 3 {
        0 => 1,
        1 | 2 => 2,
        _ => match packet.get(1)? & 0x3F {
            0 => return None,
            count => u32::from(count),
        },
    };
    if frames > MAX_FRAMES || frame * frames > MAX_PACKET_SAMPLES {
        return None;
    }
    Some(frame * frames)
}

fn opus_head(channels: u8, pre_skip: u16, input_rate: u32) -> Vec<u8> {
    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead");
    head.push(1);
    head.push(channels);
    head.extend_from_slice(&pre_skip.to_le_bytes());
    head.extend_from_slice(&input_rate.to_le_bytes());
    head.extend_from_slice(&0u16.to_le_bytes());
    head.push(0);
    head
}

fn opus_tags(comments: &[String]) -> Vec<u8> {
    let mut tags = Vec::new();
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(VENDOR.len() as u32).to_le_bytes());
    tags.extend_from_slice(VENDOR.as_bytes());
    tags.extend_from_slice(&(comments.len() as u32).to_le_bytes());
    for comment in comments {
        tags.extend_from_slice(&(comment.len() as u32).to_le_bytes());
        tags.extend_from_slice(comment.as_bytes());
    }
    tags
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
