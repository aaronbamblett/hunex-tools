use super::compile::*;
use super::normalize::{normalize, NormInstruction, NormOperand};
use crate::source;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

fn statement_list(src: &str, scene: &[(&str, &str)]) -> Vec<NormInstruction> {
    let statements = source::parse(src).unwrap();
    let compiled = compile(&statements, scene).unwrap();
    let list = normalize(compiled.program()).unwrap();
    let (start, end) = compiled.spans()[0];
    list[start..end].to_vec()
}

fn num(value: u32) -> NormOperand {
    NormOperand::Num(value)
}

fn text(value: &str) -> NormOperand {
    NormOperand::Text(value.as_bytes().to_vec())
}

fn ins(opcode: u32, operands: Vec<NormOperand>) -> NormInstruction {
    NormInstruction::new(opcode, operands)
}

#[test]
fn flag_write_matches_the_worked_example() {
    assert_eq!(
        statement_list("_WKST(F023,eq,1);", &[]),
        vec![
            ins(0, vec![num(1)]),
            ins(3, vec![text("eq")]),
            ins(0, vec![num(23)]),
            ins(0, vec![num(0)]),
            ins(233, vec![]),
        ]
    );
}

#[test]
fn shok_keeps_source_values_in_reverse_order() {
    assert_eq!(
        statement_list("_SHOK(0,7,300,2);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(2)]),
            ins(0, vec![num(300)]),
            ins(0, vec![num(7)]),
            ins(0, vec![num(0)]),
            ins(832, vec![]),
        ]
    );
}

#[test]
fn shok_five_arguments_use_the_source_delay() {
    assert_eq!(
        statement_list("_SHOK(6,0,300,1,400);", &[]),
        vec![
            ins(0, vec![num(400)]),
            ins(0, vec![num(1)]),
            ins(0, vec![num(300)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(6)]),
            ins(832, vec![]),
        ]
    );
}

#[test]
fn cktt_keeps_leading_zeros_as_six() {
    let list = statement_list("_CKTT(19,06,41);", &[]);
    assert_eq!(
        list,
        vec![
            ins(0, vec![num(41)]),
            ins(0, vec![num(6)]),
            ins(0, vec![num(19)]),
            ins(897, vec![]),
        ]
    );
}

#[test]
fn stcg_image_and_symbol_names_compile_to_the_same_bytes() {
    assert_eq!(
        statement_list("_STCG(tsu_img0001);", &[]),
        vec![ins(3, vec![text("tsu_img0001")]), ins(552, vec![])]
    );
    assert_eq!(
        statement_list("_STCG(img0242);", &[]),
        vec![ins(3, vec![text("img0242")]), ins(552, vec![])]
    );
}

#[test]
fn stcg_rejects_a_non_name_argument() {
    let statements = source::parse("_STCG(5);").unwrap();
    assert!(compile(&statements, &[]).is_err());
}

#[test]
fn fcsp_with_a_duration_pushes_it_and_calls_0x25e() {
    assert_eq!(
        statement_list("_FCSP(2000);", &[]),
        vec![ins(0, vec![num(2000)]), ins(0x25E, vec![])]
    );
    assert_eq!(statement_list("_FCSP();", &[]), vec![ins(0x3A1, vec![])]);
}

#[test]
fn sbas_pushes_parent_then_child_and_calls_0x25c() {
    assert_eq!(
        statement_list("_SBAS(6,5);", &[]),
        vec![
            ins(0, vec![num(5)]),
            ins(0, vec![num(6)]),
            ins(0x25C, vec![])
        ]
    );
    assert_eq!(
        statement_list("_SBAS(37,-1);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(1)]),
            ins(0x21, vec![]),
            ins(0, vec![num(37)]),
            ins(0x25C, vec![]),
        ]
    );
}

#[test]
fn tomp_with_an_index_calls_0x1c2_and_bare_tomp_keeps_0x1c0() {
    assert_eq!(
        statement_list("_TOMP(8);", &[]),
        vec![ins(0, vec![num(8)]), ins(0x1C2, vec![])]
    );
    assert_eq!(statement_list("_TOMP();", &[]), vec![ins(0x1C0, vec![])]);
}

#[test]
fn negative_integers_emit_zero_minus_and_sub() {
    assert_eq!(
        statement_list("_TTCT(-1);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(1)]),
            ins(0x21, vec![]),
            ins(316, vec![]),
        ]
    );
}

#[test]
fn ckdp_negative_arguments_emit_postfix_subtraction() {
    assert_eq!(
        statement_list("_CKDP(2,10,51,-375,-283,1000,35);", &[]),
        vec![
            ins(0, vec![num(35)]),
            ins(0, vec![num(1000)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(283)]),
            ins(33, vec![]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(375)]),
            ins(33, vec![]),
            ins(0, vec![num(51)]),
            ins(0, vec![num(10)]),
            ins(0, vec![num(2)]),
            ins(896, vec![]),
        ]
    );
}

