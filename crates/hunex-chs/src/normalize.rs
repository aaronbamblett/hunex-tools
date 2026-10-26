//! Normalized instruction form used to compare compiled output against real files.
//!
//! [`normalize`] turns a [`Program`] into plain instructions: every
//! Debug (`0x7F`) instruction is dropped, `PushString` (`0x03`) operands become the
//! string value, `0x7B` keeps its two numeric operands while its third pool operand
//! becomes the string value, and `PushOffset` (`0x01`) label targets become the index
//! of the target instruction after Debug removal. Everything else is preserved exactly.
//!
//! ```
//! use hunex_chs::{Item, Operand, Program, ProgramParts, Format};
//! use hunex_chs::normalize::{normalize, NormOperand};
//!
//! let program = Program::from_parts(ProgramParts {
//!     format: Format::Hunex,
//!     header_size: 8,
//!     includes: Vec::new(),
//!     header_padding: Vec::new(),
//!     items: vec![Item::new(0x03, vec![Operand::Str(0)]), Item::new(0x1B, vec![])],
//!     labels: Vec::new(),
//!     strings: vec![hunex_chs::PoolEntry::new(b"hi".to_vec(), true).unwrap()],
//!     tail: Vec::new(),
//! })
//! .unwrap();
//! let list = normalize(&program).unwrap();
//! assert_eq!(list[0].operands(), &[NormOperand::Text(b"hi".to_vec())]);
//! ```

use alloc::vec::Vec;
use core::fmt;

use crate::{LabelId, Operand, Program, StrId, OP_DEBUG};

/// One operand of a normalized instruction: a preserved number, a resolved string,
/// or a resolved jump target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormOperand {
    /// A numeric operand, kept exactly as written.
    Num(u32),
    /// The exact bytes of the pool string the operand pointed at.
    Text(Vec<u8>),
    /// The kept-instruction index a `PushOffset` (`0x01`) target resolves to. It is
    /// deliberately distinct from [`NormOperand::Num`] so a target never compares
    /// equal to a literal number.
    Target(usize),
}

/// One normalized instruction: an opcode and its operands in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormInstruction {
    opcode: u32,
    operands: Vec<NormOperand>,
}

impl NormInstruction {
    /// Creates a normalized instruction from an opcode and its operands.
    pub fn new(opcode: u32, operands: Vec<NormOperand>) -> NormInstruction {
        NormInstruction { opcode, operands }
    }

    /// The original opcode id, preserved through normalization.
    pub fn opcode(&self) -> u32 {
        self.opcode
    }

    /// The operands in order.
    pub fn operands(&self) -> &[NormOperand] {
        &self.operands
    }
}

/// Errors from normalizing a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizeError {
    /// A label operand names a label that does not exist, or names a target past the
    /// end of the code. Valid programs never hold one.
    UnknownLabel(LabelId),
    /// A string operand names a string that does not exist. Valid programs never hold one.
    UnknownString(StrId),
    /// A normalized target index does not fit a dword. Real programs never reach one.
    TargetOutOfRange,
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            NormalizeError::UnknownLabel(id) => write!(f, "unknown label {id}"),
            NormalizeError::UnknownString(id) => write!(f, "unknown string {id}"),
            NormalizeError::TargetOutOfRange => write!(f, "target index does not fit a dword"),
        }
    }
}

impl core::error::Error for NormalizeError {}

/// Normalizes a program: drops every Debug instruction, resolves `PushString` and
/// `0x7B` pool operands to their string bytes, and rewrites `PushOffset` targets to
/// kept-instruction indices. A target on a dropped Debug maps to the next kept
/// instruction, and a target on the end of the code maps to the list length. Any
/// other dword operand is preserved exactly, including Return (`0x1B`).
pub fn normalize(program: &Program) -> Result<Vec<NormInstruction>, NormalizeError> {
    let items = program.items();
    let mut kept_before = Vec::with_capacity(items.len() + 1);
    let mut kept = 0usize;
    for item in items {
        kept_before.push(kept);
        if item.opcode() != OP_DEBUG {
            kept += 1;
        }
    }
    kept_before.push(kept);

    let mut out = Vec::new();
    for item in items {
        if item.opcode() == OP_DEBUG {
            continue;
        }
        let mut operands = Vec::with_capacity(item.operands().len());
        for operand in item.operands() {
            operands.push(match *operand {
                Operand::Dword(value) => NormOperand::Num(value),
                Operand::Str(id) => NormOperand::Text(
                    program
                        .string(id)
                        .ok_or(NormalizeError::UnknownString(id))?
                        .to_vec(),
                ),
                Operand::Label(id) => {
                    let target = *program
                        .labels()
                        .get(id)
                        .ok_or(NormalizeError::UnknownLabel(id))?;
                    let index = kept_before
                        .get(target)
                        .copied()
                        .ok_or(NormalizeError::UnknownLabel(id))?;
                    if u32::try_from(index).is_err() {
                        return Err(NormalizeError::TargetOutOfRange);
                    }
                    NormOperand::Target(index)
                }
            });
        }
        out.push(NormInstruction {
            opcode: item.opcode(),
            operands,
        });
    }
    Ok(out)
}
