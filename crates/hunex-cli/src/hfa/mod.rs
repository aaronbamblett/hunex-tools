//! HFA script archives: list, unpack into a directory with a manifest, and pack back.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::ExitCode;

use hunex_hfa::{Archive, Entry};
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

fn list_archive(path: &str) -> Result<(), CliError> {
    let bytes = fs::read(path).map_err(|e| error(path, e))?;
    let archive = Archive::parse(&bytes).map_err(|e| error(path, e))?;
    let mut text = format!("{} entries\n", archive.entries().len());
    for (index, entry) in archive.entries().iter().enumerate() {
        let kind = if lenzu_codec::is_lenzu(entry.data()) {
            "lenzu"
        } else {
            "raw"
        };
        text.push_str(&format!(
            "{index:>4}  {:>9}  {kind:<5}  {}\n",
            entry.size(),
            entry.name()
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
    let bytes = fs::read(path).map_err(|e| error(path, e))?;
    let archive = Archive::parse(&bytes).map_err(|e| error(path, e))?;

    let mut files = Vec::new();
    let mut manifest = format!("{MANIFEST_HEADER}\n");
    for entry in archive.entries() {
        check_file_name(entry.name()).map_err(|e| error(path, e))?;
        let (flag, content) = if decompress && lenzu_codec::is_lenzu(entry.data()) {
            let plain = lenzu_codec::decompress(entry.data())
                .map_err(|e| error(&format!("{path}: {}", entry.name()), e))?;
            ('1', plain)
        } else {
            ('0', entry.data().to_vec())
        };
        manifest.push_str(&format!("{flag}\t{}\n", entry.name()));
        files.push((entry.name().to_string(), content));
    }

    let dir = Path::new(dir);
    fs::create_dir_all(dir).map_err(|e| error(&dir.display().to_string(), e))?;
    for (name, content) in &files {
        let target = dir.join(name);
        fs::write(&target, content).map_err(|e| error(&target.display().to_string(), e))?;
    }
    let manifest_path = dir.join(MANIFEST);
    fs::write(&manifest_path, manifest)
        .map_err(|e| error(&manifest_path.display().to_string(), e))?;
    println!("unpacked {} entries to {}", files.len(), dir.display());
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

/// Names become file names in the unpack directory, so they must not name a path.
pub(crate) fn check_file_name(name: &str) -> Result<(), String> {
    let bad = name.is_empty()
        || name == "."
        || name == ".."
        || name.len() > 255
        || name == MANIFEST
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        });
    if bad {
        Err(format!("entry name `{name}` cannot be a file name"))
    } else {
        Ok(())
    }
}
