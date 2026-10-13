//! The sample CBG files, gated on HUNEX_CBG_DIR: a folder that holds `.cbg` files, searched
//! recursively. Only `.cbg` files are read.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::sha256_hex;
use hunex_cbg::{decode, encode, encode_simple};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "cbg") {
            out.push(path);
        }
    }
}

fn samples() -> Option<Vec<PathBuf>> {
    let Some(dir) = std::env::var_os("HUNEX_CBG_DIR").map(PathBuf::from) else {
        eprintln!("HUNEX_CBG_DIR is not set; skipping the CBG samples");
        return None;
    };
    let mut files = Vec::new();
    collect(&dir, &mut files);
    files.sort();
    Some(files)
}

#[test]
fn all_samples_decode_and_re_encode_byte_for_byte() {
    let Some(files) = samples() else {
        return;
    };
    let (mut l, mut rgb, mut rgba) = (0usize, 0usize, 0usize);
    let mut simple_bytes = 0usize;
    let mut original_bytes = 0usize;
    for path in &files {
        let bytes = fs::read(path).unwrap();
        let image = decode(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        match image.header.bits_per_pixel {
            8 => l += 1,
            24 => rgb += 1,
            _ => rgba += 1,
        }
        let canonical = encode(&image).unwrap();
        assert_eq!(canonical, bytes, "{}: re-encode differs", path.display());
        let simple = encode_simple(&image).unwrap();
        assert_eq!(
            decode(&simple).unwrap(),
            image,
            "{}: simple form differs",
            path.display()
        );
        simple_bytes += simple.len();
        original_bytes += bytes.len();
    }
    println!(
        "{} CBG files: {l} L, {rgb} RGB, {rgba} RGBA; all re-encode byte for byte; simple form {simple_bytes} bytes against {original_bytes} original",
        files.len()
    );
    assert_eq!(files.len(), 537);
    assert_eq!((l, rgb, rgba), (14, 16, 507));
}

#[test]
fn the_spec_sample_hashes_match() {
    let Some(files) = samples() else {
        return;
    };
    let wanted = [
        (
            "rule_01.cbg",
            "22693a80a0eb33cf2d46aec75f9e6211ad9ebdeceb814ea4b219fbbb451031cc",
        ),
        (
            "alphagradation128.cbg",
            "abebc89cbf53b41a8ab7129aaa3968161ddb52404b632ddce1e6925212201d51",
        ),
        (
            "face_on_ja.cbg",
            "6860a1d1c0868c95b80d13096eb669300bb6bc580b686b7ad21b632956e2a2bb",
        ),
        (
            "BG_load.cbg",
            "6091222ad740065d470f695f8a559a7825e6ac3cbe73b5d81813042cd9b1b208",
        ),
    ];
    for (name, hash) in wanted {
        let path = files
            .iter()
            .find(|path| path.file_name().is_some_and(|n| n == name))
            .unwrap_or_else(|| panic!("{name} is not in HUNEX_CBG_DIR"));
        let image = decode(&fs::read(path).unwrap()).unwrap();
        assert_eq!(sha256_hex(&image.rgba), hash, "{name}");
    }
}
