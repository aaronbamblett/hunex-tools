use super::*;
use alloc::vec;

pub(crate) const MARKER: u32 = 0x1234_5678;
pub(crate) const CODE_BASE: usize = 0x30;

fn put(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn build(magic: &[u8], includes: &[&[u8]], code: &[u32], pool: &[u8]) -> Vec<u8> {
    build_with_fill(magic, includes, 0, code, pool)
}

fn build_with_fill(
    magic: &[u8],
    includes: &[&[u8]],
    fill: u8,
    code: &[u32],
    pool: &[u8],
) -> Vec<u8> {
    let mut out = magic.to_vec();
    put(&mut out, (CODE_BASE - magic.len()) as u32);
    put(&mut out, includes.len() as u32);
    for name in includes {
        out.extend_from_slice(name);
        out.push(0);
    }
    assert!(out.len() <= CODE_BASE, "test header too long");
    out.resize(CODE_BASE, fill);
    for &word in code {
        put(&mut out, word);
    }
    out.extend_from_slice(pool);
    out
}

pub(crate) fn hunex(code: &[u32], pool: &[u8]) -> Vec<u8> {
    build(HUNEX_MAGIC, &[], code, pool)
}

fn set_dword(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn assert_round_trip(bytes: &[u8]) {
    let script = Script::parse(bytes).unwrap();
    assert_eq!(script.encode(), bytes);
}

#[test]
fn round_trip_minimal_hunex() {
    assert_round_trip(&hunex(&[0x00, 5, 0x03, 0x14, 0x1B], b"hi\0"));
}

#[test]
fn round_trip_buriko() {
    assert_round_trip(&build(
        BURIKO_MAGIC,
        &[],
        &[0x00, 5, 0x03, 0x14, 0x1B],
        b"hi\0",
    ));
}

#[test]
fn round_trip_with_includes() {
    assert_round_trip(&build(
        HUNEX_MAGIC,
        &[b"ab", b"c"],
        &[0x7B, MARKER, 1, 2, 0x1B],
        b"",
    ));
}

#[test]
fn round_trip_nonzero_header_padding() {
    let bytes = build_with_fill(HUNEX_MAGIC, &[b"ab"], 0xA5, &[0x03, 0x0C, 0x1B], b"x\0");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.header_padding(), &[0xA5; 10][..]);
    assert_eq!(script.pool_bytes(), b"x\0");
    assert_eq!(script.encode(), bytes);
}

#[test]
fn minimal_hunex_script() {
    let bytes = hunex(&[0x00, 5, 0x03, 0x14, 0x1B], b"hi\0");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.format(), Format::Hunex);
    assert_eq!(script.header_size(), 21);
    assert_eq!(script.code_base(), 0x30);
    assert!(script.includes().is_empty());
    assert_eq!(script.string_pool_start(), 0x44);
    let offsets: Vec<usize> = script.instructions().iter().map(|i| i.offset()).collect();
    assert_eq!(offsets, vec![0x30, 0x38, 0x40]);
    assert_eq!(script.instructions()[0].operands(), &[5]);
    assert_eq!(script.string(&script.instructions()[1]), Ok("hi"));
}

#[test]
fn buriko_header_variant() {
    let bytes = build(BURIKO_MAGIC, &[], &[0x00, 5, 0x03, 0x14, 0x1B], b"hi\0");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.format(), Format::Buriko);
    assert_eq!(script.header_size(), 0x14);
    assert_eq!(script.code_base(), 0x30);
    assert_eq!(script.string_pool_start(), 0x44);
    assert_eq!(script.string(&script.instructions()[1]), Ok("hi"));
}

#[test]
fn include_names_are_listed() {
    let bytes = build(HUNEX_MAGIC, &[b"ab", b"c"], &[0x1B], b"");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.includes(), &[b"ab".to_vec(), b"c".to_vec()][..]);
    assert_eq!(script.instructions().len(), 1);
}

