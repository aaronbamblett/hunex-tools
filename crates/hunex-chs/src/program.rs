//! Symbolic form of a script: typed operands, labels and a string table.
//!
//! [`Script::lift`] turns a parsed script into a [`Program`]. [`Program::lower`] recomputes
//! every code pointer and string pointer and returns a [`Script`] that encodes to bytes.
//!
//! Pool rule: lifted strings keep the file's order, and entries no code references are kept.
//! Lowering writes the pool from the first original string that code references, because the
//! decoder takes the smallest string pointer as the pool start. From there, original strings
//! keep their order. New or changed strings (`add_string`, `set_string`) follow, in table
//! order, and only if code references them. Tail bytes after the last NUL are written last.
//!
//! Known risk: a pointer that is computed at runtime, or stored in a plain 0x00 dword, is not
//! a [`Operand::Label`] or [`Operand::Str`], so lowering does not move it. The 250 files show
//! no such pointer. One relocated script ran in the game on 2026-10-13; that does not cover
//! runtime-computed pointers.

use alloc::vec::Vec;
use core::fmt;

use crate::{
    opcode_info, Error, Format, Instruction, Script, HEADER_FIELDS_SIZE, MAX_OPERANDS, OP_DEBUG,
    OP_PUSH_STRING, OP_RETURN,
};

const OP_CODE_POINTER: u32 = 0x01;
const OP_SKIP_MARKER: u32 = 0x7B;

/// Operand slots that hold a string pointer. The third operand of 0x7B is one, in the files;
/// lifting keeps it as a plain dword when it does not name a string.
fn is_string_slot(opcode: u32, index: usize) -> bool {
    matches!(
        (opcode, index),
        (OP_PUSH_STRING | OP_DEBUG, 0) | (OP_SKIP_MARKER, 2)
    )
}

/// Index of a label in the program's label table.
pub type LabelId = usize;
/// Index of a string in the program's string table.
pub type StrId = usize;

/// One operand of an item. Code and string pointers are symbolic, so lowering can recompute them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// A plain dword that is not a pointer.
    Dword(u32),
    /// A code pointer to the label with this id.
    Label(LabelId),
    /// A string pointer to the string with this id.
    Str(StrId),
}

/// One instruction of a program: an opcode and its operands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    opcode: u32,
    operands: Vec<Operand>,
}

impl Item {
    /// Creates an item from an opcode and its operands.
    pub fn new(opcode: u32, operands: Vec<Operand>) -> Item {
        Item { opcode, operands }
    }

    /// The opcode.
    pub fn opcode(&self) -> u32 {
        self.opcode
    }

    /// The operands in order.
    pub fn operands(&self) -> &[Operand] {
        &self.operands
    }

    /// Checks the opcode, operand count and operand types. The first operand of 0x01 must be a
    /// label. The first operand of 0x03 and 0x7F, and the third operand of 0x7B, may be strings.
    /// Other operands are dwords, and the first operand of 0x03 and 0x7F must be a string.
    pub fn check(&self, labels: usize, strings: usize) -> Result<(), EditError> {
        let info = opcode_info(self.opcode).ok_or(EditError::UnregisteredOpcode(self.opcode))?;
        if self.operands.len() != info.operands {
            return Err(EditError::OperandCount {
                opcode: self.opcode,
                expected: info.operands,
                found: self.operands.len(),
            });
        }
        for (index, operand) in self.operands.iter().enumerate() {
            let expects_label = self.opcode == OP_CODE_POINTER && index == 0;
            let expects_string = is_string_slot(self.opcode, index);
            let string_only = matches!((self.opcode, index), (OP_PUSH_STRING | OP_DEBUG, 0));
            match operand {
                Operand::Label(id) if expects_label => {
                    if *id >= labels {
                        return Err(EditError::UnknownLabel(*id));
                    }
                }
                Operand::Str(id) if expects_string => {
                    if *id >= strings {
                        return Err(EditError::UnknownString(*id));
                    }
                }
                Operand::Dword(_) if !expects_label && !string_only => {}
                _ => {
                    return Err(EditError::OperandType {
                        opcode: self.opcode,
                        index,
                    })
                }
            }
        }
        Ok(())
    }
}

