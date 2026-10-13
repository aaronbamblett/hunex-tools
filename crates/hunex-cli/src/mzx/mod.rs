//! PC MZX0 streams: decompress, and compress with the greedy encoder.

use std::fs;
use std::process::ExitCode;

use hunex_mzx::{compress, decompress};

use crate::cli::{finish, parse, usage, CliError};

/// Help text for the `mzx` (MZX0 streams) command.
pub const HELP: &str = "\
hunex mzx: PC MZX0 streams (image tiles)

Usage:
  hunex mzx decompress <in> -o <out>
  hunex mzx compress <in> -o <out>   greedy: literals and back-references
  hunex mzx help
";

const DECOMPRESS_USAGE: &str = "usage: hunex mzx decompress <in> -o <out>\n";
const COMPRESS_USAGE: &str = "usage: hunex mzx compress <in> -o <out>\n";

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("decompress") => convert(&args[1..], DECOMPRESS_USAGE, true),
        Some("compress") => convert(&args[1..], COMPRESS_USAGE, false),
        Some(other) => {
            eprintln!("hunex mzx: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn convert(args: &[String], help: &str, decode: bool) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(help);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(help);
    };
    finish(run_conversion(input, output, decode))
}

fn run_conversion(input: &str, output: &str, decode: bool) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let converted = if decode {
        decompress(&bytes).map_err(|e| CliError::Message(format!("{input}: {e}")))?
    } else {
        compress(&bytes)
    };
    fs::write(output, converted).map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    Ok(())
}
