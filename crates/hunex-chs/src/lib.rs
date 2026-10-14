//! Decoder and exact re-encoder for HuneX compiled script (`.chs`) files.
//!
//! [`Script::parse`] returns owned instructions, header facts and the string pool.
//! [`Script::encode`] rebuilds the file from those parts; for a parsed file it returns the
//! original bytes exactly.
//!
//! The [`source`] module parses Switch script text into statements, and the [`normalize`]
//! module reduces a [`Program`] to the Debug-free form used to compare against real files.
//!
//! Known limit: the code section has no explicit length. The string pool is taken to start
//! at the smallest string pointer (code base plus pointer), and the walk must end on a return
//! exactly there. A crafted file whose smallest pointer targets a return in the middle of the
//! code is accepted, and the code after that return is treated as pool. Real files cannot be
//! told apart from that case, and the exe has no explicit code length either.
//!
//! ```
//! // A script with no strings: the magic, a header size of 8, no includes, then one return.
//! let mut bytes = b"HunexCompiledScriptVer1.00\0".to_vec();
//! bytes.extend_from_slice(&8u32.to_le_bytes());
//! bytes.extend_from_slice(&0u32.to_le_bytes());
//! bytes.extend_from_slice(&0x1Bu32.to_le_bytes());
//! let script = hunex_chs::Script::parse(&bytes).unwrap();
//! assert_eq!(script.encode(), bytes);
//! assert_eq!(script.lift().unwrap().lower().unwrap().encode(), bytes);
//! ```

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

pub mod compare;
#[cfg(test)]
mod compare_tests;
pub mod compile;
#[cfg(test)]
mod compile_tests;
pub mod normalize;
#[cfg(test)]
mod normalize_tests;
mod opcodes;
mod program;
#[cfg(test)]
mod program_tests;
pub mod recipes;
pub mod source;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod tests;

pub use opcodes::{opcode_info, OpcodeInfo, OPCODES};
pub use program::{
    EditError, Item, LabelId, LowerError, Operand, PoolEntry, Program, ProgramParts, StrId,
};

use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

const HUNEX_MAGIC: &[u8] = b"HunexCompiledScriptVer1.00\0";
const BURIKO_MAGIC: &[u8] = b"BurikoCompiledScriptVer1.00\0";
const OP_PUSH_STRING: u32 = 0x03;
const OP_RETURN: u32 = 0x1B;
const OP_DEBUG: u32 = 0x7F;
const MAX_OPERANDS: usize = 3;
const HEADER_FIELDS_SIZE: usize = 8;

/// The two script formats. They differ in their magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// The Hunex format, with the `HUNEX` magic.
    Hunex,
    /// The Buriko format, with its own magic.
    Buriko,
}

impl Format {
    fn magic(self) -> &'static [u8] {
        match self {
            Format::Hunex => HUNEX_MAGIC,
            Format::Buriko => BURIKO_MAGIC,
        }
    }
}

/// One instruction: its byte offset, its opcode and its dword operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    offset: usize,
    opcode: u32,
    operands: [u32; MAX_OPERANDS],
    operand_count: u8,
}

impl Instruction {
    /// Byte offset of the instruction in the file.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The opcode, the first dword of the instruction.
    pub fn opcode(&self) -> u32 {
        self.opcode
    }

    /// The operands that follow the opcode, in file order.
    pub fn operands(&self) -> &[u32] {
        &self.operands[..usize::from(self.operand_count)]
    }

    /// Size of the instruction in bytes: four bytes for the opcode and four for each operand.
    pub fn size(&self) -> usize {
        4 * (1 + usize::from(self.operand_count))
    }
}

/// A parsed script. The code is a list of instructions and the string pool is the bytes after them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    format: Format,
    header_size: u32,
    code_base: usize,
    includes: Vec<Vec<u8>>,
    header_padding: Vec<u8>,
    string_pool_start: usize,
    instructions: Vec<Instruction>,
    pool: Vec<u8>,
}