#[test]
fn skip_marker_takes_three_operands() {
    let bytes = hunex(&[0x7B, MARKER, 1, 2, 0x1B], b"");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.instructions().len(), 2);
    assert_eq!(script.instructions()[0].opcode(), 0x7B);
    assert_eq!(script.instructions()[0].operands(), &[MARKER, 1, 2]);
    assert_eq!(script.instructions()[1].offset(), 0x30 + 16);
}

#[test]
fn instruction_index_lookup() {
    let bytes = hunex(&[0x00, 5, 0x03, 0x14, 0x1B], b"hi\0");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.instruction_index_at(0x38), Some(1));
    assert_eq!(script.instruction_index_at(0x39), None);
    assert_eq!(script.instruction_index_at(0x44), None);
}

#[test]
fn debug_record_resolves_source_name() {
    let bytes = hunex(&[0x7F, 0x10, 9, 0x1B], b"f.ks\0");
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.instructions()[0].operands(), &[0x10, 9]);
    assert_eq!(script.string(&script.instructions()[0]), Ok("f.ks"));
}

#[test]
fn pool_bytes_that_decode_as_instructions_do_not_move_the_pool() {
    let bytes = hunex(&[0x03, 0x0C, 0x1B], &[0x03, 0, 0, 0, 0, 0, 0, 0, b'x', 0]);
    let script = Script::parse(&bytes).unwrap();
    assert_eq!(script.string_pool_start(), 0x3C);
    assert_eq!(script.instructions().len(), 2);
    assert_eq!(script.instructions()[1].opcode(), 0x1B);
}

#[test]
fn registered_table_entries() {
    assert_eq!(OPCODES.len(), 166);
    assert!(OPCODES.iter().all(|info| info.operands <= MAX_OPERANDS));
    assert_eq!(opcode_info(0x7B).map(|i| i.operands), Some(3));
    assert_eq!(opcode_info(0x7F).map(|i| i.operands), Some(2));
    assert_eq!(opcode_info(0x1B).and_then(|i| i.mnemonic), Some("Return"));
    assert_eq!(opcode_info(0x276).map(|i| i.operands), Some(0));
    assert!(opcode_info(0x04).is_none());
    assert!(OPCODES.windows(2).all(|pair| pair[0].id < pair[1].id));
}

#[test]
fn error_bad_magic() {
    assert_eq!(Script::parse(b""), Err(Error::BadMagic));
    assert_eq!(
        Script::parse(b"HunexCompiledScriptVer1.01\0"),
        Err(Error::BadMagic)
    );
}

#[test]
fn error_truncated_header() {
    assert_eq!(
        Script::parse(HUNEX_MAGIC),
        Err(Error::Truncated { offset: 0x1B })
    );
}

#[test]
fn error_truncated_code_base() {
    let mut bytes = HUNEX_MAGIC.to_vec();
    put(&mut bytes, 0x1000);
    put(&mut bytes, 0);
    assert!(matches!(
        Script::parse(&bytes),
        Err(Error::Truncated { .. })
    ));
}

#[test]
fn error_header_size_below_fields() {
    let mut bytes = HUNEX_MAGIC.to_vec();
    put(&mut bytes, 0);
    put(&mut bytes, 0);
    bytes.resize(0x30, 0);
    bytes.extend_from_slice(&[0x1B, 0, 0, 0]);
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::BadHeader { offset: 0x1B })
    );
}

#[test]
fn error_include_names_run_past_code_base() {
    let mut bytes = HUNEX_MAGIC.to_vec();
    put(&mut bytes, 0x0C);
    put(&mut bytes, 1);
    bytes.extend_from_slice(b"abcd");
    bytes.extend_from_slice(&[0x1B, 0, 0, 0]);
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::BadHeader { offset: 0x23 })
    );
}

#[test]
fn error_unregistered_opcode() {
    let bytes = hunex(&[0x00, 1, 0x04, 0x1B], b"");
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::UnregisteredOpcode {
            offset: 0x38,
            opcode: 0x04
        })
    );
}

