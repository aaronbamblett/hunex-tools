//! Keeps docs/opcodes.md equal to the opcode table. Run with HUNEX_UPDATE_DOCS=1 to rewrite it.

use hunex_chs::OPCODES;

const DOC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/opcodes.md");

fn render() -> String {
    let base = OPCODES.iter().filter(|info| info.id < 0x100).count();
    let commands = OPCODES.len() - base;
    let mut out = String::new();
    out.push_str("# Opcode table\n\n");
    out.push_str(
        "Generated from `crates/hunex-chs/src/opcodes.rs` by the `opcodes_doc` test. \
         Do not edit by hand; regenerate with `HUNEX_UPDATE_DOCS=1 cargo test -p hunex-chs --test opcodes_doc`.\n\n",
    );
    out.push_str(&format!(
        "{} registered ids: {base} base opcodes (0x00 to 0xFE) and {commands} commands (0x100 and up). \
         Any other id is an error when decoding.\n\n",
        OPCODES.len()
    ));
    out.push_str(
        "Inline dwords are the operands stored after the opcode. Opcode 0x7B skips three dwords, \
         which are kept as its operands. The first operand of 0x01, 0x03 and 0x7F is a code or \
         string pointer; the others are plain dwords.\n\n",
    );
    out.push_str("Mnemonics are only given where the notes name an opcode and the exe analysis does not contradict the name; \"-\" means no name.\n\n");
    out.push_str("| Id | Mnemonic | Inline dwords |\n|---|---|---|\n");
    for info in OPCODES {
        out.push_str(&format!(
            "| `0x{:03x}` | {} | {} |\n",
            info.id,
            info.mnemonic.unwrap_or("-"),
            info.operands
        ));
    }
    out
}

#[test]
fn opcodes_doc_matches_table() {
    if !std::path::Path::new(DOC).exists() {
        eprintln!("docs/opcodes.md is not in this package; skipping");
        return;
    }
    let rendered = render();
    if std::env::var_os("HUNEX_UPDATE_DOCS").is_some() {
        std::fs::write(DOC, &rendered).expect("write docs/opcodes.md");
        return;
    }
    let on_disk = std::fs::read_to_string(DOC).unwrap_or_default();
    assert!(
        on_disk == rendered,
        "docs/opcodes.md is out of date; run HUNEX_UPDATE_DOCS=1 cargo test -p hunex-chs --test opcodes_doc"
    );
}
