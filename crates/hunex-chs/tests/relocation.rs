use std::path::PathBuf;

use hunex_chs::{Item, Operand, Program, Script};

const MARKER: u32 = 0x1234_5678;
const PUSH_STRING: u32 = 0x03;
const SKIP: u32 = 0x7B;
const DEBUG: u32 = 0x7F;
const CODE_POINTER: u32 = 0x01;
const PROBE: &[u8] = b"relocation probe";

#[derive(Debug, PartialEq)]
enum Shape {
    Dword(u32),
    Target(usize),
    Text(Vec<u8>),
}

fn logical(program: &Program) -> Vec<(u32, Vec<Shape>)> {
    program
        .items()
        .iter()
        .map(|item| {
            let shapes = item
                .operands()
                .iter()
                .map(|operand| match *operand {
                    Operand::Dword(value) => Shape::Dword(value),
                    Operand::Label(id) => Shape::Target(program.labels()[id]),
                    Operand::Str(id) => Shape::Text(program.string(id).unwrap().to_vec()),
                })
                .collect();
            (item.opcode(), shapes)
        })
        .collect()
}

fn counts(program: &Program) -> [usize; 4] {
    let mut counts = [0usize; 4];
    for item in program.items() {
        match item.opcode() {
            SKIP => counts[0] += 1,
            PUSH_STRING => counts[1] += 1,
            DEBUG => counts[2] += 1,
            CODE_POINTER => counts[3] += 1,
            _ => {}
        }
    }
    counts
}

fn chs_files() -> Option<Vec<PathBuf>> {
    let Some(dir) = std::env::var_os("HUNEX_CHS_DIR") else {
        eprintln!("HUNEX_CHS_DIR is not set; skipping relocation checks");
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

fn skip_item() -> Item {
    Item::new(
        SKIP,
        vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Dword(0)],
    )
}

#[test]
fn real_files_survive_insertions_and_a_new_string() {
    let Some(files) = chs_files() else {
        return;
    };
    let mut totals = [0usize; 4];
    let mut edited_totals = [0usize; 4];
    for path in &files {
        let name = path.display();
        let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{name}: {error}"));
        let original = Script::parse(&bytes)
            .unwrap_or_else(|error| panic!("{name}: {error}"))
            .lift()
            .unwrap_or_else(|error| panic!("{name}: {error}"));

        let mut edited = original.clone();
        edited.insert(0, skip_item()).unwrap();
        let middle = edited.items().len() / 2;
        edited.insert(middle, skip_item()).unwrap();
        let probe = edited.add_string(PROBE).unwrap();
        edited
            .insert(
                middle + 1,
                Item::new(PUSH_STRING, vec![Operand::Str(probe)]),
            )
            .unwrap();

        let relowered = edited
            .lower()
            .unwrap_or_else(|error| panic!("{name}: lower: {error}"))
            .encode();
        let reread = Script::parse(&relowered)
            .unwrap_or_else(|error| panic!("{name}: reparse: {error}"))
            .lift()
            .unwrap_or_else(|error| panic!("{name}: relift: {error}"));
        assert_eq!(
            logical(&reread),
            logical(&edited),
            "{name}: relocated program differs from the edited program"
        );

        let mut restored = reread.clone();
        restored.remove(middle + 1).unwrap();
        restored.remove(middle).unwrap();
        restored.remove(0).unwrap();
        assert_eq!(
            logical(&restored),
            logical(&original),
            "{name}: removing the inserted items does not restore the original program"
        );

        let original_counts = counts(&original);
        let edited_counts = counts(&reread);
        assert_eq!(
            edited_counts,
            [
                original_counts[0] + 2,
                original_counts[1] + 1,
                original_counts[2],
                original_counts[3]
            ],
            "{name}: totals changed by something other than the inserted items"
        );
        for index in 0..4 {
            totals[index] += original_counts[index];
            edited_totals[index] += edited_counts[index];
        }
    }
    println!(
        "relocated {} files: original totals {:?}, edited totals {:?}",
        files.len(),
        totals,
        edited_totals
    );
    assert_eq!(totals, [23139, 115090, 297516, 1656]);
    assert_eq!(edited_totals, [23139 + 500, 115090 + 250, 297516, 1656]);
}
