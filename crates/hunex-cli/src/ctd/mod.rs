//! Script text (`.ctd`): export lines as text, import them back, and print lines by id.

use std::fs;
use std::io::{self, Write as _};
use std::process::ExitCode;

use hunex_ctd::{format_id, parse_id, Text, MAX_LINES};

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex ctd: script text (.ctd)

Usage:
  hunex ctd export <file.ctd> -o <lines.txt>   one line per id: $000000, TAB, text
  hunex ctd import <lines.txt> -o <file.ctd>   ids must run $000000, $000001, ... in order
  hunex ctd get <file.ctd> <$id>...            print the lines for the given ids
  hunex ctd help

Both stored (LenZu) and decompressed files are accepted. Text escapes: \\\\ for a backslash,
\\t for a tab, \\r and \\n for CR and LF. Import writes a literal-only LenZu stream, which is
larger than the original.
";

const EXPORT_USAGE: &str = "usage: hunex ctd export <file.ctd> -o <lines.txt>\n";
const IMPORT_USAGE: &str = "usage: hunex ctd import <lines.txt> -o <file.ctd>\n";
const GET_USAGE: &str = "usage: hunex ctd get <file.ctd> <$id>...\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("export") => export(&args[1..]),
        Some("import") => import(&args[1..]),
        Some("get") => get(&args[1..]),
        Some(other) => {
            eprintln!("hunex ctd: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn export(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(EXPORT_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(EXPORT_USAGE);
    };
    finish(export_file(input, output))
}

fn export_file(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = read(input)?;
    let text = Text::from_bytes(&bytes).map_err(|e| message(input, e))?;
    if text.len() > MAX_LINES {
        return Err(message(input, "too many lines for six-digit ids"));
    }
    let mut out = String::new();
    for (index, line) in text.lines().iter().enumerate() {
        let id = format_id(index).unwrap_or_default();
        out.push_str(&format!("{id}\t{}\n", escape(line)));
    }
    fs::write(output, out).map_err(|e| message(output, e))?;
    Ok(())
}

fn import(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(IMPORT_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(IMPORT_USAGE);
    };
    finish(import_file(input, output))
}

fn import_file(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = read(input)?;
    let source = String::from_utf8(bytes).map_err(|_| message(input, "lines file is not UTF-8"))?;
    let mut text = Text::new();
    let mut pieces: Vec<&str> = source.split('\n').collect();
    if pieces.last() == Some(&"") {
        pieces.pop();
    }
    for (position, piece) in pieces.iter().enumerate() {
        let number = position + 1;
        let piece = piece.strip_suffix('\r').unwrap_or(piece);
        let (id, raw) = piece
            .split_once('\t')
            .ok_or_else(|| line_error(input, number, "expected an id, a TAB and the text"))?;
        let expected = text.len();
        match parse_id(id) {
            Some(found) if found == expected => {}
            Some(found) => {
                return Err(line_error(
                    input,
                    number,
                    &format!("expected ${expected:06}, found ${found:06}"),
                ))
            }
            None => return Err(line_error(input, number, &format!("bad id `{id}`"))),
        }
        let decoded = unescape(raw).map_err(|reason| line_error(input, number, &reason))?;
        text.push_line(&decoded)
            .map_err(|_| line_error(input, number, "a line cannot contain CR or LF"))?;
    }
    fs::write(output, text.to_stored()).map_err(|e| message(output, e))?;
    Ok(())
}

fn get(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &[], &[]) else {
        return usage(GET_USAGE);
    };
    let Some((&input, ids)) = parsed.positional.split_first() else {
        return usage(GET_USAGE);
    };
    if ids.is_empty() {
        return usage(GET_USAGE);
    }
    finish(get_lines(input, ids))
}

fn get_lines(input: &str, ids: &[&str]) -> Result<(), CliError> {
    let bytes = read(input)?;
    let text = Text::from_bytes(&bytes).map_err(|e| message(input, e))?;
    let mut out = String::new();
    for id in ids {
        let line = text
            .lookup(id)
            .ok_or_else(|| message(input, format!("no line for {id}")))?;
        out.push_str(&format!("{id}\t{}\n", escape(line)));
    }
    io::stdout().lock().write_all(out.as_bytes())?;
    Ok(())
}

fn read(path: &str) -> Result<Vec<u8>, CliError> {
    fs::read(path).map_err(|e| message(path, e))
}

fn message(context: &str, error: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
}

fn line_error(input: &str, number: usize, reason: &str) -> CliError {
    CliError::Message(format!("{input}: line {number}: {reason}"))
}

fn escape(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    for character in line.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

fn unescape(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut characters = raw.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => out.push('\\'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('n') => out.push('\n'),
            Some(other) => return Err(format!("unknown escape \\{other}")),
            None => return Err("text ends with a backslash".to_string()),
        }
    }
    Ok(out)
}
