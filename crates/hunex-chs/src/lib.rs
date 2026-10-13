//! Decoder for HuneX compiled script (`.chs`) files.
//!
//! The input is copied, so a parsed [`Script`] owns its data.

#![no_std]

extern crate alloc;

mod opcodes;
#[cfg(test)]
mod tests;

pub use opcodes::{opcode_info, OpcodeInfo, OPCODES};

use alloc::vec::Vec;
use core::fmt;

const HUNEX_MAGIC: &[u8] = b"HunexCompiledScriptVer1.00\0";
const BURIKO_MAGIC: &[u8] = b"BurikoCompiledScriptVer1.00\0";
const OP_PUSH_STRING: u32 = 0x03;
const OP_RETURN: u32 = 0x1B;
const OP_DEBUG: u32 = 0x7F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Hunex,
    Buriko,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub offset: usize,
    pub opcode: u32,
    pub operands: Vec<u32>,
}

impl Instruction {
    pub fn size(&self) -> usize {
        4 * (1 + self.operands.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub format: Format,
    pub header_size: u32,
    pub code_base: usize,
    pub includes: Vec<Vec<u8>>,
    pub string_pool_start: usize,
    pub instructions: Vec<Instruction>,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BadMagic,
    Truncated { offset: usize },
    UnregisteredOpcode { offset: usize, opcode: u32 },
    PoolBoundary { offset: usize },
    BadStringOperand { offset: usize, value: u32 },
    InvalidUtf8 { offset: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not a HuneX or Buriko compiled script"),
            Error::Truncated { offset } => write!(f, "file truncated at 0x{offset:x}"),
            Error::UnregisteredOpcode { offset, opcode } => {
                write!(f, "unregistered opcode 0x{opcode:x} at 0x{offset:x}")
            }
            Error::PoolBoundary { offset } => write!(
                f,
                "code walk does not end on a return exactly at the string pool (instruction at 0x{offset:x})"
            ),
            Error::BadStringOperand { offset, value } => write!(
                f,
                "string operand 0x{value:x} at 0x{offset:x} is not a string start"
            ),
            Error::InvalidUtf8 { offset } => write!(f, "string at 0x{offset:x} is not UTF-8"),
        }
    }
}

impl core::error::Error for Error {}

impl Script {
    pub fn parse(bytes: &[u8]) -> Result<Script, Error> {
        let (format, magic) = if bytes.starts_with(HUNEX_MAGIC) {
            (Format::Hunex, HUNEX_MAGIC)
        } else if bytes.starts_with(BURIKO_MAGIC) {
            (Format::Buriko, BURIKO_MAGIC)
        } else {
            return Err(Error::BadMagic);
        };
        let data = bytes.to_vec();

        let header_size = read_u32(&data, magic.len())?;
        let code_base = magic
            .len()
            .checked_add(header_size as usize)
            .filter(|&base| base <= data.len())
            .ok_or(Error::Truncated { offset: data.len() })?;

        let include_count = read_u32(&data, magic.len() + 4)?;
        let mut position = magic.len() + 8;
        let mut includes = Vec::new();
        for _ in 0..include_count {
            let rest = &data[position..];
            let length = rest
                .iter()
                .position(|&byte| byte == 0)
                .ok_or(Error::Truncated { offset: data.len() })?;
            includes.push(rest[..length].to_vec());
            position += length + 1;
        }

        let (greedy, end, stop) = decode_all(&data, code_base);
        let first_string = greedy
            .iter()
            .filter(|instruction| is_string_opcode(instruction.opcode))
            .filter_map(|instruction| {
                instruction
                    .operands
                    .first()
                    .map(|&v| (v, instruction.offset))
            })
            .min();

        let pool = match (first_string, stop) {
            (Some((value, offset)), _) => {
                match code_base
                    .checked_add(value as usize)
                    .filter(|&pool| pool <= end)
                {
                    Some(pool) => pool,
                    None => match stop {
                        Some(error) => return Err(error),
                        None => return Err(Error::BadStringOperand { offset, value }),
                    },
                }
            }
            (None, Some(error)) => return Err(error),
            (None, None) => data.len(),
        };

        let code: Vec<Instruction> = greedy
            .into_iter()
            .take_while(|instruction| instruction.offset < pool)
            .collect();
        match code.last() {
            Some(last) if last.opcode == OP_RETURN && last.offset + last.size() == pool => {}
            Some(last) => {
                return Err(Error::PoolBoundary {
                    offset: last.offset,
                })
            }
            None => return Err(Error::PoolBoundary { offset: code_base }),
        }

        for instruction in &code {
            if !is_string_opcode(instruction.opcode) {
                continue;
            }
            let value = instruction.operands[0];
            let target = code_base.checked_add(value as usize);
            if !target.is_some_and(|target| is_string_start(&data, pool, target)) {
                return Err(Error::BadStringOperand {
                    offset: instruction.offset,
                    value,
                });
            }
        }

        Ok(Script {
            format,
            header_size,
            code_base,
            includes,
            string_pool_start: pool,
            instructions: code,
            data,
        })
    }

    pub fn string_bytes(&self, instruction: &Instruction) -> Result<&[u8], Error> {
        let value = instruction.operands.first().copied().unwrap_or(0);
        let bad = Error::BadStringOperand {
            offset: instruction.offset,
            value,
        };
        if !is_string_opcode(instruction.opcode) {
            return Err(bad);
        }
        let start = self
            .code_base
            .checked_add(value as usize)
            .filter(|&start| is_string_start(&self.data, self.string_pool_start, start))
            .ok_or(bad)?;
        let length = self.data[start..]
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(bad)?;
        Ok(&self.data[start..start + length])
    }

    pub fn string(&self, instruction: &Instruction) -> Result<&str, Error> {
        let bytes = self.string_bytes(instruction)?;
        core::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8 {
            offset: instruction.offset,
        })
    }
}

fn is_string_opcode(opcode: u32) -> bool {
    opcode == OP_PUSH_STRING || opcode == OP_DEBUG
}

fn is_string_start(data: &[u8], pool: usize, target: usize) -> bool {
    target >= pool
        && target < data.len()
        && target
            .checked_sub(1)
            .and_then(|previous| data.get(previous))
            == Some(&0)
}

fn read_u32(data: &[u8], position: usize) -> Result<u32, Error> {
    let truncated = Error::Truncated { offset: position };
    let end = position.checked_add(4).ok_or(truncated)?;
    match data.get(position..end) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(truncated),
    }
}

fn decode_one(data: &[u8], position: usize) -> Result<Instruction, Error> {
    let opcode = read_u32(data, position)?;
    let info = opcode_info(opcode).ok_or(Error::UnregisteredOpcode {
        offset: position,
        opcode,
    })?;
    let mut operands = Vec::with_capacity(info.operands);
    for index in 0..info.operands {
        operands.push(read_u32(data, position + 4 + 4 * index)?);
    }
    Ok(Instruction {
        offset: position,
        opcode,
        operands,
    })
}

/// Decodes from `start` until the end of the data or the first undecodable position.
/// Returns the instructions, the position where decoding stopped, and the reason it stopped.
fn decode_all(data: &[u8], start: usize) -> (Vec<Instruction>, usize, Option<Error>) {
    let mut instructions = Vec::new();
    let mut position = start;
    while position < data.len() {
        match decode_one(data, position) {
            Ok(instruction) => {
                position += instruction.size();
                instructions.push(instruction);
            }
            Err(error) => return (instructions, position, Some(error)),
        }
    }
    (instructions, position, None)
}
