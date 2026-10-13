//! Switch RomFS checks, gated on HUNEX_TSUKI_ROMFS, HUNEX_MAHOYO_ROMFS, HUNEX_TSUKI_DUMP and
//! HUNEX_MAHOYO_DUMP. Scratch files go to HUNEX_WORK, which should be a folder under /var/tmp
//! (not /tmp, which is RAM here); the tests remove what they create.
//!
//! Memory: every check reads archives entry by entry (seek and read), never a whole data file,
//! except the 12 MB script text archives, which are read whole.

mod common;

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;

use common::sha256_hex;

fn romfs(var: &str) -> Option<PathBuf> {
    match std::env::var_os(var) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("{var} is not set; skipping");
            None
        }
    }
}

fn work_root() -> PathBuf {
    let root = match std::env::var_os("HUNEX_WORK") {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir(),
    };
    root.join(format!("hunex-switch-test-{}", std::process::id()))
}

fn hunex(args: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_hunex"))
        .args(args)
        .output()
        .expect("run hunex");
    assert!(
        output.status.success(),
        "hunex {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Index and names of a set, from its .hed and .nam files (small; read whole).
fn set_index(dir: &Path, base: &str) -> Vec<hunex_mrg::Meta> {
    let hed = fs::read(dir.join(format!("{base}.hed"))).unwrap();
    let nam = fs::read(dir.join(format!("{base}.nam"))).ok();
    hunex_mrg::index(&hed, nam.as_deref()).unwrap()
}

/// Reads one entry's stored bytes from a set's data file, with a seek.
fn read_entry(file: &mut fs::File, meta: &hunex_mrg::Meta) -> Vec<u8> {
    let length = meta.row.byte_span() as usize;
    let mut buffer = vec![0u8; length];
    file.seek(SeekFrom::Start(meta.row.byte_start())).unwrap();
    let read = file.read(&mut buffer).unwrap();
    buffer.truncate(read);
    buffer
}

#[test]
fn allscr_scripts_decode_byte_identical_to_the_dumps() {
    for (romfs_var, dump_var, expected) in [
        ("HUNEX_TSUKI_ROMFS", "HUNEX_TSUKI_DUMP", 549usize),
        ("HUNEX_MAHOYO_ROMFS", "HUNEX_MAHOYO_DUMP", 249usize),
    ] {
        let (Some(romfs), Some(dump)) = (romfs(romfs_var), romfs(dump_var)) else {
            continue;
        };
        let work = work_root().join(romfs_var);
        fs::create_dir_all(&work).unwrap();
        let input = romfs.join("allscr.mrg");
        hunex(&[
            "switch-script",
            "dump",
            input.to_str().unwrap(),
            "-o",
            work.to_str().unwrap(),
        ]);
        let mut identical = 0usize;
        let mut differing = Vec::new();
        for index in 0..expected {
            let name = format!("scr{index:04}.txt");
            let ours = fs::read(work.join(&name)).unwrap();
            let theirs = fs::read(dump.join(&name)).unwrap();
            if ours == theirs {
                identical += 1;
            } else {
                differing.push(name);
            }
        }
        assert_eq!(
            fs::read(work.join("nam.bin")).unwrap(),
            fs::read(dump.join("nam.bin")).unwrap()
        );
        println!("{romfs_var}: {identical}/{expected} scripts identical to the dumps");
        assert!(differing.is_empty(), "{romfs_var}: differing {differing:?}");
        fs::remove_dir_all(&work).unwrap();
    }
}

#[test]
fn entry_counts_match_the_spec_tables() {
    let Some(romfs) = romfs("HUNEX_TSUKI_ROMFS") else {
        return;
    };
    let expected = [
        ("allpachd", 14_765usize, 14_705usize),
        ("allpachdml", 67, 0),
        ("w_allpac", 29_267, 0),
        ("w_allpaccg", 1_521, 0),
        ("w_allpacml", 344, 0),
        ("w_allui", 48, 0),
        ("parts", 5, 0),
    ];
    for (set, entries, empty) in expected {
        let index = set_index(&romfs, set);
        let counted_empty = index.iter().filter(|meta| meta.row.is_empty()).count();
        assert_eq!(index.len(), entries, "{set}: entries");
        assert_eq!(counted_empty, empty, "{set}: empty slots");
    }
    let extensions = |set: &str| {
        let mut counts = std::collections::BTreeMap::new();
        for meta in set_index(&romfs, set) {
            let Some(name) = meta.name else { continue };
            if name == hunex_mrg::PLACEHOLDER {
                continue;
            }
            let extension = name.rsplit('.').next().unwrap_or("").to_string();
            *counts.entry(extension).or_insert(0usize) += 1;
        }
        counts
    };
    let w_allpac = extensions("w_allpac");
    assert_eq!(w_allpac.get("NXZ"), Some(&7_294));
    assert_eq!(w_allpac.get("NXGZ"), Some(&20_688));
    assert_eq!(w_allpac.get("JPG"), Some(&1_273));
    assert_eq!(w_allpac.get("PNG"), Some(&6));
    assert_eq!(w_allpac.get("MZP"), Some(&6));
    let allpachd = extensions("allpachd");
    assert_eq!(allpachd.get("NXZ"), Some(&5));
    assert_eq!(allpachd.get("NXGZ"), Some(&49));
    assert_eq!(allpachd.get("MZP"), Some(&6));

    if let Some(mahoyo) = romfs_path("HUNEX_MAHOYO_ROMFS") {
        for (set, entries) in [
            ("allpachdml", 89usize),
            ("allpacml", 167),
            ("allui", 70),
            ("jallpac", 22_156),
            ("jallpachd", 11_414),
            ("parts", 2),
        ] {
            assert_eq!(set_index(&mahoyo, set).len(), entries, "mahoyo {set}");
        }
    }
    println!("entry counts match the spec tables");
}

fn romfs_path(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).map(PathBuf::from)
}

fn decode_named(romfs: &Path, set: &str, index: usize, name: &str) -> hunex_nx::Rgba {
    let meta = set_index(romfs, set).into_iter().nth(index).unwrap();
    assert_eq!(meta.name.as_deref(), Some(name), "{set} entry {index}");
    let mut file = fs::File::open(romfs.join(format!("{set}.mrg"))).unwrap();
    let stored = read_entry(&mut file, &meta);
    let plain = hunex_nx::unwrap(&stored).unwrap();
    let container = hunex_nx::Bntx::parse(&plain).unwrap();
    hunex_nx::decode_texture(&container, 0).unwrap()
}

#[test]
fn the_two_spec_backgrounds_decode_to_their_hashes() {
    if let Some(romfs) = romfs("HUNEX_TSUKI_ROMFS") {
        let image = decode_named(&romfs, "w_allpac", 2598, "IMG5012.NXGZ");
        assert_eq!((image.width, image.height), (1280, 720));
        let hash = sha256_hex(&image.pixels);
        println!("IMG5012 {}x{} sha256 {hash}", image.width, image.height);
        assert_eq!(
            hash,
            "ef58e090d43ed16f91c058d47feb16f1e53fe86f1ddac744d45d70530d879579"
        );
    }
    if let Some(romfs) = romfs("HUNEX_MAHOYO_ROMFS") {
        let image = decode_named(&romfs, "jallpac", 850, "IMG0179.NXZ");
        assert_eq!((image.width, image.height), (1400, 840));
        let hash = sha256_hex(&image.pixels);
        println!("IMG0179 {}x{} sha256 {hash}", image.width, image.height);
        assert_eq!(
            hash,
            "96de0854b88fa552ddcba7e1ee6ea3072f9bea4dc483b8e2889be5d70a3ce8b6"
        );
    }
}

#[test]
fn script_text_line_counts_and_lookup() {
    for (var, lines) in [
        ("HUNEX_TSUKI_ROMFS", 43_969usize),
        ("HUNEX_MAHOYO_ROMFS", 24_134usize),
    ] {
        let Some(romfs) = romfs(var) else {
            continue;
        };
        let bytes = fs::read(romfs.join("script_text.mrg")).unwrap();
        let container = hunex_mzp::parse_switch(&bytes).unwrap();
        let pairs = hunex_switch_text::pairs(&container.entries).unwrap();
        assert_eq!(pairs.len(), 5, "{var}: pairs");
        for (index, pair) in pairs.iter().enumerate() {
            assert_eq!(pair.len(), lines, "{var}: pair {index} line count");
        }
        assert!(
            pairs[1].lookup("$000000").unwrap().is_some(),
            "{var}: first English line"
        );
        println!("{var}: five pairs of {lines} lines");
    }
}

#[test]
fn sampled_textures_decode_across_sets_with_no_errors() {
    let games = [
        (
            "HUNEX_TSUKI_ROMFS",
            vec!["w_allpac", "w_allpacml", "w_allui", "allpachd"],
        ),
        (
            "HUNEX_MAHOYO_ROMFS",
            vec!["jallpac", "jallpachd", "allui", "allpacml"],
        ),
    ];
    let target = 300usize;
    let total_sets: usize = games.iter().map(|(_, sets)| sets.len()).sum();
    let per_set = target / total_sets;
    let mut decoded = std::collections::BTreeMap::new();
    let mut unsupported = std::collections::BTreeMap::new();
    let mut errors = Vec::new();
    let mut entries = 0usize;
    for (var, sets) in games {
        let Some(romfs) = romfs(var) else {
            continue;
        };
        for set in sets {
            let mut file = fs::File::open(romfs.join(format!("{set}.mrg"))).unwrap();
            let index = set_index(&romfs, set);
            let candidates: Vec<&hunex_mrg::Meta> = index
                .iter()
                .filter(|meta| {
                    !meta.row.is_empty()
                        && meta
                            .name
                            .as_deref()
                            .is_some_and(|n| n.ends_with(".NXZ") || n.ends_with(".NXGZ"))
                })
                .collect();
            let step = (candidates.len() / per_set).max(1);
            for meta in candidates.iter().step_by(step).take(per_set) {
                let stored = read_entry(&mut file, meta);
                entries += 1;
                let outcome = hunex_nx::unwrap(&stored).and_then(|plain| {
                    let container = hunex_nx::Bntx::parse(&plain)?;
                    let mut formats = Vec::new();
                    for texture in 0..container.textures.len() {
                        formats.push(container.textures[texture].format);
                        hunex_nx::decode_texture(&container, texture)?;
                    }
                    Ok(formats)
                });
                match outcome {
                    Ok(formats) => {
                        for format in formats {
                            *decoded.entry(format).or_insert(0usize) += 1;
                        }
                    }
                    Err(hunex_nx::Error::UnsupportedFormat { code, .. }) => {
                        *unsupported.entry(code).or_insert(0usize) += 1;
                    }
                    Err(error) => errors.push(format!("{set} {}: {error}", meta.index)),
                }
            }
        }
    }
    println!("sampled {entries} NX entries across {total_sets} sets");
    for (format, count) in &decoded {
        println!("  format 0x{format:04x}: {count} textures decoded");
    }
    for (format, count) in &unsupported {
        println!(
            "  format 0x{format:04x}: {count} entries not decoded (BC3 channel reading is open)"
        );
    }
    assert!(errors.is_empty(), "decode errors: {errors:?}");
}

#[test]
fn text_export_writes_one_line_per_script_id() {
    let Some(romfs) = romfs("HUNEX_TSUKI_ROMFS") else {
        return;
    };
    let work = work_root().join("text");
    fs::create_dir_all(&work).unwrap();
    let output = work.join("lines.txt");
    hunex(&[
        "switch-text",
        "export",
        romfs.join("script_text.mrg").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--pair",
        "1",
    ]);
    let text = fs::read_to_string(&output).unwrap();
    assert_eq!(text.lines().count(), 43_969);
    assert!(text.lines().next().unwrap().starts_with("$000000\t"));
    fs::remove_dir_all(&work).unwrap();
}
