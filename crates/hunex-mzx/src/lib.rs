//! MZX0 streams: a command stream over 16-bit little-endian words.
//!
//! The stream starts with `MZX0` and the decompressed size in bytes. Each command byte holds
//! the command in its low two bits and an argument in its high six bits:
//!
//! - 0, repeat: writes the last output word `arg + 1` times.
//! - 1, back-reference: one more byte `d`; copies `arg + 1` words from `d + 1` words back.
//!   The copy may overlap what it writes.
//! - 2, ring: writes the ring entry `arg`; the ring holds the last 64 literal words.
//! - 3, literal: `arg + 1` words follow, two bytes each.
//!
//! Output is cut to the decompressed size, and a final command may run past it.
//!
//! ```
//! let plain = b"abcdabcdabcd";
//! let stream = hunex_mzx::compress(plain);
//! assert!(hunex_mzx::is_mzx0(&stream));
//! assert_eq!(hunex_mzx::decompress(&stream).unwrap(), plain);
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

#[cfg(test)]
mod tests;

use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

/// The magic that starts every MZX0 stream: the ASCII name.
pub const MAGIC: &[u8; 4] = b"MZX0";
const HEADER: usize = 8;
const RING: usize = 64;
const MAX_ARG_WORDS: usize = 64;
/// The largest back-reference distance the encoder writes. The original streams never use 256
/// (stored byte 255), so the encoder stays at 255.
const MAX_DISTANCE: usize = 255;
/// Words per window. PC streams zero the repeat value at the start of each window.
pub const WINDOW_WORDS: usize = 4096;
/// The largest declared output `decompress` accepts (256 MiB). Callers that know the size they
/// expect should use `decompress_within`.
pub const MAX_DECOMPRESSED: usize = 1 << 28;
const HASH_BITS: u32 = 16;

/// Errors from reading an MZX0 stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The stream does not start with `MZX0`.
    BadMagic,
    /// The stream ends before the command or word at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// A back-reference names words before the start of the output.
    BadReference {
        /// A back-reference names words before the start of the output.
        offset: usize,
    },
    /// Bytes remain after the output is complete.
    TrailingBytes {
        /// The stream has bytes after its last command.
        offset: usize,
    },
    /// The declared output size is beyond the limit the caller set.
    Limit {
        /// The declared output size is larger than the limit the caller set.
        declared: usize,
        /// The limit that was set.
        limit: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not an MZX0 stream"),
            Error::Truncated { offset } => write!(f, "MZX0 stream truncated at 0x{offset:x}"),
            Error::BadReference { offset } => {
                write!(f, "back-reference at 0x{offset:x} is before the start")
            }
            Error::TrailingBytes { offset } => write!(f, "bytes after the output at 0x{offset:x}"),
            Error::Limit { declared, limit } => {
                write!(
                    f,
                    "declared output of {declared} bytes is over the limit of {limit}"
                )
            }
        }
    }
}

impl core::error::Error for Error {}

/// True when the bytes start with the `MZX0` magic.
pub fn is_mzx0(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

/// How a stream family differs from the PC streams.
#[derive(Clone, Copy)]
struct Flavour {
    /// The last word and every ring slot before the first literal.
    initial: u16,
    /// Literal bytes are stored XORed with 0xFF.
    xor_literals: bool,
    /// The input must end exactly after the last command.
    strict_end: bool,
}

const PC: Flavour = Flavour {
    initial: 0,
    xor_literals: false,
    strict_end: true,
};

const SWITCH: Flavour = Flavour {
    initial: 0xFFFF,
    xor_literals: true,
    strict_end: false,
};

/// How a PC stream resets the repeat value at each window boundary (every `WINDOW_WORDS` output
/// words). The streams do not say which reading the game uses; the decoder uses `Command`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowRule {
    /// No reset: a repeat writes the previous output word.
    None,
    /// A command that starts on a boundary writes zero as its repeat value. A repeat that starts
    /// before a boundary keeps its value across it.
    Command,
    /// The repeat value is zeroed at each boundary the repeat reaches, so a repeat that crosses a
    /// boundary writes zeros after it.
    Word,
}

