use super::*;
use alloc::vec;

#[test]
fn writes_and_parses_back_the_same_entries() {
    let big = vec![7u8; 5000];
    let entries: Vec<&[u8]> = vec![b"a", b"", &big, b"tail-entry-of-eleven"];
    let bytes = write(&entries).unwrap();
    assert_eq!(&bytes[..6], MAGIC);
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 4);
    let parsed = parse(&bytes).unwrap();
    assert_eq!(parsed.entries, entries);
    assert_eq!(write(&parsed.entries).unwrap(), bytes);
}

#[test]
fn entries_are_padded_to_eight_bytes_with_ff_including_the_last() {
    let bytes = write(&[b"abc"]).unwrap();
    assert_eq!(bytes.len(), 8 + 8 + 3 + 5);
    assert!(bytes[16 + 3..].iter().all(|&byte| byte == 0xFF));
    let aligned = write(&[b"12345678"]).unwrap();
    assert_eq!(aligned.len(), 16 + 8 + 8);
}

#[test]
fn record_fields_follow_the_span_rule() {
    let first = vec![1u8; SECTOR - 16];
    let second = vec![2u8; 100];
    let bytes = write(&[&first, &second]).unwrap();
    let record = |index: usize| {
        let at = 8 + 8 * index;
        [0, 2, 4, 6].map(|field| u16::from_le_bytes([bytes[at + field], bytes[at + field + 1]]))
    };
    assert_eq!(record(0), [0, 0, 1, (SECTOR - 16) as u16]);
    assert_eq!(parse(&bytes).unwrap().entries[1], second.as_slice());
}

#[test]
fn rejects_bad_magic_and_truncation() {
    assert_eq!(parse(b"nope00\0\0"), Err(Error::BadMagic));
    let bytes = write(&[b"abcdefgh"]).unwrap();
    assert!(matches!(parse(&bytes[..10]), Err(Error::Truncated { .. })));
    assert!(parse(&bytes[..bytes.len() - 9]).is_err());
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
fn random_and_mutated_containers_never_panic() {
    let a = vec![9u8; 3000];
    let seed = write(&[&a, b"xyz", b"12345678901"]).unwrap();
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..20_000 {
        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        let _ = parse(&mutated);
        let cut = (rng.next() as usize) % seed.len();
        let _ = parse(&seed[..cut]);
    }
}

#[test]
fn switch_rule_takes_the_span_high_bits_and_the_low_field() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(&9u16.to_le_bytes());
    bytes.extend_from_slice(&0x44c0u16.to_le_bytes());
    bytes.extend_from_slice(&[0x55; 0x44c0]);
    let parsed = parse_switch(&bytes).unwrap();
    assert_eq!(parsed.entries[0].len(), 0x44c0);
    assert!(parse_switch(&bytes[..bytes.len() - 1]).is_err());
}

#[test]
fn an_empty_record_past_the_end_of_the_input_is_an_error_not_a_panic() {
    let mut bytes = b"mrgd00".to_vec();
    bytes.extend_from_slice(&1u16.to_le_bytes());
    for value in [100u16, 1, 1, 0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    assert!(parse(&bytes).is_err());
    assert!(parse_switch(&bytes).is_err());
}