/// Errors from reading a script file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The file does not start with a known magic.
    BadMagic,
    /// The data ends before the field at `offset`.
    Truncated {
        /// Byte offset where the data ends.
        offset: usize,
    },
    /// A header field has a value this crate does not accept.
    BadHeader {
        /// Byte offset of the bad header field.
        offset: usize,
    },
    /// An instruction uses an opcode that is not in the opcode table.
    UnregisteredOpcode {
        /// Byte offset of the instruction.
        offset: usize,
        /// The opcode that is not in the table.
        opcode: u32,
    },
    /// The code runs into the string pool at `offset`.
    PoolBoundary {
        /// Byte offset of the pool boundary.
        offset: usize,
    },
    /// A string operand does not point at a valid string.
    BadStringOperand {
        /// Byte offset of the instruction.
        offset: usize,
        /// The operand that is not a valid string pointer.
        value: u32,
    },
    /// A code operand does not point at an instruction.
    BadCodePointer {
        /// Byte offset of the instruction.
        offset: usize,
        /// The operand that is not a valid code pointer.
        value: u32,
    },
    /// A string is not valid UTF-8.
    InvalidUtf8 {
        /// Parses a script file. The file must start with a known magic and have a valid header.
        /// Byte offset of the string.
        offset: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BadMagic => write!(f, "not a HuneX or Buriko compiled script"),
            Error::Truncated { offset } => write!(f, "file truncated at 0x{offset:x}"),
            Error::BadHeader { offset } => write!(f, "malformed header at 0x{offset:x}"),
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
            Error::BadCodePointer { offset, value } => write!(
                f,
                "code pointer 0x{value:x} at 0x{offset:x} does not land on an instruction start"
            ),
            Error::InvalidUtf8 { offset } => write!(f, "string at 0x{offset:x} is not UTF-8"),
        }
    }
}

impl core::error::Error for Error {}

impl Script {
    /// Parses a script file. The file must start with a known magic and have a valid header.
    pub fn parse(bytes: &[u8]) -> Result<Script, Error> {
        let (format, magic) = if bytes.starts_with(HUNEX_MAGIC) {
            (Format::Hunex, HUNEX_MAGIC)
        } else if bytes.starts_with(BURIKO_MAGIC) {
            (Format::Buriko, BURIKO_MAGIC)
        } else {
            return Err(Error::BadMagic);
        };

        let header_size = read_u32(bytes, magic.len())?;
        if (header_size as usize) < HEADER_FIELDS_SIZE {
            return Err(Error::BadHeader {
                offset: magic.len(),
            });
        }
        let code_base = magic
            .len()
            .checked_add(header_size as usize)
            .filter(|&base| base <= bytes.len())
            .ok_or(Error::Truncated {
                offset: bytes.len(),
            })?;

        let include_count_offset = magic.len() + 4;
        let include_count = read_u32(bytes, include_count_offset)?;
        let mut position = magic.len() + HEADER_FIELDS_SIZE;
        if include_count as usize > code_base - position {
            return Err(Error::BadHeader {
                offset: include_count_offset,
            });
        }
        let mut includes = Vec::with_capacity(include_count as usize);
        for _ in 0..include_count {
            let names = &bytes[position..code_base];
            let length = names
                .iter()
                .position(|&byte| byte == 0)
                .ok_or(Error::BadHeader { offset: position })?;
            includes.push(names[..length].to_vec());
            position += length + 1;
        }
        let header_padding = bytes[position..code_base].to_vec();

        let mut instructions = Vec::new();
        let mut position = code_base;
        let mut smallest_string: Option<(u32, usize)> = None;
        loop {
            let pool = smallest_string.map(|(value, _)| pool_start(code_base, value));
            if pool.is_some_and(|pool| position >= pool) || position >= bytes.len() {
                break;
            }
            let instruction = decode_one(bytes, position, pool)?;
            if is_string_opcode(instruction.opcode) {
                let value = instruction.operands()[0];
                if smallest_string.is_none_or(|(smallest, _)| value < smallest) {
                    smallest_string = Some((value, instruction.offset));
                }
            }
            position += instruction.size();
            instructions.push(instruction);
        }

        let pool = match smallest_string {
            Some((value, offset)) => {
                let pool = pool_start(code_base, value);
                if pool > bytes.len() {
                    return Err(Error::BadStringOperand { offset, value });
                }
                pool
            }
            None => bytes.len(),
        };
        let last_offset = instructions.last().map_or(code_base, |last| last.offset);
        match instructions.last() {
            Some(last) if position == pool && last.opcode == OP_RETURN => {}
            _ => {
                return Err(Error::PoolBoundary {
                    offset: last_offset,
                })
            }
        }

        let pool_bytes = bytes[pool..].to_vec();
        for instruction in &instructions {
            if is_string_opcode(instruction.opcode) {
                let value = instruction.operands()[0];
                if string_range(&pool_bytes, pool, code_base, value).is_none() {
                    return Err(Error::BadStringOperand {
                        offset: instruction.offset,
                        value,
                    });
                }
            }
        }

        Ok(Script {
            format,
            header_size,
            code_base,
            includes,
            header_padding,
            string_pool_start: pool,
            instructions,
            pool: pool_bytes,
        })
    }

