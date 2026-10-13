//! The LenZu compressor: greedy LZ77 with hash chains, then a Huffman table built from the token
//! counts, written in the form the stream's count selects.

use alloc::vec;
use alloc::vec::Vec;

use crate::{build_tree, write_fields, BitWriter, Node, Tree, SYMBOLS};

/// Longest literal run. The originals never exceed 54 bytes.
const RUN_MAX: usize = 54;
/// Shortest match: the length field's base.
const MATCH_MIN: usize = 2;
/// Longest match: the base plus the largest length symbol, 127.
const MATCH_MAX: usize = 129;
/// Farthest match: the base plus the two 7-bit distance fields.
const DISTANCE_MAX: usize = 16_385;
/// Deepest code the compressor writes. The originals reach 13.
const DEPTH_MAX: usize = 13;
/// Dense tables start at this many listed symbols (the decoder's rule).
const DENSE_FROM: usize = 103;
const HASH_BITS: u32 = 16;
const CHAIN_LIMIT: usize = 64;
const NONE: u32 = u32::MAX;

#[derive(Clone, Copy)]
enum Token {
    Literal { start: usize, length: usize },
    Match { length: usize, distance: usize },
}

/// Compresses `plain` to a LenZu stream that the decoder reads back.
///
/// Matches are at least 2 bytes, at most 129, and at most 16385 bytes back. A match never copies
/// bytes it has just written. Literal runs are at most 54 bytes. The table holds the token counts;
/// its weights are scaled down only if a code would be deeper than 13.
pub fn compress(plain: &[u8]) -> Vec<u8> {
    let tokens = tokenize(plain);
    let mut counts = [0u64; SYMBOLS];
    for token in &tokens {
        match *token {
            Token::Literal { length, .. } => counts[length - 1] += 1,
            Token::Match { length, distance } => {
                counts[length - MATCH_MIN] += 1;
                counts[(distance - MATCH_MIN) >> 7] += 1;
            }
        }
    }
    if counts.iter().all(|&count| count == 0) {
        counts[0] = 1;
    }
    let (weights, tree) = shallow_table(&counts);
    let codes = codes(&tree);

    let mut out = Vec::new();
    write_fields(&mut out, plain);
    write_table(&mut out, &weights);
    let mut writer = BitWriter::default();
    for token in &tokens {
        match *token {
            Token::Literal { start, length } => {
                writer.push_bit(0);
                push_code(&mut writer, &codes, length - 1);
                for &byte in &plain[start..start + length] {
                    writer.push_bits(u32::from(byte), 8);
                }
            }
            Token::Match { length, distance } => {
                writer.push_bit(1);
                push_code(&mut writer, &codes, length - MATCH_MIN);
                let far = distance - MATCH_MIN;
                push_code(&mut writer, &codes, far >> 7);
                writer.push_bits((far & 0x7F) as u32, 7);
            }
        }
    }
    out.extend_from_slice(&writer.finish());
    out
}

/// Greedy parse. Each position takes the longest match among the recent positions with the same
/// three bytes, or else starts or extends a literal run.
fn tokenize(plain: &[u8]) -> Vec<Token> {
    let n = plain.len();
    let mut head = vec![NONE; 1 << HASH_BITS];
    let mut prev = vec![NONE; n];
    let mut tokens = Vec::new();
    let mut pending: Option<(usize, usize)> = None;
    let mut position = 0usize;
    while position < n {
        let found = find_match(plain, position, &head, &prev);
        if let Some((length, distance)) = found {
            flush_literals(&mut tokens, &mut pending);
            tokens.push(Token::Match { length, distance });
            for step in 0..length {
                insert(plain, position + step, &mut head, &mut prev);
            }
            position += length;
        } else {
            match pending {
                Some((start, length)) if length < RUN_MAX => pending = Some((start, length + 1)),
                _ => {
                    flush_literals(&mut tokens, &mut pending);
                    pending = Some((position, 1));
                }
            }
            insert(plain, position, &mut head, &mut prev);
            position += 1;
        }
    }
    flush_literals(&mut tokens, &mut pending);
    tokens
}