/// One NUL-terminated pool entry. `original` entries come from the file; the rest are new or
/// changed and are written after the original entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolEntry {
    bytes: Vec<u8>,
    original: bool,
}

impl PoolEntry {
    /// Creates a pool entry. Fails when the bytes contain a NUL before the end.
    pub fn new(bytes: Vec<u8>, original: bool) -> Result<PoolEntry, EditError> {
        if bytes.contains(&0) {
            return Err(EditError::NulInString);
        }
        Ok(PoolEntry { bytes, original })
    }

    /// The bytes of the string, without the terminating zero.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// True when the entry came from the file, not from an edit.
    pub fn is_original(&self) -> bool {
        self.original
    }
}

/// Everything a [`Program`] holds, for building one directly (the assembler does this).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramParts {
    /// The format the file was read as.
    pub format: Format,
    /// The header size field, in bytes.
    pub header_size: u32,
    /// The include names, in file order.
    pub includes: Vec<Vec<u8>>,
    /// Header bytes between the include list and the code, kept as read.
    pub header_padding: Vec<u8>,
    /// The items in file order.
    pub items: Vec<Item>,
    /// Target item index of each label. A target equal to `items.len()` is the end of code.
    pub labels: Vec<usize>,
    /// The string pool entries in table order.
    pub strings: Vec<PoolEntry>,
    /// Bytes after the last string, kept as read.
    pub tail: Vec<u8>,
}

/// A script in symbolic form. Labels and string references are ids, so editing one item does not break pointers elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    format: Format,
    header_size: u32,
    includes: Vec<Vec<u8>>,
    header_padding: Vec<u8>,
    items: Vec<Item>,
    labels: Vec<usize>,
    strings: Vec<PoolEntry>,
    tail: Vec<u8>,
}

/// Errors from building or editing a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    /// The opcode is not in the opcode table.
    UnregisteredOpcode(u32),
    /// An operand count that does not match the opcode.
    OperandCount {
        /// The opcode whose operands are wrong.
        opcode: u32,
        /// The operand count the opcode takes.
        expected: usize,
        /// The operand count that was given.
        found: usize,
    },
    /// An operand has the wrong kind for its slot.
    OperandType {
        /// The opcode whose operand is wrong.
        opcode: u32,
        /// Index of the operand.
        index: usize,
    },
    /// A label operand names a label that does not exist.
    UnknownLabel(LabelId),
    /// A string operand names a string that does not exist.
    UnknownString(StrId),
    /// An index is past the end of its table.
    IndexOutOfRange(usize),
    /// The instruction has a label on it. Move the label with `move_label` first.
    LabelOnRemovedInstruction {
        /// Index of the item that refers to the label.
        index: usize,
        /// The label that the item refers to.
        label: LabelId,
    },
    /// A string contains a NUL byte before its end.
    NulInString,
    /// The header is not one that the lowering can write.
    BadHeader,
}

/// Errors from lowering a program back to a script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LowerError {
    /// The code must end with a return, because the decoder finds the pool after it.
    MissingFinalReturn,
    /// A tail without any string would be read as code.
    TailWithoutStrings,
    /// An original string is referenced by an operand other than a 0x03 or 0x7F pointer, and
    /// it lies before the pool start that those pointers define.
    PointerBeforePool,
    /// The first pool entry is not referenced by a 0x03 or 0x7F pointer, so the decoder would
    /// not find the pool start.
    NoStringAnchor,
    /// A label or string reference points at nothing.
    DanglingReference,
    /// A pointer does not fit the file's 32-bit fields.
    OutOfRange,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EditError::UnregisteredOpcode(opcode) => write!(f, "unregistered opcode 0x{opcode:x}"),
            EditError::OperandCount {
                opcode,
                expected,
                found,
            } => write!(
                f,
                "opcode 0x{opcode:x} takes {expected} operands, found {found}"
            ),
            EditError::OperandType { opcode, index } => write!(
                f,
                "operand {index} of opcode 0x{opcode:x} has the wrong kind"
            ),
            EditError::UnknownLabel(id) => write!(f, "unknown label {id}"),
            EditError::UnknownString(id) => write!(f, "unknown string {id}"),
            EditError::IndexOutOfRange(index) => write!(f, "index {index} is out of range"),
            EditError::LabelOnRemovedInstruction { index, label } => write!(
                f,
                "label {label} is on instruction {index}; move it before removing"
            ),
            EditError::NulInString => write!(f, "a string may not contain NUL"),
            EditError::BadHeader => write!(f, "header size does not match the header fields"),
        }
    }
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::MissingFinalReturn => write!(f, "the code does not end with a return"),
            LowerError::TailWithoutStrings => {
                write!(f, "pool tail bytes with no string to anchor them")
            }
            LowerError::PointerBeforePool => write!(
                f,
                "a string is referenced before the pool start that 0x03 and 0x7F pointers define"
            ),
            LowerError::NoStringAnchor => write!(
                f,
                "the first pool entry is not referenced by a 0x03 or 0x7F pointer"
            ),
            LowerError::DanglingReference => write!(f, "a label or string reference is dangling"),
            LowerError::OutOfRange => write!(f, "a pointer does not fit in a dword"),
        }
    }
}