#[test]
fn error_walk_not_on_pool_boundary() {
    let bytes = hunex(&[0x03, 0x0C, 0x00, 1], b"ab\0");
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::PoolBoundary { offset: 0x38 })
    );
}

#[test]
fn error_string_operand_not_a_string_start() {
    let bytes = hunex(&[0x03, 0x14, 0x03, 0x15, 0x1B], b"ab\0cd\0");
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::BadStringOperand {
            offset: 0x38,
            value: 0x15
        })
    );
}

#[test]
fn error_push_string_without_nul_before_eof() {
    let bytes = hunex(&[0x03, 0x0C, 0x1B], b"ab");
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::BadStringOperand {
            offset: 0x30,
            value: 0x0C
        })
    );
}

#[test]
fn error_debug_string_without_nul_before_eof() {
    let bytes = hunex(&[0x7F, 0x10, 1, 0x1B], b"ab");
    assert_eq!(
        Script::parse(&bytes),
        Err(Error::BadStringOperand {
            offset: 0x30,
            value: 0x10
        })
    );
}

#[test]
fn invalid_utf8_keeps_raw_bytes() {
    let bytes = hunex(&[0x03, 0x0C, 0x1B], &[0xFF, 0x00]);
    let script = Script::parse(&bytes).unwrap();
    let instruction = &script.instructions()[0];
    assert_eq!(script.string_bytes(instruction), Ok(&[0xFF][..]));
    assert_eq!(
        script.string(instruction),
        Err(Error::InvalidUtf8 { offset: 0x30 })
    );
}

#[test]
fn string_accessor_rejects_non_string_instruction() {
    let bytes = hunex(&[0x03, 0x14, 0x00, 0x14, 0x1B], b"ab\0");
    let script = Script::parse(&bytes).unwrap();
    assert!(matches!(
        script.string(&script.instructions()[1]),
        Err(Error::BadStringOperand { .. })
    ));
}

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

fn exercise(bytes: &[u8]) {
    if let Ok(script) = Script::parse(bytes) {
        for instruction in script.instructions() {
            let _ = script.string(instruction);
        }
    }
}

fn seed() -> Vec<u8> {
    hunex(&[0x7B, MARKER, 1, 2, 0x7F, 0x20, 7, 0x1B], b"ab\0\0cd\0")
}

#[test]
fn truncated_inputs_error_before_pool_and_never_panic() {
    let valid = seed();
    let script = Script::parse(&valid).unwrap();
    for length in 0..valid.len() {
        exercise(&valid[..length]);
        if length < script.string_pool_start() {
            assert!(Script::parse(&valid[..length]).is_err(), "length {length}");
        }
    }
}

#[test]
fn hostile_header_fields_are_rejected_without_panic() {
    for value in [0xFFFF_FFFF, 0xFFFF_FFF0, 0x7FFF_FFFF] {
        let mut header_size = seed();
        set_dword(&mut header_size, 0x1B, value);
        assert!(
            Script::parse(&header_size).is_err(),
            "header_size {value:#x}"
        );

        let mut include_count = seed();
        set_dword(&mut include_count, 0x1F, value);
        assert!(
            Script::parse(&include_count).is_err(),
            "include count {value:#x}"
        );
    }
    for value in [0, 8] {
        let mut header_size = seed();
        set_dword(&mut header_size, 0x1B, value);
        exercise(&header_size);

        let mut include_count = seed();
        set_dword(&mut include_count, 0x1F, value);
        exercise(&include_count);
    }
}

#[test]
fn random_and_mutated_inputs_never_panic() {
    let seed = seed();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for round in 0..20_000 {
        let magic = if round % 2 == 0 {
            HUNEX_MAGIC
        } else {
            BURIKO_MAGIC
        };
        let mut random = magic.to_vec();
        for _ in 0..rng.below(200) {
            random.push(rng.next_u64() as u8);
        }
        exercise(&random);

        let mut mutated = seed.clone();
        for _ in 0..1 + rng.below(4) {
            let index = rng.below(mutated.len());
            mutated[index] = rng.next_u64() as u8;
        }
        exercise(&mutated);
    }
}
