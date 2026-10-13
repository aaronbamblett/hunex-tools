use super::*;
use alloc::vec;

fn table(starts: &[u32], length: u32) -> Vec<u8> {
    let mut out = Vec::new();
    for value in starts.iter().chain([&length, &SENTINEL]) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    out
}

#[test]
fn lines_are_slices_between_starts_without_crlf() {
    let text = b"one\r\ntwo\r\n\r\n";
    let offsets = table(&[0, 5, 10, 12], 12);
    let pair = Pair::parse(&offsets, text).unwrap();
    assert_eq!(pair.len(), 3);
    assert_eq!(pair.lines().unwrap(), ["one", "two", ""]);
    assert_eq!(pair.line(3).unwrap(), None);
    assert_eq!(pair.raw_line(0), Some(&b"one\r\n"[..]));
}

#[test]
fn lookup_uses_six_digit_references() {
    let text = b"a\r\nb\r\n";
    let pair = Pair::parse(&table(&[0, 3, 6], 6), text).unwrap();
    assert_eq!(pair.lookup("$000001").unwrap(), Some("b"));
    assert_eq!(pair.lookup("$000002").unwrap(), None);
    assert_eq!(pair.lookup("$1").unwrap(), None);
    assert_eq!(ref_index("$000079"), Some(79));
    assert_eq!(ref_index("000079"), None);
    assert_eq!(format_ref(79), "$000079");
}

#[test]
fn rejects_bad_tables_and_text() {
    assert_eq!(
        Pair::parse(&[0, 0, 0], b""),
        Err(Error::BadTableLength { length: 3 })
    );
    assert_eq!(Pair::parse(&[0, 0, 0, 0], b""), Err(Error::BadTable));
    assert_eq!(
        Pair::parse(&table(&[0, 5, 2], 6), b"abcdef"),
        Err(Error::BadOffsets { slot: 2 })
    );
    assert_eq!(
        Pair::parse(&table(&[0], 9), b"abc"),
        Err(Error::BadOffsets { slot: 1 })
    );
    let bad_utf8 = [0xFF, b'\r', b'\n'];
    let pair = Pair::parse(&table(&[0, 3, 3], 3), &bad_utf8).unwrap();
    assert_eq!(pair.line(0), Err(Error::NotUtf8 { line: 0 }));
}

#[test]
fn pairs_come_from_consecutive_entries() {
    let text_a = b"x\r\n";
    let text_b = b"y\r\n";
    let table_a = table(&[0, 3], 3);
    let table_b = table(&[0, 3], 3);
    let entries: Vec<&[u8]> = vec![&table_a, text_a, &table_b, text_b];
    let all = pairs(&entries).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[1].line(0).unwrap(), Some("y"));
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
fn random_tables_never_panic() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..10_000 {
        let length = (rng.next() % 64) as usize;
        let table: Vec<u8> = (0..length).map(|_| rng.next() as u8).collect();
        let text: Vec<u8> = (0..(rng.next() % 64) as usize)
            .map(|_| rng.next() as u8)
            .collect();
        if let Ok(pair) = Pair::parse(&table, &text) {
            let _ = pair.lines();
            let _ = pair.lookup("$000001");
        }
    }
}