#[test]
fn sttr_negative_expression_keeps_the_subtraction_shape() {
    assert_eq!(
        statement_list("_STTR(23,-1);", &[]),
        vec![
            ins(0, vec![num(256)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(1)]),
            ins(33, vec![]),
            ins(0, vec![num(23)]),
            ins(603, vec![]),
        ]
    );
}

#[test]
fn stgs_empty_field_uses_the_sentinel_default() {
    assert_eq!(
        statement_list("_STGS(0,,255,600,3);", &[]),
        vec![
            ins(0, vec![num(3)]),
            ins(0, vec![num(600)]),
            ins(0, vec![num(255)]),
            ins(0, vec![num(2147483648)]),
            ins(0, vec![num(0)]),
            ins(581, vec![]),
        ]
    );
}

#[test]
fn wttm_absent_field_uses_the_absent_default() {
    assert_eq!(
        statement_list("_WTTM(330,1);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(1)]),
            ins(0, vec![num(330)]),
            ins(272, vec![]),
        ]
    );
}

#[test]
fn selr_numeric_text_pushes_the_raw_spelling_as_a_string() {
    assert_eq!(
        statement_list("_SELR(0,1);", &[]),
        vec![
            ins(3, vec![text("1")]),
            ins(0, vec![num(0)]),
            ins(352, vec![]),
        ]
    );
}