impl core::error::Error for EditError {}
impl core::error::Error for LowerError {}

impl Program {
    /// Builds a program from its parts. Checks that the labels and string references are valid.
    pub fn from_parts(parts: ProgramParts) -> Result<Program, EditError> {
        let ProgramParts {
            format,
            header_size,
            includes,
            header_padding,
            items,
            labels,
            strings,
            tail,
        } = parts;
        let mut names = 0usize;
        for name in &includes {
            if name.contains(&0) {
                return Err(EditError::NulInString);
            }
            names += name.len() + 1;
        }
        if header_size as usize != HEADER_FIELDS_SIZE + names + header_padding.len() {
            return Err(EditError::BadHeader);
        }
        if tail.contains(&0) {
            return Err(EditError::NulInString);
        }
        for &target in &labels {
            if target > items.len() {
                return Err(EditError::IndexOutOfRange(target));
            }
        }
        for item in &items {
            item.check(labels.len(), strings.len())?;
        }
        Ok(Program {
            format,
            header_size,
            includes,
            header_padding,
            items,
            labels,
            strings,
            tail,
        })
    }

    /// The format the file was read as.
    pub fn format(&self) -> Format {
        self.format
    }

    /// The header size field, in bytes.
    pub fn header_size(&self) -> u32 {
        self.header_size
    }

    /// The include names, in file order.
    pub fn includes(&self) -> &[Vec<u8>] {
        &self.includes
    }

    /// Header bytes between the include list and the code, kept as read.
    pub fn header_padding(&self) -> &[u8] {
        &self.header_padding
    }

    /// The items in file order.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Target item index of each label, indexed by [`LabelId`].
    pub fn labels(&self) -> &[usize] {
        &self.labels
    }

    /// The string pool entries in table order.
    pub fn strings(&self) -> &[PoolEntry] {
        &self.strings
    }

    /// The bytes of the string with this id, or `None` when the id is out of range.
    pub fn string(&self, id: StrId) -> Option<&[u8]> {
        self.strings.get(id).map(PoolEntry::bytes)
    }

    /// Bytes after the last string, kept as read.
    pub fn tail(&self) -> &[u8] {
        &self.tail
    }

    /// Inserts `item` before the item at `index` (`index == items().len()` appends). Labels on
    /// the item at `index` stay on that item, which moves to `index + 1`.
    pub fn insert(&mut self, index: usize, item: Item) -> Result<(), EditError> {
        if index > self.items.len() {
            return Err(EditError::IndexOutOfRange(index));
        }
        item.check(self.labels.len(), self.strings.len())?;
        self.items.insert(index, item);
        for target in &mut self.labels {
            if *target >= index {
                *target += 1;
            }
        }
        Ok(())
    }

    /// Removes the item at `index`. It fails if a label is on that item; move the label first.
    pub fn remove(&mut self, index: usize) -> Result<Item, EditError> {
        if index >= self.items.len() {
            return Err(EditError::IndexOutOfRange(index));
        }
        if let Some(label) = self.labels.iter().position(|&target| target == index) {
            return Err(EditError::LabelOnRemovedInstruction { index, label });
        }
        for target in &mut self.labels {
            if *target > index {
                *target -= 1;
            }
        }
        Ok(self.items.remove(index))
    }