    /// Rebuilds the file: magic, header, code instructions, then the string pool.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(self.format.magic());
        push_u32(&mut out, self.header_size);
        push_u32(&mut out, self.includes.len() as u32);
        for name in &self.includes {
            out.extend_from_slice(name);
            out.push(0);
        }
        out.extend_from_slice(&self.header_padding);
        for instruction in &self.instructions {
            push_u32(&mut out, instruction.opcode);
            for &operand in instruction.operands() {
                push_u32(&mut out, operand);
            }
        }
        out.extend_from_slice(&self.pool);
        out
    }

    /// Index of the instruction that starts at `offset`, or `None` when no instruction starts there.
    pub fn format(&self) -> Format {
        self.format
    }

    /// The header size field, in bytes.
    pub fn header_size(&self) -> u32 {
        self.header_size
    }

    /// Byte offset where the code starts.
    pub fn code_base(&self) -> usize {
        self.code_base
    }

    /// The include names, in file order.
    pub fn includes(&self) -> &[Vec<u8>] {
        &self.includes
    }

    /// Header bytes between the end of the include list and the code base (zero in real files).
    pub fn header_padding(&self) -> &[u8] {
        &self.header_padding
    }

    /// Byte offset where the string pool starts.
    pub fn string_pool_start(&self) -> usize {
        self.string_pool_start
    }

    /// The string pool: every byte from the pool start to the end of the file.
    pub fn pool_bytes(&self) -> &[u8] {
        &self.pool
    }

    /// The instructions in file order.
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// Index of the instruction that starts at `offset`, or `None` when no instruction starts there.
    pub fn instruction_index_at(&self, offset: usize) -> Option<usize> {
        self.instructions
            .binary_search_by_key(&offset, |instruction| instruction.offset)
            .ok()
    }

    /// The raw bytes of the string that the instruction points at, without the terminating zero.
    pub fn string_bytes(&self, instruction: &Instruction) -> Result<&[u8], Error> {
        let value = instruction.operands().first().copied().unwrap_or(0);
        let bad = Error::BadStringOperand {
            offset: instruction.offset,
            value,
        };
        if !is_string_opcode(instruction.opcode) {
            return Err(bad);
        }
        let range =
            string_range(&self.pool, self.string_pool_start, self.code_base, value).ok_or(bad)?;
        Ok(&self.pool[range])
    }

    /// The string that the instruction points at, decoded as UTF-8.
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

fn pool_start(code_base: usize, value: u32) -> usize {
    code_base.saturating_add(value as usize)
}

/// Index range, within the pool bytes, of the string a pointer names. The pointer must land
/// inside the pool and be a string start: the pool start (which follows the final return's
/// zero byte) or a position preceded by NUL. The string must end with NUL before the pool does.
fn string_range(
    pool: &[u8],
    pool_start: usize,
    code_base: usize,
    value: u32,
) -> Option<Range<usize>> {
    let start = code_base
        .checked_add(value as usize)?
        .checked_sub(pool_start)?;
    if start >= pool.len() {
        return None;
    }
    if start > 0 && pool[start - 1] != 0 {
        return None;
    }
    let length = pool[start..].iter().position(|&byte| byte == 0)?;
    Some(start..start + length)
}

fn read_u32(bytes: &[u8], position: usize) -> Result<u32, Error> {
    let truncated = Error::Truncated { offset: position };
    let end = position.checked_add(4).ok_or(truncated)?;
    match bytes.get(position..end) {
        Some(&[a, b, c, d]) => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(truncated),
    }
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Decodes the instruction at `position`. When the pool start is known, the instruction must
/// lie entirely before it; its bytes are not read otherwise.
fn decode_one(bytes: &[u8], position: usize, pool: Option<usize>) -> Result<Instruction, Error> {
    let boundary = Error::PoolBoundary { offset: position };
    if pool.is_some_and(|pool| position + 4 > pool) {
        return Err(boundary);
    }
    let opcode = read_u32(bytes, position)?;
    let info = opcode_info(opcode).ok_or(Error::UnregisteredOpcode {
        offset: position,
        opcode,
    })?;
    let instruction_size = 4 * (1 + info.operands);
    if pool.is_some_and(|pool| position + instruction_size > pool) {
        return Err(boundary);
    }
    let mut operands = [0; MAX_OPERANDS];
    for (index, operand) in operands.iter_mut().enumerate().take(info.operands) {
        *operand = read_u32(bytes, position + 4 + 4 * index)?;
    }
    Ok(Instruction {
        offset: position,
        opcode,
        operands,
        operand_count: info.operands as u8,
    })
}
