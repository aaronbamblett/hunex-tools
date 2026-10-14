use super::normalize::*;
use crate::{Format, Item, Operand, PoolEntry, Program, ProgramParts};
use alloc::vec;

const MARKER: u32 = 0x1234_5678;

fn program() -> Program {
    Program::from_parts(ProgramParts {
        format: Format::Hunex,
        header_size: 8,
        includes: vec![],
        header_padding: vec![],
        items: vec![
            Item::new(0x00, vec![Operand::Dword(5)]),
            Item::new(0x7F, vec![Operand::Str(1), Operand::Dword(12)]),
            Item::new(0x01, vec![Operand::Label(0)]),
            Item::new(0x03, vec![Operand::Str(0)]),
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Str(0)],
            ),
            Item::new(0x1B, vec![]),
        ],
        labels: vec![3],
        strings: vec![
            PoolEntry::new(b"hi".to_vec(), true).unwrap(),
            PoolEntry::new(b"dbg".to_vec(), true).unwrap(),
        ],
        tail: vec![],
    })
    .unwrap()
}

#[test]
fn debug_is_dropped_and_strings_and_targets_resolve() {
    let list = normalize(&program()).unwrap();
    assert_eq!(list.len(), 5);
    assert!(list.iter().all(|item| item.opcode() != 0x7F));
    assert_eq!(
        list[0],
        NormInstruction::new(0x00, vec![NormOperand::Num(5)])
    );
    assert_eq!(
        list[1],
        NormInstruction::new(0x01, vec![NormOperand::Target(2)])
    );
    assert_eq!(
        list[2],
        NormInstruction::new(0x03, vec![NormOperand::Text(b"hi".to_vec())])
    );
    assert_eq!(
        list[3],
        NormInstruction::new(
            0x7B,
            vec![
                NormOperand::Num(MARKER),
                NormOperand::Num(0),
                NormOperand::Text(b"hi".to_vec())
            ]
        )
    );
    assert_eq!(list[4], NormInstruction::new(0x1B, vec![]));
}

#[test]
fn target_on_debug_maps_to_next_kept_instruction() {
    let mut edits = program();
    edits.move_label(0, 1).unwrap();
    let list = normalize(&edits).unwrap();
    assert_eq!(list[1].operands(), &[NormOperand::Target(1)]);
}

#[test]
fn target_on_end_of_code_maps_to_list_length() {
    let mut edits = program();
    edits.move_label(0, edits.items().len()).unwrap();
    let list = normalize(&edits).unwrap();
    assert_eq!(list[1].operands(), &[NormOperand::Target(5)]);
}

#[test]
fn target_never_compares_equal_to_a_literal_number() {
    let list = normalize(&program()).unwrap();
    assert_ne!(
        list[1].operands(),
        &[NormOperand::Num(2)],
        "a jump target must not equal the same literal number"
    );
}

#[test]
fn plain_dword_third_operand_of_skip_is_preserved() {
    let mut edits = program();
    edits.remove(4).unwrap();
    edits
        .insert(
            4,
            Item::new(
                0x7B,
                vec![Operand::Dword(MARKER), Operand::Dword(0), Operand::Dword(9)],
            ),
        )
        .unwrap();
    let list = normalize(&edits).unwrap();
    assert_eq!(
        list[3].operands()[2],
        NormOperand::Num(9),
        "a non-string 0x7B operand stays numeric"
    );
}

#[test]
fn first_difference_finds_the_diverging_index() {
    let list = normalize(&program()).unwrap();
    assert_eq!(first_difference(&[], &[]), None);
    assert_eq!(first_difference(&list, &list.clone()), None);
    let mut changed = list.clone();
    changed[3] = NormInstruction::new(0x7B, vec![NormOperand::Num(0)]);
    assert_eq!(first_difference(&list, &changed), Some(3));
    assert_eq!(first_difference(&list, &list[..3]), Some(3));
    assert_eq!(first_difference(&list[..3], &list), Some(3));
}

#[test]
fn plain_dword_operands_pass_through() {
    let mut edits = program();
    edits.remove(2).unwrap();
    edits
        .insert(2, Item::new(0x00, vec![Operand::Dword(0)]))
        .unwrap();
    let list = normalize(&edits).unwrap();
    assert_eq!(
        list[1],
        NormInstruction::new(0x00, vec![NormOperand::Num(0)])
    );
}
