//! The allscr writer: dump, pack with --base, compare. Gated on HUNEX_SWITCH_ROMFS_A (a RomFS
//! folder with allscr.mrg). Scratch files go to HUNEX_WORK (skipped when it is not set).

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
fn allscr_dump_pack_is_identical_and_an_edit_changes_one_entry() {
    let Some(romfs) = std::env::var_os("HUNEX_SWITCH_ROMFS_A").map(PathBuf::from) else {
        eprintln!("HUNEX_SWITCH_ROMFS_A is not set; skipping the allscr writer checks");
        return;
    };
    let Some(work) = std::env::var_os("HUNEX_WORK").map(PathBuf::from) else {
        eprintln!("HUNEX_WORK is not set; skipping the allscr writer checks");
        return;
    };
    let work = work.join(format!("allscr-pack-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();
    let original = romfs.join("allscr.mrg");
    let dumped = work.join("dump");
    let packed = work.join("packed.mrg");
    let base = original.to_str().unwrap();

    hunex(&[
        "switch-script",
        "dump",
        base,
        "-o",
        dumped.to_str().unwrap(),
    ]);
    hunex(&[
        "switch-script",
        "pack",
        dumped.to_str().unwrap(),
        "-o",
        packed.to_str().unwrap(),
        "--base",
        base,
    ]);
    assert!(
        fs::read(&packed).unwrap() == fs::read(&original).unwrap(),
        "dump -> pack --base is not byte-identical"
    );

    // Edit one script: only that entry changes, and its stream decodes to the edit.
    let edit_script = 100usize;
    let edited = work.join("edited");
    let _ = fs::remove_dir_all(&edited);
    fs::create_dir_all(&edited).unwrap();
    for entry in fs::read_dir(&dumped).unwrap() {
        let path = entry.unwrap().path();
        fs::copy(&path, edited.join(path.file_name().unwrap())).unwrap();
    }
    let target = edited.join(format!("scr{edit_script:04}.txt"));
    let mut text = fs::read(&target).unwrap();
    text.extend_from_slice(b"\n");
    fs::write(&target, &text).unwrap();
    let edited_mrg = work.join("edited.mrg");
    hunex(&[
        "switch-script",
        "pack",
        edited.to_str().unwrap(),
        "-o",
        edited_mrg.to_str().unwrap(),
        "--base",
        base,
    ]);

    let before_bytes = fs::read(&original).unwrap();
    let before = hunex_mzp::parse_switch(&before_bytes).unwrap();
    let after_bytes = fs::read(&edited_mrg).unwrap();
    let after = hunex_mzp::parse_switch(&after_bytes).unwrap();
    assert_eq!(before.entries.len(), after.entries.len());
    let mut changed = Vec::new();
    for index in 0..before.entries.len() {
        let (old, new) = (before.entries[index], after.entries[index]);
        let same = if index < 3 {
            old == new
        } else {
            let old_len = hunex_mzx::switch::stream_len(old).unwrap();
            let new_len = hunex_mzx::switch::stream_len(new).unwrap();
            old[..old_len] == new[..new_len]
        };
        if !same {
            changed.push(index);
        }
    }
    assert_eq!(
        changed,
        vec![edit_script + 3],
        "only the edited entry may change"
    );
    let decoded = hunex_mzx::switch::decompress(after.entries[edit_script + 3]).unwrap();
    assert_eq!(decoded, text, "the edited stream decodes to the edit");
    let _ = fs::remove_dir_all(&work);
}
