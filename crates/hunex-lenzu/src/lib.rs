//! LenZu compressed streams, as stored in HuneX engine archives.
//!
//! [`decompress`] reads the variant seen in the game's archives: seven-bit Huffman symbols and
//! back-reference distances with seven low bits, base two. It checks the header checksum.
//!
//! [`compress_literal`] writes a stream of literal runs only. The format allows it and the
//! decoder reads it back, but the stream is larger than its input. It is not yet known whether
//! the game accepts such streams.

#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

pub const MAGIC: &[u8] = b"LenZuCompressor\0";

const SIZE_AT: usize = 0x20;
const CHECKSUM_HIGH_AT: usize = 0x24;
const CHECKSUM_LOW_AT: usize = 0x28;
const WIDTH_RAW_AT: usize = 0x31;
const WIDTH_MIN_AT: usize = 0x32;
const LOW_COUNT_AT: usize = 0x34;
const BASE_AT: usize = 0x35;
const TABLE_AT: usize = 0x36;
const SYMBOLS: usize = 128;
const SYMBOL_BITS: u32 = 7;
const LITERAL_MAX: usize = 128;
const CHECKSUM_KEYS: [u64; 4] = [0xE9, 0x115, 0x137, 0x1B1];
const OPTIONS: [u8; 6] = [0x06, 0x07, 0x07, 0x0E, 0x07, 0x02];
const CONSTANT_A: u32 = 49;
const CONSTANT_B: u32 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BadMagic,
    Truncated {
        offset: usize,
    },
    /// The stream uses parameters other than the seven-bit variant.
    Unsupported,
    BadTable {
        offset: usize,
    },
    BadDistance {
        offset: usize,
    },
    ChecksumMismatch {
        stored: u64,
        computed: u64,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not a LenZu stream"),
            Error::Truncated { offset } => write!(f, "LenZu stream truncated at 0x{offset:x}"),
            Error::Unsupported => write!(f, "LenZu stream uses an unsupported variant"),
            Error::BadTable { offset } => write!(f, "bad Huffman table at 0x{offset:x}"),
            Error::BadDistance { offset } => {
                write!(f, "back-reference before the start at output 0x{offset:x}")
            }
            Error::ChecksumMismatch { stored, computed } => write!(
                f,
                "checksum mismatch: stored 0x{stored:016x}, computed 0x{computed:016x}"
            ),
        }
    }
}

impl core::error::Error for Error {}

/// Returns true if the bytes start with the LenZu magic.
pub fn is_lenzu(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

/// Decodes a LenZu stream and checks its header checksum.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Error> {
    if !is_lenzu(input) {
        return Err(Error::BadMagic);
    }
    let size = u32_at(input, SIZE_AT)? as usize;
    let stored = (u64::from(u32_at(input, CHECKSUM_HIGH_AT)?) << 32)
        | u64::from(u32_at(input, CHECKSUM_LOW_AT)?);
    let width = byte_at(input, WIDTH_RAW_AT)?.max(byte_at(input, WIDTH_MIN_AT)?);
    let low_count = byte_at(input, LOW_COUNT_AT)? as u32;
    let base = byte_at(input, BASE_AT)? as usize;
    if u32::from(width) != SYMBOL_BITS || low_count > 16 {
        return Err(Error::Unsupported);
    }

    let (weights, body) = read_table(input)?;
    let tree = build_tree(&weights)?;
    let mut bits = Bits {
        data: input,
        position: body * 8,
    };

    let mut out = Vec::new();
    while out.len() < size {
        if bits.bit()? == 0 {
            let count = read_symbol(&tree, &mut bits)? as usize + 1;
            for _ in 0..count {
                out.push(bits.bits(8)? as u8);
                if out.len() == size {
                    break;
                }
            }
        } else {
            let length = read_symbol(&tree, &mut bits)? as usize + base;
            let high = u32::from(read_symbol(&tree, &mut bits)?);
            let low = if low_count == 0 {
                0
            } else {
                bits.bits(low_count)?
            };
            let distance = ((low | (high << low_count)) as usize) + base;
            if distance == 0 || distance > out.len() {
                return Err(Error::BadDistance { offset: out.len() });
            }
            for _ in 0..length {
                if out.len() == size {
                    break;
                }
                let byte = out[out.len() - distance];
                out.push(byte);
            }
        }
    }

    let computed = checksum(&out);
    if computed != stored {
        return Err(Error::ChecksumMismatch { stored, computed });
    }
    Ok(out)
}

/// Writes a stream that holds the input only as literal runs. The result is larger than the input.
///
/// Inputs must be smaller than 4 GiB, because the size field is 32 bits.
pub fn compress_literal(plain: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    write_fields(&mut out, plain);
    out.push(SYMBOLS as u8);
    for _ in 0..SYMBOLS {
        out.extend_from_slice(&1u32.to_le_bytes());
    }
    let mut writer = BitWriter::default();
    for chunk in plain.chunks(LITERAL_MAX) {
        writer.push_bit(0);
        writer.push_bits((SYMBOLS - chunk.len()) as u32, SYMBOL_BITS);
        for &byte in chunk {
            writer.push_bits(u32::from(byte), 8);
        }
    }
    out.extend_from_slice(&writer.finish());
    out
}

/// Writes the fixed header fields and options, up to the table count byte.
fn write_fields(out: &mut Vec<u8>, plain: &[u8]) {
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&CONSTANT_A.to_le_bytes());
    out.extend_from_slice(&CONSTANT_B.to_le_bytes());
    out.extend_from_slice(&[0; 8]);
    out.extend_from_slice(&(plain.len() as u32).to_le_bytes());
    let sum = checksum(plain);
    out.extend_from_slice(&((sum >> 32) as u32).to_le_bytes());
    out.extend_from_slice(&(sum as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&OPTIONS);
}

