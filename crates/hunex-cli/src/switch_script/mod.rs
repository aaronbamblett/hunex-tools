//! Switch scripts: decompress the scripts of `allscr.mrg` to files, and rebuild the archive from
//! them.

use std::fs;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::ExitCode;

use hunex_mzp::{header_for, padding_len, switch_header_len, switch_spans, PADDING};
use hunex_mzx::switch::{compress, decompress, stream_len};

use crate::cli::{finish, parse, usage, CliError};

/// Help text for the `switch-script` (Switch scripts) command.
pub const HELP: &str = "\
hunex switch-script: Switch scripts (allscr.mrg)

Usage:
  hunex switch-script dump <allscr.mrg> -o <dir>
  hunex switch-script pack <dir> -o <allscr.mrg> --base <original.mrg>
  hunex switch-script help

dump writes scr0000.txt, scr0001.txt, ... (entry 3 onwards, decompressed, byte for byte), and
nam.bin (entry 0, the script name table).

pack rebuilds the archive. A scr file that matches the original script is stored as it was in
the original. A changed scr file is encoded again. Entries 1 and 2 and any entry without a file
come from --base. nam.bin, when present, replaces entry 0.
";

const DUMP_USAGE: &str = "usage: hunex switch-script dump <allscr.mrg> -o <dir>\n";
const PACK_USAGE: &str =
    "usage: hunex switch-script pack <dir> -o <allscr.mrg> --base <original.mrg>\n";

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("dump") => dump(&args[1..]),
        Some("pack") => pack(&args[1..]),
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
    outln!("dumped {} scripts to {output}", spans.len() - 3);
    Ok(())
}

fn pack(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o", "--base"], &[]) else {
        return usage(PACK_USAGE);
    };
    let (&[dir], Some(output), Some(base)) = (
        parsed.positional.as_slice(),
        parsed.value("-o"),
        parsed.value("--base"),
    ) else {
        return usage(PACK_USAGE);
    };
    finish(pack_file(dir, output, base))
}

/// Where the bytes of one entry come from when the archive is written.
enum Source {
    /// A byte range of the original archive.
    Base { offset: u64, len: usize },
    /// Bytes read from a file in the dump folder or encoded from one.
    Bytes(Vec<u8>),
}

impl Source {
    fn len(&self) -> usize {
        match self {
            Source::Base { len, .. } => *len,
            Source::Bytes(bytes) => bytes.len(),
        }
    }
}

fn pack_file(dir: &str, output: &str, base: &str) -> Result<(), CliError> {
    let message = |path: &str, e: &dyn std::fmt::Display| CliError::Message(format!("{path}: {e}"));
    let mut file = fs::File::open(base).map_err(|e| message(base, &e))?;
    let mut first = [0u8; 8];
    file.read_exact(&mut first).map_err(|e| message(base, &e))?;
    let header_len = switch_header_len(&first).map_err(|e| message(base, &e))?;
    let mut table = vec![0u8; header_len];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut table).map_err(|e| message(base, &e))?;
    let spans = switch_spans(&table).map_err(|e| message(base, &e))?;
    let data_start = header_len as u64;
    let stored = |file: &mut fs::File, offset: u64, len: usize| -> Result<Vec<u8>, CliError> {
        let mut buffer = vec![0u8; len];
        file.seek(SeekFrom::Start(data_start + offset))?;
        file.read_exact(&mut buffer)
            .map_err(|e| message(base, &e))?;
        Ok(buffer)
    };
    let mut sources: Vec<Source> = Vec::with_capacity(spans.len());
    // Entry 0: the name table. A nam.bin in the dump replaces it.
    let names_path = Path::new(dir).join("nam.bin");
    if names_path.exists() {
        sources.push(Source::Bytes(
            fs::read(&names_path).map_err(|e| message(&names_path.display().to_string(), &e))?,
        ));
    } else {
        sources.push(Source::Base {
            offset: spans[0].offset,
            len: spans[0].size,
        });
    }
    // Entries 1 and 2 are kept as they are.
    for span in &spans[1..3] {
        sources.push(Source::Base {
            offset: span.offset,
            len: span.size,
        });
    }
    // Scripts: a file identical to the original keeps the original stream; any other file is encoded.
    let mut unchanged = 0usize;
    let mut encoded = 0usize;
    for (number, span) in spans.iter().enumerate().skip(3) {
        let script = number - 3;
        let path = Path::new(dir).join(format!("scr{script:04}.txt"));
        let original = stored(&mut file, span.offset, span.size)?;
        let original_len = stream_len(&original)
            .map_err(|e| message(base, &format_args!("entry {number}: {e}")))?;
        let original_plain = decompress(&original)
            .map_err(|e| message(base, &format_args!("entry {number}: {e}")))?;
        if !path.exists() {
            sources.push(Source::Base {
                offset: span.offset,
                len: original_len,
            });
            unchanged += 1;
            continue;
        }
        let text = fs::read(&path).map_err(|e| message(&path.display().to_string(), &e))?;
        if text == original_plain {
            sources.push(Source::Base {
                offset: span.offset,
                len: original_len,
            });
            unchanged += 1;
        } else {
            sources.push(Source::Bytes(compress(&text)));
            encoded += 1;
        }
    }
    let lengths: Vec<usize> = sources.iter().map(Source::len).collect();
    let table = header_for(&lengths).map_err(|e| message(output, &e))?;
    let out = fs::File::create(output).map_err(|e| message(output, &e))?;
    let mut out = BufWriter::new(out);
    out.write_all(&table)?;
    for source in &sources {
        let len = source.len();
        match source {
            Source::Base { offset, .. } => {
                let bytes = stored(&mut file, *offset, len)?;
                out.write_all(&bytes)?;
            }
            Source::Bytes(bytes) => out.write_all(bytes)?,
        }
        out.write_all(&vec![PADDING; padding_len(len)])?;
    }
    out.flush()?;
    outln!(
        "packed {} entries into {output}: {unchanged} unchanged, {encoded} encoded",
        sources.len()
    );
    Ok(())
}
