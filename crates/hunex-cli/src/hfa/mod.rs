//! HFA script archives: list, unpack into a directory with a manifest, and pack back.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::path::Path;
use std::process::ExitCode;

use hunex_hfa::{read_toc, table_len, Archive, Entry, TocEntry};
use hunex_lenzu as lenzu_codec;

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex hfa: HFA script archives (.hfa)

Usage:
  hunex hfa list <archive.hfa>
  hunex hfa unpack <archive.hfa> -o <dir> [--decompress]
  hunex hfa pack <dir> -o <archive.hfa> [--base <original.hfa>]
  hunex hfa help

unpack writes each entry as a file, plus hfa-manifest.txt, which records the order and
which entries were decompressed. pack rebuilds the archive from that directory. With --base,
an entry whose content matches the base archive's entry reuses the base's stored bytes.
";

const MANIFEST: &str = "hfa-manifest.txt";
const MANIFEST_HEADER: &str = "hunex-hfa-manifest 1";
const LIST_USAGE: &str = "usage: hunex hfa list <archive.hfa>\n";
const UNPACK_USAGE: &str = "usage: hunex hfa unpack <archive.hfa> -o <dir> [--decompress]\n";
const PACK_USAGE: &str = "usage: hunex hfa pack <dir> -o <archive.hfa> [--base <original.hfa>]\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("list") => list(&args[1..]),
        Some("unpack") => unpack(&args[1..]),
        Some("pack") => pack(&args[1..]),
        Some(other) => {
            eprintln!("hunex hfa: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn error(context: &str, message: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {message}"))
}

fn list(args: &[String]) -> ExitCode {
    let parsed = match parse(args, &[], &[]) {
        Ok(parsed) => parsed,
        Err(_) => return usage(LIST_USAGE),
    };
    let [path] = parsed.positional.as_slice() else {
        return usage(LIST_USAGE);
    };
    finish(list_archive(path))
}

/// Reads the header and table of an archive file. Returns the table's length and its entries;
/// the entry data is not read.
fn read_table(file: &mut fs::File, path: &str) -> Result<(usize, Vec<TocEntry>), CliError> {
    let file_len = file.metadata().map_err(|e| error(path, e))?.len();
    let mut head = [0u8; 16];
    file.read_exact(&mut head).map_err(|e| error(path, e))?;
    let len = table_len(&head).map_err(|e| error(path, e))?;
    if len as u64 > file_len {
        return Err(error(path, "the table is longer than the file"));
    }
    let mut table = vec![0u8; len];
    file.seek(SeekFrom::Start(0)).map_err(|e| error(path, e))?;
    file.read_exact(&mut table).map_err(|e| error(path, e))?;
    let data_len = (file_len - len as u64) as usize;
    let toc = read_toc(&table, data_len).map_err(|e| error(path, e))?;
    Ok((len, toc))
}

/// Reads `size` bytes of one entry's data.
fn read_bytes(file: &mut fs::File, path: &str, at: u64, size: usize) -> Result<Vec<u8>, CliError> {
    file.seek(SeekFrom::Start(at)).map_err(|e| error(path, e))?;
    let mut bytes = vec![0u8; size];
    file.read_exact(&mut bytes).map_err(|e| error(path, e))?;
    Ok(bytes)
}

fn list_archive(path: &str) -> Result<(), CliError> {
    let mut file = fs::File::open(path).map_err(|e| error(path, e))?;
    let (table_len, toc) = read_table(&mut file, path)?;
    let mut text = format!("{} entries\n", toc.len());
    for (index, entry) in toc.iter().enumerate() {
        let start = (table_len + entry.offset) as u64;
        let prefix = read_bytes(&mut file, path, start, entry.size.min(16))?;
        let kind = if lenzu_codec::is_lenzu(&prefix) {
            "lenzu"
        } else {
            "raw"
        };
        text.push_str(&format!(
            "{index:>4}  {:>9}  {kind:<5}  {}\n",
            entry.size, entry.name
        ));
    }
    std::io::stdout().lock().write_all(text.as_bytes())?;
    Ok(())
}