fn flush_literals(tokens: &mut Vec<Token>, pending: &mut Option<(usize, usize)>) {
    if let Some((start, length)) = pending.take() {
        tokens.push(Token::Literal { start, length });
    }
}

fn hash(plain: &[u8], position: usize) -> usize {
    let key = (u32::from(plain[position]) << 16)
        | (u32::from(plain[position + 1]) << 8)
        | u32::from(plain[position + 2]);
    (key.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize
}

fn insert(plain: &[u8], position: usize, head: &mut [u32], prev: &mut [u32]) {
    if position + 3 <= plain.len() {
        let slot = hash(plain, position);
        prev[position] = head[slot];
        head[slot] = position as u32;
    }
}

/// The longest match at `position`, as (length, distance), if it is at least `MATCH_MIN`.
fn find_match(plain: &[u8], position: usize, head: &[u32], prev: &[u32]) -> Option<(usize, usize)> {
    if position + 3 > plain.len() {
        return None;
    }
    let limit = MATCH_MAX.min(plain.len() - position);
    let mut best: Option<(usize, usize)> = None;
    let mut candidate = head[hash(plain, position)];
    for _ in 0..CHAIN_LIMIT {
        if candidate == NONE {
            break;
        }
        let source = candidate as usize;
        let distance = position - source;
        if distance > DISTANCE_MAX {
            break;
        }
        if distance >= MATCH_MIN {
            let max = limit.min(distance);
            let length = (0..max)
                .take_while(|&k| plain[source + k] == plain[position + k])
                .count();
            if length >= MATCH_MIN && best.is_none_or(|(best_length, _)| length > best_length) {
                best = Some((length, distance));
                if length == limit {
                    break;
                }
            }
        }
        candidate = prev[source];
    }
    best
}

/// The weights to write and the tree they give. Halves the counts, keeping every used symbol at
/// weight at least 1, until the deepest code is at most `DEPTH_MAX`.
fn shallow_table(counts: &[u64; SYMBOLS]) -> ([u64; SYMBOLS], Tree) {
    let mut shift = 0u32;
    loop {
        let mut weights = [0u64; SYMBOLS];
        for (weight, &count) in weights.iter_mut().zip(counts) {
            if count > 0 {
                *weight = (count >> shift).max(1);
            }
        }
        let tree = build_tree(&weights).expect("the table has a used symbol");
        if tree.max_depth() <= DEPTH_MAX {
            return (weights, tree);
        }
        shift += 1;
    }
}

/// The code of each symbol: its bits, most significant first, and its length.
fn codes(tree: &Tree) -> Vec<(u32, u32)> {
    let mut table = vec![(0u32, 0u32); SYMBOLS];
    let mut stack = vec![(tree.root, 0u32, 0u32)];
    while let Some((node, code, length)) = stack.pop() {
        match tree.nodes[node] {
            Node::Leaf(symbol) => table[usize::from(symbol)] = (code, length),
            Node::Branch { zero, one } => {
                stack.push((zero, code << 1, length + 1));
                stack.push((one, (code << 1) | 1, length + 1));
            }
        }
    }
    table
}

fn push_code(writer: &mut BitWriter, codes: &[(u32, u32)], symbol: usize) {
    let (code, length) = codes[symbol];
    writer.push_bits(code, length);
}

/// Writes the table: the count byte, then the dense weights or the sparse pairs.
fn write_table(out: &mut Vec<u8>, weights: &[u64; SYMBOLS]) {
    let used = weights.iter().filter(|&&weight| weight > 0).count();
    out.push(used as u8);
    if used >= DENSE_FROM {
        for &weight in weights {
            out.extend_from_slice(&(weight as u32).to_le_bytes());
        }
    } else {
        for (symbol, &weight) in weights.iter().enumerate() {
            if weight > 0 {
                out.push(symbol as u8);
                out.extend_from_slice(&(weight as u32).to_le_bytes());
            }
        }
    }
}
