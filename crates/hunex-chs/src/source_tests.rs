use super::source::*;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

fn names(statements: &[Statement]) -> Vec<&str> {
    statements.iter().map(Statement::name).collect()
}

#[test]
fn empty_and_whitespace_only_scripts_parse_to_nothing() {
    assert_eq!(parse("").unwrap(), vec![]);
    assert_eq!(parse("  \r\n\t　").unwrap(), vec![]);
}

#[test]
fn leading_bom_and_trailing_nuls_are_allowed() {
    let statements = parse("\u{FEFF}_PGST(1);\0\0").unwrap();
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].offset(), 0);
    assert_eq!(statements[0].name(), "_PGST");
}

#[test]
fn offsets_are_char_indices_after_bom_removal() {
    let statements = parse("_ZM(あ);_PGST(1);").unwrap();
    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].offset(), 0);
    assert_eq!(statements[1].offset(), 7);
    assert_eq!(names(&statements), ["_ZM", "_PGST"]);
}

#[test]
fn empty_argument_list_and_empty_fields() {
    let statements = parse("_END_();_STCH(0,,);").unwrap();
    assert_eq!(statements[0].args(), &[]);
    assert_eq!(
        statements[1]
            .args()
            .iter()
            .map(Value::kind)
            .collect::<Vec<_>>(),
        [ValueKind::Int, ValueKind::Empty, ValueKind::Empty]
    );
    assert_eq!(statements[1].args()[1].raw(), "");
}

#[test]
fn family_suffix_canonicalises_but_keeps_spelling() {
    let statements = parse("_ZM01502(x);_ZZ01401();_ZY01600(LV00000);_ZZ(TOP);").unwrap();
    assert_eq!(names(&statements), ["_ZM", "_ZZ", "_ZY", "_ZZ"]);
    assert_eq!(statements[0].raw_name(), "_ZM01502");
    assert_eq!(statements[2].raw_name(), "_ZY01600");
    assert_eq!(statements[3].raw_name(), "_ZZ");
}

#[test]
fn non_hex_suffix_is_not_canonicalised() {
    let statements = parse("_ZMG0(x);").unwrap();
    assert_eq!(statements[0].name(), "_ZMG0");
}

#[test]
fn text_replaces_carets_with_lf() {
    let statements = parse("_ZM($000008^$000009);").unwrap();
    assert_eq!(
        statements[0].args()[0],
        Value::Text {
            raw: String::from("$000008^$000009"),
            text: String::from("$000008\n$000009"),
        }
    );
}

#[test]
fn tag_splits_on_colons() {
    let statements = parse("_CMD(`001:10:20);").unwrap();
    assert_eq!(
        statements[0].args()[0],
        Value::Tag {
            raw: String::from("`001:10:20"),
            tag: String::from("001"),
            parts: vec![String::from("10"), String::from("20")],
        }
    );
}

#[test]
fn flags_map_to_banks() {
    let statements = parse("_WKST(F023,G5,_D7);").unwrap();
    let args = statements[0].args();
    assert_eq!(args[0].kind(), ValueKind::Flag);
    assert!(matches!(
        args[0],
        Value::Flag {
            index: 23,
            bank: 0,
            ..
        }
    ));
    assert!(matches!(
        args[1],
        Value::Flag {
            index: 5,
            bank: 1,
            ..
        }
    ));
    assert!(matches!(
        args[2],
        Value::Flag {
            index: 7,
            bank: 2,
            ..
        }
    ));
}

#[test]
fn ints_cover_decimal_negative_hex_and_leading_zeros() {
    let statements = parse("_CMD(06,-375,0x20D00,0);").unwrap();
    let values: Vec<i64> = statements[0]
        .args()
        .iter()
        .map(|arg| match arg {
            Value::Int { value, .. } => *value,
            other => panic!("expected int, found {other:?}"),
        })
        .collect();
    assert_eq!(values, [6, -375, 0x20D00, 0]);
}

#[test]
fn expression_keeps_addition_and_subtraction_structure() {
    let statements = parse("_STCH(262-22,158+1);").unwrap();
    assert_eq!(
        statements[0].args()[0],
        Value::Expression {
            raw: String::from("262-22"),
            expr: Expr::Sub(Box::new(Expr::Lit(262)), Box::new(Expr::Lit(22))),
        }
    );
    assert_eq!(
        statements[0].args()[1],
        Value::Expression {
            raw: String::from("158+1"),
            expr: Expr::Add(Box::new(Expr::Lit(158)), Box::new(Expr::Lit(1))),
        }
    );
}

#[test]
fn expression_supports_parens_unary_minus_and_spaces() {
    let statements = parse("_CMD((1+2)- -3);").unwrap();
    assert_eq!(
        statements[0].args()[0],
        Value::Expression {
            raw: String::from("(1+2)- -3"),
            expr: Expr::Sub(
                Box::new(Expr::Add(Box::new(Expr::Lit(1)), Box::new(Expr::Lit(2)))),
                Box::new(Expr::Neg(Box::new(Expr::Lit(3)))),
            ),
        }
    );
}

