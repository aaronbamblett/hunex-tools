//! Lower parsed Switch statements to a [`Program`] using the engine recipe table.
//!
//! [`compile`] reads [`Statement`]s and the caller-supplied scene number map and
//! emits prologue dispatch blocks, labels, conditionals, jumps, messages, typed
//! encoders and the epilogue return, tracking each numeric push made from a source
//! argument so shape comparison can relax only those. It emits no Debug
//! instructions, interns strings in first-use order, writes the Steam header
//! layout (size 21, no includes, thirteen zero padding bytes), and fails with the
//! offending statement offset on unknown commands, unmatched predicates, missing
//! scene numbers, unknown comparison operators, unresolved labels and unproven shapes.
//! Zero-emission recipes emit no instructions but still apply their state and label
//! effects. Selecting one whose predicate is not
//! proven on the corpus records a [`CompileWarning`] without changing emission.
//!
//! The scene number to scene name map is an input, derived per the spec's scene
//! number rule (for example from a names list such as `nam.bin` entries): each
//! `_JUMP` number must resolve through it, and a missing number is an error.
//!
//! ```
//! let statements = hunex_chs::source::parse("_END_();").unwrap();
//! let compiled = hunex_chs::compile::compile(&statements, &[]).unwrap();
//! let list = hunex_chs::normalize::normalize(compiled.program()).unwrap();
//! assert_eq!(list.last().unwrap().opcode(), 0x1B);
//! ```

use alloc::string::String;
use alloc::vec::Vec;

use crate::recipes::{self, Encoder, Special, StateKey};
use crate::source::{Statement, Value};
use crate::{Format, Item, LabelId, Operand, PoolEntry, Program, ProgramParts, StrId};

/// A numeric push made from a source argument. Shape comparison may relax only these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralSlot {
    /// The emitted instruction index.
    pub index: usize,
    /// The zero-based source field the number came from.
    pub field: usize,
    /// The encoder that made it.
    pub encoder: SlotEncoder,
    /// The raw source spelling it was made from.
    pub raw: String,
}

/// The encoders that can make a tracked numeric push.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotEncoder {
    /// The ordinary source-value encoder on an int or expression.
    Value,
    /// A backtick tag payload parsed as a numeric expression.
    TagPart(u8),
}

/// The reason every [`CompileWarning`] carries: the selected zero-emission
/// predicate is not proven on the corpus.
pub const ZERO_EMISSION_UNPROVEN: &str = "zero-emission predicate not proven on the corpus";

/// One selected zero-emission predicate that is not proven on the corpus. The
/// compiler emits nothing for it, exactly as for proven predicates, but keeps
/// the selection visible here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileWarning {
    offset: usize,
    command: String,
    args: Vec<String>,
    reason: &'static str,
}

impl CompileWarning {
    /// The source character offset of the statement that selected the predicate.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The canonical command name of that statement.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The raw argument spellings of that statement, in source order.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Why the selection warns: [`ZERO_EMISSION_UNPROVEN`].
    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

/// The lowered scene: the program plus the per-instruction and per-statement facts
/// the comparison needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiled {
    program: Program,
    owners: Vec<i64>,
    spans: Vec<(usize, usize)>,
    literal_slots: Vec<LiteralSlot>,
    selected: Vec<Option<&'static str>>,
    warnings: Vec<CompileWarning>,
}

impl Compiled {
    /// The lowered program, with labels and a first-use-order string pool.
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// The owning statement of each emitted instruction: the statement index, `-1`
    /// for the prologue and `-2` for the epilogue return.
    pub fn owners(&self) -> &[i64] {
        &self.owners
    }

    /// The half-open instruction span of each source statement, in source order.
    pub fn spans(&self) -> &[(usize, usize)] {
        &self.spans
    }

    /// Every numeric push made from a source argument, by instruction index.
    pub fn literal_slots(&self) -> &[LiteralSlot] {
        &self.literal_slots
    }

    /// The selected recipe id of each source statement, in source order.
    pub fn selected(&self) -> &[Option<&'static str>] {
        &self.selected
    }

    /// One entry per selected zero-emission predicate that is not proven on the
    /// corpus, in source order. Emitting nothing for them is unchanged.
    pub fn warnings(&self) -> &[CompileWarning] {
        &self.warnings
    }
}

