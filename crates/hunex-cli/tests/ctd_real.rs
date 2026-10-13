//! Export, import and get on the real script text files, gated on HUNEX_CTD_DIR.
//! Scratch files go to HUNEX_WORK and are removed afterwards. The checks are skipped when it is not set.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use hunex_lenzu as lenzu;

const FILES: [&str; 4] = [
    "script_text_en.ctd",
    "script_text_ja.ctd",
    "script_text_zc.ctd",
    "script_text_zt.ctd",
];

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
fn export_import_and_get_round_trip_all_four_files() {
    let Some(dir) = std::env::var_os("HUNEX_CTD_DIR").map(PathBuf::from) else {
        eprintln!("HUNEX_CTD_DIR is not set; skipping ctd CLI checks");
        return;
    };
    let Some(base) = std::env::var_os("HUNEX_WORK").map(PathBuf::from) else {
        eprintln!("HUNEX_WORK is not set; skipping the ctd CLI checks");
        return;
    };
    let work = base.join(format!("hunex-ctd-e2e-{}", std::process::id()));
    fs::create_dir_all(&work).unwrap();

    for name in FILES {
        let original = dir.join(name);
        let lines = work.join(format!("{name}.txt"));
        let rebuilt = work.join(format!("{name}.rebuilt"));
        hunex(&[
            "ctd",
            "export",
            original.to_str().unwrap(),
            "-o",
            lines.to_str().unwrap(),
        ]);
        hunex(&[
            "ctd",
            "import",
            lines.to_str().unwrap(),
            "-o",
            rebuilt.to_str().unwrap(),
        ]);
        let expected = lenzu::decompress(&fs::read(&original).unwrap()).unwrap();
        let actual = lenzu::decompress(&fs::read(&rebuilt).unwrap()).unwrap();
        assert!(
            actual == expected,
            "{name}: export, import, decompress differs from the original body"
        );
        let exported = fs::read_to_string(&lines).unwrap();
        assert_eq!(
            exported.lines().count(),
            24_134,
            "{name}: exported line count"
        );
    }

    let en = dir.join("script_text_en.ctd");
    let got = hunex(&["ctd", "get", en.to_str().unwrap(), "$000000", "$000001"]);
    let text = String::from_utf8(got.stdout).unwrap();
    let mut parts = text.lines();
    assert!(parts.next().unwrap().starts_with("$000000\t"));
    assert!(parts.next().unwrap().starts_with("$000001\t"));
    assert!(parts.next().is_none());

    fs::remove_dir_all(&work).unwrap();
    println!("ctd: export, import and decompress match the originals for all four files");
}
