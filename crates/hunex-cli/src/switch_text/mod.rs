//! Switch script text: export one language of `script_text.mrg` as `$` lines.

use std::fs;
use std::process::ExitCode;

use hunex_mzp::parse_switch;
use hunex_switch_text::{format_ref, pairs};

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex switch-text: Switch script text (script_text.mrg)

Usage:
  hunex switch-text export <script_text.mrg> -o <lines.txt> [--pair N]
  hunex switch-text help

Writes one line per script id: $000000, a TAB, then the text. --pair picks the language
(0 to 4; default 1). Backslash, tab, CR and LF are escaped as \\\\, \\t, \\r and \\n.
";

const EXPORT_USAGE: &str =
    "usage: hunex switch-text export <script_text.mrg> -o <lines.txt> [--pair N]\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("export") => export(&args[1..]),
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
    println!("exported {} lines of pair {pair} to {output}", chosen.len());
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