/// Why a scene failed to lower. Every variant carries the statement index and its
/// source character offset in [`CompileError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileErrorKind {
    /// No recipe table holds the command.
    UnknownCommand,
    /// No recipe predicate matched the statement's arguments and state.
    NoMatchingRecipe,
    /// A `_JUMP` number with no entry in the caller-supplied scene name map.
    UnknownSceneNumber,
    /// An `_IF__` operator outside the comparison-op map.
    UnknownComparisonOp,
    /// A label reference no statement defines. It reports the first statement that
    /// refers to the missing label.
    UnresolvedLabel,
    /// An arithmetic shape outside `+`, `-` and parentheses (`*`, `/`, `%`).
    UnsupportedExpression,
    /// A `tag_part:N` payload that is missing or not numeric.
    BadTagPart,
    /// An argument the selected encoder cannot encode, such as a tag with `value`.
    BadEncoder,
    /// A body argument past the provided argument count.
    MissingArgument,
    /// A state assignment argument that is not a decimal integer.
    BadStateArg,
    /// A number above `u32::MAX` or below `-(u32::MAX)` that no dword push can hold.
    NumberOutOfRange,
    /// A label definition for a name that is already defined.
    DuplicateLabel,
    /// The lowered parts failed program validation. Builders never produce one.
    InvalidProgram,
}

/// A lowering failure: what failed, at which statement and source character offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    kind: CompileErrorKind,
    statement: usize,
    offset: usize,
}

impl CompileError {
    /// What failed.
    pub fn kind(&self) -> &CompileErrorKind {
        &self.kind
    }

    /// The index of the offending source statement.
    pub fn statement(&self) -> usize {
        self.statement
    }

    /// The source character offset of the offending statement.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let what = match &self.kind {
            CompileErrorKind::UnknownCommand => "unknown command",
            CompileErrorKind::NoMatchingRecipe => "no recipe predicate matched",
            CompileErrorKind::UnknownSceneNumber => "unknown scene number",
            CompileErrorKind::UnknownComparisonOp => "unknown comparison operator",
            CompileErrorKind::UnresolvedLabel => "unresolved label",
            CompileErrorKind::UnsupportedExpression => "unsupported expression",
            CompileErrorKind::BadTagPart => "bad tag payload",
            CompileErrorKind::BadEncoder => "argument the encoder cannot encode",
            CompileErrorKind::MissingArgument => "body argument past the argument count",
            CompileErrorKind::BadStateArg => "state argument is not a decimal integer",
            CompileErrorKind::NumberOutOfRange => "number out of range for a dword push",
            CompileErrorKind::DuplicateLabel => "duplicate label definition",
            CompileErrorKind::InvalidProgram => "lowered program failed validation",
        };
        write!(f, "{what} at statement {}", self.statement)
    }
}

impl core::error::Error for CompileError {}

const OP_PUSH: u32 = 0x00;
const OP_TARGET: u32 = 0x01;
const OP_STRING: u32 = 0x03;
const OP_JUMP: u32 = 0x18;
const OP_JUMP_IF: u32 = 0x19;
const OP_ADD: u32 = 0x20;
const OP_SUB: u32 = 0x21;
const OP_CMP_EQ: u32 = 0x30;
const OP_FLAG_READ: u32 = 0xEA;
const OP_MESSAGE_META: u32 = 0x7B;
const META_MAGIC: u32 = 0x1234_5678;
const OP_DISPATCH_FLAG: u32 = 0xF7;
const OP_ERROR: u32 = 0xF9;
const OP_RETURN: u32 = 0x1B;
const OP_GOTO_TABLE: u32 = 0x162;
const OP_SKIP_CALL: u32 = 0x122;
const OP_JUMP_SETUP: u32 = 0xF6;
const OP_JUMP_CALL: u32 = 0xF3;
const OP_MESSAGE_CALL: u32 = 0x140;
const PLATFORM_CALL_DEFAULT: u32 = 0x131;
const SKIP_SENTINEL: u32 = 0xFFFF_FFFF;

/// The Steam header layout the compiler writes: no includes and the thirteen
/// zero padding bytes the sample files carry, so the code starts at 0x30.
const HEADER_SIZE: u32 = 21;
const HEADER_PADDING: [u8; 13] = [0; 13];

struct Builder<'a> {
    statements: &'a [Statement],
    scene_names: &'a [(&'a str, &'a str)],
    items: Vec<Item>,
    strings: Vec<Vec<u8>>,
    label_names: Vec<Option<String>>,
    label_targets: Vec<Option<usize>>,
    pending_refs: Vec<(usize, usize)>,
    owners: Vec<i64>,
    spans: Vec<(usize, usize)>,
    slots: Vec<LiteralSlot>,
    selected: Vec<Option<&'static str>>,
    warnings: Vec<CompileWarning>,
    wnty: i64,
    f822: i64,
    f823: i64,
    message_index: u32,
}

