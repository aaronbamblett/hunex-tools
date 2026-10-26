//! Parser for HuneX Switch script text (`_CMD(args);` statements).
//!
//! [`parse`] reads a script into [`Statement`]s in source order. Offsets are Unicode
//! character indices after leading byte-order marks are removed. Argument spelling is
//! preserved; each argument is also classified into a [`Value`].
//!
//! The classification keeps the raw spelling and records a kind: empty, text
//! (contains `$`, with `^` replaced by LF), tag (leading backtick), flag
//! (`F`/`G`/`_D` plus decimal digits), int (decimal or `0x` hexadecimal),
//! expression (literals, flag reads and `+ - | & ( )`; `* / %` are rejected), image
//! (starts with `img`) and symbol (everything else). Digits are ASCII. See
//! `docs/formats/chs-format.md` for how the compiler lowers each kind.
//!
//! ```
//! let statements = hunex_chs::source::parse("_PGST(-1);_ZM01502($000008^$000009);").unwrap();
//! assert_eq!(statements.len(), 2);
//! assert_eq!(statements[1].name(), "_ZM");
//! assert_eq!(statements[1].raw_name(), "_ZM01502");
//! ```

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// One `_NAME(arguments);` statement of a Switch script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    name: String,
    raw_name: String,
    offset: usize,
    args: Vec<Value>,
    malformed: bool,
}

impl Statement {
    /// The canonical command name. `_ZZ`/`_ZM`/`_ZY` with an all-hexadecimal suffix
    /// are shortened to the family name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The command name exactly as written, before canonicalisation.
    pub fn raw_name(&self) -> &str {
        &self.raw_name
    }

    /// Unicode character index of the statement start, after BOM removal.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The typed arguments in source order. An absent argument is represented by the
    /// argument count: position `i` is absent when `i >= args().len()`.
    pub fn args(&self) -> &[Value] {
        &self.args
    }

    /// True for the 14 literal `_KWRS);` tokens of the corpus, which have no argument
    /// list. They parse as a `_KWRS` statement with no arguments.
    pub fn malformed(&self) -> bool {
        self.malformed
    }
}

/// The kind of a parsed argument, without its payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// A supplied zero-length field.
    Empty,
    /// A field containing `$`, with every `^` replaced by LF.
    Text,
    /// A field with a leading backtick, split on `:`.
    Tag,
    /// `F`/`G`/`_D` plus decimal digits: banks 0, 1 and 2.
    Flag,
    /// Decimal digits with an optional leading minus, or `0x` hexadecimal digits.
    Int,
    /// Digits, whitespace and arithmetic punctuation, parsed into an [`Expr`].
    Expression,
    /// Any other field starting with `img`.
    Image,
    /// Any remaining field.
    Symbol,
}

/// One typed script argument. Every variant keeps the raw spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A supplied zero-length field.
    Empty,
    /// A field containing `$`. `text` has every `^` replaced by LF.
    Text {
        /// The field exactly as written.
        raw: String,
        /// The field with `^` replaced by LF.
        text: String,
    },
    /// A field with a leading backtick. `tag` is the code before the first `:` and
    /// `parts` are the payload fields after it.
    Tag {
        /// The field exactly as written.
        raw: String,
        /// The tag code, with original spelling.
        tag: String,
        /// The payload fields after the tag code.
        parts: Vec<String>,
    },
    /// A flag reference. Banks are 0 for `F`, 1 for `G` and 2 for `_D`.
    Flag {
        /// The field exactly as written.
        raw: String,
        /// The decimal flag index.
        index: u32,
        /// The flag bank: 0, 1 or 2.
        bank: u8,
    },
    /// A decimal or `0x` hexadecimal integer.
    Int {
        /// The field exactly as written.
        raw: String,
        /// The parsed value.
        value: i64,
    },
    /// An arithmetic argument, parsed into an AST.
    Expression {
        /// The field exactly as written.
        raw: String,
        /// The parsed expression.
        expr: Expr,
    },
    /// Any other field starting with `img`, with original spelling.
    Image {
        /// The field exactly as written.
        raw: String,
    },
    /// Any remaining field, with original spelling.
    Symbol {
        /// The field exactly as written.
        raw: String,
    },
}