fn checksum(plain: &[u8]) -> u64 {
    let mut accumulator: u64 = 0;
    for (index, &byte) in plain.iter().enumerate() {
        accumulator = accumulator
            .wrapping_add(u64::from(byte))
            .wrapping_mul(CHECKSUM_KEYS[index % 4]);
    }
    accumulator
}

fn byte_at(input: &[u8], offset: usize) -> Result<u8, Error> {
    input
        .get(offset)
        .copied()
        .ok_or(Error::Truncated { offset })
}

fn u32_at(input: &[u8], offset: usize) -> Result<u32, Error> {
    match input.get(offset..offset + 4) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(Error::Truncated { offset }),
    }
}

/// Reads the Huffman weights. Returns them with the offset where the bitstream starts.
fn read_table(input: &[u8]) -> Result<([u64; SYMBOLS], usize), Error> {
    let declared = match byte_at(input, TABLE_AT)? as usize {
        0 => SYMBOLS,
        count => count,
    };
    let mut weights = [0u64; SYMBOLS];
    // Dense when 4 * 128 < (width + 4) * count, with a one-byte symbol index.
    if 4 * SYMBOLS < 5 * declared {
        for (symbol, weight) in weights.iter_mut().enumerate() {
            *weight = u64::from(u32_at(input, TABLE_AT + 1 + 4 * symbol)?);
        }
        return Ok((weights, TABLE_AT + 1 + 4 * SYMBOLS));
    }
    let mut seen = [false; SYMBOLS];
    let mut position = TABLE_AT + 1;
    for _ in 0..declared {
        let symbol = byte_at(input, position)? as usize;
        if symbol >= SYMBOLS || seen[symbol] {
            return Err(Error::BadTable { offset: position });
        }
        seen[symbol] = true;
        weights[symbol] = u64::from(u32_at(input, position + 1)?);
        position += 5;
    }
    Ok((weights, position))
}

#[derive(Clone, Copy)]
enum Node {
    Leaf(u8),
    Branch { zero: usize, one: usize },
}

struct Tree {
    nodes: Vec<Node>,
    root: usize,
}

/// Builds the code tree: repeatedly join the two lightest parentless nodes (ties go to the
/// earlier node) until the joined weight covers every leaf weight.
fn build_tree(weights: &[u64; SYMBOLS]) -> Result<Tree, Error> {
    let mut nodes: Vec<Node> = (0..SYMBOLS)
        .map(|symbol| Node::Leaf(symbol as u8))
        .collect();
    let mut weight: Vec<u64> = weights.to_vec();
    let mut parented = vec![false; SYMBOLS];
    let total: u64 = weights.iter().sum();
    let mut root = None;
    loop {
        let mut candidates: Vec<usize> = (0..nodes.len())
            .filter(|&index| !parented[index] && weight[index] > 0)
            .collect();
        if candidates.is_empty() {
            return Err(Error::BadTable { offset: TABLE_AT });
        }
        if candidates.len() < 2 {
            break;
        }
        candidates.sort_by_key(|&index| (weight[index], index));
        let (light, heavy) = (candidates[0], candidates[1]);
        let sum = weight[light] + weight[heavy];
        nodes.push(Node::Branch {
            zero: heavy,
            one: light,
        });
        weight.push(sum);
        parented.push(false);
        parented[light] = true;
        parented[heavy] = true;
        root = Some(nodes.len() - 1);
        if sum >= total {
            break;
        }
    }
    let root = match root {
        Some(index) => index,
        None => weights
            .iter()
            .position(|&value| value > 0)
            .ok_or(Error::BadTable { offset: TABLE_AT })?,
    };
    Ok(Tree { nodes, root })
}

fn read_symbol(tree: &Tree, bits: &mut Bits) -> Result<u8, Error> {
    let mut node = tree.root;
    loop {
        match tree.nodes[node] {
            Node::Leaf(symbol) => return Ok(symbol),
            Node::Branch { zero, one } => {
                node = if bits.bit()? == 1 { one } else { zero };
            }
        }
    }
}

struct Bits<'a> {
    data: &'a [u8],
    position: usize,
}

impl Bits<'_> {
    fn bit(&mut self) -> Result<u32, Error> {
        let index = self.position / 8;
        let byte = self
            .data
            .get(index)
            .copied()
            .ok_or(Error::Truncated { offset: index })?;
        let bit = (byte >> (7 - self.position % 8)) & 1;
        self.position += 1;
        Ok(u32::from(bit))
    }

    fn bits(&mut self, count: u32) -> Result<u32, Error> {
        let mut value = 0;
        for _ in 0..count {
            value = (value << 1) | self.bit()?;
        }
        Ok(value)
    }
}

#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    pending: u8,
    count: u8,
}

impl BitWriter {
    fn push_bit(&mut self, bit: u32) {
        self.pending = (self.pending << 1) | (bit & 1) as u8;
        self.count += 1;
        if self.count == 8 {
            self.out.push(self.pending);
            self.pending = 0;
            self.count = 0;
        }
    }

    fn push_bits(&mut self, value: u32, count: u32) {
        for shift in (0..count).rev() {
            self.push_bit((value >> shift) & 1);
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.count > 0 {
            self.out.push(self.pending << (8 - self.count));
        }
        self.out
    }
}