/// Decodes a PC stream. The whole input must be consumed.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Error> {
    decompress_within(input, MAX_DECOMPRESSED)
}

/// Decodes a PC stream whose declared output is at most `limit` bytes.
pub fn decompress_within(input: &[u8], limit: usize) -> Result<Vec<u8>, Error> {
    decode(input, PC, WindowRule::Command, None, limit)
}

/// Decodes a PC stream under the given window rule. The whole input must be consumed.
pub fn decompress_with(input: &[u8], rule: WindowRule) -> Result<Vec<u8>, Error> {
    decode(input, PC, rule, None, MAX_DECOMPRESSED)
}

/// One command of a PC stream, as the decoder ran it. `start` is the output word index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// A literal command: `words` words that follow the command byte.
    Literal {
        /// Output word index where the command starts.
        start: usize,
        /// Number of words the command writes.
        words: usize,
    },
    /// A repeat command, which writes the last output word `words` times.
    Repeat {
        /// Output word index where the command starts.
        start: usize,
        /// Number of words the command writes.
        words: usize,
    },
    /// `distance` is the number of words back the copy starts (the stored byte plus one).
    BackReference {
        /// A back-reference command, which copies `words` words from `distance` words back.
        start: usize,
        /// Output word index where the command starts.
        words: usize,
        /// Number of words the command writes.
        distance: usize,
    },
    /// `written_at` is the output index of the literal word the ring slot holds.
    Ring {
        /// A ring command, which writes one word from the ring of recent literal words.
        start: usize,
        /// Output word index where the command starts.
        slot: usize,
        /// Output word index of the literal the ring slot holds.
        written_at: usize,
    },
}

/// The commands of a PC stream, decoded under `WindowRule::Command`.
pub fn trace(input: &[u8]) -> Result<Vec<Step>, Error> {
    let mut steps = Vec::new();
    decode(
        input,
        PC,
        WindowRule::Command,
        Some(&mut steps),
        MAX_DECOMPRESSED,
    )?;
    Ok(steps)
}

/// What a set of PC streams does with the window structure. The original encoder keeps every
/// non-literal command inside one window and never copies from before the window start; these
/// counts show whether a stream stays inside that envelope.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// Streams added.
    pub streams: usize,
    /// Commands across all streams.
    pub commands: usize,
    /// The largest back-reference distance, in words (the stored byte plus one).
    pub max_distance: usize,
    /// Back-references at distance 256 (stored byte 255).
    pub distance_256: usize,
    /// Repeats, back-references and ring commands whose output crosses a window boundary.
    pub crossings: usize,
    /// Back-references whose source lies before the start of the window they write in.
    pub source_before_window: usize,
    /// Ring commands across all streams.
    pub ring_uses: usize,
    /// Ring uses whose entry was written before the start of the current window.
    pub ring_from_earlier_window: usize,
    /// Non-literal commands that write the first word of a window (position 0 included). The
    /// window rule decides the value of such a word, so the encoder never writes one.
    pub window_starts_not_literal: usize,
    /// Repeat commands (command 0). The encoder writes none.
    pub repeats: usize,
}