impl Value {
    /// The kind of this value.
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Empty => ValueKind::Empty,
            Value::Text { .. } => ValueKind::Text,
            Value::Tag { .. } => ValueKind::Tag,
            Value::Flag { .. } => ValueKind::Flag,
            Value::Int { .. } => ValueKind::Int,
            Value::Expression { .. } => ValueKind::Expression,
            Value::Image { .. } => ValueKind::Image,
            Value::Symbol { .. } => ValueKind::Symbol,
        }
    }

    /// The field exactly as written.
    pub fn raw(&self) -> &str {
        match self {
            Value::Empty => "",
            Value::Text { raw, .. }
            | Value::Tag { raw, .. }
            | Value::Flag { raw, .. }
            | Value::Int { raw, .. }
            | Value::Expression { raw, .. }
            | Value::Image { raw }
            | Value::Symbol { raw } => raw,
        }
    }
}

/// An argument expression with unary minus, addition, subtraction, bitwise OR/AND and parentheses.
///
/// Parsing caps nesting at [`MAX_EXPR_NESTING`] and the node count at
/// [`MAX_EXPR_NODES`], so walking, lowering or dropping a parsed tree never
/// recurses deeper than the nesting cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// An integer literal.
    Lit(i64),
    /// VM flag read.
    Flag {
        /// Variable index.
        index: u32,
        /// VM bank: F=0, G=1, compatibility=2.
        bank: u8,
    },
    /// Bitwise OR.
    Or(Box<Expr>, Box<Expr>),
    /// Bitwise AND.
    And(Box<Expr>, Box<Expr>),
    /// Unary minus.
    Neg(Box<Expr>),
    /// Addition.
    Add(Box<Expr>, Box<Expr>),
    /// Subtraction.
    Sub(Box<Expr>, Box<Expr>),
}

/// The deepest nesting the expression parser accepts: parentheses and unary
/// signs each count one level. Deeper input is an error, never a stack
/// overflow.
pub const MAX_EXPR_NESTING: usize = 256;

/// The most nodes one parsed expression may hold. Larger input is an error,
/// never a stack overflow while lowering or dropping the tree.
pub const MAX_EXPR_NODES: usize = 4096;

/// Errors from parsing script text. Every variant carries the Unicode character index
/// (after BOM removal) of the offending statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Text that starts no statement where one was expected.
    UnexpectedSyntax {
        /// Character index where the text could not be parsed.
        offset: usize,
    },
    /// A statement whose argument list never closes with `);`.
    UnterminatedStatement {
        /// Character index of the statement start.
        offset: usize,
    },
    /// An expression with an operator outside `+`, `-`, `|` and `&` (`*`, `/`, `%`) or with
    /// any other shape the AST does not cover.
    UnsupportedExpression {
        /// Character index of the statement start.
        offset: usize,
    },
    /// A decimal or hexadecimal number that does not fit an `i64`.
    NumberOutOfRange {
        /// Character index of the statement start.
        offset: usize,
    },
    /// An expression nested deeper than [`MAX_EXPR_NESTING`] levels or holding
    /// more than [`MAX_EXPR_NODES`] nodes.
    ExpressionTooDeep {
        /// Character index of the statement start.
        offset: usize,
    },
}

impl ParseError {
    /// The character index (after BOM removal) carried by the error.
    pub fn offset(self) -> usize {
        match self {
            ParseError::UnexpectedSyntax { offset }
            | ParseError::UnterminatedStatement { offset }
            | ParseError::UnsupportedExpression { offset }
            | ParseError::NumberOutOfRange { offset }
            | ParseError::ExpressionTooDeep { offset } => offset,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParseError::UnexpectedSyntax { .. } => write!(f, "unparsed source"),
            ParseError::UnterminatedStatement { .. } => write!(f, "unterminated statement"),
            ParseError::UnsupportedExpression { .. } => write!(f, "unsupported expression"),
            ParseError::NumberOutOfRange { .. } => write!(f, "number out of range"),
            ParseError::ExpressionTooDeep { .. } => {
                write!(f, "expression too deep or too large")
            }
        }
    }
}

