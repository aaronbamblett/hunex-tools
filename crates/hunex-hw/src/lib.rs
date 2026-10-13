//! The `.hw` audio container of the HuneX engine.
//!
//! Layout (integers little-endian): a u32 header length (0x40), the tag `hw` and two spaces, a
//! u32 payload length (file size minus 0x40), the total sample frames (the granule of the last
//! Ogg page), the sample rate, the channel count, and 40 zero bytes. The payload is an Ogg
//! Vorbis stream, byte for byte.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::fmt;

use hunex_ogg::{first_packet, pages, Error as OggError, NO_GRANULE};

pub const HEADER_SIZE: usize = 0x40;
pub const TAG: &[u8; 4] = b"hw  ";

const VORBIS_ID_LENGTH: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Truncated {
        offset: usize,
    },
    BadHeaderLength {
        found: u32,
    },
    BadTag,
    BadPayloadLength {
        stored: u32,
        actual: u32,
    },
    NonZeroPadding {
        offset: usize,
    },
    /// The payload's first packet is not a Vorbis identification header.
    NotVorbis,
    BadIdentification,
    /// The last Ogg page has no granule, or one too large for the header.
    BadGranule,
    /// The header field does not match the Ogg data. `field` names it.
    HeaderMismatch {
        field: &'static str,
    },
    TooLarge,
    Ogg(OggError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Truncated { offset } => write!(f, "file truncated at 0x{offset:x}"),
            Error::BadHeaderLength { found } => {
                write!(f, "header length is 0x{found:x}, expected 0x40")
            }
            Error::BadTag => write!(f, "tag is not `hw  `"),
            Error::BadPayloadLength { stored, actual } => write!(
                f,
                "payload length field is {stored}, but the payload is {actual} bytes"
            ),
            Error::NonZeroPadding { offset } => {
                write!(f, "header padding at 0x{offset:x} is not zero")
            }
            Error::NotVorbis => write!(
                f,
                "the audio is not Ogg Vorbis (the first packet is not a Vorbis header)"
            ),
            Error::BadIdentification => write!(f, "the Vorbis identification header is malformed"),
            Error::BadGranule => write!(f, "the last Ogg page has no usable granule position"),
            Error::HeaderMismatch { field } => {
                write!(f, "header {field} does not match the Ogg data")
            }
            Error::TooLarge => write!(f, "the file is too large for the header fields"),
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

/// Facts read from an Ogg Vorbis stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VorbisInfo {
    pub channels: u8,
    pub sample_rate: u32,
    pub sample_frames: u32,
}

/// A parsed `.hw` file. The payload is borrowed from the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hw<'a> {
    pub sample_frames: u32,
    pub sample_rate: u32,
    pub channels: u32,
    payload: &'a [u8],
}

impl<'a> Hw<'a> {
    /// Parses a `.hw` file and checks its header against the Ogg payload.
    pub fn parse(bytes: &'a [u8]) -> Result<Hw<'a>, Error> {
        let header = bytes.get(..HEADER_SIZE).ok_or(Error::Truncated {
            offset: bytes.len(),
        })?;
        let header_length = read_u32(header, 0);
        if header_length as usize != HEADER_SIZE {
            return Err(Error::BadHeaderLength {
                found: header_length,
            });
        }
        if &header[4..8] != TAG {
            return Err(Error::BadTag);
        }
        let payload = &bytes[HEADER_SIZE..];
        let stored = read_u32(header, 8);
        let actual = u32::try_from(payload.len()).map_err(|_| Error::TooLarge)?;
        if stored != actual {
            return Err(Error::BadPayloadLength { stored, actual });
        }
        if let Some(offset) = header[0x18..].iter().position(|&byte| byte != 0) {
            return Err(Error::NonZeroPadding {
                offset: 0x18 + offset,
            });
        }
        let frames = read_u32(header, 0x0C);
        let rate = read_u32(header, 0x10);
        let channels = read_u32(header, 0x14);

        let info = vorbis_info(payload)?;
        if info.sample_frames != frames {
            return Err(Error::HeaderMismatch {
                field: "sample frames",
            });
        }
        if info.sample_rate != rate {
            return Err(Error::HeaderMismatch {
                field: "sample rate",
            });
        }
        if u32::from(info.channels) != channels {
            return Err(Error::HeaderMismatch { field: "channels" });
        }
        Ok(Hw {
            sample_frames: frames,
            sample_rate: rate,
            channels,
            payload,
        })
    }

    /// The Ogg Vorbis stream.
    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

/// Reads the Vorbis identification header and the length of an Ogg Vorbis stream.
pub fn vorbis_info(ogg: &[u8]) -> Result<VorbisInfo, Error> {
    let packet = first_packet(ogg)?.ok_or(Error::NotVorbis)?;
    if packet.len() < VORBIS_ID_LENGTH || packet[0] != 1 || &packet[1..7] != b"vorbis" {
        return Err(Error::NotVorbis);
    }
    let version = read_u32(&packet, 7);
    let channels = packet[11];
    let sample_rate = read_u32(&packet, 12);
    let framing = packet[29] & 1;
    if version != 0 || channels == 0 || sample_rate == 0 || framing != 1 {
        return Err(Error::BadIdentification);
    }
    let last = pages(ogg)?
        .last()
        .map(|page| page.granule)
        .ok_or(Error::Truncated { offset: 0 })?;
    if last == NO_GRANULE || last > u64::from(u32::MAX) {
        return Err(Error::BadGranule);
    }
    Ok(VorbisInfo {
        channels,
        sample_rate,
        sample_frames: last as u32,
    })
}

/// Wraps an Ogg Vorbis file in the `.hw` header.
pub fn wrap(ogg: &[u8]) -> Result<Vec<u8>, Error> {
    let info = vorbis_info(ogg)?;
    let length = u32::try_from(ogg.len()).map_err(|_| Error::TooLarge)?;
    let mut out = Vec::with_capacity(HEADER_SIZE + ogg.len());
    out.extend_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
    out.extend_from_slice(TAG);
    out.extend_from_slice(&length.to_le_bytes());
    out.extend_from_slice(&info.sample_frames.to_le_bytes());
    out.extend_from_slice(&info.sample_rate.to_le_bytes());
    out.extend_from_slice(&u32::from(info.channels).to_le_bytes());
    out.resize(HEADER_SIZE, 0);
    out.extend_from_slice(ogg);
    Ok(out)
}

/// The Ogg Vorbis stream inside a `.hw` file.
pub fn unwrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    Ok(Hw::parse(bytes)?.payload().to_vec())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
