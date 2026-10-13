//! The envelope of the original LenZu streams, gated on HUNEX_LENZU_DIR (the folder with the
//! stored `.chs` and `script_text_*.ctd` entries) and HUNEX_HFA_DIR (the folder with
//! data00100.hfa, for the `.ccit` tables).

use std::fs;
use std::path::PathBuf;

use hunex_hfa::Archive;
use hunex_lenzu::Envelope;

fn dir(var: &str) -> Option<PathBuf> {
    match std::env::var_os(var) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("{var} is not set; skipping");
            None
        }
    }
}

#[test]
fn the_original_streams_envelope() {
    let Some(lenzu) = dir("HUNEX_LENZU_DIR") else {
        return;
    };
    let mut names: Vec<String> = fs::read_dir(&lenzu)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".chs") || n.ends_with(".ctd"))
        .collect();
    names.sort();
    let mut scripts = Envelope::default();
    let mut texts = Envelope::default();
    for name in &names {
        let bytes = fs::read(lenzu.join(name)).unwrap();
        let target = if name.ends_with(".ctd") {
            &mut texts
        } else {
            &mut scripts
        };
        target.add(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    println!("scripts (.chs): {scripts:?}");
    println!("script text (.ctd): {texts:?}");

    if let Some(hfa) = dir("HUNEX_HFA_DIR") {
        let bytes = fs::read(hfa.join("data00100.hfa")).unwrap();
        let archive = Archive::parse(&bytes).unwrap();
        let mut tables = Envelope::default();
        for entry in archive.entries() {
            if entry.name().ends_with(".ccit") {
                tables.add(entry.data()).unwrap();
            }
        }
        println!("glyph tables (.ccit): {tables:?}");
    }
}
