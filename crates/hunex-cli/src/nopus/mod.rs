//! Nintendo OPUS audio: info, and remux to Ogg Opus.

use std::fs;
use std::process::ExitCode;

use hunex_nopus::Stream;

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex nopus: Nintendo OPUS audio (Switch stream files)

Usage:
  hunex nopus info <in.opus>               channels, rate, packets and length
  hunex nopus to-ogg <in.opus> -o <out.ogg>  remux to Ogg Opus, packets unchanged
  hunex nopus help

Ogg Opus is an intermediate. To use the audio in the game, decode it and encode Ogg Vorbis
(for example with ffmpeg and libvorbis), then wrap it with `hunex hw wrap`.
";

const INFO_USAGE: &str = "usage: hunex nopus info <in.opus>\n";
const TO_OGG_USAGE: &str = "usage: hunex nopus to-ogg <in.opus> -o <out.ogg>\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("info") => info(&args[1..]),
        Some("to-ogg") => to_ogg(&args[1..]),
        Some(other) => {
            eprintln!("hunex nopus: unknown command `{other}`");
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
    let stream = Stream::parse(&bytes).map_err(|e| message(path, e))?;
    let seconds = stream.decoded_samples() as f64 / f64::from(hunex_nopus::SAMPLE_RATE_OUT);
    println!("file size:      {} bytes", bytes.len());
    println!("channels:       {}", stream.channels);
    println!("input rate:     {} Hz", stream.sample_rate);
    println!("pre-skip:       {} samples", stream.pre_skip);
    println!("packets:        {}", stream.packets.len());
    println!(
        "total samples:  {} (48 kHz, with pre-skip)",
        stream.total_samples()
    );
    println!(
        "decoded length: {} samples ({seconds:.6} s)",
        stream.decoded_samples()
    );
    Ok(())
}

fn to_ogg(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(TO_OGG_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(TO_OGG_USAGE);
    };
    finish(to_ogg_file(input, output))
}

fn to_ogg_file(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| message(input, e))?;
    let stream = Stream::parse(&bytes).map_err(|e| message(input, e))?;
    let ogg = stream.to_ogg().map_err(|e| message(input, e))?;
    fs::write(output, ogg).map_err(|e| message(output, e))?;
    Ok(())
}

fn message(context: &str, error: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
}
