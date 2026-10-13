use super::*;

#[test]
fn parses_crlf_lines_and_writes_them_back() {
    let body = b"first\r\n\r\n\xe6\x97\xa5\xe6\x9c\xac\r\n";
    let text = Text::parse(body).unwrap();
    assert_eq!(text.lines(), ["first", "", "\u{65e5}\u{672c}"]);
    assert_eq!(text.write(), body);
}

#[test]
fn empty_body_has_no_lines() {
    let text = Text::parse(b"").unwrap();
    assert!(text.is_empty());
    assert_eq!(text.write(), b"");
}

#[test]
fn rejects_bare_line_breaks_and_unterminated_lines() {
    assert_eq!(Text::parse(b"a\nb\r\n"), Err(Error::BareLf { offset: 1 }));
    assert_eq!(Text::parse(b"a\rb\r\n"), Err(Error::BareCr { offset: 1 }));
    assert_eq!(
        Text::parse(b"a\r\nb"),
        Err(Error::Unterminated { offset: 3 })
    );
    assert_eq!(Text::parse(b"a\r\n\r"), Err(Error::BareCr { offset: 3 }));
    assert_eq!(
        Text::parse(&[0xFF, b'\r', b'\n']),
        Err(Error::InvalidUtf8 { offset: 0 })
    );
}

#[test]
fn ids_are_six_digits_after_a_dollar_sign() {
    assert_eq!(parse_id("$000079"), Some(79));
    assert_eq!(parse_id("$999999"), Some(999_999));
    assert_eq!(parse_id("000079"), None);
    assert_eq!(parse_id("$79"), None);
    assert_eq!(parse_id("$0000790"), None);
    assert_eq!(parse_id("$00007a"), None);
    assert_eq!(format_id(0).as_deref(), Some("$000000"));
    assert_eq!(format_id(79).as_deref(), Some("$000079"));
    assert_eq!(format_id(MAX_LINES), None);
}

#[test]
fn lookup_edit_and_append() {
    let mut text = Text::parse(b"zero\r\none\r\n").unwrap();
    assert_eq!(text.lookup("$000001"), Some("one"));
    assert_eq!(text.lookup("$000002"), None);
    text.set_line(0, "changed").unwrap();
    assert_eq!(text.write(), b"changed\r\none\r\n");
    assert_eq!(text.push_line("two"), Ok(2));
    assert_eq!(text.lookup("$000002"), Some("two"));
    assert_eq!(
        text.set_line(9, "x"),
        Err(Error::IndexOutOfRange { index: 9 })
    );
    assert_eq!(text.push_line("a\nb"), Err(Error::LineBreakInText));
    assert_eq!(text.push_line("a\rb"), Err(Error::LineBreakInText));
}

#[test]
fn stored_and_decompressed_forms_both_load() {
    let text = Text::parse(b"alpha\r\nbeta\r\n").unwrap();
    let stored = text.to_stored();
    assert!(hunex_lenzu::is_lenzu(&stored));
    assert_eq!(Text::from_bytes(&stored), Ok(text.clone()));
    assert_eq!(Text::from_bytes(&text.write()), Ok(text));
}

#[test]
fn corrupt_stored_form_is_reported() {
    let mut stored = Text::parse(b"alpha\r\n").unwrap().to_stored();
    stored[0x28] ^= 0x40;
    assert!(matches!(Text::from_bytes(&stored), Err(Error::LenZu(_))));
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn random_bodies_never_panic() {
    let seed = b"one\r\ntwo\r\n\xe6\x97\xa5\r\n".to_vec();
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..20_000 {
        let mut body = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % body.len();
            body[index] = [b'\r', b'\n', b'x', 0xE6, 0xFF][(rng.next() % 5) as usize];
        }
        if let Ok(text) = Text::parse(&body) {
            assert_eq!(text.write(), body);
        }
        let cut = (rng.next() as usize) % (seed.len() + 1);
        let _ = Text::parse(&seed[..cut]);
    }
}
