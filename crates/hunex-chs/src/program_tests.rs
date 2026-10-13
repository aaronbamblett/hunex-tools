use super::*;
use crate::tests::{hunex, MARKER};
use alloc::vec;

fn labelled() -> Vec<u8> {
    hunex(&[0x01, 0x08, 0x00, 5, 0x03, 0x1C, 0x1B], b"hi\0")
}

fn lift(bytes: &[u8]) -> Program {
    Script::parse(bytes).unwrap().lift().unwrap()
}

fn lower_bytes(program: &Program) -> Vec<u8> {
    program.lower().unwrap().encode()
}

#[test]
fn lift_lower_round_trip_with_labels_and_strings() {
    let bytes = labelled();
    let program = lift(&bytes);
    assert_eq!(program.items().len(), 4);
    assert_eq!(program.labels(), &[1]);
    assert_eq!(program.items()[0].operands(), &[Operand::Label(0)]);
    assert_eq!(program.items()[2].operands(), &[Operand::Str(0)]);
    assert_eq!(program.string(0), Some(&b"hi"[..]));
    assert_eq!(lower_bytes(&program), bytes);
}

#[test]
fn unreferenced_original_string_is_kept_in_place() {
    let bytes = hunex(&[0x03, 0x14, 0x03, 0x19, 0x1B], b"a\0zz\0b\0");
    let program = lift(&bytes);
    assert_eq!(program.strings().len(), 3);
    assert_eq!(lower_bytes(&program), bytes);
}

#[test]
fn pointer_into_middle_of_code_is_a_lift_error() {
    let bytes = hunex(&[0x01, 0x09, 0x00, 5, 0x1B], b"");
    assert_eq!(
        Script::parse(&bytes).unwrap().lift(),
        Err(Error::BadCodePointer {
            offset: 0x30,
            value: 9
        })
    );
}

#[test]
fn insert_shifts_labels_after_it() {
    let mut program = lift(&labelled());
    program
        .insert(
            0,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Dword(0)],
            ),
        )
        .unwrap();
    assert_eq!(program.labels(), &[2]);
    let bytes = lower_bytes(&program);
    let reparsed = lift(&bytes);
    assert_eq!(reparsed.labels(), &[2]);
    assert_eq!(reparsed.items()[2].opcode(), 0x00);
}

#[test]
fn insert_at_label_keeps_label_on_old_instruction() {
    let mut program = lift(&labelled());
    program
        .insert(
            1,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Dword(0)],
            ),
        )
        .unwrap();
    assert_eq!(program.labels(), &[2]);
    assert_eq!(program.items()[2].opcode(), 0x00);
}

#[test]
fn removing_a_labelled_instruction_is_an_error_until_the_label_moves() {
    let mut program = lift(&labelled());
    assert_eq!(
        program.remove(1),
        Err(EditError::LabelOnRemovedInstruction { index: 1, label: 0 })
    );
    program.move_label(0, 2).unwrap();
    let removed = program.remove(1).unwrap();
    assert_eq!(removed.opcode(), 0x00);
    assert_eq!(program.labels(), &[1]);
    assert_eq!(program.items()[1].opcode(), 0x03);
}

#[test]
fn appended_string_is_written_after_original_strings() {
    let mut program = lift(&labelled());
    let id = program.add_string(b"new text").unwrap();
    program
        .insert(3, Item::new(0x03, vec![Operand::Str(id)]))
        .unwrap();
    let script = program.lower().unwrap();
    let reparsed = Script::parse(&script.encode()).unwrap();
    assert_eq!(reparsed.pool_bytes(), b"hi\0new text\0");
    let relifted = reparsed.lift().unwrap();
    assert_eq!(relifted.string(1), Some(&b"new text"[..]));
    assert_eq!(relifted.items()[3].operands(), &[Operand::Str(1)]);
}

#[test]
fn replaced_string_moves_and_old_bytes_are_dropped_when_unreferenced() {
    let mut program = lift(&labelled());
    program.set_string(0, b"changed").unwrap();
    let reparsed = Script::parse(&program.lower().unwrap().encode()).unwrap();
    assert_eq!(reparsed.pool_bytes(), b"changed\0");
}

#[test]
fn unreferenced_new_string_is_not_written() {
    let mut program = lift(&labelled());
    program.add_string(b"unused").unwrap();
    assert_eq!(lower_bytes(&program), labelled());
}

fn skip_with_string() -> Vec<u8> {
    hunex(&[0x03, 0x1C, 0x7B, MARKER, 1, 0x1C, 0x1B], b"hi\0")
}

