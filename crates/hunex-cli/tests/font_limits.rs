//! Crafted glyph tables and atlases: the render must reject them with an error, not abort.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_glyph_wider_than_its_cell_is_an_error_not_an_abort() {
    let dir = scratch("font-limits");
    let table = dir.join("huge.ccit");
    fs::write(
        &table,
        words(&[1, 8, 0x41, 0, 1, 20, 0, 0, 0, u32::MAX, 73]),
    )
    .unwrap();
    let atlas = dir.join("blank.mzp");
    let layout = hunex_image::Layout::default_for(1024, 1024);
    let packed =
        hunex_image::encode_mzp(&vec![0u8; 1024 * 1024 * 4], 1024, 1024, Some(&layout)).unwrap();
    fs::write(&atlas, packed).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_hunex"))
        .args(["font", "render"])
        .arg(&table)
        .args(["--atlas"])
        .arg(&atlas)
        .args(["--text", "A", "-o"])
        .arg(dir.join("huge.png"))
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("wider than its cell"));
}
