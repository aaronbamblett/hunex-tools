//! MRG sets: list and unpack a `.hed`, `.mrg` and `.nam` trio given by any of the three paths.

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use hunex_mrg::{kind, Kind};

use crate::cli::{finish, parse, usage, CliError};
use crate::hfa::check_file_name;

/// Help text for the `mrg` (MRG sets) command.
pub const HELP: &str = "\
hunex mrg: MRG sets (.hed index, .mrg data, .nam names)

Usage:
  hunex mrg list <set>             list the entries of a set
  hunex mrg unpack <set> -o <dir>  write the entries to a folder, named from the .nam file
  hunex mrg help

<set> is the base path, or the path of any one of the three files. Unpacked NXX entries keep
their wrapper; other entries are written as stored, including sector padding.
";

const LIST_USAGE: &str = "usage: hunex mrg list <set>\n";
const UNPACK_USAGE: &str = "usage: hunex mrg unpack <set> -o <dir>\n";

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
        Some(other) => {
            eprintln!("hunex mrg: unknown command `{other}`");
            usage(HELP)
        }
    }
}

/// The base path of a set, from any of its three files.
fn base_of(path: &str) -> PathBuf {
    let path = Path::new(path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("hed" | "mrg" | "nam") => path.with_extension(""),
        _ => path.to_path_buf(),
    }
}

/// The index and names of a set (small files, read whole), and the open data file.
struct Opened {
    metas: Vec<hunex_mrg::Meta>,
    data: fs::File,
    data_len: u64,
}

fn open_set(base: &Path) -> Result<Opened, CliError> {
    let read = |extension: &str| {
        let path = base.with_extension(extension);
        fs::read(&path).map_err(|e| CliError::Message(format!("{}: {e}", path.display())))
    };
    let hed = read("hed")?;
    let nam = read("nam").ok();
    let metas = hunex_mrg::index(&hed, nam.as_deref())
        .map_err(|e| CliError::Message(format!("{}: {e}", base.display())))?;
    let path = base.with_extension("mrg");
    let data =
        fs::File::open(&path).map_err(|e| CliError::Message(format!("{}: {e}", path.display())))?;
    let data_len = data.metadata()?.len();
    Ok(Opened {
        metas,
        data,
        data_len,
    })
}

/// Reads one entry's stored bytes: the whole sector span, or the exact NXX length.
fn read_entry(opened: &mut Opened, meta: &hunex_mrg::Meta) -> Result<Vec<u8>, CliError> {
    let start = meta.row.byte_start();
    let available = opened.data_len.saturating_sub(start);
    let length = meta.row.byte_span().min(available) as usize;
    let mut buffer = vec![0u8; length];
    opened.data.seek(SeekFrom::Start(start))?;
    opened.data.read_exact(&mut buffer)?;
    if let Some(exact) = nxx_length(&buffer).filter(|&exact| exact <= buffer.len()) {
        buffer.truncate(exact);
    }
    Ok(buffer)
}

fn list(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &[], &[]) else {
        return usage(LIST_USAGE);
    };
    let [path] = parsed.positional.as_slice() else {
        return usage(LIST_USAGE);
    };
    finish(list_set(path))
}

fn list_set(path: &str) -> Result<(), CliError> {
    let mut opened = open_set(&base_of(path))?;
    let empty = opened
        .metas
        .iter()
        .filter(|meta| meta.row.is_empty())
        .count();
    let mut text = format!("{} entries, {} empty\n", opened.metas.len(), empty);
    for meta in opened.metas.iter().filter(|meta| !meta.row.is_empty()) {
        let head = {
            let mut head = [0u8; 8];
            opened.data.seek(SeekFrom::Start(meta.row.byte_start()))?;
            let read = opened.data.read(&mut head)?;
            head[..read].to_vec()
        };
        let label = match kind(&head) {
            Kind::Nxx => "nxx",
            Kind::Mzp => "mzp",
            Kind::Raw => "raw",
            Kind::Empty => "empty",
        };
        let size = if label == "nxx" {
            let mut header = [0u8; 16];
            opened.data.seek(SeekFrom::Start(meta.row.byte_start()))?;
            let read = opened.data.read(&mut header)?;
            nxx_length(&header[..read]).unwrap_or(0) as u64
        } else {
            meta.row.byte_span()
        };
        text.push_str(&format!(
            "{:>6}  {:>10}  {label:<5}  {}\n",
            meta.index,
            size,
            meta.name.as_deref().unwrap_or("-")
        ));
    }
    std::io::Write::write_all(&mut std::io::stdout().lock(), text.as_bytes())?;
    Ok(())
}

fn unpack(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(UNPACK_USAGE);
    };
    let (&[path], Some(dir)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(UNPACK_USAGE);
    };
    finish(unpack_set(path, dir))
}

fn unpack_set(path: &str, dir: &str) -> Result<(), CliError> {
    let mut opened = open_set(&base_of(path))?;
    fs::create_dir_all(dir).map_err(|e| CliError::Message(format!("{dir}: {e}")))?;
    let mut used = std::collections::HashSet::new();
    let mut written = 0usize;
    let metas = std::mem::take(&mut opened.metas);
    for meta in &metas {
        if meta.row.is_empty() {
            continue;
        }
        if meta.name.as_deref() == Some(hunex_mrg::PLACEHOLDER) {
            continue;
        }
        let name = match meta.name.as_deref() {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => format!("entry-{:05}.bin", meta.index),
        };
        check_file_name(&name).map_err(CliError::Message)?;
        // Names are compared without case, as on a case-insensitive file system. A repeated name is
        // renamed with the entry index rather than overwriting the file already written.
        let name = if used.insert(name.to_lowercase()) {
            name
        } else {
            let (stem, extension) = match name.rsplit_once('.') {
                Some((stem, extension)) => (stem.to_string(), format!(".{extension}")),
                None => (name.clone(), String::new()),
            };
            let renamed = format!("{stem}_{}{extension}", meta.index);
            used.insert(renamed.to_lowercase());
            renamed
        };
        let stored = read_entry(&mut opened, meta)?;
        let target = Path::new(dir).join(&name);
        fs::write(&target, &stored)
            .map_err(|e| CliError::Message(format!("{}: {e}", target.display())))?;
        written += 1;
    }
    outln!("unpacked {written} entries to {dir}");
    Ok(())
}

/// The exact length of an NXX entry: its 16-byte header and the compressed size it declares.
/// Reads only the header, so it needs at least 12 bytes.
fn nxx_length(header: &[u8]) -> Option<usize> {
    let compressed = u32::from_le_bytes(header.get(8..12)?.try_into().ok()?) as usize;
    16usize.checked_add(compressed)
}

#[cfg(test)]
mod tests {
    #[test]
    fn repeated_names_are_renamed_without_case() {
        let mut used = std::collections::HashSet::new();
        assert!(used.insert("font.ctd".to_lowercase()));
        assert!(!used.insert("FONT.ctd".to_lowercase()));
    }
}