impl<'a> Builder<'a> {
    fn error(&self, statement: usize, kind: CompileErrorKind) -> CompileError {
        let offset = self.statements.get(statement).map_or(0, Statement::offset);
        CompileError {
            kind,
            statement,
            offset,
        }
    }

    fn state(&self, key: StateKey) -> i64 {
        match key {
            StateKey::Wnty => self.wnty,
            StateKey::F822 => self.f822,
            StateKey::F823 => self.f823,
        }
    }

    fn set_state_key(&mut self, key: StateKey, value: i64) {
        match key {
            StateKey::Wnty => self.wnty = value,
            StateKey::F822 => self.f822 = value,
            StateKey::F823 => self.f823 = value,
        }
    }

    fn intern(&mut self, bytes: &[u8]) -> StrId {
        if let Some(id) = self.strings.iter().position(|entry| entry == bytes) {
            return id;
        }
        self.strings.push(bytes.to_vec());
        self.strings.len() - 1
    }

    fn label_id(&mut self, name: &str) -> LabelId {
        if let Some(id) = self
            .label_names
            .iter()
            .position(|known| known.as_deref() == Some(name))
        {
            return id;
        }
        self.label_names.push(Some(String::from(name)));
        self.label_targets.push(None);
        self.label_names.len() - 1
    }

    /// A label outside the script namespace, for the prologue entry point. It
    /// has no name, so no script spelling can define or reference it.
    fn fresh_label(&mut self) -> LabelId {
        self.label_names.push(None);
        self.label_targets.push(None);
        self.label_names.len() - 1
    }

    fn define_id(&mut self, id: LabelId, statement: usize) -> Result<(), CompileError> {
        if self.label_targets[id].is_some() {
            return Err(self.error(statement, CompileErrorKind::DuplicateLabel));
        }
        self.label_targets[id] = Some(self.items.len());
        Ok(())
    }

    fn define(&mut self, name: &str, statement: usize) -> Result<(), CompileError> {
        let id = self.label_id(name);
        self.define_id(id, statement)
    }

    fn push(&mut self, item: Item, owner: i64) {
        self.items.push(item);
        self.owners.push(owner);
    }

    fn push_dword(&mut self, value: u32, owner: i64) {
        self.push(
            Item::new(OP_PUSH, Vec::from([Operand::Dword(value)])),
            owner,
        );
    }

    fn push_tracked(
        &mut self,
        value: u32,
        owner: i64,
        field: usize,
        encoder: SlotEncoder,
        raw: &str,
    ) {
        let index = self.items.len();
        self.push_dword(value, owner);
        self.slots.push(LiteralSlot {
            index,
            field,
            encoder,
            raw: String::from(raw),
        });
    }

    fn push_str(&mut self, bytes: &[u8], owner: i64) {
        let id = self.intern(bytes);
        self.push(Item::new(OP_STRING, Vec::from([Operand::Str(id)])), owner);
    }

    fn push_target(&mut self, name: &str, owner: i64, statement: usize) {
        let id = self.label_id(name);
        self.push_target_id(id, owner, statement);
    }

    fn push_target_id(&mut self, id: LabelId, owner: i64, statement: usize) {
        self.pending_refs.push((id, statement));
        self.push(Item::new(OP_TARGET, Vec::from([Operand::Label(id)])), owner);
    }

    fn push_call(&mut self, opcode: u32, owner: i64) {
        self.push(Item::new(opcode, Vec::new()), owner);
    }

    fn push_lit(&mut self, opcode: u32, operands: &[crate::recipes::LitOp], owner: i64) {
        let mut out = Vec::with_capacity(operands.len());
        for operand in operands {
            out.push(match operand {
                crate::recipes::LitOp::Num(value) => Operand::Dword(*value),
                crate::recipes::LitOp::Text(text) => Operand::Str(self.intern(text.as_bytes())),
            });
        }
        self.push(Item::new(opcode, out), owner);
    }

    /// Pushes one integer as dwords. Values outside `-(u32::MAX)..=u32::MAX`
    /// are an error: no dword push can hold them, and truncating with `as`
    /// would silently emit the wrong number.
    fn push_int(
        &mut self,
        value: i64,
        owner: i64,
        statement: usize,
        field: usize,
        encoder: SlotEncoder,
        raw: &str,
    ) -> Result<(), CompileError> {
        if value > u32::MAX as i64 || value < -(u32::MAX as i64) {
            return Err(self.error(statement, CompileErrorKind::NumberOutOfRange));
        }
        if value < 0 {
            let magnitude = value.wrapping_neg() as u32;
            self.push_tracked(0, owner, field, encoder, raw);
            self.push_tracked(magnitude, owner, field, encoder, raw);
            self.push_call(OP_SUB, owner);
        } else {
            self.push_tracked(value as u32, owner, field, encoder, raw);
        }
        Ok(())
    }

