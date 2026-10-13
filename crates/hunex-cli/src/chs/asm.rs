//! Text form of a script. [`dump`] writes it, and [`assemble`] and [`build`] read it back.
//!
//! Lines, one statement each. `#` starts a comment line.
//!
//! - `format hunex` or `format buriko`, and `header_size N`, both required.
//! - `include "name"`, once per include, in order.
//! - `header_padding 00 a5 ...`, optional.
//! - `L_0000:` defines a label on the next instruction (or the end of code).
//! - An instruction is a mnemonic (`PushString`) or a hex id (`0x0f7`), then operands:
//!   a number (`0x10`, `16`), a label (`L_0000`), a string (`S_0000`), or a quoted string,
//!   which becomes a new string.
//! - `string S_0000 "text"` is an original pool entry, in pool order. `string new S_0000 "text"`
//!   is a new string, written after the original ones.
//! - `tail "text"`, optional bytes after the last pool NUL.
//!
//! Quoted text escapes `\n \r \t \\ \"` and `\xNN`. Text that is valid UTF-8 is written as is;
//! other bytes are written as `\xNN`.

use std::collections::BTreeMap;
use std::fmt;

use hunex_chs::{
    opcode_info, Format, Item, LabelId, Operand, PoolEntry, Program, ProgramParts, OPCODES,
};

/// An error from reading assembly text, with the line it is on when known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmError {
    /// The line number of the error, when known.
    pub line: Option<usize>,
    /// What is wrong, in plain text.
    pub message: String,
}

impl AsmError {
    fn at(line: usize, message: impl Into<String>) -> AsmError {
        AsmError {
            line: Some(line),
            message: message.into(),
        }
    }

