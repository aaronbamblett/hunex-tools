use super::compile::*;
use super::normalize::{normalize, NormInstruction, NormOperand};
use crate::source;
use alloc::format;
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
fn strr_pushes_angle_then_slot_and_calls_0x25d() {
    assert_eq!(
        statement_list("_STRR(16,-1386);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(1386)]),
            ins(0x21, vec![]),
            ins(0, vec![num(16)]),
            ins(0x25D, vec![]),
        ]
    );
    assert_eq!(
        statement_list("_STRR(38,9000);", &[]),
        vec![
            ins(0, vec![num(9000)]),
            ins(0, vec![num(38)]),
            ins(0x25D, vec![]),
        ]
    );
}

#[test]
fn fclt_pushes_type_and_calls_0x264() {
    assert_eq!(
        statement_list("_FCLT(9);", &[]),
        vec![ins(0, vec![num(9)]), ins(0x264, vec![])]
    );
}

#[test]
fn sela_save_calls_0x262_without_operands() {
    assert_eq!(
        statement_list("_SELA_SAVE();", &[]),
        vec![ins(0x262, vec![])]
    );
}

#[test]
fn fadw_pushes_preset_then_operand_then_mode_and_calls_0x263() {
    assert_eq!(
        statement_list("_FADW(4,0,0);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(4)]),
            ins(0x263, vec![]),
        ]
    );
    assert_eq!(
        statement_list("_FADW(4,-90,16);", &[]),
        vec![
            ins(0, vec![num(16)]),
            ins(0, vec![num(0)]),
            ins(0, vec![num(90)]),
            ins(0x21, vec![]),
            ins(0, vec![num(4)]),
            ins(0x263, vec![]),
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

fn assert_if_opcode(operator: &str, opcode: u32) {
    assert_eq!(crate::recipes::comparison_op(operator), Some(opcode));
    assert_eq!(
        statement_list(&format!("_IF__(F024,{operator},7,DONE);_ZZ(DONE);"), &[]),
        vec![
            ins(0, vec![num(24)]),
            ins(0, vec![num(0)]),
            ins(0xea, vec![]),
            ins(0, vec![num(7)]),
            ins(opcode, vec![]),
            ins(1, vec![NormOperand::Target(17)]),
            ins(0x19, vec![num(0)]),
        ]
    );
}

#[test]
fn if_eq_calls_0x30() {
    assert_if_opcode("eq", 0x30);
}

#[test]
fn if_ne_calls_0x31() {
    assert_if_opcode("ne", 0x31);
}

#[test]
fn if_le_calls_0x32() {
    assert_if_opcode("le", 0x32);
}

#[test]
fn if_ge_calls_0x33() {
    assert_if_opcode("ge", 0x33);
}

#[test]
fn if_lt_calls_0x34() {
    assert_if_opcode("lt", 0x34);
}

#[test]
fn if_gt_calls_0x35() {
    assert_if_opcode("gt", 0x35);
}

fn steam_if_jumps(list: &[NormInstruction], flag_value: i32) -> bool {
    let mut stack = Vec::new();
    for instruction in list {
        match instruction.opcode() {
            0 => match instruction.operands() {
                [NormOperand::Num(value)] => stack.push(*value as i32),
                other => panic!("unexpected push {other:?}"),
            },
            0xea => {
                assert_eq!(stack.pop(), Some(0));
                assert_eq!(stack.pop(), Some(24));
                stack.push(flag_value);
            }
            0x20 | 0x21 | 0x25 | 0x26 | 0x30..=0x35 => {
                // S 0x14007d120..0x14007d2b0 pop right, then compare left to right.
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                let value = match instruction.opcode() {
                    0x20 => left.wrapping_add(right),
                    0x21 => left.wrapping_sub(right),
                    0x25 => left & right,
                    0x26 => left | right,
                    0x30 => i32::from(left == right),
                    0x31 => i32::from(left != right),
                    0x32 => i32::from(left <= right),
                    0x33 => i32::from(left >= right),
                    0x34 => i32::from(left < right),
                    0x35 => i32::from(left > right),
                    _ => unreachable!(),
                };
                stack.push(value);
            }
            1 => assert!(matches!(instruction.operands(), [NormOperand::Target(_)])),
            0x19 => {
                assert_eq!(instruction.operands(), &[num(0)]);
                let predicate = stack.pop().unwrap();
                assert!(stack.is_empty());
                return predicate == 0;
            }
            other => panic!("unexpected opcode {other:#x}"),
        }
    }
    panic!("missing conditional jump");
}

#[test]
fn if_compares_signed_left_to_right_and_jumps_on_false() {
    type ComparisonTestCase = (&'static str, fn(i32, i32) -> bool);
    let operators: [ComparisonTestCase; 6] = [
        ("eq", |left, right| left == right),
        ("ne", |left, right| left != right),
        ("le", |left, right| left <= right),
        ("ge", |left, right| left >= right),
        ("lt", |left, right| left < right),
        ("gt", |left, right| left > right),
    ];
    for (operator, compare) in operators {
        for (left, right) in [(-3, 7), (7, -3), (7, 7)] {
            let list = statement_list(
                &format!("_IF__(F024,{operator},{right},DONE);_ZZ(DONE);"),
                &[],
            );
            assert_eq!(
                steam_if_jumps(&list, left),
                !compare(left, right),
                "{left} {operator} {right}"
            );
        }
        let list = statement_list(
            &format!("_IF__(F024,{operator},((7|8)&7)+2,DONE);_ZZ(DONE);"),
            &[],
        );
        for left in [-3, 9, 15] {
            assert_eq!(steam_if_jumps(&list, left), !compare(left, 9));
        }
    }
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
        "zero-emission rule not proven on the corpus"
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

#[test]
fn masked_persistent_or_reads_combines_and_stores() {
    assert_eq!(
        statement_list("_WKST(G016,eq,(G016|2)&65535);", &[]),
        vec![
            ins(0, vec![num(16)]),
            ins(0, vec![num(1)]),
            ins(0xea, vec![]),
            ins(0, vec![num(2)]),
            ins(0x26, vec![]),
            ins(0, vec![num(65535)]),
            ins(0x25, vec![]),
            ins(3, vec![text("eq")]),
            ins(0, vec![num(16)]),
            ins(0, vec![num(1)]),
            ins(0xe9, vec![]),
        ]
    );
}

#[test]
fn masked_or_accepts_hexadecimal_operands() {
    for (operand, decimal) in [
        ("1", 1),
        ("2", 2),
        ("0x2", 2),
        ("0xff", 255),
        ("0xABCD", 43981),
    ] {
        assert_eq!(
            statement_list(&format!("_WKST(G016,eq,(G016|({operand}))&65535);"), &[]),
            statement_list(&format!("_WKST(G016,eq,(G016|({decimal}))&65535);"), &[]),
            "{operand}",
        );
    }
}

#[test]
fn mply_start_offset_is_carried_in_mode_without_changing_fades_or_volume() {
    assert_eq!(
        statement_list("_MPLY(8,,`001:3000,,`017:80,,`069:502272);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(0, vec![num(3000)]),
            ins(0, vec![num(0x8000_0001 | (502272 << 2))]),
            ins(0, vec![num(80)]),
            ins(0, vec![num(8)]),
            ins(0x180, vec![]),
        ]
    );
    assert_eq!(
        statement_list("_MPLY(8,,`069:0);", &[]),
        statement_list("_MPLY(8);", &[])
    );
    for src in [
        "_MPLY(8,,`069:536870912);",
        "_MPLY(8,,`069:-1);",
        "_MPLY(8,,`069:1,,`069:2);",
    ] {
        assert!(compile(&crate::source::parse(src).unwrap(), &[]).is_err());
    }
}

#[test]
fn voice_pair_stages_starts_then_waits() {
    assert_eq!(
        statement_list("_VPL2(VOICE,03d00);", &[]),
        vec![
            ins(0, vec![num(0)]),
            ins(3, vec![text("VOICE")]),
            ins(0x1a0, vec![]),
            ins(0x25f, vec![]),
        ]
    );
    assert_eq!(statement_list("_VCWT();", &[]), vec![ins(0x260, vec![])]);
}

#[test]
fn rtm2_preserves_music_without_returning() {
    assert_eq!(statement_list("_RTM2();", &[]), vec![ins(0x261, vec![])]);
}

#[test]
fn sepl_category_shapes_keep_playback_pushes_but_drop_category() {
    for (suffix, delay, fade, volume) in [
        (",,`001:200,,`017:30,,`068:500,,`070:2", 500, 200, 30),
        (",,`001:200,,`017:30,,`070:3", 0, 200, 30),
        (",,`017:30,,`068:500,,`070:10", 500, 0, 30),
        (",,`017:30,,`070:16", 0, 0, 30),
        (",,`070:2", 0, 0, 100),
    ] {
        assert_eq!(
            statement_list(&format!("_SEPL(19,1,FATE_SE142{suffix});"), &[]),
            vec![
                ins(0, vec![num(delay)]),
                ins(0, vec![num(fade)]),
                ins(0, vec![num(64)]),
                ins(0, vec![num(volume)]),
                ins(3, vec![text("FATE_SE142")]),
                ins(0, vec![num(1)]),
                ins(0, vec![num(19)]),
                ins(0x190, vec![]),
            ]
        );
    }
}

#[test]
fn ssw2_scr0510_zero_shapes_emit_no_instructions() {
    for source in ["_SSW2(33,1,6,500,600,0.5,-1,0);", "_SSW2(9,0,0,0,0,0,0,0);"] {
        assert_eq!(statement_list(source, &[]), vec![]);
    }
}

#[test]
fn ssw2_scr0424_800ms_shapes_approximate_with_persistent_stsw() {
    for slot in [6, 7] {
        assert_eq!(
            statement_list(&format!("_SSW2({slot},3,7,1000,5000,1,800,0);"), &[]),
            vec![ins(0, vec![num(slot)]), ins(0x274, vec![])]
        );
    }
}

#[test]
fn tsqk_pushes_the_eleven_record_fields_and_calls_0x273() {
    for (source, expected) in [
        (
            "_TSQK(3,0,1,10,20,2,3,4,6,-1,0);",
            vec![
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(1)]),
                ins(0x21, vec![]),
                ins(0, vec![num(6)]),
                ins(0, vec![num(4)]),
                ins(0, vec![num(3)]),
                ins(0, vec![num(2)]),
                ins(0, vec![num(20)]),
                ins(0, vec![num(10)]),
                ins(0, vec![num(1)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(3)]),
                ins(0x273, vec![]),
            ],
        ),
        (
            "_TSQK();",
            vec![
                ins(0, vec![num(0)]),
                ins(0, vec![num(0xffffffff)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(1)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(0)]),
                ins(0x273, vec![]),
            ],
        ),
    ] {
        assert_eq!(statement_list(source, &[]), expected, "{source}");
    }

    let text = "_TSQK(3,0,1,10,20,2,3,4,6,-1,0);_TSQK(-1,1,3,7,15,60,120,1,0,-1,5);";
    let statements = source::parse(text).unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let script = compiled.program().lower().unwrap();
    assert_eq!(
        script
            .instructions()
            .iter()
            .filter(|instruction| instruction.opcode() == 0x273)
            .count(),
        2
    );
}

#[test]
fn tiny_script_compiles_to_a_header_matching_real_files() {
    let statements = source::parse("_END_();").unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let bytes = compiled.program().lower().unwrap().encode();
    assert_eq!(&bytes[..27], b"HunexCompiledScriptVer1.00\0");
    assert_eq!(u32::from_le_bytes(bytes[27..31].try_into().unwrap()), 21);
    assert_eq!(u32::from_le_bytes(bytes[31..35].try_into().unwrap()), 0);
    assert_eq!(&bytes[35..48], &[0u8; 13]);
    let script = crate::Script::parse(&bytes).unwrap();
    assert_eq!(script.encode(), bytes);
}

#[test]
fn tqst_pushes_bank_before_slot_and_calls_0x271() {
    for (source, expected) in [
        (
            "_TQST(7,0);",
            vec![
                ins(0, vec![num(0)]),
                ins(0, vec![num(7)]),
                ins(0x271, vec![]),
            ],
        ),
        (
            "_TQST(7,1);",
            vec![
                ins(0, vec![num(1)]),
                ins(0, vec![num(7)]),
                ins(0x271, vec![]),
            ],
        ),
        (
            "_TQST(-1,1);",
            vec![
                ins(0, vec![num(1)]),
                ins(0, vec![num(0)]),
                ins(0, vec![num(1)]),
                ins(0x21, vec![]),
                ins(0x271, vec![]),
            ],
        ),
    ] {
        assert_eq!(statement_list(source, &[]), expected, "{source}");
    }
}

#[test]
fn shake_bank_spellings_compile_and_round_trip() {
    let text = "_STQK(7,0,0,34,-1,0,0);_STQK(-1,6,12,34,800,0,0);_STSW(13);                _TQST(7,0);_TQST(-1,1);_TQST(7,1);";
    let statements = source::parse(text).unwrap();
    let compiled = compile(&statements, &[]).unwrap();
    let script = compiled.program().lower().unwrap();
    assert_eq!(
        script
            .instructions()
            .iter()
            .filter(|instruction| instruction.opcode() == 0x271)
            .count(),
        3
    );
}
