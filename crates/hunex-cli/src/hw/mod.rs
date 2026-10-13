//! `.hw` audio: info, unwrap to Ogg Vorbis, and wrap an Ogg Vorbis file.

use std::fs;
use std::process::ExitCode;

use hunex_hw::{wrap, Hw};

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex hw: .hw audio (64-byte header and an Ogg Vorbis stream)

Usage:
  hunex hw info <file.hw>             header fields and the Vorbis facts
  hunex hw unwrap <file.hw> -o <out.ogg>
  hunex hw wrap <in.ogg> -o <out.hw>  the input must be Ogg Vorbis
  hunex hw help

The engine plays Ogg Vorbis. Opus is not used by the game's .hw files.
";

const INFO_USAGE: &str = "usage: hunex hw info <file.hw>\n";
const UNWRAP_USAGE: &str = "usage: hunex hw unwrap <file.hw> -o <out.ogg>\n";
const WRAP_USAGE: &str = "usage: hunex hw wrap <in.ogg> -o <out.hw>\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("info") => info(&args[1..]),
        Some("unwrap") => unwrap_command(&args[1..]),
        Some("wrap") => wrap_command(&args[1..]),
        Some(other) => {
            eprintln!("hunex hw: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn info(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &[], &[]) else {
        return usage(INFO_USAGE);
    };
    let [path] = parsed.positional.as_slice() else {
        return usage(INFO_USAGE);
    };
    finish(info_file(path))
}

fn info_file(path: &str) -> Result<(), CliError> {
    let bytes = fs::read(path).map_err(|e| message(path, e))?;
    let hw = Hw::parse(&bytes).map_err(|e| message(path, e))?;
    let seconds = f64::from(hw.sample_frames) / f64::from(hw.sample_rate);
    println!("file size:     {} bytes", bytes.len());
    println!("payload:       {} bytes (Ogg Vorbis)", hw.payload().len());
    println!("sample rate:   {} Hz", hw.sample_rate);
    println!("channels:      {}", hw.channels);
    println!("sample frames: {} ({seconds:.6} s)", hw.sample_frames);
    Ok(())
}

fn unwrap_command(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(UNWRAP_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(UNWRAP_USAGE);
    };
    finish(unwrap_file(input, output))
}

fn unwrap_file(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| message(input, e))?;
    let payload = Hw::parse(&bytes).map_err(|e| message(input, e))?;
    fs::write(output, payload.payload()).map_err(|e| message(output, e))?;
    Ok(())
}

fn wrap_command(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(WRAP_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(WRAP_USAGE);
    };
    finish(wrap_file(input, output))
}

fn wrap_file(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| message(input, e))?;
    let hw = wrap(&bytes).map_err(|e| message(input, e))?;
    fs::write(output, hw).map_err(|e| message(output, e))?;
    Ok(())
}

fn message(context: &str, error: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
}
