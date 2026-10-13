//! Checks that the example in docs/asm-format.md assembles, lowers and re-encodes to itself.

use hunex_cli::chs::asm;

fn example(doc: &str) -> String {
    let start = doc
        .find("```text\n")
        .expect("docs/asm-format.md has a text example")
        + "```text\n".len();
    let end = start + doc[start..].find("```").expect("example is closed");
    doc[start..end].to_string()
}

#[test]
fn asm_format_example_assembles() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/asm-format.md");
    if !std::path::Path::new(path).exists() {
        eprintln!("docs/asm-format.md is not in this package; skipping");
        return;
    }
    let doc = std::fs::read_to_string(path).expect("read docs/asm-format.md");
    let text = example(&doc);
    let bytes = asm::build(text.as_bytes()).unwrap_or_else(|error| panic!("example: {error}"));
    let script = hunex_chs::Script::parse(&bytes).expect("example parses");
    assert_eq!(script.encode(), bytes);
    assert_eq!(script.instructions().len(), 4);
}
