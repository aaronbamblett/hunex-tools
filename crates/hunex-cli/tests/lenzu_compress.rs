//! The LenZu compressor on the real streams, gated on HUNEX_LENZU_DIR (the folder with the stored
//! `.chs` and `script_text_*.ctd` entries; its sibling `raw` folder holds the decompressed bodies)
//! and HUNEX_HFA_DIR (for the `.ccit` tables in data00100.hfa).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use hunex_hfa::Archive;
use hunex_lenzu::{compress, decompress, Envelope};

fn dir(var: &str) -> Option<PathBuf> {
    match std::env::var_os(var) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("{var} is not set; skipping");
            None
        }
    }
}

/// Checks a set of compressed streams against the original envelope, and returns the totals.
fn check(
    label: &str,
    plains: &[(String, Vec<u8>)],
    stored: &[(String, Vec<u8>)],
) -> (usize, usize) {
    let mut ours = Envelope::default();
    let (mut stored_total, mut ours_total) = (0usize, 0usize);
    let mut seconds = 0.0f64;
    for ((name, plain), (_, original)) in plains.iter().zip(stored) {
        let start = Instant::now();
        let packed = compress(plain);
        seconds += start.elapsed().as_secs_f64();
        assert_eq!(
            decompress(&packed).unwrap(),
            *plain,
            "{label} {name}: round trip"
        );
        ours.add(&packed).unwrap();
        stored_total += original.len();
        ours_total += packed.len();
    }
    println!(
        "{label}: {} streams, stored {stored_total} bytes, compressed {ours_total} bytes ({:.3} of stored), compress time {seconds:.2} s",
        plains.len(),
        ours_total as f64 / stored_total as f64
    );
    println!("{label} envelope of the compressed streams: {ours:?}");
    assert!(
        ours.max_code_depth <= 13,
        "{label}: depth {}",
        ours.max_code_depth
    );
    assert!(
        ours.longest_match <= 129 && ours.shortest_match >= 2,
        "{label}: match lengths"
    );
    assert!(
        ours.longest_distance <= 16_385,
        "{label}: distance {}",
        ours.longest_distance
    );
    assert_eq!(ours.overlapping_matches, 0, "{label}: overlapping matches");
    assert!(
        ours.longest_literal_run <= 54,
        "{label}: literal run {}",
        ours.longest_literal_run
    );
    assert_eq!(
        ours.count_is_used_symbols, ours.streams,
        "{label}: declared count"
    );
    assert_eq!(ours.padding_zero_streams, ours.streams, "{label}: padding");
    assert_eq!(ours.options.len(), 1, "{label}: options");
    (stored_total, ours_total)
}

fn read_dir_sorted(dir: &Path, suffixes: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| suffixes.iter().any(|s| n.ends_with(s)))
        .collect();
    names.sort();
    names
}

#[test]
fn compressor_round_trips_the_real_streams_inside_the_envelope() {
    let Some(stored_dir) = dir("HUNEX_LENZU_DIR") else {
        return;
    };
    let raw_dir = stored_dir.parent().unwrap().join("raw");

    let scripts = read_dir_sorted(&stored_dir, &[".chs"]);
    let scripts_stored: Vec<(String, Vec<u8>)> = scripts
        .iter()
        .map(|n| (n.clone(), fs::read(stored_dir.join(n)).unwrap()))
        .collect();
    let scripts_plain: Vec<(String, Vec<u8>)> = scripts
        .iter()
        .map(|n| (n.clone(), fs::read(raw_dir.join(n)).unwrap()))
        .collect();
    check("scripts (.chs)", &scripts_plain, &scripts_stored);

    let texts = read_dir_sorted(&stored_dir, &[".ctd"]);
    let texts_stored: Vec<(String, Vec<u8>)> = texts
        .iter()
        .map(|n| (n.clone(), fs::read(stored_dir.join(n)).unwrap()))
        .collect();
    let texts_plain: Vec<(String, Vec<u8>)> = texts
        .iter()
        .map(|n| (n.clone(), fs::read(raw_dir.join(n)).unwrap()))
        .collect();
    check("script text (.ctd)", &texts_plain, &texts_stored);

    if let Some(hfa) = dir("HUNEX_HFA_DIR") {
        let bytes = fs::read(hfa.join("data00100.hfa")).unwrap();
        let archive = Archive::parse(&bytes).unwrap();
        let mut tables_stored = Vec::new();
        let mut tables_plain = Vec::new();
        for entry in archive
            .entries()
            .iter()
            .filter(|e| e.name().ends_with(".ccit"))
        {
            tables_stored.push((entry.name().to_string(), entry.data().to_vec()));
            tables_plain.push((entry.name().to_string(), decompress(entry.data()).unwrap()));
        }
        check("glyph tables (.ccit)", &tables_plain, &tables_stored);
    }
}

#[test]
fn compressor_edge_cases_round_trip() {
    for plain in [
        Vec::new(),
        vec![7u8],
        vec![0u8; 100_000],
        (0..70_000u32)
            .map(|i| ((i.wrapping_mul(2_654_435_761u32)) >> 24) as u8)
            .collect::<Vec<u8>>(),
    ] {
        assert_eq!(decompress(&compress(&plain)).unwrap(), plain);
    }
}

#[test]
fn the_cli_compresses_by_default_and_writes_literal_runs_with_the_flag() {
    use std::process::Command;
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("lenzu-cli");
    fs::create_dir_all(&dir).unwrap();
    let plain: Vec<u8> = b"HunexCompiledScriptVer1.00 ".repeat(400);
    let input = dir.join("plain.bin");
    fs::write(&input, &plain).unwrap();
    let run = |args: &[&str], output: &Path| {
        let status = Command::new(env!("CARGO_BIN_EXE_hunex"))
            .args(["lenzu", "compress"])
            .arg(&input)
            .args(args)
            .arg("-o")
            .arg(output)
            .status()
            .unwrap();
        assert!(status.success());
    };
    let packed = dir.join("packed.lz");
    let literal = dir.join("literal.lz");
    run(&[], &packed);
    run(&["--literal"], &literal);
    let packed_bytes = fs::read(&packed).unwrap();
    let literal_bytes = fs::read(&literal).unwrap();
    assert_eq!(decompress(&packed_bytes).unwrap(), plain);
    assert_eq!(decompress(&literal_bytes).unwrap(), plain);
    assert!(packed_bytes.len() * 4 < literal_bytes.len());
}