    fn global(message: impl Into<String>) -> AsmError {
        AsmError {
            line: None,
            message: message.into(),
        }
    }
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for AsmError {}

/// Formats a program as the text that the `chs dump` command writes in its assembly mode.
pub fn dump(program: &Program) -> String {
    let mut out = String::new();
    let format = match program.format() {
        Format::Hunex => "hunex",
        Format::Buriko => "buriko",
    };
    out.push_str(&format!("format {format}\n"));
    out.push_str(&format!("header_size {}\n", program.header_size()));
    for name in program.includes() {
        out.push_str(&format!("include {}\n", quote(name)));
    }
    if !program.header_padding().is_empty() {
        out.push_str(&format!(
            "header_padding {}\n",
            hex(program.header_padding())
        ));
    }
    out.push('\n');

    let items = program.items();
    let mut labels_at: Vec<Vec<LabelId>> = alloc_labels(items.len());
    for (id, &target) in program.labels().iter().enumerate() {
        if let Some(slot) = labels_at.get_mut(target) {
            slot.push(id);
        }
    }
    for (index, labels) in labels_at.iter().enumerate() {
        for id in labels {
            out.push_str(&format!("L_{id:04}:\n"));
        }
        if let Some(item) = items.get(index) {
            out.push_str("    ");
            out.push_str(&instruction_text(item));
            out.push('\n');
        }
    }
    out.push('\n');

    for (id, entry) in program.strings().iter().enumerate() {
        let kind = if entry.is_original() {
            "string"
        } else {
            "string new"
        };
        out.push_str(&format!("{kind} S_{id:04} {}\n", quote(entry.bytes())));
    }
    if !program.tail().is_empty() {
        out.push_str(&format!("tail {}\n", quote(program.tail())));
    }
    out
}

fn alloc_labels(items: usize) -> Vec<Vec<LabelId>> {
    (0..=items).map(|_| Vec::new()).collect()
}

fn instruction_text(item: &Item) -> String {
    let mut text = match opcode_info(item.opcode()).and_then(|info| info.mnemonic) {
        Some(mnemonic) => mnemonic.to_string(),
        None => format!("0x{:03x}", item.opcode()),
    };
    for operand in item.operands() {
        text.push(' ');
        match *operand {
            Operand::Dword(value) => text.push_str(&format!("0x{value:x}")),
            Operand::Label(id) => text.push_str(&format!("L_{id:04}")),
            Operand::Str(id) => text.push_str(&format!("S_{id:04}")),
        }
    }
    text
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote(bytes: &[u8]) -> String {
    let utf8 = core::str::from_utf8(bytes).is_ok();
    let mut out = vec![b'"'];
    for &byte in bytes {
        match byte {
            b'"' => out.extend_from_slice(b"\\\""),
            b'\\' => out.extend_from_slice(b"\\\\"),
            b'\n' => out.extend_from_slice(b"\\n"),
            b'\r' => out.extend_from_slice(b"\\r"),
            b'\t' => out.extend_from_slice(b"\\t"),
            0x20..=0x7E => out.push(byte),
            0x80..=0xFF if utf8 => out.push(byte),
            _ => out.extend_from_slice(format!("\\x{byte:02x}").as_bytes()),
        }
    }
    out.push(b'"');
    String::from_utf8_lossy(&out).into_owned()
}

/// Assembles text into a program.
pub fn assemble(text: &[u8]) -> Result<Program, AsmError> {
    let mut statements = Vec::new();
    for (index, raw) in text.split(|&byte| byte == b'\n').enumerate() {
        let line = index + 1;
        let tokens = tokenize(raw, line)?;
        if !tokens.is_empty() {
            statements.push((line, tokens));
        }
    }
    assemble_statements(&statements)
}

/// Assembles text and lowers it to the original file bytes.
pub fn build(text: &[u8]) -> Result<Vec<u8>, AsmError> {
    let program = assemble(text)?;
    let script = program
        .lower()
        .map_err(|error| AsmError::global(error.to_string()))?;
    Ok(script.encode())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Quoted(Vec<u8>),
}

fn tokenize(raw: &[u8], line: usize) -> Result<Vec<Token>, AsmError> {
    let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
    let mut tokens = Vec::new();
    let mut position = 0;
    while position < raw.len() {
        let byte = raw[position];
        if matches!(byte, b' ' | b'\t') {
            position += 1;
        } else if byte == b'#' && tokens.is_empty() {
            break;
        } else if byte == b'"' {
            let (bytes, next) = parse_quoted(raw, position, line)?;
            if next < raw.len() && !matches!(raw[next], b' ' | b'\t') {
                return Err(AsmError::at(line, "expected a space after a string"));
            }
            tokens.push(Token::Quoted(bytes));
            position = next;
        } else {
            let start = position;
            while position < raw.len() && !matches!(raw[position], b' ' | b'\t' | b'"') {
                position += 1;
            }
            let word = core::str::from_utf8(&raw[start..position])
                .map_err(|_| AsmError::at(line, "text outside strings must be UTF-8"))?;
            tokens.push(Token::Word(word.to_string()));
        }
    }
    Ok(tokens)
}

fn parse_quoted(raw: &[u8], start: usize, line: usize) -> Result<(Vec<u8>, usize), AsmError> {
    let mut bytes = Vec::new();
    let mut position = start + 1;
    loop {
        let Some(&byte) = raw.get(position) else {
            return Err(AsmError::at(line, "unterminated string"));
        };
        position += 1;
        match byte {
            b'"' => return Ok((bytes, position)),
            b'\\' => {
                let Some(&escape) = raw.get(position) else {
                    return Err(AsmError::at(line, "unterminated escape"));
                };
                position += 1;
                match escape {
                    b'n' => bytes.push(b'\n'),
                    b'r' => bytes.push(b'\r'),
                    b't' => bytes.push(b'\t'),
                    b'\\' => bytes.push(b'\\'),
                    b'"' => bytes.push(b'"'),
                    b'x' => {
                        let digits = raw
                            .get(position..position + 2)
                            .filter(|digits| digits.iter().all(u8::is_ascii_hexdigit))
                            .ok_or_else(|| AsmError::at(line, "\\x needs two hex digits"))?;
                        let text = core::str::from_utf8(digits).unwrap_or("00");
                        bytes.push(
                            u8::from_str_radix(text, 16)
                                .map_err(|_| AsmError::at(line, "bad \\x escape"))?,
                        );
                        position += 2;
                    }
                    _ => return Err(AsmError::at(line, "unknown escape in string")),
                }
            }
            0x20..=0x7E | 0x80..=0xFF => bytes.push(byte),
            _ => return Err(AsmError::at(line, "control character must be escaped")),
        }
    }
}

fn first_word(tokens: &[Token]) -> Option<&str> {
    match tokens.first() {
        Some(Token::Word(word)) => Some(word.as_str()),
        _ => None,
    }
}

fn parse_number(word: &str) -> Option<u32> {
    match word.strip_prefix("0x") {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None => word.parse().ok(),
    }
}

fn assemble_statements(statements: &[(usize, Vec<Token>)]) -> Result<Program, AsmError> {
    let mut format = None;
    let mut header_size = None;
    let mut includes = Vec::new();
    let mut header_padding = None;
    let mut tail = None;
    let mut label_names: BTreeMap<String, LabelId> = BTreeMap::new();
    let mut label_targets = Vec::new();
    let mut string_names: BTreeMap<String, usize> = BTreeMap::new();
    let mut strings = Vec::new();
    let mut item_count = 0usize;
    let mut items = Vec::new();

    for (line, tokens) in statements {
        let line = *line;
        match first_word(tokens) {
            Some(word) if word.ends_with(':') => {
                let name = &word[..word.len() - 1];
                if tokens.len() != 1 || !name.starts_with("L_") {
                    return Err(AsmError::at(line, "a label is L_NAME: on its own line"));
                }
                if label_names.contains_key(name) {
                    return Err(AsmError::at(line, format!("label {name} is defined twice")));
                }
                label_names.insert(name.to_string(), label_targets.len());
                label_targets.push(item_count);
            }
            Some("string") => {
                let (name, original, bytes) = string_statement(tokens, line)?;
                if !name.starts_with("S_") {
                    return Err(AsmError::at(line, "string names start with S_"));
                }
                if string_names.contains_key(&name) {
                    return Err(AsmError::at(
                        line,
                        format!("string {name} is defined twice"),
                    ));
                }
                string_names.insert(name, strings.len());
                strings.push(
                    PoolEntry::new(bytes, original)
                        .map_err(|error| AsmError::at(line, error.to_string()))?,
                );
            }
            Some("format" | "header_size" | "include" | "header_padding" | "tail") => {}
            _ => item_count += 1,
        }
    }

    for (line, tokens) in statements {
        let line = *line;
        match first_word(tokens) {
            Some(word) if word.ends_with(':') || word == "string" => {}
            Some("format") => {
                let value = match tokens.as_slice() {
                    [_, Token::Word(value)] => value.as_str(),
                    _ => return Err(AsmError::at(line, "expected: format hunex|buriko")),
                };
                let parsed = match value {
                    "hunex" => Format::Hunex,
                    "buriko" => Format::Buriko,
                    _ => return Err(AsmError::at(line, "format is hunex or buriko")),
                };
                if format.replace(parsed).is_some() {
                    return Err(AsmError::at(line, "format is given twice"));
                }
            }
            Some("header_size") => {
                let value = match tokens.as_slice() {
                    [_, Token::Word(value)] => parse_number(value),
                    _ => None,
                }
                .ok_or_else(|| AsmError::at(line, "expected: header_size NUMBER"))?;
                if header_size.replace(value).is_some() {
                    return Err(AsmError::at(line, "header_size is given twice"));
                }
            }
            Some("include") => match tokens.as_slice() {
                [_, Token::Quoted(name)] => includes.push(name.clone()),
                _ => return Err(AsmError::at(line, "expected: include \"NAME\"")),
            },
            Some("header_padding") => {
                let mut bytes = Vec::new();
                for token in &tokens[1..] {
                    let Token::Word(word) = token else {
                        return Err(AsmError::at(line, "header_padding takes hex bytes"));
                    };
                    let byte = (word.len() == 2)
                        .then(|| u8::from_str_radix(word, 16).ok())
                        .flatten()
                        .ok_or_else(|| AsmError::at(line, "header_padding takes hex bytes"))?;
                    bytes.push(byte);
                }
                if header_padding.replace(bytes).is_some() {
                    return Err(AsmError::at(line, "header_padding is given twice"));
                }
            }
            Some("tail") => match tokens.as_slice() {
                [_, Token::Quoted(bytes)] => {
                    if tail.replace(bytes.clone()).is_some() {
                        return Err(AsmError::at(line, "tail is given twice"));
                    }
                }
                _ => return Err(AsmError::at(line, "expected: tail \"TEXT\"")),
            },
            _ => {
                let item = instruction(
                    tokens,
                    line,
                    &label_names,
                    &string_names,
                    &mut strings,
                    label_targets.len(),
                )?;
                items.push(item);
            }
        }
    }

    let format = format.ok_or_else(|| AsmError::global("missing format directive"))?;
    let header_size =
        header_size.ok_or_else(|| AsmError::global("missing header_size directive"))?;
    Program::from_parts(ProgramParts {
        format,
        header_size,
        includes,
        header_padding: header_padding.unwrap_or_default(),
        items,
        labels: label_targets,
        strings,
        tail: tail.unwrap_or_default(),
    })
    .map_err(|error| AsmError::global(error.to_string()))
}

fn string_statement(tokens: &[Token], line: usize) -> Result<(String, bool, Vec<u8>), AsmError> {
    let usage = "expected: string [new] S_NAME \"TEXT\"";
    match tokens {
        [_, Token::Word(name), Token::Quoted(bytes)] => Ok((name.clone(), true, bytes.clone())),
        [_, Token::Word(new), Token::Word(name), Token::Quoted(bytes)] if new == "new" => {
            Ok((name.clone(), false, bytes.clone()))
        }
        _ => Err(AsmError::at(line, usage)),
    }
}

fn instruction(
    tokens: &[Token],
    line: usize,
    labels: &BTreeMap<String, LabelId>,
    string_names: &BTreeMap<String, usize>,
    strings: &mut Vec<PoolEntry>,
    label_count: usize,
) -> Result<Item, AsmError> {
    let word = first_word(tokens).ok_or_else(|| AsmError::at(line, "expected an instruction"))?;
    let opcode = match OPCODES.iter().find(|info| info.mnemonic == Some(word)) {
        Some(info) => info.id,
        None => {
            let id = parse_number(word)
                .filter(|_| word.starts_with("0x"))
                .ok_or_else(|| AsmError::at(line, format!("unknown instruction {word}")))?;
            if opcode_info(id).is_none() {
                return Err(AsmError::at(line, format!("unregistered opcode {word}")));
            }
            id
        }
    };

    let mut operands = Vec::new();
    for token in &tokens[1..] {
        let operand = match token {
            Token::Quoted(bytes) => {
                strings.push(
                    PoolEntry::new(bytes.clone(), false)
                        .map_err(|error| AsmError::at(line, error.to_string()))?,
                );
                Operand::Str(strings.len() - 1)
            }
            Token::Word(name) if name.starts_with("L_") => Operand::Label(
                *labels
                    .get(name)
                    .ok_or_else(|| AsmError::at(line, format!("undefined label {name}")))?,
            ),
            Token::Word(name) if name.starts_with("S_") => Operand::Str(
                *string_names
                    .get(name)
                    .ok_or_else(|| AsmError::at(line, format!("undefined string {name}")))?,
            ),
            Token::Word(number) => Operand::Dword(
                parse_number(number)
                    .ok_or_else(|| AsmError::at(line, format!("bad operand {number}")))?,
            ),
        };
        operands.push(operand);
    }
    let item = Item::new(opcode, operands);
    item.check(label_count, strings.len())
        .map_err(|error| AsmError::at(line, error.to_string()))?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hunex_chs::Script;

    fn synthetic() -> Vec<u8> {
        let mut out = b"HunexCompiledScriptVer1.00\0".to_vec();
        out.extend_from_slice(&21u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.resize(0x30, 0);
        let code: [u32; 8] = [0x01, 0x08, 0x03, 0x20, 0x7F, 0x23, 9, 0x1B];
        for word in code {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(b"ab\0\xff\n\"x\0cd");
        out
    }

    #[test]
    fn dump_and_build_reproduce_the_bytes() {
        let bytes = synthetic();
        let program = Script::parse(&bytes).unwrap().lift().unwrap();
        let text = dump(&program);
        assert_eq!(build(text.as_bytes()).unwrap(), bytes, "{text}");
        assert_eq!(assemble(text.as_bytes()).unwrap(), program);
    }

    #[test]
    fn dump_escapes_bytes_that_are_not_text() {
        assert_eq!(quote(b"a\n\"\\\xff"), "\"a\\n\\\"\\\\\\xff\"");
        assert_eq!(quote("\u{65e5}\u{672c}".as_bytes()), "\"\u{65e5}\u{672c}\"");
    }

    fn error_at(text: &str) -> AsmError {
        assemble(text.as_bytes()).unwrap_err()
    }

    const HEADER: &str = "format hunex\nheader_size 21\n";

    #[test]
    fn errors_carry_line_numbers() {
        let unknown = error_at(&format!("{HEADER}    Bogus 1\n"));
        assert_eq!(unknown.line, Some(3));
        assert!(unknown.message.contains("unknown instruction"));

        let undefined = error_at(&format!("{HEADER}    PushOffset L_0009\n"));
        assert_eq!(undefined.line, Some(3));
        assert!(undefined.message.contains("undefined label"));

        let duplicate = error_at(&format!("{HEADER}L_0000:\n    0x1b\nL_0000:\n"));
        assert_eq!(duplicate.line, Some(5));

        let unterminated = error_at(&format!("{HEADER}string S_0000 \"abc\n"));
        assert_eq!(unterminated.line, Some(3));
        assert!(unterminated.message.contains("unterminated"));

        let escape = error_at(&format!("{HEADER}string S_0000 \"a\\q\"\n"));
        assert_eq!(escape.line, Some(3));

        let hex = error_at(&format!("{HEADER}string S_0000 \"\\x4g\"\n"));
        assert_eq!(hex.line, Some(3));

        let control = error_at(&format!("{HEADER}string S_0000 \"a\tb\"\n"));
        assert_eq!(control.line, Some(3));

        let shape = error_at(&format!("{HEADER}    PushOffset 5\n"));
        assert_eq!(shape.line, Some(3));
        assert!(shape.message.contains("wrong kind"));

        let missing = error_at("header_size 21\n");
        assert_eq!(missing.line, None);
        assert!(missing.message.contains("format"));
    }

    #[test]
    fn garbage_never_panics() {
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        for _ in 0..5_000 {
            let mut text = Vec::new();
            for _ in 0..(state % 64) {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let pool = b"\"\\ \tabL_S_0x:\n\x9f";
                text.push(pool[(state % pool.len() as u64) as usize]);
            }
            let _ = assemble(&text);
        }
    }
}
