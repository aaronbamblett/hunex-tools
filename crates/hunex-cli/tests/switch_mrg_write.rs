//! Rebuilds every MRG set in the Switch RomFS from its own index, names and entries, and checks
//! the result against the original files. Gated: set HUNEX_SWITCH_ROMFS_A (a RomFS folder) and
//! HUNEX_SWITCH_SET_B (a folder with the sets of game set B). Data is streamed in 8 MB chunks.

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use hunex_mrg::{
    decompressed_sectors, index_bytes, name_bytes, parse_index, parse_names, plan, SECTOR,
};

const CHUNK: usize = 8 << 20;

fn sets_in(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .expect("read set folder")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "hed"))
        .map(|path| path.with_extension(""))
        .collect();
    found.sort();
    found
}

fn check_set(base: &Path) {
    let hed = fs::read(base.with_extension("hed")).unwrap();
    let nam = fs::read(base.with_extension("nam")).ok();
    let rows = parse_index(&hed).unwrap();
    let mut data = File::open(base.with_extension("mrg")).unwrap();
    let file_len = data.metadata().unwrap().len();

    let mut slots = Vec::with_capacity(rows.len());
    let mut head = [0u8; 8];
    for row in &rows {
        if row.is_empty() {
            slots.push(None);
            continue;
        }
        data.seek(SeekFrom::Start(row.byte_start())).unwrap();
        data.read_exact(&mut head).unwrap();
        let decompressed = decompressed_sectors(&head, row.on_disk_sectors);
        assert_eq!(
            decompressed, row.decompressed_sectors,
            "{base:?}: decompressed count rule"
        );
        slots.push(Some((row.on_disk_sectors, decompressed)));
    }
    let planned = plan(&slots).unwrap();
    assert_eq!(planned, rows, "{base:?}: rebuilt index rows");
    assert_eq!(index_bytes(&planned), hed, "{base:?}: index bytes");

    if let Some(nam) = &nam {
        let names = parse_names(nam);
        assert!(
            names.len() == rows.len() || names.len() == rows.len() + 1,
            "{base:?}: name count"
        );
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        assert_eq!(
            name_bytes(&refs, rows.len()).unwrap(),
            *nam,
            "{base:?}: name bytes"
        );
    }

    // Entries, in index order, cover the data file exactly.
    let mut covered = 0u64;
    let mut original = File::open(base.with_extension("mrg")).unwrap();
    let mut from_entries = vec![0u8; CHUNK];
    let mut from_file = vec![0u8; CHUNK];
    for row in &rows {
        if row.is_empty() {
            continue;
        }
        assert_eq!(
            row.byte_start(),
            covered,
            "{base:?}: entries are contiguous"
        );
        covered += row.byte_span();
        let mut left = row.byte_span() as usize;
        data.seek(SeekFrom::Start(row.byte_start())).unwrap();
        while left > 0 {
            let n = left.min(CHUNK);
            data.read_exact(&mut from_entries[..n]).unwrap();
            original.read_exact(&mut from_file[..n]).unwrap();
            assert!(
                from_entries[..n] == from_file[..n],
                "{base:?}: entry bytes differ"
            );
            left -= n;
        }
    }
    assert_eq!(
        covered, file_len,
        "{base:?}: entries cover the whole data file"
    );
    assert_eq!(covered % SECTOR as u64, 0);
}

#[test]
fn switch_mrg_sets_rebuild_from_their_entries() {
    let mut checked = 0;
    for var in ["HUNEX_SWITCH_ROMFS_A", "HUNEX_SWITCH_SET_B"] {
        let Some(dir) = std::env::var_os(var).map(PathBuf::from) else {
            eprintln!("{var} is not set; skipping the MRG rebuild checks for it");
            continue;
        };
        for base in sets_in(&dir) {
            check_set(&base);
            checked += 1;
        }
    }
    eprintln!("rebuilt {checked} MRG sets");
}
