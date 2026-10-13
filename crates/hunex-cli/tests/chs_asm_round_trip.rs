use std::path::PathBuf;

use hunex_chs::Script;
use hunex_cli::chs::asm;

fn chs_files() -> Option<Vec<PathBuf>> {
    let Some(dir) = std::env::var_os("HUNEX_CHS_DIR") else {
        eprintln!("HUNEX_CHS_DIR is not set; skipping asm round-trip checks");
        return None;
    };
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read HUNEX_CHS_DIR")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "chs"))
        .collect();
    files.sort();
    Some(files)
}

#[test]
fn real_files_dump_then_assemble_byte_for_byte() {
    let Some(files) = chs_files() else {
        return;
    };
    let mut identical = 0usize;
    for path in &files {
        let name = path.display();
        let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{name}: {error}"));
        let program = Script::parse(&bytes)
            .unwrap_or_else(|error| panic!("{name}: {error}"))
            .lift()
            .unwrap_or_else(|error| panic!("{name}: lift: {error}"));
        let text = asm::dump(&program);
        let rebuilt =
            asm::build(text.as_bytes()).unwrap_or_else(|error| panic!("{name}: assemble: {error}"));
        if rebuilt == bytes {
            identical += 1;
        } else {
            let offset = rebuilt
                .iter()
                .zip(&bytes)
                .position(|(a, b)| a != b)
                .unwrap_or(rebuilt.len().min(bytes.len()));
            panic!("{name}: dump/assemble differs first at offset 0x{offset:x}");
        }
    }
    println!("asm: {identical} of {} files identical", files.len());
    assert_eq!(identical, 250);
}