    /// Pushes an expression in postfix order with an explicit work stack, so
    /// lowering never recurses per term no matter how long the chain is.
    fn push_expr(
        &mut self,
        expr: &crate::source::Expr,
        owner: i64,
        statement: usize,
        field: usize,
        encoder: SlotEncoder,
        raw: &str,
    ) -> Result<(), CompileError> {
        enum Task<'expr> {
            Push(&'expr crate::source::Expr),
            Zero,
            Op(u32),
        }
        let mut stack = Vec::from([Task::Push(expr)]);
        while let Some(task) = stack.pop() {
            match task {
                Task::Push(crate::source::Expr::Lit(value)) => {
                    self.push_int(*value, owner, statement, field, encoder, raw)?;
                }
                Task::Push(crate::source::Expr::Neg(inner)) => {
                    stack.push(Task::Op(OP_SUB));
                    stack.push(Task::Push(inner));
                    stack.push(Task::Zero);
                }
                Task::Push(crate::source::Expr::Add(left, right)) => {
                    stack.push(Task::Op(OP_ADD));
                    stack.push(Task::Push(right));
                    stack.push(Task::Push(left));
                }
                Task::Push(crate::source::Expr::Sub(left, right)) => {
                    stack.push(Task::Op(OP_SUB));
                    stack.push(Task::Push(right));
                    stack.push(Task::Push(left));
                }
                Task::Zero => self.push_tracked(0, owner, field, encoder, raw),
                Task::Op(opcode) => self.push_call(opcode, owner),
            }
        }
        Ok(())
    }

    /// Compiles a raw numeric string from a tag payload: a decimal or
    /// hexadecimal integer, or `+`/`-`/parentheses arithmetic. Anything else
    /// (including `*`, `/` and `%`) is an error, never a guess.
    fn push_numeric_str(
        &mut self,
        raw: &str,
        owner: i64,
        statement: usize,
        field: usize,
        encoder: SlotEncoder,
        slot_raw: &str,
    ) -> Result<(), CompileError> {
        let trimmed = raw.trim();
        if let Some(value) = parse_int_str(trimmed) {
            self.push_int(value, owner, statement, field, encoder, slot_raw)?;
            return Ok(());
        }
        let chars: Vec<char> = trimmed.chars().collect();
        let mut reader = NumReader {
            chars: &chars,
            pos: 0,
            depth: 0,
        };
        match reader.sum() {
            Ok(value) if reader.pos == chars.len() => {
                self.push_int(value, owner, statement, field, encoder, slot_raw)?;
                Ok(())
            }
            _ => Err(self.error(statement, CompileErrorKind::UnsupportedExpression)),
        }
    }

    /// Encodes one supplied argument with the named encoder, tracking numeric
    /// pushes from `value` and `tag_part` so shape comparison can relax them.
    fn encode_arg(
        &mut self,
        value: &Value,
        encoder: Encoder,
        owner: i64,
        statement: usize,
        field: usize,
    ) -> Result<(), CompileError> {
        let bad = self.error(statement, CompileErrorKind::BadEncoder);
        match encoder {
            Encoder::Str => self.push_str(value.raw().as_bytes(), owner),
            Encoder::FlagIndex => match value {
                Value::Flag { index, .. } => self.push_dword(*index, owner),
                _ => return Err(bad),
            },
            Encoder::FlagBank => match value {
                Value::Flag { bank, .. } => self.push_dword(u32::from(*bank), owner),
                _ => return Err(bad),
            },
            Encoder::TagPart(part) => match value {
                Value::Tag { raw, parts, .. } => {
                    let payload = parts
                        .get(usize::from(part))
                        .ok_or_else(|| self.error(statement, CompileErrorKind::BadTagPart))?;
                    if parse_int_str(payload.trim()).is_none() && !is_simple_numeric(payload) {
                        return Err(self.error(statement, CompileErrorKind::BadTagPart));
                    }
                    self.push_numeric_str(
                        payload,
                        owner,
                        statement,
                        field,
                        SlotEncoder::TagPart(part),
                        raw,
                    )?;
                }
                _ => return Err(bad),
            },
            Encoder::Value => match value {
                Value::Flag { index, bank, .. } => {
                    self.push_dword(*index, owner);
                    self.push_dword(u32::from(*bank), owner);
                    self.push_call(OP_FLAG_READ, owner);
                }
                Value::Int { value, raw } => {
                    self.push_int(*value, owner, statement, field, SlotEncoder::Value, raw)?;
                }
                Value::Expression { expr, raw } => {
                    self.push_expr(expr, owner, statement, field, SlotEncoder::Value, raw)?;
                }
                Value::Text { text, .. } => self.push_str(text.as_bytes(), owner),
                Value::Image { raw } | Value::Symbol { raw } => {
                    self.push_str(raw.as_bytes(), owner);
                }
                Value::Empty | Value::Tag { .. } => return Err(bad),
            },
        }
        Ok(())
    }
}