impl Envelope {
    /// Adds one PC stream. Fails if the stream does not decode.
    pub fn add(&mut self, stream: &[u8]) -> Result<(), Error> {
        self.streams += 1;
        for step in trace(stream)? {
            self.commands += 1;
            if !matches!(step, Step::Literal { .. }) && start_of(step).is_multiple_of(WINDOW_WORDS)
            {
                self.window_starts_not_literal += 1;
            }
            match step {
                Step::Literal { .. } => {}
                Step::Repeat { start, words } => {
                    self.repeats += 1;
                    if crosses(start, words) {
                        self.crossings += 1;
                    }
                }
                Step::BackReference {
                    start,
                    words,
                    distance,
                } => {
                    self.max_distance = self.max_distance.max(distance);
                    if distance == 256 {
                        self.distance_256 += 1;
                    }
                    if crosses(start, words) {
                        self.crossings += 1;
                    }
                    if start - distance < window_start(start) {
                        self.source_before_window += 1;
                    }
                }
                Step::Ring {
                    start, written_at, ..
                } => {
                    self.ring_uses += 1;
                    if written_at < window_start(start) {
                        self.ring_from_earlier_window += 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// True when every stream stays inside the original encoder's envelope: distances up to 255
    /// words, no non-literal command across a window boundary, and no back-reference source
    /// before its window.
    pub fn within_original_envelope(&self) -> bool {
        self.max_distance <= MAX_DISTANCE && self.crossings == 0 && self.source_before_window == 0
    }
}

fn start_of(step: Step) -> usize {
    match step {
        Step::Literal { start, .. }
        | Step::Repeat { start, .. }
        | Step::BackReference { start, .. }
        | Step::Ring { start, .. } => start,
    }
}

fn window_start(position: usize) -> usize {
    (position / WINDOW_WORDS) * WINDOW_WORDS
}

fn crosses(start: usize, words: usize) -> bool {
    start / WINDOW_WORDS != (start + words - 1) / WINDOW_WORDS
}

fn decode(
    input: &[u8],
    flavour: Flavour,
    rule: WindowRule,
    mut trace: Option<&mut Vec<Step>>,
    limit: usize,
) -> Result<Vec<u8>, Error> {
    if !is_mzx0(input) {
        return Err(Error::BadMagic);
    }
    let size = read_u32(input, 4)? as usize;
    if size > limit {
        return Err(Error::Limit {
            declared: size,
            limit,
        });
    }
    let words_needed = size.div_ceil(2);
    let mut out: Vec<u16> = Vec::with_capacity(words_needed.min(input.len() * 64));
    let mut ring = [flavour.initial; RING];
    let mut ring_origin = [0usize; RING];
    let mut literals = 0usize;
    let mut position = HEADER;
    while out.len() < words_needed {
        let command = *input
            .get(position)
            .ok_or(Error::Truncated { offset: position })?;
        position += 1;
        let argument = usize::from(command >> 2);
        let start = out.len();
        let step: Option<Step>;
        match command & 3 {
            0 => {
                let previous = out.last().copied().unwrap_or(flavour.initial);
                match rule {
                    WindowRule::Word => {
                        let mut value = previous;
                        for _ in 0..=argument {
                            if out.len().is_multiple_of(WINDOW_WORDS) {
                                value = 0;
                            }
                            out.push(value);
                        }
                    }
                    _ => {
                        let at_boundary =
                            rule == WindowRule::Command && out.len().is_multiple_of(WINDOW_WORDS);
                        let value = if at_boundary { 0 } else { previous };
                        out.extend(core::iter::repeat_n(value, argument + 1));
                    }
                }
                step = Some(Step::Repeat {
                    start,
                    words: argument + 1,
                });
            }
            1 => {
                let distance = usize::from(
                    *input
                        .get(position)
                        .ok_or(Error::Truncated { offset: position })?,
                );
                position += 1;
                if distance + 1 > out.len() {
                    return Err(Error::BadReference {
                        offset: position - 1,
                    });
                }
                let source = out.len() - (distance + 1);
                for offset in 0..=argument {
                    let word = out[source + offset];
                    out.push(word);
                }
                step = Some(Step::BackReference {
                    start,
                    words: argument + 1,
                    distance: distance + 1,
                });
            }
            2 => {
                out.push(ring[argument]);
                step = Some(Step::Ring {
                    start,
                    slot: argument,
                    written_at: ring_origin[argument],
                });
            }
            _ => {
                for _ in 0..=argument {
                    let bytes = input
                        .get(position..position + 2)
                        .ok_or(Error::Truncated { offset: position })?;
                    let (mut low, mut high) = (bytes[0], bytes[1]);
                    if flavour.xor_literals {
                        low ^= 0xFF;
                        high ^= 0xFF;
                    }
                    let word = u16::from_le_bytes([low, high]);
                    position += 2;
                    ring[literals % RING] = word;
                    ring_origin[literals % RING] = out.len();
                    out.push(word);
                    literals += 1;
                }
                step = Some(Step::Literal {
                    start,
                    words: argument + 1,
                });
            }
        }
        if let (Some(steps), Some(step)) = (trace.as_deref_mut(), step) {
            steps.push(step);
        }
    }
    if flavour.strict_end && position != input.len() {
        return Err(Error::TrailingBytes { offset: position });
    }
    out.truncate(words_needed);
    let mut bytes: Vec<u8> = out.iter().flat_map(|word| word.to_le_bytes()).collect();
    bytes.truncate(size);
    Ok(bytes)
}

/// Switch-console streams: the last word and ring start as 0xFFFF, literal bytes are stored
/// XORed with 0xFF, and the decoder stops once the output reaches the declared size.
pub mod switch {
    use super::*;

    /// Decodes a Switch stream. The output stops at the declared size, and the input may have bytes after the last command.
    pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Error> {
        decode(input, SWITCH, WindowRule::None, None, MAX_DECOMPRESSED)
    }

    /// The length of a Switch stream in bytes: its header and the commands that write its output.
    /// Bytes after the last command, which a container may hold, are not part of the stream.
    pub fn stream_len(input: &[u8]) -> Result<usize, Error> {
        if !is_mzx0(input) {
            return Err(Error::BadMagic);
        }
        let size = read_u32(input, 4)? as usize;
        let words_needed = size.div_ceil(2);
        let mut position = HEADER;
        let mut words = 0usize;
        while words < words_needed {
            let command = *input
                .get(position)
                .ok_or(Error::Truncated { offset: position })?;
            position += 1;
            let argument = usize::from(command >> 2);
            match command & 3 {
                0 => words += argument + 1,
                1 => {
                    input
                        .get(position)
                        .ok_or(Error::Truncated { offset: position })?;
                    position += 1;
                    words += argument + 1;
                }
                2 => words += 1,
                _ => {
                    let bytes = (argument + 1) * 2;
                    input
                        .get(position..position + bytes)
                        .ok_or(Error::Truncated { offset: position })?;
                    position += bytes;
                    words += argument + 1;
                }
            }
        }
        Ok(position)
    }

    /// The commands of a Switch stream, in order, with the output positions they write.
    pub fn trace(input: &[u8]) -> Result<Vec<Step>, Error> {
        let mut steps = Vec::new();
        decode(
            input,
            SWITCH,
            WindowRule::None,
            Some(&mut steps),
            MAX_DECOMPRESSED,
        )?;
        Ok(steps)
    }

    /// Encodes a Switch stream with literals and back-references, under the same rules as the PC
    /// encoder: no repeat commands, no ring recalls, copies of at most 255 words that stay inside
    /// their 4096-word window, and a literal first word in each window. Literal bytes are XORed
    /// with 0xFF.
    pub fn compress(plain: &[u8]) -> Vec<u8> {
        encode(plain, true)
    }

    /// A literal-only Switch stream. Literal bytes are XORed with 0xFF, and an odd final byte is
    /// padded with a zero byte.
    pub fn compress_literal(plain: &[u8]) -> Vec<u8> {
        let mut out = header(plain.len());
        for chunk in plain.chunks(2 * MAX_ARG_WORDS) {
            let words = chunk.len().div_ceil(2);
            out.push((((words - 1) as u8) << 2) | 3);
            for (index, &byte) in chunk.iter().enumerate() {
                out.push(byte ^ 0xFF);
                if index + 1 == chunk.len() && chunk.len() % 2 == 1 {
                    out.push(0);
                }
            }
        }
        out
    }
}

/// Encodes with runs, back-references and literals. Decoding the result gives `plain`.
pub fn compress(plain: &[u8]) -> Vec<u8> {
    encode(plain, false)
}

fn encode(plain: &[u8], xor_literals: bool) -> Vec<u8> {
    let words = to_words(plain);
    let mut out = header(plain.len());
    let mut literal: Vec<u16> = Vec::new();
    let mut last = 0u16;
    let mut table = vec![0u32; 1 << HASH_BITS];
    let mut i = 0usize;
    while i < words.len() {
        // The first word of each window comes from a literal, so the window rule never decides
        // it. A run never reaches the next boundary either.
        let at_window_start = i.is_multiple_of(WINDOW_WORDS);
        let value = last;
        if !at_window_start && words[i] == value {
            let boundary = (i / WINDOW_WORDS + 1) * WINDOW_WORDS;
            let limit = MAX_ARG_WORDS.min(boundary - i);
            let run = (i..words.len())
                .take(limit)
                .take_while(|&k| words[k] == value)
                .count();
            if run >= 2 {
                // The run copies the word before it: a back-reference at distance 1 that overlaps
                // its own output. The encoder writes no repeat commands.
                flush(&mut out, &mut literal, xor_literals);
                out.push((((run - 1) << 2) | 1) as u8);
                out.push(0);
                i += run;
                continue;
            }
        }
        let matched = if i + 1 < words.len() && !at_window_start {
            let candidate = table[hash(words[i], words[i + 1])] as usize;
            if candidate > 0 {
                let source = candidate - 1;
                let distance = i - source;
                // The copy stays inside the window of `i`: its source is no earlier than the
                // window start, and its output does not cross the window end.
                let reach = MAX_DISTANCE.min(i - window_start(i));
                let boundary = window_start(i) + WINDOW_WORDS;
                if (1..=reach).contains(&distance) {
                    let length = (0..MAX_ARG_WORDS.min(words.len() - i).min(boundary - i))
                        .take_while(|&t| words[source + t] == words[i + t])
                        .count();
                    (length >= 2).then_some((distance, length))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        match matched {
            Some((distance, length)) => {
                flush(&mut out, &mut literal, xor_literals);
                out.push((((length - 1) as u8) << 2) | 1);
                out.push((distance - 1) as u8);
                for t in 0..length {
                    register(&mut table, &words, i + t);
                    last = words[i + t];
                }
                i += length;
            }
            None => {
                register(&mut table, &words, i);
                literal.push(words[i]);
                last = words[i];
                i += 1;
                if literal.len() == MAX_ARG_WORDS {
                    flush(&mut out, &mut literal, xor_literals);
                }
            }
        }
    }
    flush(&mut out, &mut literal, xor_literals);
    out
}

/// Encodes with literal commands only. Every word is written in a literal run.
pub fn compress_literal(plain: &[u8]) -> Vec<u8> {
    let words = to_words(plain);
    let mut out = header(plain.len());
    for chunk in words.chunks(MAX_ARG_WORDS) {
        out.push(((chunk.len() - 1) << 2) as u8 | 3);
        for word in chunk {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out
}

fn header(size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(size as u32).to_le_bytes());
    out
}

fn to_words(plain: &[u8]) -> Vec<u16> {
    plain
        .chunks(2)
        .map(|pair| u16::from_le_bytes([pair[0], *pair.get(1).unwrap_or(&0)]))
        .collect()
}

fn flush(out: &mut Vec<u8>, literal: &mut Vec<u16>, xor_literals: bool) {
    if literal.is_empty() {
        return;
    }
    out.push((((literal.len() - 1) as u8) << 2) | 3);
    for word in literal.iter() {
        let stored = if xor_literals { word ^ 0xFFFF } else { *word };
        out.extend_from_slice(&stored.to_le_bytes());
    }
    literal.clear();
}

fn hash(first: u16, second: u16) -> usize {
    let key = (u32::from(first) << 16) | u32::from(second);
    (key.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize
}

fn register(table: &mut [u32], words: &[u16], position: usize) {
    if position + 1 < words.len() {
        table[hash(words[position], words[position + 1])] = (position + 1) as u32;
    }
}

fn read_u32(input: &[u8], offset: usize) -> Result<u32, Error> {
    match input.get(offset..offset + 4) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(Error::Truncated { offset }),
    }
}