fn unpack(args: &[String]) -> ExitCode {
    let parsed = match parse(args, &["-o"], &["--decompress"]) {
        Ok(parsed) => parsed,
        Err(_) => return usage(UNPACK_USAGE),
    };
    let (&[path], Some(dir)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(UNPACK_USAGE);
    };
    finish(unpack_archive(path, dir, parsed.flag("--decompress")))
}

fn unpack_archive(path: &str, dir: &str, decompress: bool) -> Result<(), CliError> {
    let mut file = fs::File::open(path).map_err(|e| error(path, e))?;
    let (table_len, toc) = read_table(&mut file, path)?;
    for entry in &toc {
        check_file_name(&entry.name).map_err(|e| error(path, e))?;
    }
    check_unique_names(toc.iter().map(|entry| entry.name.as_str())).map_err(|e| error(path, e))?;

    let dir = Path::new(dir);
    fs::create_dir_all(dir).map_err(|e| error(&dir.display().to_string(), e))?;
    let mut manifest = format!("{MANIFEST_HEADER}\n");
    for entry in &toc {
        let at = (table_len + entry.offset) as u64;
        let stored = read_bytes(&mut file, path, at, entry.size)?;
        let (flag, content) = if decompress && lenzu_codec::is_lenzu(&stored) {
            let plain = lenzu_codec::decompress(&stored)
                .map_err(|e| error(&format!("{path}: {}", entry.name), e))?;
            ('1', plain)
        } else {
            ('0', stored)
        };
        let target = dir.join(&entry.name);
        fs::write(&target, &content).map_err(|e| error(&target.display().to_string(), e))?;
        manifest.push_str(&format!("{flag}\t{}\n", entry.name));
    }
    let manifest_path = dir.join(MANIFEST);
    fs::write(&manifest_path, manifest)
        .map_err(|e| error(&manifest_path.display().to_string(), e))?;
    println!("unpacked {} entries to {}", toc.len(), dir.display());
    Ok(())
}

fn pack(args: &[String]) -> ExitCode {
    let parsed = match parse(args, &["-o", "--base"], &[]) {
        Ok(parsed) => parsed,
        Err(_) => return usage(PACK_USAGE),
    };
    let (&[dir], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(PACK_USAGE);
    };
    finish(pack_archive(dir, output, parsed.value("--base")))
}

struct ManifestLine {
    decompressed: bool,
    name: String,
}

fn read_manifest(dir: &Path) -> Result<Vec<ManifestLine>, CliError> {
    let path = dir.join(MANIFEST);
    let text = fs::read_to_string(&path).map_err(|e| error(&path.display().to_string(), e))?;
    let mut lines = text.lines();
    if lines.next() != Some(MANIFEST_HEADER) {
        return Err(error(&path.display().to_string(), "not an hfa manifest"));
    }
    let mut entries = Vec::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (flag, name) = line
            .split_once('\t')
            .ok_or_else(|| error(&path.display().to_string(), format!("bad line `{line}`")))?;
        let decompressed = match flag {
            "0" => false,
            "1" => true,
            _ => {
                return Err(error(
                    &path.display().to_string(),
                    format!("bad flag in `{line}`"),
                ))
            }
        };
        check_file_name(name).map_err(|e| error(&path.display().to_string(), e))?;
        entries.push(ManifestLine {
            decompressed,
            name: name.to_string(),
        });
    }
    Ok(entries)
}