impl core::error::Error for ParseError {}

const MALFORMED_TOKEN: &str = "_KWRS);";

fn starts_with(chars: &[char], pos: usize, token: &str) -> bool {
    let token: Vec<char> = token.chars().collect();
    chars.len() - pos >= token.len() && chars[pos..pos + token.len()] == token
}

fn is_word_char(value: char) -> bool {
    value == '_' || value.is_alphanumeric()
}

fn is_hex_digit(value: char) -> bool {
    value.is_ascii_hexdigit()
}

fn is_all_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Shortens `_ZZ`/`_ZM`/`_ZY` followed entirely by hexadecimal digits to the family
/// name. Anything else is returned unchanged.
fn canonicalize(raw: &str) -> &str {
    for family in ["_ZZ", "_ZM", "_ZY"] {
        if let Some(suffix) = raw.strip_prefix(family) {
            if !suffix.is_empty() && suffix.chars().all(is_hex_digit) {
                return family;
            }
        }
    }
    raw
}

/// Parses a script into its statements in source order. A leading byte-order mark is
/// allowed and trailing NUL bytes are ignored.
pub fn parse(text: &str) -> Result<Vec<Statement>, ParseError> {
    let stripped = text.trim_start_matches('\u{FEFF}').trim_end_matches('\0');
    let chars: Vec<char> = stripped.chars().collect();
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < chars.len() {
        if chars[pos].is_whitespace() {
            pos += 1;
            continue;
        }
        if starts_with(&chars, pos, MALFORMED_TOKEN) {
            out.push(Statement {
                name: String::from("_KWRS"),
                raw_name: String::from("_KWRS"),
                offset: pos,
                args: Vec::new(),
                malformed: true,
            });
            pos += MALFORMED_TOKEN.chars().count();
            continue;
        }
        let start = pos;
        let Some((raw_name, arg_start)) = match_name(&chars, pos) else {
            return Err(ParseError::UnexpectedSyntax { offset: start });
        };
        pos = arg_start;
        let mut depth: i32 = 1;
        while pos < chars.len() && depth > 0 {
            if chars[pos] == '(' {
                depth += 1;
            } else if chars[pos] == ')' {
                depth -= 1;
            }
            pos += 1;
        }
        if depth != 0 || chars.get(pos) != Some(&';') {
            return Err(ParseError::UnterminatedStatement { offset: start });
        }
        let argtext: String = chars[arg_start..pos - 1].iter().collect();
        let mut raws: Vec<String> = Vec::new();
        if !argtext.is_empty() {
            let argchars: Vec<char> = argtext.chars().collect();
            let mut nested: i32 = 0;
            let mut last = 0;
            for (index, cell) in argchars.iter().enumerate() {
                if *cell == '(' {
                    nested += 1;
                } else if *cell == ')' {
                    nested -= 1;
                }
                if *cell == ',' && nested == 0 {
                    raws.push(argchars[last..index].iter().collect());
                    last = index + 1;
                }
            }
            raws.push(argchars[last..].iter().collect());
        }
        let mut args = Vec::with_capacity(raws.len());
        for raw in &raws {
            args.push(classify(raw, start)?);
        }
        out.push(Statement {
            name: String::from(canonicalize(&raw_name)),
            raw_name,
            offset: start,
            args,
            malformed: false,
        });
        pos += 1;
    }
    Ok(out)
}

/// Matches `_\w+\(` at `pos`. Returns the raw name and the index after the `(`.
fn match_name(chars: &[char], pos: usize) -> Option<(String, usize)> {
    if chars.get(pos) != Some(&'_') {
        return None;
    }
    let mut end = pos + 1;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    if end == pos + 1 || chars.get(end) != Some(&'(') {
        return None;
    }
    Some((chars[pos..end].iter().collect(), end + 1))
}