#[test]
fn skip_marker_third_operand_is_a_string_and_relocates() {
    let bytes = skip_with_string();
    let mut program = lift(&bytes);
    assert_eq!(
        program.items()[1].operands(),
        &[Operand::Dword(MARKER), Operand::Dword(1), Operand::Str(0)]
    );
    program
        .insert(0, Item::new(0x00, vec![Operand::Dword(7)]))
        .unwrap();
    let relowered = program.lower().unwrap().encode();
    assert_ne!(relowered, bytes);
    let reread = lift(&relowered);
    assert_eq!(reread.items()[2].operands()[2], Operand::Str(0));
    assert_eq!(reread.string(0), Some(&b"hi"[..]));
}

#[test]
fn skip_marker_third_operand_that_is_not_a_string_stays_a_dword() {
    let bytes = hunex(&[0x03, 0x1C, 0x7B, MARKER, 1, 0, 0x1B], b"hi\0");
    let program = lift(&bytes);
    assert_eq!(program.items()[1].operands()[2], Operand::Dword(0));
    assert_eq!(lower_bytes(&program), bytes);
}

#[test]
fn skip_marker_string_can_be_added_by_edit() {
    let mut program = lift(&labelled());
    let id = program.add_string(b"skip text").unwrap();
    program
        .insert(
            0,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Str(id)],
            ),
        )
        .unwrap();
    let reread = lift(&program.lower().unwrap().encode());
    assert_eq!(reread.items()[0].operands()[2], Operand::Str(1));
    assert_eq!(reread.string(1), Some(&b"skip text"[..]));
    assert_eq!(
        program.insert(
            0,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Str(9)]
            )
        ),
        Err(EditError::UnknownString(9))
    );
}

#[test]
fn skip_marker_reference_cannot_move_the_pool_start() {
    let mut program = lift(&hunex(&[0x03, 0x0C, 0x1B], b"a\0b\0"));
    program.remove(0).unwrap();
    program
        .insert(
            0,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Str(1)],
            ),
        )
        .unwrap();
    assert_eq!(program.lower(), Err(LowerError::PointerBeforePool));
}

#[test]
fn new_string_referenced_only_by_skip_marker_is_not_an_anchor() {
    let mut program = lift(&hunex(&[0x03, 0x0C, 0x1B], b"a\0"));
    program.remove(0).unwrap();
    let id = program.add_string(b"new").unwrap();
    program
        .insert(
            0,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Str(id)],
            ),
        )
        .unwrap();
    assert_eq!(program.lower(), Err(LowerError::NoStringAnchor));
}

#[test]
fn edit_errors() {
    let mut program = lift(&labelled());
    assert_eq!(program.set_string(0, b"a\0b"), Err(EditError::NulInString));
    assert_eq!(
        program.insert(0, Item::new(0x04, vec![])),
        Err(EditError::UnregisteredOpcode(0x04))
    );
    assert_eq!(
        program.insert(0, Item::new(0x00, vec![])),
        Err(EditError::OperandCount {
            opcode: 0,
            expected: 1,
            found: 0
        })
    );
    assert_eq!(
        program.insert(0, Item::new(0x01, vec![Operand::Dword(5)])),
        Err(EditError::OperandType {
            opcode: 1,
            index: 0
        })
    );
    assert_eq!(
        program.insert(0, Item::new(0x03, vec![Operand::Str(9)])),
        Err(EditError::UnknownString(9))
    );
    assert_eq!(
        program.insert(0, Item::new(0x00, vec![Operand::Label(0)])),
        Err(EditError::OperandType {
            opcode: 0,
            index: 0
        })
    );
    assert_eq!(
        program.insert(99, Item::new(0x1B, vec![])),
        Err(EditError::IndexOutOfRange(99))
    );
}

#[test]
fn lower_requires_a_final_return() {
    let mut program = lift(&labelled());
    program.remove(3).unwrap();
    assert_eq!(program.lower(), Err(LowerError::MissingFinalReturn));
}

#[test]
fn from_parts_checks_header_size_against_fields() {
    let parts = ProgramParts {
        format: Format::Hunex,
        header_size: 21,
        includes: vec![],
        header_padding: vec![],
        items: vec![Item::new(0x1B, vec![])],
        labels: vec![],
        strings: vec![],
        tail: vec![],
    };
    assert_eq!(
        Program::from_parts(parts.clone()).map(|_| ()),
        Err(EditError::BadHeader)
    );
    let mut fixed = parts;
    fixed.header_size = 8 + 13;
    fixed.header_padding = vec![0; 13];
    assert!(Program::from_parts(fixed).is_ok());
}

#[test]
fn program_label_at_end_of_code_lowers() {
    let mut program = lift(&hunex(&[0x00, 5, 0x1B], b""));
    let end = program.add_label(program.items().len()).unwrap();
    assert_eq!(program.labels()[end], 2);
    assert!(program.lower().is_ok());
}
