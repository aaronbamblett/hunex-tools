//! LenZu compressed streams: decompress, and a literal-only compress.

use std::fs;
use std::process::ExitCode;

use hunex_lenzu as codec;

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex lenzu: LenZu compressed streams

Usage:
  hunex lenzu decompress <in> -o <out>
  hunex lenzu compress <in> -o <out>    literal only: valid in the format, but larger than the input
  hunex lenzu help
";

const DECOMPRESS_USAGE: &str = "usage: hunex lenzu decompress <in> -o <out>\n";
const COMPRESS_USAGE: &str = "usage: hunex lenzu compress <in> -o <out>\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("decompress") => convert(&args[1..], DECOMPRESS_USAGE, Mode::Decompress),
        Some("compress") => convert(&args[1..], COMPRESS_USAGE, Mode::Compress),
        Some(other) => {
            eprintln!("hunex lenzu: unknown command `{other}`");
            usage(HELP)
        }
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Decompress,
    Compress,
}

fn convert(args: &[String], help: &str, mode: Mode) -> ExitCode {
    let parsed = match parse(args, &["-o"], &[]) {
        Ok(parsed) => parsed,
        Err(_) => return usage(help),
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(help);
    };
    finish(run_conversion(input, output, mode))
}

fn run_conversion(input: &str, output: &str, mode: Mode) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let converted = match mode {
        Mode::Decompress => {
            codec::decompress(&bytes).map_err(|e| CliError::Message(format!("{input}: {e}")))?
        }
        Mode::Compress => codec::compress_literal(&bytes),
    };
    fs::write(output, &converted).map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    Ok(())
}