#[test]
fn wkst_wintye2_keeps_the_enum_spelling_in_a_literal() {
    let statements = source::parse("_WKST(F1105,eq,WINTYE_2);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    assert_eq!(compiled.selected(), &[Some("_WKST:wintye2")]);
    assert_eq!(
        statement_list("_WKST(F1105,eq,WINTYE_2);", &[]),
        vec![
            ins(0, vec![num(2)]),
            ins(3, vec![text("eq")]),
            ins(0, vec![num(1105)]),
            ins(0, vec![num(0)]),
            ins(233, vec![]),
        ]
    );
}

#[test]
fn wkst_unknown_enum_has_no_matching_recipe() {
    let statements = source::parse("_WKST(F1105,eq,SOMETHING_ELSE);").unwrap();
    let error = compile(&statements, &[]).unwrap_err();
    assert_eq!(error.kind(), &CompileErrorKind::NoMatchingRecipe);
    assert_eq!(error.offset(), 0);
}

#[test]
fn jump_resolves_the_scene_name_through_the_input_map() {
    assert_eq!(
        statement_list("_JUMP(201);", &[("201", "1_1")]),
        vec![
            ins(0, vec![num(0)]),
            ins(246, vec![]),
            ins(3, vec![text("1_1")]),
            ins(243, vec![]),
        ]
    );
}

#[test]
fn jump_missing_scene_number_is_an_error_with_offset() {
    let statements = source::parse("_PGST(1);_JUMP(999);").unwrap();
    let error = compile(&statements, &[("201", "1_1")]).unwrap_err();
    assert_eq!(error.kind(), &CompileErrorKind::UnknownSceneNumber);
    assert_eq!(error.statement(), 1);
    assert_eq!(error.offset(), statements[1].offset());
}

#[test]
fn message_emits_metadata_mode_and_call() {
    let statements = source::parse("_ZM01502($000008^$000009);_WKST(F823,eq,0);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let list = normalize(compiled.program()).unwrap();
    let body = "$000008\n$000009";
    let (first, second) = (compiled.spans()[0], compiled.spans()[1]);
    assert_eq!(
        list[first.0..first.1],
        vec![
            ins(123, vec![num(0x12345678), num(0), text(body)]),
            ins(0, vec![num(1)]),
            ins(0, vec![num(1)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(0)]),
            ins(3, vec![text(body)]),
            ins(320, vec![]),
        ]
    );
    assert_eq!(list[second.0..second.1], vec![ins(321, vec![])]);
}

#[test]
fn dispatch_and_conditional_match_the_worked_stream() {
    let src =
        "_ZZ01401();_ZZ01501(START);_IF__(F024,eq,1,LV00000);_RTM_();_ZY01600(LV00000);_END_();";
    let statements = source::parse(src).unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let list = normalize(compiled.program()).unwrap();
    let target = NormOperand::Target;
    let error = "指定されたラベルは見つかりませんでした。";
    assert_eq!(
        list,
        vec![
            ins(0, vec![num(0)]),
            ins(247, vec![]),
            ins(1, vec![target(10)]),
            ins(25, vec![num(1)]),
            ins(3, vec![text("START")]),
            ins(247, vec![]),
            ins(1, vec![target(10)]),
            ins(25, vec![num(1)]),
            ins(3, vec![text(error)]),
            ins(249, vec![]),
            ins(0, vec![num(24)]),
            ins(0, vec![num(0)]),
            ins(234, vec![]),
            ins(0, vec![num(1)]),
            ins(48, vec![]),
            ins(1, vec![target(20)]),
            ins(25, vec![num(0)]),
            ins(244, vec![]),
            ins(1, vec![target(20)]),
            ins(24, vec![]),
            ins(300, vec![]),
            ins(27, vec![]),
        ]
    );
}

#[test]
fn platform_if_emits_the_comparison_as_a_call() {
    let statements = source::parse("_ZZ(LV00000);_IFP4(,LV00000);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let list = normalize(compiled.program()).unwrap();
    let (start, end) = compiled.spans()[1];
    assert_eq!(
        list[start..end],
        vec![
            ins(0, vec![num(0)]),
            ins(305, vec![]),
            ins(48, vec![]),
            ins(1, vec![NormOperand::Target(10)]),
            ins(25, vec![num(0)]),
        ]
    );
}

#[test]
fn zero_emission_recipes_leave_empty_spans_but_set_state() {
    let statements = source::parse("_WKST(F822,eq,1);_WNTY(0);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    assert_eq!(compiled.spans(), &[(6, 6), (6, 8)]);
    let list = normalize(compiled.program()).unwrap();
    assert_eq!(list[6..8], vec![ins(0, vec![num(1)]), ins(344, vec![])],);
}

#[test]
fn unproven_zero_emission_selections_warn_without_emitting() {
    let statements = source::parse("_WKST(F822,eq,1);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    assert_eq!(compiled.spans(), &[(6, 6)]);
    let warnings = compiled.warnings();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].offset(), statements[0].offset());
    assert_eq!(warnings[0].command(), "_WKST");
    assert_eq!(
        warnings[0].args(),
        &[String::from("F822"), String::from("eq"), String::from("1")]
    );
    assert_eq!(
        warnings[0].reason(),
        "zero-emission predicate not proven on the corpus"
    );

    let statements = source::parse("_WKST(F848,eq,63);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    assert_eq!(compiled.selected(), &[Some("_WKST:5")]);
    assert!(compiled.warnings().is_empty());
}

#[test]
fn unknown_command_reports_its_statement_offset() {
    let statements = source::parse("_PGST(1);_NOPE(1);").unwrap();
    let error = compile(&statements, &[]).unwrap_err();
    assert_eq!(error.kind(), &CompileErrorKind::UnknownCommand);
    assert_eq!(error.statement(), 1);
    assert_eq!(error.offset(), statements[1].offset());
}

#[test]
fn unresolved_label_reports_the_referring_statement() {
    let statements = source::parse("_PGST(1);_GOTO(NOWHERE);").unwrap();
    let error = compile(&statements, &[]).unwrap_err();
    assert_eq!(error.kind(), &CompileErrorKind::UnresolvedLabel);
    assert_eq!(error.statement(), 1);
    assert_eq!(error.offset(), statements[1].offset());
}

#[test]
fn numeric_pushes_from_source_arguments_are_tracked() {
    let statements = source::parse("_CKTT(19,06,41);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let fields: Vec<usize> = compiled
        .literal_slots()
        .iter()
        .map(|slot| slot.field)
        .collect();
    assert_eq!(fields, vec![2, 1, 0]);
    assert!(compiled
        .literal_slots()
        .iter()
        .all(|slot| slot.encoder == SlotEncoder::Value));
}

#[test]
fn integers_past_u32_are_a_range_error() {
    for source in ["_PGST(4294967296);", "_PGST(-4294967296);"] {
        let statements = source::parse(source).unwrap();
        let error = compile(&statements, &[]).unwrap_err();
        assert_eq!(error.kind(), &CompileErrorKind::NumberOutOfRange);
        assert_eq!(error.statement(), 0);
    }
}

#[test]
fn integers_at_the_dword_limits_still_compile() {
    for source in ["_PGST(4294967295);", "_PGST(-4294967295);"] {
        let statements = source::parse(source).unwrap();
        compile(&statements, &[]).unwrap();
    }
}

#[test]
fn defining_a_label_twice_is_an_error() {
    let statements = source::parse("_ZZ(a);_ZZ(a);_END_();").unwrap();
    let error = compile(&statements, &[]).unwrap_err();
    assert_eq!(error.kind(), &CompileErrorKind::DuplicateLabel);
    assert_eq!(error.statement(), 1);
    assert_eq!(error.offset(), statements[1].offset());
}

#[test]
fn the_prologue_entry_label_is_outside_the_script_namespace() {
    let statements = source::parse("_ZZ(entry);_GOTO(entry);_END_();").unwrap();
    compile(&statements, &[]).unwrap();
    let statements = source::parse("_GOTO(entry);_END_();").unwrap();
    let error = compile(&statements, &[]).unwrap_err();
    assert_eq!(error.kind(), &CompileErrorKind::UnresolvedLabel);
}

#[test]
fn string_pool_uses_first_use_order() {
    let statements = source::parse("_SELR(0,1);").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let strings: Vec<&[u8]> = compiled
        .program()
        .strings()
        .iter()
        .map(|entry| entry.bytes())
        .collect();
    assert_eq!(strings[0], crate::recipes::PROLOGUE_ERROR.as_bytes());
    assert_eq!(strings[1], "1".as_bytes());
}
