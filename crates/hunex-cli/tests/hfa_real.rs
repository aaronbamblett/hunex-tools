//! Real-archive checks, gated on HUNEX_HFA_DIR (the folder with data01000.hfa .. data01400.hfa).
//! The stored chs entries are read from the sibling `../chs` folder and the decompressed
//! reference scripts from `../raw`. HUNEX_WORK picks the scratch folder. The checks that write to it are
//! skipped when it is not set, and the end-to-end test removes its folder when it finishes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hunex_hfa::Archive;
use hunex_lenzu as lenzu;

const ARCHIVES: [&str; 5] = [
    "data01000",
    "data01100",
    "data01200",
    "data01300",
    "data01400",
];

fn hfa_dir() -> Option<PathBuf> {
    match std::env::var_os("HUNEX_HFA_DIR") {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("HUNEX_HFA_DIR is not set; skipping HFA checks");
            None
        }
    }
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
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

#[test]
fn archives_parse_and_stored_entries_match_the_chs_folder() {
    let Some(dir) = hfa_dir() else {
        return;
    };
    let chs = dir.join("../chs");
    let mut total = 0;
    for name in ARCHIVES {
        let bytes = read(&dir.join(format!("{name}.hfa")));
        let archive = Archive::parse(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
        for entry in archive.entries() {
            assert_eq!(
                entry.data(),
                read(&chs.join(entry.name())).as_slice(),
                "{name}/{}: stored entry differs from chs/",
                entry.name()
            );
            total += 1;
        }
    }
    println!("stored entries: {total} match chs/");
    assert_eq!(total, 250);
}

#[test]
fn entries_decompress_to_the_raw_scripts_and_compress_literal_round_trips() {
    let Some(dir) = hfa_dir() else {
        return;
    };
    let raw = dir.join("../raw");
    let mut decompressed = 0;
    for name in ARCHIVES {
        let bytes = read(&dir.join(format!("{name}.hfa")));
        let archive = Archive::parse(&bytes).unwrap();
        for entry in archive.entries() {
            let plain = lenzu::decompress(entry.data())
                .unwrap_or_else(|error| panic!("{name}/{}: {error}", entry.name()));
            let reference = read(&raw.join(entry.name()));
            assert!(
                plain == reference,
                "{name}/{}: decompressed bytes differ from raw/",
                entry.name()
            );
            let literal = lenzu::compress_literal(&plain);
            assert_eq!(
                lenzu::decompress(&literal).unwrap(),
                plain,
                "{name}/{}: literal round trip",
                entry.name()
            );
            decompressed += 1;
        }
    }
    println!("decompressed {decompressed} entries; all match raw/; literal round trips pass");
    assert_eq!(decompressed, 250);
}

#[test]
fn archives_rewrite_byte_for_byte_from_their_entries() {
    let Some(dir) = hfa_dir() else {
        return;
    };
    for name in ARCHIVES {
        let bytes = read(&dir.join(format!("{name}.hfa")));
        let archive = Archive::parse(&bytes).unwrap();
        assert!(archive.write().unwrap() == bytes, "{name}: rewrite differs");
    }
    println!("all five archives rewrite byte-for-byte");
}

fn work_root() -> Option<PathBuf> {
    let dir = std::env::var_os("HUNEX_WORK").map(PathBuf::from);
    if dir.is_none() {
        eprintln!("HUNEX_WORK is not set; skipping the archive round trip");
    }
    dir.map(|dir| dir.join(format!("hunex-tools-e2e-{}", std::process::id())))
}

/// Replaces the quoted text of the first `string S_` line of an assembly listing.
fn edit_first_string(listing: &[u8]) -> Vec<u8> {
    let mut lines: Vec<Vec<u8>> = listing
        .split(|&byte| byte == b'\n')
        .map(<[u8]>::to_vec)
        .collect();
    let line = lines
        .iter_mut()
        .find(|line| line.starts_with(b"string S_") && line.contains(&b'"'))
        .expect("listing has a string line");
    let open = line.iter().position(|&byte| byte == b'"').unwrap();
    line.truncate(open);
    line.extend_from_slice(b"\"hunex edit test\"");
    lines.join(&b'\n')
}

#[test]
fn end_to_end_unpack_pack_and_edit_one_script() {
    let Some(dir) = hfa_dir() else {
        return;
    };
    let Some(work) = work_root() else {
        return;
    };
    fs::create_dir_all(&work).unwrap();

    for name in ARCHIVES.into_iter().chain(["data00200"]) {
        let original = dir.join(format!("{name}.hfa"));
        let unpacked = work.join(format!("{name}.dir"));
        let repacked = work.join(format!("{name}.repacked.hfa"));
        hunex(&[
            "hfa",
            "unpack",
            original.to_str().unwrap(),
            "-o",
            unpacked.to_str().unwrap(),
            "--decompress",
        ]);
        hunex(&[
            "hfa",
            "pack",
            unpacked.to_str().unwrap(),
            "-o",
            repacked.to_str().unwrap(),
            "--base",
            original.to_str().unwrap(),
        ]);
        assert!(
            read(&repacked) == read(&original),
            "{name}: unchanged repack differs"
        );
    }
    println!("end to end: unchanged unpack/pack cycles are byte-identical for the five script archives and data00200");

    let original = dir.join("data01100.hfa");
    let edit_dir = work.join("edit.dir");
    let listing = work.join("1_0.asm");
    let edited_chs = edit_dir.join("1_0.chs");
    let edited_hfa = work.join("edited.hfa");
    let again_dir = work.join("again.dir");
    hunex(&[
        "hfa",
        "unpack",
        original.to_str().unwrap(),
        "-o",
        edit_dir.to_str().unwrap(),
        "--decompress",
    ]);
    let asm = hunex(&["chs", "dump", "--asm", edited_chs.to_str().unwrap()]);
    fs::write(&listing, edit_first_string(&asm.stdout)).unwrap();
    hunex(&[
        "chs",
        "asm",
        listing.to_str().unwrap(),
        "-o",
        edited_chs.to_str().unwrap(),
    ]);
    let edited = read(&edited_chs);
    hunex(&[
        "hfa",
        "pack",
        edit_dir.to_str().unwrap(),
        "-o",
        edited_hfa.to_str().unwrap(),
        "--base",
        original.to_str().unwrap(),
    ]);
    hunex(&[
        "hfa",
        "unpack",
        edited_hfa.to_str().unwrap(),
        "-o",
        again_dir.to_str().unwrap(),
        "--decompress",
    ]);
    assert!(
        read(&again_dir.join("1_0.chs")) == edited,
        "edited 1_0.chs did not survive the pack"
    );

    let original_bytes = read(&original);
    let before = Archive::parse(&original_bytes).unwrap();
    let after_bytes = read(&edited_hfa);
    let after = Archive::parse(&after_bytes).unwrap();
    let mut changed = Vec::new();
    for (old, new) in before.entries().iter().zip(after.entries()) {
        assert_eq!(old.name(), new.name());
        if old.data() != new.data() {
            changed.push(old.name().to_string());
        }
    }
    assert_eq!(changed, ["1_0.chs"], "only the edited entry may change");
    println!("end to end: edited 1_0.chs survives; only that entry changed in the archive");

    fs::remove_dir_all(&work).unwrap();
}

#[test]
fn data00000_unpacks_and_packs_back_byte_for_byte() {
    let Some(dir) = hfa_dir() else {
        return;
    };
    let original = dir.join("data00000.hfa");
    if !original.exists() {
        eprintln!("data00000.hfa is not in HUNEX_HFA_DIR; skipping");
        return;
    }
    // The archive is 157 MB; its copies go to HUNEX_WORK (a folder under /var/tmp), not /tmp.
    let Some(base) = std::env::var_os("HUNEX_WORK").map(PathBuf::from) else {
        eprintln!("HUNEX_WORK is not set; skipping the data00000 round trip");
        return;
    };
    let work = base.join(format!("data00000-{}", std::process::id()));
    fs::create_dir_all(&work).unwrap();
    let archive = path_str(&original);

    let raw = work.join("raw");
    hunex(&["hfa", "unpack", archive, "-o", path_str(&raw)]);
    let manifest = fs::read_to_string(raw.join("hfa-manifest.txt")).unwrap();
    assert!(manifest.contains("\tface_on_ja(2).cbg\tface_on_ja.cbg\n"));
    let packed = work.join("raw.hfa");
    hunex(&["hfa", "pack", path_str(&raw), "-o", path_str(&packed)]);
    assert_eq!(read(&packed), read(&original), "raw unpack and pack differ");

    let decompressed = work.join("decompressed");
    hunex(&[
        "hfa",
        "unpack",
        archive,
        "-o",
        path_str(&decompressed),
        "--decompress",
    ]);
    let again = work.join("decompressed.hfa");
    hunex(&[
        "hfa",
        "pack",
        path_str(&decompressed),
        "-o",
        path_str(&again),
        "--base",
        archive,
    ]);
    assert_eq!(
        read(&again),
        read(&original),
        "decompressed unpack and pack differ"
    );

    fs::remove_dir_all(&work).unwrap();
}

fn path_str(path: &Path) -> &str {
    path.to_str().unwrap()
}
