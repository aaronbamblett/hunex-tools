//! The script text writer: export, import with --base, compare. Gated on HUNEX_SWITCH_ROMFS_A (a
//! RomFS folder with script_text.mrg). Scratch files go to HUNEX_WORK (skipped when unset).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn hunex(args: &[&str]) {
    let status = Command::new(env!("CARGO_BIN_EXE_hunex"))
        .args(args)
        .status()
        .expect("run hunex");
    assert!(status.success(), "hunex {args:?} failed");
}

#[test]
fn script_text_round_trips_and_an_edited_line_survives() {
    let Some(romfs) = std::env::var_os("HUNEX_SWITCH_ROMFS_A").map(PathBuf::from) else {
        eprintln!("HUNEX_SWITCH_ROMFS_A is not set; skipping the script text writer checks");
        return;
    };
    let Some(work) = std::env::var_os("HUNEX_WORK").map(PathBuf::from) else {
        eprintln!("HUNEX_WORK is not set; skipping the script text writer checks");
        return;
    };
    let work = work.join(format!("switch-text-import-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();
    let original = romfs.join("script_text.mrg");
    let base = original.to_str().unwrap();
    let lines = work.join("pair1.txt");
    let rebuilt = work.join("rebuilt.mrg");

    hunex(&[
        "switch-text",
        "export",
        base,
        "-o",
        lines.to_str().unwrap(),
        "--pair",
        "1",
    ]);
    hunex(&[
        "switch-text",
        "import",
        lines.to_str().unwrap(),
        "-o",
        rebuilt.to_str().unwrap(),
        "--base",
        base,
        "--pair",
        "1",
    ]);
    assert!(
        fs::read(&rebuilt).unwrap() == fs::read(&original).unwrap(),
        "export -> import is not byte-identical"
    );

    let text = fs::read_to_string(&lines).unwrap();
    let edited_text: String = text
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 1 {
                "$000001\tEDITED LINE FOR THE CHECK".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let edited = work.join("edited.txt");
    fs::write(&edited, edited_text).unwrap();
    let edited_mrg = work.join("edited.mrg");
    hunex(&[
        "switch-text",
        "import",
        edited.to_str().unwrap(),
        "-o",
        edited_mrg.to_str().unwrap(),
        "--base",
        base,
        "--pair",
        "1",
    ]);
    let again = work.join("again.txt");
    hunex(&[
        "switch-text",
        "export",
        edited_mrg.to_str().unwrap(),
        "-o",
        again.to_str().unwrap(),
        "--pair",
        "1",
    ]);
    let again_text = fs::read_to_string(&again).unwrap();
    assert_eq!(
        again_text.lines().nth(1).unwrap(),
        "$000001\tEDITED LINE FOR THE CHECK"
    );
    let differing = again_text
        .lines()
        .zip(text.lines())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(differing, 1, "only the edited line may differ");
    let _ = fs::remove_dir_all(&work);
}
