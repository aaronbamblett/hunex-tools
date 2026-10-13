//! HuneX compiled scripts (`.chs`): listing, round-trip checks and the text form.

pub mod asm;
mod dump;

use std::fs;
use std::process::ExitCode;

use hunex_chs::Script;

use crate::cli::{finish, usage, CliError};

/// Help text for the `chs` (compiled scripts) command.
pub const HELP: &str = "\
hunex chs: HuneX compiled scripts (.chs)

Usage:
  hunex chs dump <file.chs>             print the instruction list
  hunex chs dump --strings <file.chs>   print only the string operands
  hunex chs dump --asm <file.chs>       print the text form (re-assemblable)
  hunex chs roundtrip <file.chs>        re-encode and compare with the file
  hunex chs asm <in.txt> -o <out.chs>   assemble the text form into a script
  hunex chs help                        this text
";

const ROUNDTRIP_USAGE: &str = "usage: hunex chs roundtrip <file.chs>\n";
const ASSEMBLE_USAGE: &str = "usage: hunex chs asm <in.txt> -o <out.chs>\n";

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("dump") => dump::run(&args[1..]),
        Some("roundtrip") => roundtrip(&args[1..]),
        Some("asm") => assemble(&args[1..]),
        Some(other) => {
            eprintln!("hunex chs: unknown command `{other}`");
            usage(HELP)
        }
    }
}

pub(crate) fn read_script(path: &str) -> Result<(Vec<u8>, Script), CliError> {
    let bytes = fs::read(path).map_err(|error| CliError::Message(format!("{path}: {error}")))?;
    let script =
        Script::parse(&bytes).map_err(|error| CliError::Message(format!("{path}: {error}")))?;
    Ok((bytes, script))
}

fn roundtrip(args: &[String]) -> ExitCode {
    let [path] = args else {
        return usage(ROUNDTRIP_USAGE);
    };
    finish(check_roundtrip(path))
}

fn check_roundtrip(path: &str) -> Result<(), CliError> {
    let (bytes, script) = read_script(path)?;
    let encoded = script.encode();
    if encoded == bytes {
        outln!("identical");
        return Ok(());
    }
    let offset = encoded
        .iter()
        .zip(&bytes)
        .position(|(a, b)| a != b)
        .unwrap_or(encoded.len().min(bytes.len()));
    Err(CliError::Message(format!(
        "{path}: differs at 0x{offset:x} (original {} bytes, re-encoded {} bytes)",
        bytes.len(),
        encoded.len()
    )))
}

fn assemble(args: &[String]) -> ExitCode {
    let mut input = None;
    let mut output = None;
    let mut arguments = args.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-o" => match arguments.next() {
                Some(value) => output = Some(value.clone()),
                None => return usage(ASSEMBLE_USAGE),
            },
            other if other.starts_with('-') || input.is_some() => return usage(ASSEMBLE_USAGE),
            other => input = Some(other.to_string()),
        }
    }
    let (Some(input), Some(output)) = (input, output) else {
        return usage(ASSEMBLE_USAGE);
    };
    finish(assemble_file(&input, &output))
}

fn assemble_file(input: &str, output: &str) -> Result<(), CliError> {
    let text = fs::read(input).map_err(|error| CliError::Message(format!("{input}: {error}")))?;
    let bytes =
        asm::build(&text).map_err(|error| CliError::Message(format!("{input}: {error}")))?;
    fs::write(output, &bytes).map_err(|error| CliError::Message(format!("{output}: {error}")))?;
    Ok(())
}