    /// Adds a label on the item at `index`; `index == items().len()` labels the end of code.
    pub fn add_label(&mut self, index: usize) -> Result<LabelId, EditError> {
        if index > self.items.len() {
            return Err(EditError::IndexOutOfRange(index));
        }
        self.labels.push(index);
        Ok(self.labels.len() - 1)
    }

    /// Moves a label to a different item index. Fails when the label or index does not exist.
    pub fn move_label(&mut self, label: LabelId, index: usize) -> Result<(), EditError> {
        if label >= self.labels.len() {
            return Err(EditError::UnknownLabel(label));
        }
        if index > self.items.len() {
            return Err(EditError::IndexOutOfRange(index));
        }
        self.labels[label] = index;
        Ok(())
    }

    /// Adds a new string and returns its id. It is written after the original strings.
    pub fn add_string(&mut self, bytes: &[u8]) -> Result<StrId, EditError> {
        self.strings.push(PoolEntry::new(bytes.to_vec(), false)?);
        Ok(self.strings.len() - 1)
    }

    /// Replaces a string. The new bytes are written after the original strings.
    pub fn set_string(&mut self, id: StrId, bytes: &[u8]) -> Result<(), EditError> {
        if id >= self.strings.len() {
            return Err(EditError::UnknownString(id));
        }
        self.strings[id] = PoolEntry::new(bytes.to_vec(), false)?;
        Ok(())
    }

    /// Recomputes every code pointer and string pointer and returns the script.
    pub fn lower(&self) -> Result<Script, LowerError> {
        match self.items.last() {
            Some(last) if last.opcode == OP_RETURN => {}
            _ => return Err(LowerError::MissingFinalReturn),
        }
        let code_base = self.format.magic().len() + self.header_size as usize;

        let mut offsets = Vec::with_capacity(self.items.len() + 1);
        let mut cursor = code_base;
        for item in &self.items {
            offsets.push(cursor);
            cursor += 4 * (1 + item.operands.len());
        }
        let code_end = cursor;
        offsets.push(code_end);

        // `anchored`: referenced by a 0x03 or 0x7F pointer, which is what the decoder uses to
        // find the pool start. `used`: referenced by any string operand.
        let mut used = alloc::vec![false; self.strings.len()];
        let mut anchored = alloc::vec![false; self.strings.len()];
        for item in &self.items {
            for (slot, operand) in item.operands.iter().enumerate() {
                if let Operand::Str(id) = operand {
                    *used.get_mut(*id).ok_or(LowerError::DanglingReference)? = true;
                    if matches!((item.opcode, slot), (OP_PUSH_STRING | OP_DEBUG, 0)) {
                        anchored[*id] = true;
                    }
                }
            }
        }
        let anchor = self
            .strings
            .iter()
            .enumerate()
            .find(|(index, entry)| entry.original && anchored[*index])
            .map(|(index, _)| index);
        let referenced_before_anchor = match anchor {
            Some(anchor) => (0..anchor).any(|index| self.strings[index].original && used[index]),
            None => self
                .strings
                .iter()
                .enumerate()
                .any(|(index, entry)| entry.original && used[index]),
        };
        if referenced_before_anchor {
            return Err(LowerError::PointerBeforePool);
        }
        let mut emitted = Vec::new();
        if let Some(first) = anchor {
            for (index, entry) in self.strings.iter().enumerate().skip(first) {
                if entry.original {
                    emitted.push(index);
                }
            }
        }
        for (index, entry) in self.strings.iter().enumerate() {
            if !entry.original && used[index] {
                emitted.push(index);
            }
        }
        if let Some(&first) = emitted.first() {
            if !anchored[first] {
                return Err(LowerError::NoStringAnchor);
            }
        }

        let mut pool = Vec::new();
        let mut addresses = alloc::vec![None; self.strings.len()];
        for index in emitted {
            addresses[index] = Some(code_end + pool.len());
            pool.extend_from_slice(self.strings[index].bytes());
            pool.push(0);
        }
        if pool.is_empty() && !self.tail.is_empty() {
            return Err(LowerError::TailWithoutStrings);
        }
        pool.extend_from_slice(&self.tail);

        let mut instructions = Vec::with_capacity(self.items.len());
        for (index, item) in self.items.iter().enumerate() {
            let mut operands = [0u32; MAX_OPERANDS];
            for (slot, operand) in item.operands.iter().enumerate() {
                let address = match *operand {
                    Operand::Dword(value) => {
                        operands[slot] = value;
                        continue;
                    }
                    Operand::Label(id) => {
                        let target = *self.labels.get(id).ok_or(LowerError::DanglingReference)?;
                        *offsets.get(target).ok_or(LowerError::DanglingReference)?
                    }
                    Operand::Str(id) => addresses
                        .get(id)
                        .copied()
                        .flatten()
                        .ok_or(LowerError::DanglingReference)?,
                };
                let pointer = address - code_base;
                operands[slot] = u32::try_from(pointer).map_err(|_| LowerError::OutOfRange)?;
            }
            instructions.push(Instruction {
                offset: offsets[index],
                opcode: item.opcode,
                operands,
                operand_count: item.operands.len() as u8,
            });
        }

        Ok(Script {
            format: self.format,
            header_size: self.header_size,
            code_base,
            includes: self.includes.clone(),
            header_padding: self.header_padding.clone(),
            string_pool_start: code_end,
            instructions,
            pool,
        })
    }
}