/// Parses an optional-minus decimal or `0x` hexadecimal integer.
fn parse_int_str(trimmed: &str) -> Option<i64> {
    let (negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed),
    };
    if digits.is_empty() {
        return None;
    }
    let magnitude = if let Some(hex) = digits.strip_prefix("0x") {
        if hex.is_empty() || !hex.chars().all(|cell| cell.is_ascii_hexdigit()) {
            return None;
        }
        i64::from_str_radix(hex, 16).ok()?
    } else {
        if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        digits.parse::<i64>().ok()?
    };
    Some(if negative {
        magnitude.wrapping_neg()
    } else {
        magnitude
    })
}

/// True when the payload holds only digits, whitespace, `+`, `-` and parentheses.
fn is_simple_numeric(payload: &str) -> bool {
    !payload.is_empty()
        && payload
            .chars()
            .all(|cell| cell.is_ascii_digit() || cell.is_whitespace() || "+-()".contains(cell))
}

/// A `+`/`-`/parentheses expression reader over a tag payload: a decimal or
/// hexadecimal integer, or `+`/`-`/parentheses arithmetic. Anything else
/// (including `*`, `/` and `%`) is an error, never a guess. Nesting past the
/// parser's [`crate::source::MAX_EXPR_NESTING`] cap fails instead of recursing.
struct NumReader<'a> {
    chars: &'a [char],
    pos: usize,
    depth: usize,
}

