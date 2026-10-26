//! Compile Switch script text to a Steam `.chs` file.
//!
//! The scene number map is either a general `--scene-map` file (one
//! `<number> <scene name>` per line) or, for the Mahoyo Switch layout, a
//! `nam.bin` file via `--names`: 32-byte records whose name is the first 30
//! bytes up to NUL, resolved to scene numbers by record index (see
//! [`parse_names`]). Exactly one of the two options is required.
//!
//! Errors and `--warnings all` lines name `line:column` positions (1-based).
//! By default, selections of zero-emission rules that are not proven on
//! the corpus print one summary line per command to stderr; compile errors
//! exit non-zero.
//!
//! Script text is the Switch format with no comments: every statement is
//! `_NAME(arguments);`.

use std::collections::BTreeMap;
use std::fs;
use std::process::ExitCode;

use crate::cli::{finish, parse, usage, CliError};

const USAGE: &str = "usage: hunex chs compile <script.txt> (--names <nam.bin> | --scene-map <map.txt>) -o <out.chs> [--warnings all]\n";

/// Where the `_JUMP` scene number map comes from.
enum SceneSource<'a> {
    /// A Mahoyo Switch `nam.bin` file, resolved by record index.
    Names(&'a str),
    /// A general scene map file, one `<number> <scene name>` per line.
    Map(&'a str),
}

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    if args
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        out!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let Ok(parsed) = parse(args, &["--names", "--scene-map", "--warnings", "-o"], &[]) else {
        return usage(USAGE);
    };
    let [input] = parsed.positional.as_slice() else {
        return usage(USAGE);
    };
    let Some(output) = parsed.value("-o") else {
        return usage(USAGE);
    };
    match parsed.value("--warnings").unwrap_or("summary") {
        "all" | "summary" => {}
        _ => return usage(USAGE),
    }
    let warnings_all = parsed.value("--warnings") == Some("all");
    let source = match (parsed.value("--names"), parsed.value("--scene-map")) {
        (Some(names), None) => SceneSource::Names(names),
        (None, Some(map)) => SceneSource::Map(map),
        _ => return usage(USAGE),
    };
    finish(compile_file(input, source, output, warnings_all))
}

/// The `_JUMP` scene number of a zero-based Mahoyo Switch `nam.bin` record, or
/// `None` when the record has no observed destination mapping.
fn scene_number(record: usize) -> Option<u32> {
    match record {
        1..=137 => Some(record as u32 + 199),
        138..=150 => Some(record as u32 + 362),
        151..=219 => Some(record as u32 + 849),
        220..=245 => Some(record as u32 + 1780),
        247 => Some(4000),
        _ => None,
    }
}

/// Parses a Mahoyo Switch `nam.bin` file into `(decimal number, scene name)`
/// pairs in record order. Empty records and records without a scene number are
/// skipped before the CRLF trailer check, so unmapped records are never errors.
pub fn parse_names(bytes: &[u8]) -> Result<Vec<(String, String)>, String> {
    if !bytes.len().is_multiple_of(32) {
        return Err(format!("size {} is not a multiple of 32", bytes.len()));
    }
    let mut out = Vec::new();
    for (record, chunk) in bytes.chunks_exact(32).enumerate() {
        let end = chunk[..30].iter().position(|&byte| byte == 0).unwrap_or(30);
        if end == 0 {
            continue;
        }
        let Some(number) = scene_number(record) else {
            continue;
        };
        if chunk[30..32] != [b'\r', b'\n'] {
            return Err(format!("record {record} has no CRLF trailer"));
        }
        let name = core::str::from_utf8(&chunk[..end])
            .map_err(|_| format!("record {record} name is not UTF-8"))?;
        out.push((number.to_string(), name.to_owned()));
    }
    Ok(out)
}

/// Parses a general scene map: UTF-8 text with one `<number> <scene name>` per
/// line. Blank lines and `#` comment lines are allowed; the number is decimal.
pub fn parse_scene_map(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut fields = trimmed.split_whitespace();
        match (fields.next(), fields.next(), fields.next()) {
            (Some(number), Some(name), None)
                if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                out.push((number.to_owned(), name.to_owned()));
            }
            _ => {
                return Err(format!(
                    "line {}: expected `<number> <scene name>`",
                    index + 1
                ));
            }
        }
    }
    Ok(out)
}

/// The script text the parser's character offsets index into: the file with a
/// leading byte-order mark removed, matching [`hunex_chs::source::parse`].
fn script_body(text: &str) -> &str {
    text.trim_start_matches('\u{FEFF}')
}