/// Classifies one raw argument field. `offset` is the statement start, used for errors.
fn classify(raw: &str, offset: usize) -> Result<Value, ParseError> {
    if raw.is_empty() {
        return Ok(Value::Empty);
    }
    if raw.contains('$') {
        return Ok(Value::Text {
            raw: String::from(raw),
            text: raw.replace('^', "\n"),
        });
    }
    if let Some(body) = raw.strip_prefix('`') {
        let parts: Vec<String> = body.split(':').map(String::from).collect();
        let mut owned = parts;
        let first = if owned.is_empty() {
            String::new()
        } else {
            owned.remove(0)
        };
        return Ok(Value::Tag {
            raw: String::from(raw),
            tag: first,
            parts: owned,
        });
    }
    if let Some(rest) = raw.strip_prefix("_D") {
        if is_all_digits(rest) {
            return Ok(Value::Flag {
                raw: String::from(raw),
                index: rest
                    .parse::<u32>()
                    .map_err(|_| ParseError::NumberOutOfRange { offset })?,
                bank: 2,
            });
        }
    }
    if raw.len() >= 2 && (raw.as_bytes()[0] == b'F' || raw.as_bytes()[0] == b'G') {
        let bank = if raw.as_bytes()[0] == b'F' { 0 } else { 1 };
        if is_all_digits(&raw[1..]) {
            return Ok(Value::Flag {
                raw: String::from(raw),
                index: raw[1..]
                    .parse::<u32>()
                    .map_err(|_| ParseError::NumberOutOfRange { offset })?,
                bank,
            });
        }
    }
    if let Some(digits) = raw.strip_prefix('-') {
        if is_all_digits(digits) {
            let magnitude: i64 = digits
                .parse()
                .map_err(|_| ParseError::NumberOutOfRange { offset })?;
            return Ok(Value::Int {
                raw: String::from(raw),
                value: 0 - magnitude,
            });
        }
    } else if is_all_digits(raw) {
        return Ok(Value::Int {
            raw: String::from(raw),
            value: raw
                .parse()
                .map_err(|_| ParseError::NumberOutOfRange { offset })?,
        });
    }
    if let Some(hex) = raw.strip_prefix("0x") {
        if !hex.is_empty() && hex.chars().all(is_hex_digit) {
            return Ok(Value::Int {
                raw: String::from(raw),
                value: i64::from_str_radix(hex, 16)
                    .map_err(|_| ParseError::NumberOutOfRange { offset })?,
            });
        }
    }
    if !raw.is_empty()
        && raw.chars().all(|cell| {
            cell.is_ascii_digit()
                || (raw.contains("0x")
                    && (raw
                        .trim_start()
                        .starts_with(|c: char| c.is_ascii_digit() || "+-(".contains(c))
                        || raw.contains(['|', '&']))
                    && "abcdefABCDEFx".contains(cell))
                || cell.is_whitespace()
                || ("+*/()%-".contains(cell)
                    || (raw.contains(['|', '&']) && "|&FG_D".contains(cell)))
        })
    {
        let chars: Vec<char> = raw.chars().collect();
        let mut reader = ExprReader {
            chars: &chars,
            pos: 0,
            offset,
            depth: 0,
            nodes: 0,
        };
        let expr = reader.bit_or()?;
        if reader.pos != chars.len() {
            return Err(ParseError::UnsupportedExpression { offset });
        }
        return Ok(Value::Expression {
            raw: String::from(raw),
            expr,
        });
    }
    if raw.starts_with("img") {
        return Ok(Value::Image {
            raw: String::from(raw),
        });
    }
    Ok(Value::Symbol {
        raw: String::from(raw),
    })
}

struct ExprReader<'a> {
    chars: &'a [char],
    pos: usize,
    offset: usize,
    depth: usize,
    nodes: usize,
}

