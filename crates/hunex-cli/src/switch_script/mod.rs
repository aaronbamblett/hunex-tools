//! Switch scripts: decompress the scripts of `allscr.mrg` to files, like the existing dumps.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use hunex_mzp::{switch_header_len, switch_spans};
use hunex_mzx::switch::decompress;
use std::io::{Read, Seek, SeekFrom};

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex switch-script: Switch scripts (allscr.mrg)

Usage:
  hunex switch-script dump <allscr.mrg> -o <dir>
  hunex switch-script help

Writes scr0000.txt, scr0001.txt, ... (entry 3 onwards, decompressed, byte for byte), and
nam.bin (entry 0, the script name table).
";

const DUMP_USAGE: &str = "usage: hunex switch-script dump <allscr.mrg> -o <dir>\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("dump") => dump(&args[1..]),
        Some(other) => {
            eprintln!("hunex switch-script: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn dump(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(DUMP_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(DUMP_USAGE);
    };
    finish(dump_file(input, output))
}

fn dump_file(input: &str, output: &str) -> Result<(), CliError> {
    let mut file = fs::File::open(input).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let mut first = [0u8; 8];
    file.read_exact(&mut first)
        .map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let header_len =
        switch_header_len(&first).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let mut table = vec![0u8; header_len];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut table)
        .map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    let spans = switch_spans(&table).map_err(|e| CliError::Message(format!("{input}: {e}")))?;
    if spans.len() < 4 {
        return Err(CliError::Message(format!(
            "{input}: fewer than four entries"
        )));
    }
    fs::create_dir_all(output).map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    let read_span = |file: &mut fs::File, span: &hunex_mzp::Span| -> Result<Vec<u8>, CliError> {
        let mut buffer = vec![0u8; span.size];
        file.seek(SeekFrom::Start(header_len as u64 + span.offset))?;
        file.read_exact(&mut buffer)
            .map_err(|e| CliError::Message(format!("{input}: {e}")))?;
        Ok(buffer)
    };
    let names = read_span(&mut file, &spans[0])?;
    fs::write(Path::new(output).join("nam.bin"), names)
        .map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    for (index, span) in spans.iter().enumerate().skip(3) {
        let stored = read_span(&mut file, span)?;
        let plain = decompress(&stored)
            .map_err(|e| CliError::Message(format!("{input} entry {index}: {e}")))?;
        let target = Path::new(output).join(format!("scr{:04}.txt", index - 3));
        fs::write(&target, plain)
            .map_err(|e| CliError::Message(format!("{}: {e}", target.display())))?;
    }
    println!("dumped {} scripts to {output}", spans.len() - 3);
    Ok(())
}