#[test]
fn expression_rejects_multiplication_division_and_remainder() {
    for raw in ["_CMD(3*4);", "_CMD(4/2);", "_CMD(5%2);"] {
        assert_eq!(
            parse(raw),
            Err(ParseError::UnsupportedExpression { offset: 0 })
        );
    }
}

#[test]
fn image_and_symbol_kinds() {
    let statements = parse("_STCH(img0408,CC,eq,あ);").unwrap();
    let kinds: Vec<ValueKind> = statements[0].args().iter().map(Value::kind).collect();
    assert_eq!(
        kinds,
        [
            ValueKind::Image,
            ValueKind::Symbol,
            ValueKind::Symbol,
            ValueKind::Symbol
        ]
    );
    assert_eq!(statements[0].args()[0].raw(), "img0408");
}

#[test]
fn commas_inside_parens_do_not_split() {
    let statements = parse("_CMD((1,2),3);").unwrap();
    assert_eq!(statements[0].args().len(), 2);
    assert_eq!(statements[0].args()[0].raw(), "(1,2)");
}

#[test]
fn malformed_kwrs_token_parses_with_no_arguments() {
    let statements = parse("_PGST(1);_KWRS);_PGST(2);").unwrap();
    assert_eq!(statements.len(), 3);
    assert!(statements[1].malformed());
    assert_eq!(statements[1].name(), "_KWRS");
    assert_eq!(statements[1].args(), &[]);
    assert!(!statements[0].malformed());
}

#[test]
fn unexpected_syntax_carries_the_char_offset() {
    assert_eq!(
        parse("_PGST(1);oops"),
        Err(ParseError::UnexpectedSyntax { offset: 9 })
    );
    assert_eq!(
        parse("hello"),
        Err(ParseError::UnexpectedSyntax { offset: 0 })
    );
}

#[test]
fn unterminated_statements_are_errors() {
    assert_eq!(
        parse("_PGST(1)"),
        Err(ParseError::UnterminatedStatement { offset: 0 })
    );
    assert_eq!(
        parse("_PGST(1"),
        Err(ParseError::UnterminatedStatement { offset: 0 })
    );
    assert_eq!(
        parse("_PGST((1);"),
        Err(ParseError::UnterminatedStatement { offset: 0 })
    );
}

#[test]
fn error_offset_is_readable() {
    assert_eq!(ParseError::UnexpectedSyntax { offset: 9 }.offset(), 9);
}

#[test]
fn deep_parens_are_an_error_not_a_stack_overflow() {
    let depth = MAX_EXPR_NESTING + 44;
    let mut arg = String::from("_PGST(");
    for _ in 0..depth {
        arg.push('(');
    }
    arg.push('1');
    for _ in 0..depth {
        arg.push(')');
    }
    arg.push_str(");");
    assert_eq!(
        parse(&arg),
        Err(ParseError::ExpressionTooDeep { offset: 0 })
    );
}

#[test]
fn a_long_addition_chain_is_an_error_not_a_stack_overflow() {
    let mut arg = String::from("_PGST(1");
    for _ in 0..MAX_EXPR_NODES {
        arg.push_str("+1");
    }
    arg.push_str(");");
    assert_eq!(
        parse(&arg),
        Err(ParseError::ExpressionTooDeep { offset: 0 })
    );
}

#[test]
fn a_long_minus_chain_is_an_error_not_a_stack_overflow() {
    let mut arg = String::from("_PGST(");
    for _ in 0..MAX_EXPR_NESTING + 44 {
        arg.push('-');
    }
    arg.push_str("1);");
    assert_eq!(
        parse(&arg),
        Err(ParseError::ExpressionTooDeep { offset: 0 })
    );
}

#[test]
fn expressions_at_the_limits_still_parse() {
    let mut arg = String::from("_PGST(");
    for _ in 0..MAX_EXPR_NESTING {
        arg.push('(');
    }
    arg.push('1');
    for _ in 0..MAX_EXPR_NESTING {
        arg.push(')');
    }
    arg.push_str(");");
    assert_eq!(parse(&arg).unwrap().len(), 1);
    let mut chain = String::from("_PGST(1");
    for _ in 0..(MAX_EXPR_NODES - 1) / 2 {
        chain.push_str("+1");
    }
    chain.push_str(");");
    assert_eq!(parse(&chain).unwrap().len(), 1);
}

#[test]
fn hex_lookalikes_remain_symbols() {
    let statements = parse("_A(C0x1,C0xAB);").unwrap();
    assert!(statements[0]
        .args()
        .iter()
        .all(|arg| matches!(arg, Value::Symbol { .. })));
}