impl ExprReader<'_> {
    fn skip_space(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn bit_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.bit_and()?;
        loop {
            self.skip_space();
            if self.chars.get(self.pos) != Some(&'|') {
                return Ok(left);
            }
            self.pos += 1;
            let right = self.bit_and()?;
            self.node()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
    }

    fn bit_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.sum()?;
        loop {
            self.skip_space();
            if self.chars.get(self.pos) != Some(&'&') {
                return Ok(left);
            }
            self.pos += 1;
            let right = self.sum()?;
            self.node()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
    }

    fn sum(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            self.skip_space();
            let op = match self.chars.get(self.pos) {
                Some(&'+') => true,
                Some(&'-') => false,
                _ => return Ok(left),
            };
            self.pos += 1;
            let right = self.unary()?;
            self.node()?;
            left = if op {
                Expr::Add(Box::new(left), Box::new(right))
            } else {
                Expr::Sub(Box::new(left), Box::new(right))
            };
        }
    }

    fn descend(&mut self) -> Result<(), ParseError> {
        if self.depth >= MAX_EXPR_NESTING {
            return Err(ParseError::ExpressionTooDeep {
                offset: self.offset,
            });
        }
        self.depth += 1;
        Ok(())
    }

    /// Counts one freshly built node and fails past [`MAX_EXPR_NODES`], so a
    /// hostile chain is rejected while the partial tree is still small enough
    /// to walk and drop safely.
    fn node(&mut self) -> Result<(), ParseError> {
        if self.nodes >= MAX_EXPR_NODES {
            return Err(ParseError::ExpressionTooDeep {
                offset: self.offset,
            });
        }
        self.nodes += 1;
        Ok(())
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        self.skip_space();
        match self.chars.get(self.pos) {
            Some(&'-') => {
                self.pos += 1;
                self.descend()?;
                let inner = self.unary()?;
                self.depth -= 1;
                self.node()?;
                Ok(Expr::Neg(Box::new(inner)))
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

    fn primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_space();
        match self.chars.get(self.pos) {
            Some(&'(') => {
                self.pos += 1;
                self.descend()?;
                let inner = self.bit_or()?;
                self.depth -= 1;
                self.skip_space();
                if self.chars.get(self.pos) != Some(&')') {
                    return Err(ParseError::UnsupportedExpression {
                        offset: self.offset,
                    });
                }
                self.pos += 1;
                Ok(inner)
            }
            Some(&'F' | &'G' | &'_') => {
                let bank = match self.chars[self.pos] {
                    'F' => 0,
                    'G' => 1,
                    _ => 2,
                };
                self.pos += 1;
                if bank == 2 {
                    if self.chars.get(self.pos) != Some(&'D') {
                        return Err(ParseError::UnsupportedExpression {
                            offset: self.offset,
                        });
                    }
                    self.pos += 1;
                }
                let start = self.pos;
                while self.chars.get(self.pos).is_some_and(char::is_ascii_digit) {
                    self.pos += 1;
                }
                let text: String = self.chars[start..self.pos].iter().collect();
                let index = text
                    .parse()
                    .map_err(|_| ParseError::UnsupportedExpression {
                        offset: self.offset,
                    })?;
                self.node()?;
                Ok(Expr::Flag { index, bank })
            }
            Some(cell) if cell.is_ascii_digit() => self.number(),
            _ => Err(ParseError::UnsupportedExpression {
                offset: self.offset,
            }),
        }
    }

    fn number(&mut self) -> Result<Expr, ParseError> {
        self.node()?;
        let start = self.pos;
        if self.chars.get(self.pos) == Some(&'0') && self.chars.get(self.pos + 1) == Some(&'x') {
            self.pos += 2;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_hexdigit() {
                self.pos += 1;
            }
            let text: String = self.chars[start + 2..self.pos].iter().collect();
            if text.is_empty() {
                return Err(ParseError::UnsupportedExpression {
                    offset: self.offset,
                });
            }
            let value =
                i64::from_str_radix(&text, 16).map_err(|_| ParseError::NumberOutOfRange {
                    offset: self.offset,
                })?;
            return Ok(Expr::Lit(value));
        }
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let value: i64 = text.parse().map_err(|_| ParseError::NumberOutOfRange {
            offset: self.offset,
        })?;
        Ok(Expr::Lit(value))
    }
}
