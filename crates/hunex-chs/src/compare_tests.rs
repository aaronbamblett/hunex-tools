use super::compare::*;
use super::normalize::{NormInstruction, NormOperand};
use alloc::vec::Vec;

fn ins(opcode: u32, operands: Vec<NormOperand>) -> NormInstruction {
    NormInstruction::new(opcode, operands)
}

fn ret() -> Vec<NormInstruction> {
    alloc::vec![ins(0x1B, Vec::new())]
}

fn input<'a>(
    expected: &'a [NormInstruction],
    actual: &'a [NormInstruction],
    slots: &'a [usize],
) -> SceneInput<'a> {
    SceneInput {
        expected,
        actual,
        slots,
        owners: &[],
        switch_chars: &[],
        steam_offsets: &[],
        pool_start: 0,
        has_problems: false,
        authored: &[],
    }
}

#[test]
fn identical_lists_are_exact() {
    let list = ret();
    assert_eq!(compare(&input(&list, &list, &[])).class, SceneClass::Exact);
}

#[test]
fn tracked_numeric_drift_is_shape() {
    let expected = alloc::vec![ins(0, alloc::vec![NormOperand::Num(330)])];
    let actual = alloc::vec![ins(0, alloc::vec![NormOperand::Num(380)])];
    assert_eq!(
        compare(&input(&expected, &actual, &[])).class,
        SceneClass::Structural
    );
    assert_eq!(
        compare(&input(&expected, &actual, &[0])).class,
        SceneClass::Shape
    );
}

#[test]
fn shape_never_relaxes_opcodes_strings_or_targets() {
    let slot = &[0];
    for (expected, actual) in [
        (
            ins(33, alloc::vec![NormOperand::Num(0)]),
            ins(33, alloc::vec![NormOperand::Num(1)]),
        ),
        (
            ins(3, alloc::vec![NormOperand::Text(b"a".to_vec())]),
            ins(3, alloc::vec![NormOperand::Text(b"b".to_vec())]),
        ),
        (
            ins(1, alloc::vec![NormOperand::Target(2)]),
            ins(1, alloc::vec![NormOperand::Target(3)]),
        ),
        (
            ins(0, alloc::vec![NormOperand::Num(0), NormOperand::Num(1)]),
            ins(0, alloc::vec![NormOperand::Num(0), NormOperand::Num(2)]),
        ),
    ] {
        let expected = alloc::vec![expected];
        let actual = alloc::vec![actual];
        assert_eq!(
            compare(&input(&expected, &actual, slot)).class,
            SceneClass::Structural
        );
    }
}

#[test]
fn target_never_equals_a_literal_number() {
    let expected = alloc::vec![ins(1, alloc::vec![NormOperand::Target(2)])];
    let actual = alloc::vec![ins(1, alloc::vec![NormOperand::Num(2)])];
    let result = compare(&input(&expected, &actual, &[0]));
    assert_eq!(result.class, SceneClass::Structural);
    assert_eq!(result.exact_divergence, Some(0));
}

#[test]
fn reviewed_authored_edits_lift_a_scene_to_authored() {
    let expected = alloc::vec![
        ins(0, alloc::vec![NormOperand::Num(1)]),
        ins(0, alloc::vec![NormOperand::Num(2)]),
        ins(0x1B, Vec::new()),
    ];
    let actual = alloc::vec![
        ins(0, alloc::vec![NormOperand::Num(1)]),
        ins(0, alloc::vec![NormOperand::Num(9)]),
        ins(0x1B, Vec::new()),
    ];
    let plain = compare(&input(&expected, &actual, &[]));
    assert_eq!(plain.class, SceneClass::Structural);
    let entry = AuthoredDiff {
        source_index: 1,
        steam_index: 1,
        source_value: &expected[1..2],
        steam_value: &actual[1..2],
        switch_char: None,
        steam_offset: 0,
        reviewed: true,
        has_evidence: true,
    };
    let with_authored = SceneInput {
        authored: &[entry],
        ..input(&expected, &actual, &[])
    };
    let result = compare(&with_authored);
    assert!(result.authored_matching);
    assert_eq!(result.class, SceneClass::Authored);
}

#[test]
fn unreviewed_authored_entries_keep_the_scene_structural() {
    let expected = alloc::vec![ins(0, alloc::vec![NormOperand::Num(2)])];
    let actual = alloc::vec![ins(0, alloc::vec![NormOperand::Num(9)])];
    let entry = AuthoredDiff {
        source_index: 0,
        steam_index: 0,
        source_value: &expected[..],
        steam_value: &actual[..],
        switch_char: None,
        steam_offset: 0,
        reviewed: false,
        has_evidence: true,
    };
    let with_authored = SceneInput {
        authored: &[entry],
        ..input(&expected, &actual, &[])
    };
    let result = compare(&with_authored);
    assert_eq!(result.class, SceneClass::Structural);
    assert_eq!(
        result.authored_failure,
        Some(AuthoredFailure::StaleOrUnreviewed)
    );
}

#[test]
fn diagnostics_prevent_acceptance() {
    let list = ret();
    let troubled = SceneInput {
        has_problems: true,
        ..input(&list, &list, &[])
    };
    assert_eq!(compare(&troubled).class, SceneClass::Structural);
}

#[test]
fn overflowing_manifest_indices_are_stale_not_a_panic() {
    let list = ret();
    let entry = AuthoredDiff {
        source_index: usize::MAX,
        steam_index: 0,
        source_value: &list,
        steam_value: &[],
        switch_char: None,
        steam_offset: 0,
        reviewed: true,
        has_evidence: true,
    };
    let with_authored = SceneInput {
        authored: &[entry],
        ..input(&list, &list, &[])
    };
    let result = compare(&with_authored);
    assert_eq!(
        result.authored_failure,
        Some(AuthoredFailure::StaleOrUnreviewed)
    );
}

#[test]
fn class_labels_match_the_reference() {
    assert_eq!(SceneClass::Exact.label(), "exact match");
    assert_eq!(SceneClass::Shape.label(), "shape match with value drift");
    assert_eq!(SceneClass::Authored.label(), "authored");
    assert_eq!(SceneClass::Structural.label(), "structural mismatch");
}