/// The 1-based `(line, column)` of a Unicode character offset in script text.
fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    for (index, cell) in text.chars().enumerate() {
        if index >= offset {
            break;
        }
        if cell == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

/// One summary line for the unproven zero-emission selections, grouped by
/// command, or `None` when there are none.
fn warning_summary(warnings: &[hunex_chs::compile::CompileWarning]) -> Option<String> {
    if warnings.is_empty() {
        return None;
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for warning in warnings {
        *counts.entry(warning.command()).or_insert(0) += 1;
    }
    let total: usize = counts.values().sum();
    let parts: Vec<String> = counts
        .iter()
        .map(|(command, count)| format!("{command} {count}"))
        .collect();
    Some(format!(
        "warning: {total} statements use zero-emission rules not proven on the corpus: {}",
        parts.join(", ")
    ))
}

fn compile_file(
    script: &str,
    source: SceneSource<'_>,
    output: &str,
    warnings_all: bool,
) -> Result<(), CliError> {
    let text = fs::read_to_string(script)
        .map_err(|error| CliError::Message(format!("{script}: {error}")))?;
    let pairs: Vec<(String, String)> = match source {
        SceneSource::Names(names) => {
            let nam =
                fs::read(names).map_err(|error| CliError::Message(format!("{names}: {error}")))?;
            parse_names(&nam).map_err(|error| CliError::Message(format!("{names}: {error}")))?
        }
        SceneSource::Map(map) => {
            let listed = fs::read_to_string(map)
                .map_err(|error| CliError::Message(format!("{map}: {error}")))?;
            parse_scene_map(&listed)
                .map_err(|error| CliError::Message(format!("{map}: {error}")))?
        }
    };
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(number, name)| (number.as_str(), name.as_str()))
        .collect();
    let body = script_body(&text);
    let statements = hunex_chs::source::parse(&text).map_err(|error| {
        let (line, column) = line_col(body, error.offset());
        CliError::Message(format!("{script}:{line}:{column}: {error}"))
    })?;
    let compiled = hunex_chs::compile::compile(&statements, &refs).map_err(|error| {
        let (line, column) = line_col(body, error.offset());
        CliError::Message(format!("{script}:{line}:{column}: {error}"))
    })?;
    if warnings_all {
        for warning in compiled.warnings() {
            let (line, column) = line_col(body, warning.offset());
            eprintln!(
                "{script}:{line}:{column}: {}({}): {}",
                warning.command(),
                warning.args().join(","),
                warning.reason()
            );
        }
    } else if let Some(summary) = warning_summary(compiled.warnings()) {
        eprintln!("{summary}");
    }
    let script = compiled
        .program()
        .lower()
        .map_err(|error| CliError::Message(format!("{script}: lower: {error}")))?;
    let bytes = script.encode();
    fs::write(output, &bytes).map_err(|error| CliError::Message(format!("{output}: {error}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{line_col, parse_names, parse_scene_map, scene_number, warning_summary};

    fn record(name: &str) -> Vec<u8> {
        let mut record = vec![0u8; 32];
        record[..name.len()].copy_from_slice(name.as_bytes());
        record[30] = b'\r';
        record[31] = b'\n';
        record
    }

    #[test]
    fn scene_numbers_follow_the_record_table() {
        assert_eq!(scene_number(0), None);
        assert_eq!(scene_number(1), Some(200));
        assert_eq!(scene_number(137), Some(336));
        assert_eq!(scene_number(138), Some(500));
        assert_eq!(scene_number(150), Some(512));
        assert_eq!(scene_number(151), Some(1000));
        assert_eq!(scene_number(219), Some(1068));
        assert_eq!(scene_number(220), Some(2000));
        assert_eq!(scene_number(245), Some(2025));
        assert_eq!(scene_number(246), None);
        assert_eq!(scene_number(247), Some(4000));
        assert_eq!(scene_number(248), None);
    }

    #[test]
    fn names_parse_skips_empty_and_unmapped_records() {
        let mut bytes = vec![0u8; 32];
        bytes.extend(record("1_0"));
        bytes.extend(record("1_1"));
        let names = parse_names(&bytes).unwrap();
        assert_eq!(
            names,
            vec![
                ("200".to_owned(), "1_0".to_owned()),
                ("201".to_owned(), "1_1".to_owned()),
            ]
        );
    }

    #[test]
    fn names_rejects_a_bad_size_and_a_bad_trailer() {
        assert!(parse_names(&[0u8; 31]).is_err());
        let mut bytes = vec![0u8; 32];
        let mut bad = record("1_0");
        bad[30] = 0;
        bytes.extend(bad);
        assert!(parse_names(&bytes).is_err());
    }

    #[test]
    fn unmapped_records_skip_the_trailer_check() {
        let mut bad = record("no-map");
        bad[30] = 0;
        bad[31] = 0;
        let names = parse_names(&bad).unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn scene_map_parses_numbers_names_comments_and_blanks() {
        let map = parse_scene_map("# scenes\n\n200 1_0\n201 1_1\n").unwrap();
        assert_eq!(
            map,
            vec![
                ("200".to_owned(), "1_0".to_owned()),
                ("201".to_owned(), "1_1".to_owned()),
            ]
        );
    }

    #[test]
    fn scene_map_rejects_bad_lines() {
        assert!(parse_scene_map("200\n").is_err());
        assert!(parse_scene_map("200 1_0 extra\n").is_err());
        assert!(parse_scene_map("abc 1_0\n").is_err());
    }

    #[test]
    fn positions_count_lines_and_columns_from_one() {
        assert_eq!(line_col("_PGST(1);_PGST(2);", 0), (1, 1));
        assert_eq!(line_col("_PGST(1);_PGST(2);", 9), (1, 10));
        assert_eq!(line_col("_PGST(1);\n_PGST(2);", 10), (2, 1));
        assert_eq!(line_col("_PGST(1);\n_PGST(2);", 12), (2, 3));
    }

    #[test]
    fn unproven_selections_summarize_per_command() {
        let statements = hunex_chs::source::parse("_WKST(F822,eq,1);_WKST(F822,eq,1);").unwrap();
        let compiled = hunex_chs::compile::compile(&statements, &[]).unwrap();
        assert_eq!(compiled.warnings().len(), 2);
        assert_eq!(
            warning_summary(compiled.warnings()).as_deref(),
            Some("warning: 2 statements use zero-emission rules not proven on the corpus: _WKST 2")
        );
        assert_eq!(warning_summary(&[]), None);
    }
}
