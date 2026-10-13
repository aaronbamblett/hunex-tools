//! LenZu compressed streams, as stored in HuneX engine archives.
//!
//! [`decompress`] reads the variant seen in the game's archives: seven-bit Huffman symbols and
//! back-reference distances with seven low bits, base two. It checks the header checksum.
//!
//! [`compress`] writes that variant: greedy LZ77 matches, and a Huffman table built from the
//! token counts. [`compress_literal`] writes a stream of literal runs only.
//!
//! ```
//! let plain = b"the same words, the same words";
//! let stored = hunex_lenzu::compress(plain);
//! assert!(hunex_lenzu::is_lenzu(&stored));
//! assert_eq!(hunex_lenzu::decompress(&stored).unwrap(), plain);
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

mod encode;
#[cfg(test)]
mod tests;

pub use encode::compress;

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

/// The magic that starts every LenZu stream: the ASCII name and one zero byte.
pub const MAGIC: &[u8] = b"LenZuCompressor\0";

const SIZE_AT: usize = 0x20;
const CHECKSUM_HIGH_AT: usize = 0x24;
const CHECKSUM_LOW_AT: usize = 0x28;
const WIDTH_RAW_AT: usize = 0x31;
const WIDTH_MIN_AT: usize = 0x32;
const LOW_COUNT_AT: usize = 0x34;
const BASE_AT: usize = 0x35;
const TABLE_AT: usize = 0x36;
pub(crate) const SYMBOLS: usize = 128;
const SYMBOL_BITS: u32 = 7;
const LITERAL_MAX: usize = 128;
const CHECKSUM_KEYS: [u64; 4] = [0xE9, 0x115, 0x137, 0x1B1];
const OPTIONS: [u8; 6] = [0x06, 0x07, 0x07, 0x0E, 0x07, 0x02];
const CONSTANT_A: u32 = 49;
const CONSTANT_B: u32 = 48;

/// Errors from reading a LenZu stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The stream does not start with the magic.
    BadMagic,
    /// The stream ends before the field or symbol at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// The stream uses parameters other than the seven-bit variant.
    Unsupported,
    /// The Huffman table is malformed at `offset`.
    BadTable {
        /// Byte offset of the malformed table entry.
        offset: usize,
    },
    /// A back-reference reaches before the start of the output.
    BadDistance {
        /// Output position of the back-reference.
        offset: usize,
    },
    /// The checksum in the header does not match the decoded bytes.
    ChecksumMismatch {
        /// The checksum stored in the header.
        stored: u64,
        /// The checksum of the decoded bytes.
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

/// What the decoder read from one stream: the header options, the table, the code tree and the
/// tokens. The analyser uses it to describe the envelope of the original streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    /// Header bytes 0x30 to 0x35.
    pub options: [u8; 6],
    /// Header field at 0x2C.
    pub field_2c: u32,
    /// True for the dense table form (128 weights, one per symbol).
    pub dense: bool,
    /// The table count byte as stored; 0 means 128.
    pub declared_count: usize,
    /// The weight of each symbol in the table (0 for symbols not listed).
    pub weights: [u64; SYMBOLS],
    /// How often each symbol is read from the bitstream.
    pub symbol_counts: [u64; SYMBOLS],
    /// The depth of the deepest leaf of the code tree.
    pub max_code_depth: usize,
    /// Number of literal runs in the stream.
    pub literal_runs: usize,
    /// Length of the longest literal run, in bytes.
    pub longest_literal_run: usize,
    /// Number of back-reference tokens.
    pub matches: usize,
    /// Match lengths and distances, in bytes (0 when there are no matches).
    pub shortest_match: usize,
    /// Length of the longest back-reference, in bytes.
    pub longest_match: usize,
    /// Distance of the shortest back-reference, in bytes.
    pub shortest_distance: usize,
    /// Distance of the longest back-reference, in bytes.
    pub longest_distance: usize,
    /// Matches longer than their distance, which copy bytes they have just written.
    pub overlapping_matches: usize,
    /// True when the unread bits of the last byte are zero.
    pub padding_is_zero: bool,
}

/// Decodes a LenZu stream and checks its header checksum.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Error> {
    decode(input).map(|(out, _)| out)
}

/// Reads a stream and reports what it contains. The output is not returned.
pub fn trace(input: &[u8]) -> Result<Trace, Error> {
    decode(input).map(|(_, trace)| trace)
}

