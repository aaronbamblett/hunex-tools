//! MZP containers: list, unpack into numbered files, and pack them back.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use hunex_mzp::{parse, write};

use crate::cli::{finish, parse as parse_args, usage, CliError};

/// Help text for the `mzp` (MZP containers) command.
pub const HELP: &str = "\
hunex mzp: MZP containers (background images)

Usage:
  hunex mzp list <file.mzp>
  hunex mzp unpack <file.mzp> -o <dir>   entries as entry-000.bin, entry-001.bin, ...
  hunex mzp pack <dir> -o <file.mzp>     packs entry-000.bin, entry-001.bin, ... in order
  hunex mzp help
";

const LIST_USAGE: &str = "usage: hunex mzp list <file.mzp>\n";
const UNPACK_USAGE: &str = "usage: hunex mzp unpack <file.mzp> -o <dir>\n";
const PACK_USAGE: &str = "usage: hunex mzp pack <dir> -o <file.mzp>\n";

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("list") => list(&args[1..]),
        Some("unpack") => unpack(&args[1..]),
        Some("pack") => pack(&args[1..]),
        Some(other) => {
            eprintln!("hunex mzp: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn list(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse_args(args, &[], &[]) else {
        return usage(LIST_USAGE);
    };
    let [path] = parsed.positional.as_slice() else {
        return usage(LIST_USAGE);
    };
    finish(list_file(path))
}

fn list_file(path: &str) -> Result<(), CliError> {
    let bytes = fs::read(path).map_err(|e| message(path, e))?;
    let container = parse(&bytes).map_err(|e| message(path, e))?;
    let mut text = format!("{} entries\n", container.entries.len());
    for (index, entry) in container.entries.iter().enumerate() {
        text.push_str(&format!("{index:>4}  {:>9}\n", entry.len()));
    }
    std::io::Write::write_all(&mut std::io::stdout().lock(), text.as_bytes())?;
    Ok(())
}

fn unpack(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse_args(args, &["-o"], &[]) else {
        return usage(UNPACK_USAGE);
    };
    let (&[path], Some(dir)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(UNPACK_USAGE);
    };
    finish(unpack_file(path, dir))
}

fn unpack_file(path: &str, dir: &str) -> Result<(), CliError> {
    let bytes = fs::read(path).map_err(|e| message(path, e))?;
    let container = parse(&bytes).map_err(|e| message(path, e))?;
    fs::create_dir_all(dir).map_err(|e| message(dir, e))?;
    for (index, entry) in container.entries.iter().enumerate() {
        let target = Path::new(dir).join(format!("entry-{index:03}.bin"));
        fs::write(&target, entry).map_err(|e| message(&target.display().to_string(), e))?;
    }
    outln!("unpacked {} entries to {dir}", container.entries.len());
    Ok(())
}

fn pack(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse_args(args, &["-o"], &[]) else {
        return usage(PACK_USAGE);
    };
    let (&[dir], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(PACK_USAGE);
    };
    finish(pack_dir(dir, output))
}

fn pack_dir(dir: &str, output: &str) -> Result<(), CliError> {
    let mut entries = Vec::new();
    loop {
        let target = Path::new(dir).join(format!("entry-{:03}.bin", entries.len()));
        if !target.exists() {
            break;
        }
        entries.push(fs::read(&target).map_err(|e| message(&target.display().to_string(), e))?);
    }
    if entries.is_empty() {
        return Err(CliError::Message(format!("{dir}: no entry-000.bin file")));
    }
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    let packed = write(&refs).map_err(|e| message(output, e))?;
    fs::write(output, packed).map_err(|e| message(output, e))?;
    outln!("packed {} entries into {output}", entries.len());
    Ok(())
}

fn message(context: &str, error: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
}