impl Script {
    /// Converts a parsed script into a program, with labels and string ids in place of raw pointers.
    pub fn lift(&self) -> Result<Program, Error> {
        let code_base = self.code_base();
        let pool_start = self.string_pool_start();
        let pool = self.pool_bytes();

        let mut strings = Vec::new();
        let mut entry_starts = Vec::new();
        let mut start = 0;
        for (position, &byte) in pool.iter().enumerate() {
            if byte == 0 {
                entry_starts.push(start);
                strings.push(PoolEntry {
                    bytes: pool[start..position].to_vec(),
                    original: true,
                });
                start = position + 1;
            }
        }
        let tail = pool[start..].to_vec();

        let mut label_targets = Vec::new();
        for instruction in self.instructions() {
            if instruction.opcode() == OP_CODE_POINTER {
                label_targets.push(code_target(self, instruction)?);
            }
        }
        label_targets.sort_unstable();
        label_targets.dedup();

        let resolve = |value: u32| -> Option<StrId> {
            code_base
                .checked_add(value as usize)
                .and_then(|address| address.checked_sub(pool_start))
                .and_then(|relative| entry_starts.binary_search(&relative).ok())
        };
        let mut items = Vec::with_capacity(self.instructions().len());
        for instruction in self.instructions() {
            let mut operands = Vec::with_capacity(instruction.operands().len());
            for (index, &value) in instruction.operands().iter().enumerate() {
                let operand = match (instruction.opcode(), index) {
                    (OP_CODE_POINTER, 0) => {
                        let target = code_target(self, instruction)?;
                        let id = label_targets.binary_search(&target).map_err(|_| {
                            Error::BadCodePointer {
                                offset: instruction.offset(),
                                value,
                            }
                        })?;
                        Operand::Label(id)
                    }
                    (OP_PUSH_STRING, 0) | (OP_DEBUG, 0) => {
                        let bad = Error::BadStringOperand {
                            offset: instruction.offset(),
                            value,
                        };
                        Operand::Str(resolve(value).ok_or(bad)?)
                    }
                    (OP_SKIP_MARKER, 2) => {
                        resolve(value).map_or(Operand::Dword(value), Operand::Str)
                    }
                    _ => Operand::Dword(value),
                };
                operands.push(operand);
            }
            items.push(Item {
                opcode: instruction.opcode(),
                operands,
            });
        }

        Ok(Program {
            format: self.format(),
            header_size: self.header_size(),
            includes: self.includes().to_vec(),
            header_padding: self.header_padding().to_vec(),
            items,
            labels: label_targets,
            strings,
            tail,
        })
    }
}

fn code_target(script: &Script, instruction: &Instruction) -> Result<usize, Error> {
    let value = instruction.operands().first().copied().unwrap_or(0);
    let bad = Error::BadCodePointer {
        offset: instruction.offset(),
        value,
    };
    let address = script.code_base().checked_add(value as usize).ok_or(bad)?;
    script.instruction_index_at(address).ok_or(bad)
}
