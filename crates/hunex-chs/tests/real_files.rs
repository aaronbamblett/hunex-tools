use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use hunex_chs::{Operand, Script};

const RETURN: u32 = 0x1B;
const PUSH_STRING: u32 = 0x03;
const CODE_POINTER: u32 = 0x01;
const SKIP: u32 = 0x7B;
const DEBUG: u32 = 0x7F;
const SKIP_MARKER: u32 = 0x1234_5678;

fn chs_files() -> Option<Vec<PathBuf>> {
    let Some(dir) = std::env::var_os("HUNEX_CHS_DIR") else {
        eprintln!("HUNEX_CHS_DIR is not set; skipping real-file checks");
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

fn text_ids(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut ids = Vec::new();
    for start in 0..bytes.len() {
        let end = start + 7;
        if bytes[start] != b'$' || end > bytes.len() {
            continue;
        }
        let digits = bytes[start + 1..end].iter().all(u8::is_ascii_digit);
        let terminated = bytes.get(end).is_none_or(|next| !next.is_ascii_digit());
        if digits && terminated {
            ids.push(&text[start..end]);
        }
    }
    ids
}

fn image_names(text: &str) -> Vec<&str> {
    text.split('\n')
        .filter(|line| {
            line.len() == 7
                && line.starts_with("img")
                && line.as_bytes()[3..].iter().all(u8::is_ascii_digit)
        })
        .collect()
}

#[test]
fn real_files_decode_with_expected_totals() {
    let Some(files) = chs_files() else {
        return;
    };
    assert_eq!(files.len(), 250, "expected 250 .chs files");

    let mut push_string = 0usize;
    let mut debug = 0usize;
    let mut skip = 0usize;
    let mut code_pointer = 0usize;

    for path in &files {
        let name = path.display();
        let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{name}: {error}"));
        let script = Script::parse(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));

        let last = script.instructions().last().expect("non-empty code");
        assert_eq!(last.opcode(), RETURN, "{name}: walk ends on a return");
        assert_eq!(
            last.offset() + last.size(),
            script.string_pool_start(),
            "{name}: walk ends exactly at the pool start"
        );

        let starts: HashSet<usize> = script.instructions().iter().map(|i| i.offset()).collect();
        for instruction in script.instructions() {
            match instruction.opcode() {
                PUSH_STRING => {
                    push_string += 1;
                    script
                        .string(instruction)
                        .unwrap_or_else(|error| panic!("{name}: {error}"));
                }
                DEBUG => {
                    debug += 1;
                    script
                        .string(instruction)
                        .unwrap_or_else(|error| panic!("{name}: {error}"));
                }
                SKIP => {
                    skip += 1;
                    assert_eq!(instruction.operands().len(), 3, "{name}: 0x7B operands");
                    assert_eq!(
                        instruction.operands()[0],
                        SKIP_MARKER,
                        "{name}: 0x7B marker"
                    );
                }
                CODE_POINTER => {
                    code_pointer += 1;
                    let target = script.code_base() + instruction.operands()[0] as usize;
                    assert!(
                        starts.contains(&target),
                        "{name}: 0x01 at 0x{:x} lands at 0x{target:x}, not an instruction start",
                        instruction.offset()
                    );
                }
                _ => {}
            }
        }
    }

    println!(
        "totals over {} files: PushString(0x03)={push_string} Debug(0x7F)={debug} \
         SkipMarker(0x7B)={skip} CodePointer(0x01)={code_pointer}",
        files.len()
    );
    assert_eq!(push_string, 115_090);
    assert_eq!(debug, 297_516);
    assert_eq!(skip, 23_139);
    assert_eq!(code_pointer, 1_656);
}

#[test]
fn main_sample_text_ids_and_images() {
    let Some(dir) = std::env::var_os("HUNEX_CHS_DIR") else {
        eprintln!("HUNEX_CHS_DIR is not set; skipping 1_0.chs checks");
        return;
    };
    let path = PathBuf::from(dir).join("1_0.chs");
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let script = Script::parse(&bytes).expect("parse 1_0.chs");

    let mut ids = BTreeSet::new();
    let mut images: Vec<&str> = Vec::new();
    for instruction in script.instructions() {
        if !matches!(instruction.opcode(), PUSH_STRING | DEBUG) {
            continue;
        }
        let text = script.string(instruction).expect("string operand");
        ids.extend(text_ids(text));
        for name in image_names(text) {
            if !images.contains(&name) {
                images.push(name);
            }
        }
    }

    println!(
        "1_0.chs: {} distinct text ids, first images {:?}",
        ids.len(),
        &images[..4]
    );
    assert_eq!(ids.len(), 80);
    assert_eq!(&images[..4], ["img0408", "img1227", "img2376", "img2326"]);
}

#[test]
fn real_files_round_trip_byte_for_byte() {
    let Some(files) = chs_files() else {
        return;
    };
    let mut identical = 0usize;
    let mut first_difference: Option<(String, usize)> = None;
    for path in &files {
        let bytes =
            std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let encoded = Script::parse(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
            .encode();
        if encoded == bytes {
            identical += 1;
        } else if first_difference.is_none() {
            let offset = encoded
                .iter()
                .zip(&bytes)
                .position(|(a, b)| a != b)
                .unwrap_or(encoded.len().min(bytes.len()));
            first_difference = Some((path.display().to_string(), offset));
        }
    }
    println!("round-trip: {identical} of {} files identical", files.len());
    if let Some((name, offset)) = first_difference {
        panic!("{name}: re-encode differs first at offset 0x{offset:x}");
    }
    assert_eq!(identical, 250);
}

#[test]
fn real_files_lift_lower_round_trip_byte_for_byte() {
    let Some(files) = chs_files() else {
        return;
    };
    let mut identical = 0usize;
    for path in &files {
        let bytes =
            std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let script =
            Script::parse(&bytes).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let program = script
            .lift()
            .unwrap_or_else(|error| panic!("{}: lift: {error}", path.display()));
        let encoded = program
            .lower()
            .unwrap_or_else(|error| panic!("{}: lower: {error}", path.display()))
            .encode();
        if encoded == bytes {
            identical += 1;
        } else {
            let offset = encoded
                .iter()
                .zip(&bytes)
                .position(|(a, b)| a != b)
                .unwrap_or(encoded.len().min(bytes.len()));
            panic!(
                "{}: lift/lower differs first at offset 0x{offset:x}",
                path.display()
            );
        }
    }
    println!("lift/lower: {identical} of {} files identical", files.len());
    assert_eq!(identical, 250);
}

#[test]
fn real_files_skip_operand_is_a_string_pointer_above_the_pool_start() {
    let Some(files) = chs_files() else {
        return;
    };
    let mut skips = 0usize;
    let mut fallbacks = 0usize;
    for path in &files {
        let name = path.display();
        let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{name}: {error}"));
        let script = Script::parse(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
        for instruction in script.instructions() {
            if instruction.opcode() == SKIP {
                skips += 1;
                let pointer = instruction.operands()[2] as usize + script.code_base();
                assert!(
                    pointer >= script.string_pool_start(),
                    "{name}: 0x7B pointer at 0x{:x} is below the pool start",
                    instruction.offset()
                );
            }
        }
        let program = script
            .lift()
            .unwrap_or_else(|error| panic!("{name}: lift: {error}"));
        for item in program.items() {
            if item.opcode() == SKIP && !matches!(item.operands()[2], Operand::Str(_)) {
                fallbacks += 1;
            }
        }
    }
    println!("0x7B: {skips} instructions, {fallbacks} third operands kept as dwords");
    assert_eq!(skips, 23_139);
    assert_eq!(fallbacks, 0);
}
