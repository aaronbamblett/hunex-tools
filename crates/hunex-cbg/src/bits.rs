//! Unsigned LEB128 integers, and the least-significant-bit-first bit order of the Huffman
//! bitstream.

use alloc::vec::Vec;

use crate::Error;

/// Appends `value` as unsigned LEB128.
pub(crate) fn write_leb(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// Reads an unsigned LEB128 integer at `position`. Returns the value and the next position.
pub(crate) fn read_leb(bytes: &[u8], position: usize) -> Result<(u64, usize), Error> {
    let mut value = 0u64;
    let mut shift = 0u32;
    let mut at = position;
    loop {
        let byte = *bytes.get(at).ok_or(Error::Truncated { offset: at })?;
        at += 1;
        if shift >= 64 || (shift == 63 && byte > 1) {
            return Err(Error::BadStream {
                what: "variable-length integer",
            });
        }
        value |= u64::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, at));
        }
        shift += 7;
    }
}

/// Collects bits, least significant bit of each byte first.
#[derive(Default)]
pub(crate) struct BitWriter {
    out: Vec<u8>,
    count: u32,
}

impl BitWriter {
    pub(crate) fn push(&mut self, bit: u8) {
        if self.count.is_multiple_of(8) {
            self.out.push(0);
        }
        if bit & 1 == 1 {
            let last = self.out.len() - 1;
            self.out[last] |= 1 << (self.count % 8);
        }
        self.count += 1;
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.out
    }
}

/// Reads bits, least significant bit of each byte first.
pub(crate) struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        BitReader { bytes, position: 0 }
    }

    pub(crate) fn bit(&mut self) -> Result<u8, Error> {
        let byte = *self.bytes.get(self.position / 8).ok_or(Error::Truncated {
            offset: self.position / 8,
        })?;
        let bit = (byte >> (self.position % 8)) & 1;
        self.position += 1;
        Ok(bit)
    }

    /// Bits read so far.
    pub(crate) fn used(&self) -> usize {
        self.position
    }

    /// True when every bit after the last one read is zero.
    pub(crate) fn rest_is_zero(&self) -> bool {
        (self.position..self.bytes.len() * 8).all(|bit| (self.bytes[bit / 8] >> (bit % 8)) & 1 == 0)
    }
}