fn decode(input: &[u8]) -> Result<(Vec<u8>, Trace), Error> {
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

    let table = read_table(input)?;
    let tree = build_tree(&table.weights)?;
    let mut bits = Bits {
        data: input,
        position: table.body * 8,
    };
    let mut trace = Trace {
        options: [0; 6],
        field_2c: u32_at(input, 0x2C)?,
        dense: table.dense,
        declared_count: table.declared,
        weights: table.weights,
        symbol_counts: [0; SYMBOLS],
        max_code_depth: tree.max_depth(),
        literal_runs: 0,
        longest_literal_run: 0,
        matches: 0,
        shortest_match: 0,
        longest_match: 0,
        shortest_distance: 0,
        longest_distance: 0,
        overlapping_matches: 0,
        padding_is_zero: true,
    };
    trace.options.copy_from_slice(
        input
            .get(0x30..0x36)
            .ok_or(Error::Truncated { offset: 0x30 })?,
    );

    let mut out = Vec::new();
    while out.len() < size {
        if bits.bit()? == 0 {
            let symbol = read_symbol(&tree, &mut bits)?;
            trace.symbol_counts[usize::from(symbol)] += 1;
            let count = symbol as usize + 1;
            trace.literal_runs += 1;
            trace.longest_literal_run = trace.longest_literal_run.max(count);
            for _ in 0..count {
                out.push(bits.bits(8)? as u8);
                if out.len() == size {
                    break;
                }
            }
        } else {
            let length_symbol = read_symbol(&tree, &mut bits)?;
            trace.symbol_counts[usize::from(length_symbol)] += 1;
            let length = length_symbol as usize + base;
            let high_symbol = read_symbol(&tree, &mut bits)?;
            trace.symbol_counts[usize::from(high_symbol)] += 1;
            let high = u32::from(high_symbol);
            let low = if low_count == 0 {
                0
            } else {
                bits.bits(low_count)?
            };
            let distance = ((low | (high << low_count)) as usize) + base;
            if distance == 0 || distance > out.len() {
                return Err(Error::BadDistance { offset: out.len() });
            }
            if trace.matches == 0 {
                trace.shortest_match = length;
                trace.shortest_distance = distance;
            }
            trace.matches += 1;
            trace.shortest_match = trace.shortest_match.min(length);
            trace.longest_match = trace.longest_match.max(length);
            trace.shortest_distance = trace.shortest_distance.min(distance);
            trace.longest_distance = trace.longest_distance.max(distance);
            if length > distance {
                trace.overlapping_matches += 1;
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

    let used = bits.position % 8;
    if used != 0 {
        let last = input[bits.position / 8];
        trace.padding_is_zero = last & ((1u8 << (8 - used)) - 1) == 0;
    }

    let computed = checksum(&out);
    if computed != stored {
        return Err(Error::ChecksumMismatch { stored, computed });
    }
    Ok((out, trace))
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
pub(crate) fn write_fields(out: &mut Vec<u8>, plain: &[u8]) {
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

struct Table {
    weights: [u64; SYMBOLS],
    /// The offset where the bitstream starts.
    body: usize,
    dense: bool,
    declared: usize,
}

/// Reads the Huffman weights and where the bitstream starts.
fn read_table(input: &[u8]) -> Result<Table, Error> {
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
        return Ok(Table {
            weights,
            body: TABLE_AT + 1 + 4 * SYMBOLS,
            dense: true,
            declared,
        });
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
    Ok(Table {
        weights,
        body: position,
        dense: false,
        declared,
    })
}

#[derive(Clone, Copy)]
pub(crate) enum Node {
    Leaf(u8),
    Branch { zero: usize, one: usize },
}

pub(crate) struct Tree {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: usize,
}

/// Builds the code tree: repeatedly join the two lightest parentless nodes (ties go to the
/// earlier node) until the joined weight covers every leaf weight.
pub(crate) fn build_tree(weights: &[u64; SYMBOLS]) -> Result<Tree, Error> {
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

impl Tree {
    /// The depth of the deepest leaf that has a weight.
    pub(crate) fn max_depth(&self) -> usize {
        let mut deepest = 0;
        let mut stack = vec![(self.root, 0usize)];
        while let Some((node, depth)) = stack.pop() {
            match self.nodes[node] {
                Node::Leaf(_) => deepest = deepest.max(depth),
                Node::Branch { zero, one } => {
                    stack.push((zero, depth + 1));
                    stack.push((one, depth + 1));
                }
            }
        }
        deepest
    }
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
pub(crate) struct BitWriter {
    out: Vec<u8>,
    pending: u8,
    count: u8,
}

impl BitWriter {
    pub(crate) fn push_bit(&mut self, bit: u32) {
        self.pending = (self.pending << 1) | (bit & 1) as u8;
        self.count += 1;
        if self.count == 8 {
            self.out.push(self.pending);
            self.pending = 0;
            self.count = 0;
        }
    }

    pub(crate) fn push_bits(&mut self, value: u32, count: u32) {
        for shift in (0..count).rev() {
            self.push_bit((value >> shift) & 1);
        }
    }

    pub(crate) fn finish(mut self) -> Vec<u8> {
        if self.count > 0 {
            self.out.push(self.pending << (8 - self.count));
        }
        self.out
    }
}

/// Facts about a set of streams, from their traces. Each field is a count or an extreme over the
/// streams added so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Envelope {
    /// Number of streams added.
    pub streams: usize,
    /// Distinct option byte sets (header bytes 0x30 to 0x35).
    pub options: BTreeSet<[u8; 6]>,
    /// Streams whose header field at 0x2C is not zero.
    pub nonzero_field_2c: usize,
    /// Streams with the dense table form.
    pub dense_tables: usize,
    /// Streams with the sparse table form.
    pub sparse_tables: usize,
    /// Declared count bytes of the dense and sparse tables.
    pub dense_counts: BTreeSet<usize>,
    /// The distinct declared counts of the sparse tables.
    pub sparse_counts: BTreeSet<usize>,
    /// The deepest code of any stream, in bits.
    pub max_code_depth: usize,
    /// Streams whose table weights equal the symbol counts of their bitstream.
    pub weights_are_counts: usize,
    /// Literal runs across all streams.
    pub literal_runs: usize,
    /// The longest literal run of any stream, in bytes.
    pub longest_literal_run: usize,
    /// Back-references across all streams.
    pub matches: usize,
    /// The shortest back-reference of any stream, in bytes.
    pub shortest_match: usize,
    /// The longest back-reference of any stream, in bytes.
    pub longest_match: usize,
    /// The shortest back-reference distance of any stream, in bytes.
    pub shortest_distance: usize,
    /// The longest back-reference distance of any stream, in bytes.
    pub longest_distance: usize,
    /// Back-references that copy bytes they have just written, which the original streams never contain.
    pub overlapping_matches: usize,
    /// Streams whose unused bits in the last byte are all zero.
    pub padding_zero_streams: usize,
    /// Streams whose declared count equals the number of symbols with a nonzero weight.
    pub count_is_used_symbols: usize,
}

impl Envelope {
    /// Adds one stream. Fails if the stream does not decode.
    pub fn add(&mut self, stream: &[u8]) -> Result<(), Error> {
        let trace = trace(stream)?;
        self.streams += 1;
        self.options.insert(trace.options);
        if trace.field_2c != 0 {
            self.nonzero_field_2c += 1;
        }
        if trace.dense {
            self.dense_tables += 1;
            self.dense_counts.insert(trace.declared_count);
        } else {
            self.sparse_tables += 1;
            self.sparse_counts.insert(trace.declared_count);
        }
        self.max_code_depth = self.max_code_depth.max(trace.max_code_depth);
        if trace.weights == trace.symbol_counts {
            self.weights_are_counts += 1;
        }
        let used = trace.weights.iter().filter(|&&w| w > 0).count();
        if trace.declared_count == used || (trace.declared_count == SYMBOLS && used == SYMBOLS) {
            self.count_is_used_symbols += 1;
        }
        self.literal_runs += trace.literal_runs;
        self.longest_literal_run = self.longest_literal_run.max(trace.longest_literal_run);
        if trace.matches > 0 {
            let first = self.matches == 0;
            self.shortest_match = if first {
                trace.shortest_match
            } else {
                self.shortest_match.min(trace.shortest_match)
            };
            self.longest_match = self.longest_match.max(trace.longest_match);
            self.shortest_distance = if first {
                trace.shortest_distance
            } else {
                self.shortest_distance.min(trace.shortest_distance)
            };
            self.longest_distance = self.longest_distance.max(trace.longest_distance);
        }
        self.matches += trace.matches;
        self.overlapping_matches += trace.overlapping_matches;
        if trace.padding_is_zero {
            self.padding_zero_streams += 1;
        }
        Ok(())
    }
}
