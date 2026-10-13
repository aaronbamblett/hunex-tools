//! Switch script text: export one language of `script_text.mrg` as `$` lines, and import lines
//! back into the container.

use std::fs;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::process::ExitCode;

use hunex_mzp::{header_for, padding_len, parse_switch, switch_header_len, switch_spans, PADDING};
use hunex_switch_text::{build_pair, format_ref, pairs, ref_index};

use crate::cli::{finish, parse, usage, CliError};

/// Help text for the `switch-text` (Switch script text) command.
pub const HELP: &str = "\
hunex switch-text: Switch script text (script_text.mrg)

Usage:
  hunex switch-text export <script_text.mrg> -o <lines.txt> [--pair N]
  hunex switch-text import <lines.txt> -o <script_text.mrg> --base <original.mrg> [--pair N]
  hunex switch-text help

Writes one line per script id: $000000, a TAB, then the text. --pair picks the language
(0 to 4; default 1). Backslash, tab, CR and LF are escaped as \\\\, \\t, \\r and \\n.

import rebuilds the container from --base, replacing one language pair with the lines of the
file. The file must have the same ids, in order, and the same number of lines as the pair.
Every other entry keeps its stored bytes.
";

const EXPORT_USAGE: &str =
    "usage: hunex switch-text export <script_text.mrg> -o <lines.txt> [--pair N]\n";
const IMPORT_USAGE: &str = "usage: hunex switch-text import <lines.txt> -o <script_text.mrg> --base <original.mrg> [--pair N]\n";

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("export") => export(&args[1..]),
        Some("import") => import(&args[1..]),
        Some(other) => {
            eprintln!("hunex switch-text: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn export(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o", "--pair"], &[]) else {
        return usage(EXPORT_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(EXPORT_USAGE);
    };
    let pair = match parsed.value("--pair") {
        Some(value) => match value.parse::<usize>() {
            Ok(pair) => pair,
            Err(_) => return usage(EXPORT_USAGE),
        },
        None => 1,
    };
    finish(export_file(input, output, pair))
}

fn export_file(input: &str, output: &str, pair: usize) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let container = parse_switch(&bytes).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let all = pairs(&container.entries).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let chosen = all
        .get(pair)
        .ok_or_else(|| CliError::Message(format!("{input}: no language pair {pair}")))?;
    let mut text = String::new();
    for index in 0..chosen.len() {
        let line = chosen
            .line(index)
            .map_err(|e| CliError::Message(format!("{input}: {e}")))?
            .unwrap_or("");
        text.push_str(&format!("{}\t{}\n", format_ref(index), escape(line)));
    }
    fs::write(output, text).map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    outln!("exported {} lines of pair {pair} to {output}", chosen.len());
    Ok(())
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

fn import(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o", "--base", "--pair"], &[]) else {
        return usage(IMPORT_USAGE);
    };
    let (&[input], Some(output), Some(base)) = (
        parsed.positional.as_slice(),
        parsed.value("-o"),
        parsed.value("--base"),
    ) else {
        return usage(IMPORT_USAGE);
    };
    let pair = match parsed.value("--pair") {
        Some(value) => match value.parse::<usize>() {
            Ok(pair) => pair,
            Err(_) => return usage(IMPORT_USAGE),
        },
        None => 1,
    };
    finish(import_file(input, output, base, pair))
}

/// Reads an exported file: one `$id<TAB>text` line per id, in order.
fn read_lines(input: &str) -> Result<Vec<Vec<u8>>, CliError> {
    let text = fs::read_to_string(input).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let mut lines = Vec::new();
    for (number, raw) in text.lines().enumerate() {
        let (id, value) = raw.split_once('\t').ok_or_else(|| {
            CliError::Message(format!("{input}: line {}: no TAB after the id", number + 1))
        })?;
        if ref_index(id) != Some(lines.len()) {
            return Err(CliError::Message(format!(
                "{input}: line {}: expected {}, found {id}",
                number + 1,
                format_ref(lines.len())
            )));
        }
        lines.push(unescape(value).map_err(|bad| {
            CliError::Message(format!(
                "{input}: line {}: unknown escape \\{bad}",
                number + 1
            ))
        })?);
    }
    Ok(lines)
}

fn unescape(value: &str) -> Result<Vec<u8>, char> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('n') => out.push('\n'),
            Some(other) => return Err(other),
            None => return Err('\\'),
        }
    }
    Ok(out.into_bytes())
}

fn import_file(input: &str, output: &str, base: &str, pair: usize) -> Result<(), CliError> {
    let message = |path: &str, e: &dyn std::fmt::Display| CliError::Message(format!("{path}: {e}"));
    let lines = read_lines(input)?;
    let mut file = fs::File::open(base).map_err(|e| message(base, &e))?;
    let mut first = [0u8; 8];
    file.read_exact(&mut first).map_err(|e| message(base, &e))?;
    let header_len = switch_header_len(&first).map_err(|e| message(base, &e))?;
    let mut table = vec![0u8; header_len];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut table).map_err(|e| message(base, &e))?;
    let spans = switch_spans(&table).map_err(|e| message(base, &e))?;
    let (table_entry, text_entry) = (2 * pair, 2 * pair + 1);
    if text_entry >= spans.len() {
        return Err(CliError::Message(format!(
            "{base}: no language pair {pair}"
        )));
    }
    let base_lines = {
        let mut stored = vec![0u8; spans[table_entry].size];
        file.seek(SeekFrom::Start(
            header_len as u64 + spans[table_entry].offset,
        ))?;
        file.read_exact(&mut stored)
            .map_err(|e| message(base, &e))?;
        let values = stored.len() / 4;
        values.saturating_sub(3)
    };
    if base_lines != lines.len() {
        return Err(CliError::Message(format!(
            "{input}: has {} lines, but pair {pair} of {base} has {base_lines}",
            lines.len()
        )));
    }
    let refs: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let (new_table, new_text) = build_pair(&refs).map_err(|e| message(input, &e))?;

    enum Source {
        Base(usize),
        Bytes(usize),
    }
    let mut lengths = Vec::with_capacity(spans.len());
    let mut sources = Vec::with_capacity(spans.len());
    let replacements = [new_table, new_text];
    for (index, span) in spans.iter().enumerate() {
        if index == table_entry || index == text_entry {
            let which = index - table_entry;
            lengths.push(replacements[which].len());
            sources.push(Source::Bytes(which));
        } else {
            lengths.push(span.size);
            sources.push(Source::Base(index));
        }
    }
    let header = header_for(&lengths).map_err(|e| message(output, &e))?;
    let out = fs::File::create(output).map_err(|e| message(output, &e))?;
    let mut out = BufWriter::new(out);
    out.write_all(&header)?;
    for (index, source) in sources.iter().enumerate() {
        match source {
            Source::Base(entry) => {
                let mut buffer = vec![0u8; spans[*entry].size];
                file.seek(SeekFrom::Start(header_len as u64 + spans[*entry].offset))?;
                file.read_exact(&mut buffer)
                    .map_err(|e| message(base, &e))?;
                out.write_all(&buffer)?;
            }
            Source::Bytes(which) => out.write_all(&replacements[*which])?,
        }
        out.write_all(&vec![PADDING; padding_len(lengths[index])])?;
    }
    out.flush()?;
    outln!(
        "imported {} lines into pair {pair} of {output}",
        lines.len()
    );
    Ok(())
}
