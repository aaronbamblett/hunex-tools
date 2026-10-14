//! Engine recipe table for lowering Switch script text to Steam programs.
//!
//! The data itself lives in the `recipes_data` module, which is the maintained
//! source (edit it by hand, keep the command tables sorted by name); this module
//! holds the table types, the prologue error string, the comparison-op map and
//! the [`recipes_for`] lookup. The scene number to scene name map is deliberately
//! not data: the caller passes it to the compiler, derived per the scene number
//! rule in `docs/formats/chs-format.md` (for example from a names list such as
//! `nam.bin` entries).
//!
//! ```
//! let recipes = hunex_chs::recipes::recipes_for("_END_").unwrap();
//! assert_eq!(recipes.len(), 1);
//! assert_eq!(recipes[0].id, "_END_:0");
//! ```

use alloc::vec::Vec;

use crate::source::ValueKind;

#[path = "recipes_data.rs"]
mod recipes_data;

pub use recipes_data::{COMMANDS, COMPARISON_OPS, PROLOGUE_ERROR};

/// One command's recipe list: the canonical command name and its ordered recipes.
/// The compiler selects the first recipe whose [`When`] matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandTable {
    /// The canonical command name, for example `"_WKST"`.
    pub name: &'static str,
    /// The ordered recipes for the command.
    pub recipes: &'static [Recipe],
}

/// One comparison operator name and its opcode, for example `eq` to `0x30`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmpOp {
    /// The operator as written in `_IF__`, for example `"eq"`.
    pub name: &'static str,
    /// The emitted comparison opcode.
    pub opcode: u32,
}

/// Compile-state slots the recipes read and write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateKey {
    /// The current `_WNTY` argument plus one selects the message mode.
    Wnty,
    /// Set by `_WKST(F822,eq,1)`; consumed by the next `_WNTY`.
    F822,
    /// Set by `_WKAD(F823,1)`; consumed by `_WKST(F823,eq,0)`.
    F823,
}

/// How one source argument becomes instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoder {
    /// Numeric expressions, string pushes for text/image/symbol, or flag reads.
    Value,
    /// A raw string push regardless of parsed kind.
    Str,
    /// One flag index push, without the read.
    FlagIndex,
    /// One flag bank push, without the read.
    FlagBank,
    /// The Nth payload of a backtick tag, parsed as a numeric expression.
    TagPart(u8),
}

/// One operand of a literal instruction in the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitOp {
    /// A fixed numeric operand.
    Num(u32),
    /// A fixed string operand, interned on first use.
    Text(&'static str),
}

/// One literal instruction in the table: an opcode and its fixed operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LitInstr {
    /// The opcode.
    pub opcode: u32,
    /// The fixed operands.
    pub operands: &'static [LitOp],
}

/// One logical push field of a `fields` recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    /// The zero-based source argument it reads.
    pub source: usize,
    /// How the supplied argument is encoded.
    pub encoder: Encoder,
    /// The instructions when the field is supplied but empty.
    pub empty: &'static [LitInstr],
    /// The instructions when the field is absent (past the argument count).
    pub absent: &'static [LitInstr],
}

/// One ordered emission part of a `body` recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyPart {
    /// Encode one source argument.
    Arg {
        /// The zero-based source argument.
        arg: usize,
        /// How it is encoded.
        encoder: Encoder,
    },
    /// Emit fixed instructions.
    Lit(&'static [LitInstr]),
    /// Push the current value of a compile-state slot.
    StatePush(StateKey),
}

/// One compile-state assignment applied after a statement emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetState {
    /// Assign a literal value.
    Lit(StateKey, i64),
    /// Parse the raw argument at the index as a decimal integer and assign it.
    Arg(StateKey, usize),
}

/// A structural operation that overrides normal field and body emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Special {
    /// `_ZZ`/`_ZY`: define a label, after emitting the `_ZY` jump.
    Label,
    /// `_XSKR`: register a skip target without defining a label.
    SkipRegistration,
    /// `_GOTO`: an unconditional jump to a label.
    Goto,
    /// `_ZM`/`_MSAD`: a message with metadata, mode and state rules.
    Message,
    /// `_JUMP`: a scene change through the caller-supplied scene name map.
    Jump,
    /// `_IFP4`/`_IFNX`/`_IFSC`/`_IFLA`: a platform-flag conditional.
    PlatformIf,
    /// `_GOTS`: a fixed 19-entry jump table.
    GotoTable,
    /// `_IF__`: a flag conditional with a comparison operator.
    If,
}

/// The match predicate of one recipe. Every present condition must hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct When {
    /// The exact provided field count, when present.
    pub arity: Option<usize>,
    /// The inclusive minimum and maximum provided field count, when present.
    pub arity_range: Option<(usize, usize)>,
    /// Zero-based argument indices pinned to exact raw strings.
    pub equals: &'static [(usize, &'static str)],
    /// Zero-based argument indices pinned to parsed kinds. Absent fields fail.
    pub kinds: &'static [(usize, ValueKind)],
    /// Zero-based argument indices pinned to allowed raw strings.
    pub one_of: &'static [(usize, &'static [&'static str])],
    /// Exact compile-state values.
    pub state: &'static [(StateKey, i64)],
}

/// One lowering recipe: its predicate, its emission parts and its state effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recipe {
    /// The stable command and index identifier used by corpus evidence.
    pub id: &'static str,
    /// The match predicate. The compiler takes the first matching recipe.
    pub when: When,
    /// Logical push fields, emitted in reverse order. Empty when unused.
    pub fields: &'static [Field],
    /// Ordered emission parts after any fields. Empty when unused.
    pub body: &'static [BodyPart],
    /// Operand-free opcodes appended after fields and body (normal recipes only).
    pub calls: &'static [u32],
    /// A structural operation overriding field and body emission, when present.
    pub special: Option<Special>,
    /// State assignments applied after emission.
    pub set_state: &'static [SetState],
    /// The pushed platform value of a [`Special::PlatformIf`], when present.
    pub platform_value: Option<i64>,
    /// The platform call of a [`Special::PlatformIf`]; `0x131` when absent.
    pub platform_call: Option<u32>,
    /// The argument holding the label of a [`Special::PlatformIf`]; 1 when absent.
    pub target_field: usize,
    /// The fixed entry count of a [`Special::GotoTable`]; 0 otherwise.
    pub target_slots: usize,
    /// True when this zero-emission predicate is not proven on the corpus: the
    /// compiler still emits nothing for it but records a warning.
    pub zero_unproven: bool,
}

/// The ordered recipes for a canonical command name, or `None` when the command
/// has no recipes. Commands with an empty recipe array are explicitly unresolved.
pub fn recipes_for(name: &str) -> Option<&'static [Recipe]> {
    COMMANDS
        .iter()
        .find(|table| table.name == name)
        .map(|table| table.recipes)
}

/// The comparison opcode for an `_IF__` operator name, or `None` when unknown.
pub fn comparison_op(name: &str) -> Option<u32> {
    COMPARISON_OPS
        .iter()
        .find(|op| op.name == name)
        .map(|op| op.opcode)
}

/// Every canonical command name in the table, in sorted order.
pub fn command_names() -> Vec<&'static str> {
    COMMANDS.iter().map(|table| table.name).collect()
}