impl NumReader<'_> {
    fn skip_space(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn sum(&mut self) -> Result<i64, ()> {
        let mut left = self.unary()?;
        loop {
            self.skip_space();
            let add = match self.chars.get(self.pos) {
                Some(&'+') => true,
                Some(&'-') => false,
                _ => return Ok(left),
            };
            self.pos += 1;
            let right = self.unary()?;
            left = if add {
                left.wrapping_add(right)
            } else {
                left.wrapping_sub(right)
            };
        }
    }

    fn descend(&mut self) -> Result<(), ()> {
        if self.depth >= crate::source::MAX_EXPR_NESTING {
            return Err(());
        }
        self.depth += 1;
        Ok(())
    }

    fn unary(&mut self) -> Result<i64, ()> {
        self.skip_space();
        match self.chars.get(self.pos) {
            Some(&'-') => {
                self.pos += 1;
                self.descend()?;
                let inner = self.unary()?;
                self.depth -= 1;
                Ok(inner.wrapping_neg())
            }
            Some(&'+') => {
                self.pos += 1;
                self.descend()?;
                let inner = self.unary()?;
                self.depth -= 1;
                Ok(inner)
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<i64, ()> {
        self.skip_space();
        match self.chars.get(self.pos) {
            Some(&'(') => {
                self.pos += 1;
                self.descend()?;
                let inner = self.sum()?;
                self.depth -= 1;
                self.skip_space();
                if self.chars.get(self.pos) != Some(&')') {
                    return Err(());
                }
                self.pos += 1;
                Ok(inner)
            }
            Some(cell) if cell.is_ascii_digit() => {
                let start = self.pos;
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                self.chars[start..self.pos]
                    .iter()
                    .collect::<String>()
                    .parse::<i64>()
                    .map_err(|_| ())
            }
            _ => Err(()),
        }
    }
}

fn raw_args(statement: &Statement) -> Vec<&str> {
    statement.args().iter().map(Value::raw).collect()
}

/// True when every present condition of the predicate holds for the statement.
fn matches(recipe: &crate::recipes::Recipe, statement: &Statement, builder: &Builder<'_>) -> bool {
    let args = statement.args();
    let when = &recipe.when;
    if when.arity.is_some_and(|arity| args.len() != arity) {
        return false;
    }
    if when
        .arity_range
        .is_some_and(|(min, max)| args.len() < min || args.len() > max)
    {
        return false;
    }
    for (index, literal) in when.equals {
        if args.get(*index).is_none_or(|value| value.raw() != *literal) {
            return false;
        }
    }
    for (index, kind) in when.kinds {
        if args.get(*index).is_none_or(|value| value.kind() != *kind) {
            return false;
        }
    }
    for (index, allowed) in when.one_of {
        if args
            .get(*index)
            .is_none_or(|value| !allowed.contains(&value.raw()))
        {
            return false;
        }
    }
    for (key, value) in when.state {
        if builder.state(*key) != *value {
            return false;
        }
    }
    true
}

impl<'a> Builder<'a> {
    fn apply_set_state(
        &mut self,
        recipe: &crate::recipes::Recipe,
        statement: usize,
        raws: &[&str],
    ) -> Result<(), CompileError> {
        for assignment in recipe.set_state {
            match *assignment {
                crate::recipes::SetState::Lit(key, value) => self.set_state_key(key, value),
                crate::recipes::SetState::Arg(key, index) => {
                    let raw = raws
                        .get(index)
                        .ok_or_else(|| self.error(statement, CompileErrorKind::MissingArgument))?;
                    let value = raw
                        .trim()
                        .parse::<i64>()
                        .map_err(|_| self.error(statement, CompileErrorKind::BadStateArg))?;
                    self.set_state_key(key, value);
                }
            }
        }
        Ok(())
    }

    fn lower_fields(
        &mut self,
        recipe: &crate::recipes::Recipe,
        statement: &Statement,
        index: usize,
        owner: i64,
    ) -> Result<(), CompileError> {
        for field in recipe.fields.iter().rev() {
            match statement.args().get(field.source) {
                None => {
                    for instr in field.absent {
                        self.push_lit(instr.opcode, instr.operands, owner);
                    }
                }
                Some(Value::Empty) => {
                    for instr in field.empty {
                        self.push_lit(instr.opcode, instr.operands, owner);
                    }
                }
                Some(value) => {
                    self.encode_arg(value, field.encoder, owner, index, field.source)?;
                }
            }
        }
        Ok(())
    }

    fn lower_body(
        &mut self,
        recipe: &crate::recipes::Recipe,
        statement: &Statement,
        index: usize,
        owner: i64,
    ) -> Result<(), CompileError> {
        for part in recipe.body {
            match *part {
                crate::recipes::BodyPart::Arg { arg, encoder } => {
                    let value = statement
                        .args()
                        .get(arg)
                        .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                    self.encode_arg(value, encoder, owner, index, arg)?;
                }
                crate::recipes::BodyPart::Lit(instructions) => {
                    for instr in instructions {
                        self.push_lit(instr.opcode, instr.operands, owner);
                    }
                }
                crate::recipes::BodyPart::StatePush(key) => {
                    let value = self.state(key);
                    if value > u32::MAX as i64 || value < -(u32::MAX as i64) {
                        return Err(self.error(index, CompileErrorKind::NumberOutOfRange));
                    }
                    self.push_dword(value as u32, owner);
                }
            }
        }
        for opcode in recipe.calls {
            self.push_call(*opcode, owner);
        }
        Ok(())
    }

    fn scene_name(&self, statement: usize, number: &str) -> Result<&'a str, CompileError> {
        self.scene_names
            .iter()
            .find(|(key, _)| *key == number)
            .map(|(_, name)| *name)
            .ok_or_else(|| self.error(statement, CompileErrorKind::UnknownSceneNumber))
    }

    fn lower_message(&mut self, statement: &Statement, index: usize, owner: i64) {
        let args = statement.args();
        let raw = args.first().map_or("", Value::raw);
        let text: String = raw
            .chars()
            .map(|cell| if cell == '^' { '\n' } else { cell })
            .collect();
        let count = self.message_index;
        self.message_index += 1;
        let mut waitflag = false;
        for later in &self.statements[index + 1..] {
            match later.name() {
                "_WNTY" | "_ZM" | "_MSAD" => break,
                "_WKST" => {
                    let raws = raw_args(later);
                    if raws.len() == 3 && raws[1] == "eq" && raws[2] == "63" {
                        waitflag = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        let mode = if waitflag {
            0
        } else if text.contains("@n") {
            1
        } else {
            (self.wnty + 1) as u32
        };
        let text_bytes = text.as_bytes();
        let meta = self.intern(text_bytes);
        self.push(
            Item::new(
                OP_MESSAGE_META,
                Vec::from([
                    Operand::Dword(META_MAGIC),
                    Operand::Dword(count),
                    Operand::Str(meta),
                ]),
            ),
            owner,
        );
        self.push_dword(mode, owner);
        self.push_dword(1, owner);
        self.push_dword(0, owner);
        self.push_dword(0, owner);
        self.push_str(text_bytes, owner);
        self.push_call(OP_MESSAGE_CALL, owner);
    }

    fn lower_special(
        &mut self,
        recipe: &crate::recipes::Recipe,
        statement: &Statement,
        index: usize,
        owner: i64,
    ) -> Result<(), CompileError> {
        let raws = raw_args(statement);
        match recipe.special {
            Some(Special::Label) => {
                if statement.name() == "_ZY" {
                    let name = raws.first().copied().unwrap_or("");
                    let owned = String::from(name);
                    self.push_target(&owned, owner, index);
                    self.push_call(OP_JUMP, owner);
                    self.define(&owned, index)?;
                } else if let Some(name) = raws.first() {
                    self.define(name, index)?;
                }
            }
            Some(Special::Goto) => {
                let name = String::from(raws.first().copied().unwrap_or(""));
                self.push_target(&name, owner, index);
                self.push_call(OP_JUMP, owner);
            }
            Some(Special::SkipRegistration) => {
                if statement.args().len() > 1 && !raws[1].is_empty() {
                    let name = String::from(raws[1]);
                    self.push_target(&name, owner, index);
                } else {
                    self.push_dword(SKIP_SENTINEL, owner);
                }
                let first = statement
                    .args()
                    .first()
                    .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                self.encode_arg(first, Encoder::Value, owner, index, 0)?;
                self.push_call(OP_SKIP_CALL, owner);
            }
            Some(Special::Message) => self.lower_message(statement, index, owner),
            Some(Special::Jump) => {
                let number = raws.first().copied().unwrap_or("");
                let name = self.scene_name(index, number)?;
                if statement.args().len() > 1 && !raws[1].is_empty() {
                    let entry = statement
                        .args()
                        .get(1)
                        .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                    self.encode_arg(entry, Encoder::Value, owner, index, 1)?;
                } else {
                    self.push_dword(0, owner);
                }
                self.push_call(OP_JUMP_SETUP, owner);
                self.push_str(name.as_bytes(), owner);
                self.push_call(OP_JUMP_CALL, owner);
            }
            Some(Special::PlatformIf) => {
                let value = recipe.platform_value.unwrap_or(0);
                let call = recipe.platform_call.unwrap_or(PLATFORM_CALL_DEFAULT);
                let target = raws
                    .get(recipe.target_field)
                    .copied()
                    .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                let owned = String::from(target);
                self.push_dword(value as u32, owner);
                self.push_call(call, owner);
                self.push_call(OP_CMP_EQ, owner);
                self.push_target(&owned, owner, index);
                self.push(Item::new(OP_JUMP_IF, Vec::from([Operand::Dword(0)])), owner);
            }
            Some(Special::GotoTable) => {
                let supplied = raws.len().saturating_sub(1);
                let missing = recipe.target_slots.saturating_sub(supplied);
                for _ in 0..missing {
                    self.push_dword(SKIP_SENTINEL, owner);
                }
                for label in raws.iter().skip(1).rev() {
                    let owned = String::from(*label);
                    self.push_target(&owned, owner, index);
                }
                let flag = statement
                    .args()
                    .first()
                    .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                self.encode_arg(flag, Encoder::FlagIndex, owner, index, 0)?;
                self.encode_arg(flag, Encoder::FlagBank, owner, index, 0)?;
                self.push_call(OP_GOTO_TABLE, owner);
            }
            Some(Special::If) => {
                let flag = statement
                    .args()
                    .first()
                    .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                let operator = raws.get(1).copied().unwrap_or("");
                let opcode = recipes::comparison_op(operator)
                    .ok_or_else(|| self.error(index, CompileErrorKind::UnknownComparisonOp))?;
                let rhs = statement
                    .args()
                    .get(2)
                    .ok_or_else(|| self.error(index, CompileErrorKind::MissingArgument))?;
                let label = String::from(raws.get(3).copied().unwrap_or(""));
                self.encode_arg(flag, Encoder::FlagIndex, owner, index, 0)?;
                self.encode_arg(flag, Encoder::FlagBank, owner, index, 0)?;
                self.push_call(OP_FLAG_READ, owner);
                self.encode_arg(rhs, Encoder::Value, owner, index, 2)?;
                self.push_call(opcode, owner);
                self.push_target(&label, owner, index);
                self.push(Item::new(OP_JUMP_IF, Vec::from([Operand::Dword(0)])), owner);
            }
            None => {
                self.lower_fields(recipe, statement, index, owner)?;
                self.lower_body(recipe, statement, index, owner)?;
            }
        }
        Ok(())
    }
}

/// Lowers parsed Switch statements to a [`Compiled`] scene.
///
/// `scene_names` maps `_JUMP` decimal spellings to Steam scene names; every used
/// number must resolve through it. The call fails with the offending statement's
/// offset on unknown commands, unmatched predicates, missing scene numbers,
/// unknown comparison operators, unresolved labels, unproven shapes and anything
/// else outside the recipe table.
pub fn compile(
    statements: &[Statement],
    scene_names: &[(&str, &str)],
) -> Result<Compiled, CompileError> {
    let first_wnty = statements
        .iter()
        .position(|statement| statement.name() == "_WNTY")
        .unwrap_or(statements.len());
    let initial_f823 = if statements[..first_wnty]
        .iter()
        .any(|statement| statement.name() == "_WKST" && raw_args(statement) == ["F822", "eq", "1"])
    {
        0
    } else {
        1
    };
    let mut builder = Builder {
        statements,
        scene_names,
        items: Vec::new(),
        strings: Vec::new(),
        label_names: Vec::new(),
        label_targets: Vec::new(),
        pending_refs: Vec::new(),
        owners: Vec::new(),
        spans: Vec::new(),
        slots: Vec::new(),
        selected: Vec::new(),
        warnings: Vec::new(),
        wnty: 0,
        f822: 0,
        f823: initial_f823,
        message_index: 0,
    };

    let entry = builder.fresh_label();
    builder.push_dword(0, -1);
    builder.push_call(OP_DISPATCH_FLAG, -1);
    builder.push_target_id(entry, -1, 0);
    builder.push(Item::new(OP_JUMP_IF, Vec::from([Operand::Dword(1)])), -1);
    for statement in statements {
        if statement.name() == "_ZZ" {
            if let Some(label) = statement.args().first() {
                let owned = String::from(label.raw());
                builder.push_str(owned.as_bytes(), -1);
                builder.push_call(OP_DISPATCH_FLAG, -1);
                builder.push_target(&owned, -1, 0);
                builder.push(Item::new(OP_JUMP_IF, Vec::from([Operand::Dword(1)])), -1);
            }
        }
    }
    builder.push_str(recipes::PROLOGUE_ERROR.as_bytes(), -1);
    builder.push_call(OP_ERROR, -1);
    builder.define_id(entry, 0)?;

    for (index, statement) in statements.iter().enumerate() {
        let start = builder.items.len();
        let owner = i64::try_from(index).unwrap_or(i64::MAX);
        let recipes = recipes::recipes_for(statement.name())
            .ok_or_else(|| builder.error(index, CompileErrorKind::UnknownCommand))?;
        let recipe = recipes
            .iter()
            .find(|candidate| matches(candidate, statement, &builder))
            .ok_or_else(|| builder.error(index, CompileErrorKind::NoMatchingRecipe))?;
        builder.selected.push(Some(recipe.id));
        if recipe.zero_unproven {
            builder.warnings.push(CompileWarning {
                offset: statement.offset(),
                command: String::from(statement.name()),
                args: raw_args(statement)
                    .iter()
                    .map(|raw| String::from(*raw))
                    .collect(),
                reason: ZERO_EMISSION_UNPROVEN,
            });
        }
        builder.lower_special(recipe, statement, index, owner)?;
        let raws = raw_args(statement);
        builder.apply_set_state(recipe, index, &raws)?;
        builder.spans.push((start, builder.items.len()));
    }

    builder.push_call(OP_RETURN, -2);

    let mut labels = Vec::with_capacity(builder.label_targets.len());
    for (id, target) in builder.label_targets.iter().enumerate() {
        let statement = builder
            .pending_refs
            .iter()
            .find(|(pending, _)| *pending == id)
            .map_or(statements.len(), |(_, statement)| *statement);
        let target =
            (*target).ok_or_else(|| builder.error(statement, CompileErrorKind::UnresolvedLabel))?;
        labels.push(target);
    }

    let strings = builder
        .strings
        .iter()
        .map(|bytes| PoolEntry::new(bytes.clone(), false))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| CompileError {
            kind: CompileErrorKind::InvalidProgram,
            statement: statements.len(),
            offset: 0,
        })?;
    let owners = builder.owners;
    let spans = builder.spans;
    let slots = builder.slots;
    let selected = builder.selected;
    let program = Program::from_parts(ProgramParts {
        format: Format::Hunex,
        header_size: HEADER_SIZE,
        includes: Vec::new(),
        header_padding: Vec::from(HEADER_PADDING),
        items: builder.items,
        labels,
        strings,
        tail: Vec::new(),
    })
    .map_err(|_| CompileError {
        kind: CompileErrorKind::InvalidProgram,
        statement: statements.len(),
        offset: 0,
    })?;
    Ok(Compiled {
        program,
        owners,
        spans,
        literal_slots: slots,
        selected,
        warnings: builder.warnings,
    })
}