fn pack_archive(dir: &str, output: &str, base: Option<&str>) -> Result<(), CliError> {
    let dir = Path::new(dir);
    let manifest = read_manifest(dir)?;
    let base_bytes = match base {
        Some(path) => Some(fs::read(path).map_err(|e| error(path, e))?),
        None => None,
    };
    let base_archive = match (&base_bytes, base) {
        (Some(bytes), Some(path)) => Some(Archive::parse(bytes).map_err(|e| error(path, e))?),
        _ => None,
    };
    let base_entries: BTreeMap<&str, &Entry> = base_archive
        .iter()
        .flat_map(|archive| archive.entries().iter())
        .map(|entry| (entry.name(), entry))
        .collect();

    let mut entries = Vec::with_capacity(manifest.len());
    for line in &manifest {
        let path = dir.join(&line.name);
        let content = fs::read(&path).map_err(|e| error(&path.display().to_string(), e))?;
        let stored = if line.decompressed {
            match base_entries.get(line.name.as_str()) {
                Some(base_entry) if same_content(base_entry.data(), &content) => {
                    Cow::Borrowed(base_entry.data())
                }
                _ => Cow::Owned(lenzu_codec::compress_literal(&content)),
            }
        } else {
            Cow::Owned(content)
        };
        entries.push(Entry::new(line.name.clone(), stored));
    }

    let packed = Archive::new(entries)
        .write()
        .map_err(|e| error(&dir.display().to_string(), e))?;
    fs::write(output, &packed).map_err(|e| error(output, e))?;
    println!("packed {} entries into {output}", manifest.len());
    Ok(())
}

/// True when the stored bytes hold the same content as `plain`, decompressing LenZu streams.
fn same_content(stored: &[u8], plain: &[u8]) -> bool {
    if lenzu_codec::is_lenzu(stored) {
        lenzu_codec::decompress(stored).is_ok_and(|decoded| decoded == plain)
    } else {
        stored == plain
    }
}

/// Names become file names in the unpack directory, so they must not name a path, a device on
/// Windows, or something Windows cannot store (a trailing dot or space).
pub(crate) fn check_file_name(name: &str) -> Result<(), String> {
    let bad = name.is_empty()
        || name == "."
        || name == ".."
        || name.len() > 255
        || name == MANIFEST
        || name.ends_with('.')
        || name.ends_with(' ')
        || reserved_device(name)
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        });
    if bad {
        Err(format!("entry name `{name}` cannot be a file name"))
    } else {
        Ok(())
    }
}

/// True for the Windows device names (CON, PRN, AUX, NUL, COM1 to COM9, LPT1 to LPT9), with or
/// without an extension, in any case.
fn reserved_device(name: &str) -> bool {
    let base = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    match base.as_str() {
        "CON" | "PRN" | "AUX" | "NUL" => true,
        other => {
            let digits = other
                .strip_prefix("COM")
                .or_else(|| other.strip_prefix("LPT"));
            matches!(
                digits,
                Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            )
        }
    }
}

/// Rejects two names that would name the same file on a case-insensitive file system.
pub(crate) fn check_unique_names<'a>(names: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = BTreeMap::new();
    for name in names {
        if let Some(first) = seen.insert(name.to_lowercase(), name) {
            return Err(format!("entries `{first}` and `{name}` name the same file"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_file_name, check_unique_names};

    #[test]
    fn windows_device_names_and_trailing_dots_or_spaces_are_rejected() {
        for name in [
            "CON", "con.bin", "PRN.txt", "aux", "NUL.x", "COM1", "com9.dat", "LPT1", "lpt9.z",
            "foo.", "foo ",
        ] {
            assert!(check_file_name(name).is_err(), "{name} should be rejected");
        }
        for name in [
            "CONX",
            "COM10",
            "LPT0",
            "console.bin",
            "a.con",
            "script_text.ctd",
        ] {
            assert!(check_file_name(name).is_ok(), "{name} should be accepted");
        }
    }

    #[test]
    fn case_insensitive_duplicates_are_rejected() {
        assert!(check_unique_names(["a.bin", "b.bin"].into_iter()).is_ok());
        assert!(check_unique_names(["Font.ctd", "font.CTD"].into_iter()).is_err());
    }
}
