use std::fmt;
use std::process::ExitCode;

use crate::{
    chs, ctd, font, hfa, hw, image, lenzu, mrg, mzp, mzx, nopus, nx, switch_script, switch_text,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const HELP: &str = "\
hunex: tools for HuneX engine file formats

Usage:
  hunex <format> <command> [options]
  hunex help
  hunex --version

Formats:
  chs     compiled scripts (.chs): dump, roundtrip, asm
  hfa     script archives (.hfa): list, unpack, pack
  lenzu   LenZu compressed streams: decompress, compress
  ctd     script text (.ctd): export, import, get
  hw      audio (.hw): info, unwrap, wrap
  font    glyph tables (.ccit): info, export, import, render
  nopus   Nintendo OPUS audio: info, to-ogg
  mzp     image containers (.mzp): list, unpack, pack
  image   background images: to-png, from-png
  mzx     MZX0 image tile streams: decompress, compress
  mrg     Switch archive sets (.hed/.mrg/.nam): list, unpack
  nx      Switch textures (NXX/BNTX): info, to-png
  switch-text    Switch script text: export
  switch-script  Switch scripts (allscr.mrg): dump

Run `hunex <format> help` for the commands of one format.
";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => {
            eprint!("{HELP}");
            ExitCode::from(2)
        }
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version" | "version") => {
            println!("hunex {VERSION}");
            ExitCode::SUCCESS
        }
        Some("chs") => chs::run(&args[1..]),
        Some("hfa") => hfa::run(&args[1..]),
        Some("lenzu") => lenzu::run(&args[1..]),
        Some("ctd") => ctd::run(&args[1..]),
        Some("hw") => hw::run(&args[1..]),
        Some("font") => font::run(&args[1..]),
        Some("nopus") => nopus::run(&args[1..]),
        Some("mzp") => mzp::run(&args[1..]),
        Some("image") => image::run(&args[1..]),
        Some("mzx") => mzx::run(&args[1..]),
        Some("mrg") => mrg::run(&args[1..]),
        Some("nx") => nx::run(&args[1..]),
        Some("switch-text") => switch_text::run(&args[1..]),
        Some("switch-script") => switch_script::run(&args[1..]),
        Some(other) => {
            eprintln!("hunex: unknown command `{other}`");
            eprint!("{HELP}");
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
pub(crate) enum CliError {
    Io(std::io::Error),
    Message(String),
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::Io(error)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(error) => write!(f, "{error}"),
            CliError::Message(message) => write!(f, "{message}"),
        }
    }
}

/// Prints the result of a command. A closed output pipe is not an error.
pub(crate) fn finish(result: Result<(), CliError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Io(error)) if error.kind() == std::io::ErrorKind::BrokenPipe => {
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("hunex: {error}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) fn usage(text: &str) -> ExitCode {
    eprint!("{text}");
    ExitCode::from(2)
}

/// Options of one command: value options take the next argument, flags stand alone.
pub(crate) struct Parsed<'a> {
    pub positional: Vec<&'a str>,
    values: Vec<(&'a str, &'a str)>,
    flags: Vec<&'a str>,
}

impl<'a> Parsed<'a> {
    pub fn value(&self, name: &str) -> Option<&'a str> {
        self.values
            .iter()
            .find(|(option, _)| *option == name)
            .map(|(_, value)| *value)
    }

    pub fn values(&self, name: &str) -> Vec<&'a str> {
        self.values
            .iter()
            .filter(|(option, _)| *option == name)
            .map(|(_, value)| *value)
            .collect()
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.contains(&name)
    }
}

pub(crate) fn parse<'a>(
    args: &'a [String],
    values: &[&str],
    flags: &[&str],
) -> Result<Parsed<'a>, String> {
    let mut parsed = Parsed {
        positional: Vec::new(),
        values: Vec::new(),
        flags: Vec::new(),
    };
    let mut arguments = args.iter();
    while let Some(argument) = arguments.next() {
        let argument = argument.as_str();
        if values.contains(&argument) {
            let value = arguments
                .next()
                .ok_or_else(|| format!("{argument} needs a value"))?;
            parsed.values.push((argument, value.as_str()));
        } else if flags.contains(&argument) {
            parsed.flags.push(argument);
        } else if argument.starts_with('-') {
            return Err(format!("unknown option {argument}"));
        } else {
            parsed.positional.push(argument);
        }
    }
    Ok(parsed)
}
