use super::*;
use alloc::vec;

fn entry(name: &str, data: &'static [u8]) -> Entry<'static> {
    Entry::new(String::from(name), Cow::Borrowed(data))
}

fn sample() -> Vec<u8> {
    Archive::new(vec![
        entry("a.chs", b"first"),
        entry("bb.chs", b""),
        entry("c.chs", b"third entry"),
    ])
    .write()
    .unwrap()
}

#[test]
fn write_then_parse_round_trips() {
    let bytes = sample();
    let archive = Archive::parse(&bytes).unwrap();
    let names: Vec<&str> = archive.entries().iter().map(Entry::name).collect();
    assert_eq!(names, ["a.chs", "bb.chs", "c.chs"]);
    assert_eq!(archive.entries()[0].data(), b"first");
    assert_eq!(archive.entries()[1].size(), 0);
    assert_eq!(archive.entries()[2].data(), b"third entry");
    assert_eq!(archive.write().unwrap(), bytes);
}

#[test]
fn layout_matches_the_documented_offsets() {
    let bytes = sample();
    assert_eq!(&bytes[..12], MAGIC);
    assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 3);
    let second = HEADER_SIZE + RECORD_SIZE;
    assert_eq!(&bytes[second..second + 6], b"bb.chs");
    assert_eq!(u32_at(&bytes, second + OFFSET_AT).unwrap(), 5);
    assert_eq!(u32_at(&bytes, second + SIZE_AT).unwrap(), 0);
    assert_eq!(bytes.len(), HEADER_SIZE + 3 * RECORD_SIZE + 5 + 11);
}

fn with_name(name: &[u8]) -> Vec<u8> {
    let mut bytes = sample();
    let mut field = [0u8; NAME_FIELD];
    field[..name.len()].copy_from_slice(name);
    bytes[HEADER_SIZE..HEADER_SIZE + NAME_FIELD].copy_from_slice(&field);
    bytes
}

#[test]
fn rejects_bad_magic_and_counts_past_the_end() {
    assert_eq!(Archive::parse(b""), Err(Error::BadMagic));
    let mut bytes = sample();
    bytes[0] = b'X';
    assert_eq!(Archive::parse(&bytes), Err(Error::BadMagic));

    let mut bytes = sample();
    bytes[12..16].copy_from_slice(&1000u32.to_le_bytes());
    assert!(matches!(
        Archive::parse(&bytes),
        Err(Error::CountPastEnd { count: 1000 })
    ));
    assert!(matches!(
        Archive::parse(&sample()[..14]),
        Err(Error::Truncated { .. })
    ));
}

#[test]
fn rejects_name_problems() {
    let full = [b'a'; NAME_FIELD];
    assert_eq!(
        Archive::parse(&with_name(&full)),
        Err(Error::NameNotTerminated { index: 0 })
    );
    assert_eq!(
        Archive::parse(&with_name(b"x\0junk")),
        Err(Error::NameTrailingData { index: 0 })
    );
    assert_eq!(
        Archive::parse(&with_name(&[0xFF, 0])),
        Err(Error::NameNotUtf8 { index: 0 })
    );
}

#[test]
fn rejects_overlapping_or_gapped_entries() {
    let mut bytes = sample();
    let second = HEADER_SIZE + RECORD_SIZE;
    bytes[second + OFFSET_AT..second + OFFSET_AT + 4].copy_from_slice(&4u32.to_le_bytes());
    assert_eq!(
        Archive::parse(&bytes),
        Err(Error::NotContiguous { index: 1 })
    );
}

#[test]
fn rejects_data_past_the_end_and_trailing_bytes() {
    let mut bytes = sample();
    let first = HEADER_SIZE;
    bytes[first + SIZE_AT..first + SIZE_AT + 4].copy_from_slice(&9999u32.to_le_bytes());
    assert_eq!(Archive::parse(&bytes), Err(Error::DataPastEnd { index: 0 }));

    let mut bytes = sample();
    bytes.push(0);
    assert!(matches!(
        Archive::parse(&bytes),
        Err(Error::TrailingBytes { .. })
    ));
}

#[test]
fn rejects_nonzero_reserved_bytes() {
    let mut bytes = sample();
    bytes[HEADER_SIZE + RESERVED_AT] = 1;
    assert_eq!(
        Archive::parse(&bytes),
        Err(Error::NonZeroReserved { index: 0 })
    );
}

#[test]
fn writer_rejects_bad_and_duplicate_names() {
    let long = "n".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        Archive::new(vec![Entry::new(long, Cow::Borrowed(b""))]).write(),
        Err(WriteError::BadName { index: 0 })
    );
    assert_eq!(
        Archive::new(vec![Entry::new(String::new(), Cow::Borrowed(b""))]).write(),
        Err(WriteError::BadName { index: 0 })
    );
    let repeated = Archive::new(vec![entry("x", b"1"), entry("x", b"2")])
        .write()
        .unwrap();
    let parsed = Archive::parse(&repeated).unwrap();
    assert_eq!(parsed.entries()[1].data(), b"2");
    let max = "m".repeat(MAX_NAME_LEN);
    let bytes = Archive::new(vec![Entry::new(max.clone(), Cow::Borrowed(b"ok"))])
        .write()
        .unwrap();
    assert_eq!(Archive::parse(&bytes).unwrap().entries()[0].name(), max);
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
fn random_and_mutated_archives_never_panic() {
    let seed = sample();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for round in 0..20_000 {
        let mut random = MAGIC.to_vec();
        for _ in 0..(rng.next() % 400) {
            random.push(rng.next() as u8);
        }
        if round % 2 == 0 {
            random.extend_from_slice(&(rng.next() as u32).to_le_bytes());
        }
        let _ = Archive::parse(&random);

        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        if round % 3 == 0 {
            let cut = (rng.next() as usize) % mutated.len();
            mutated.truncate(cut);
        }
        if let Ok(archive) = Archive::parse(&mutated) {
            let _ = archive.write();
        }
    }
}

#[test]
fn the_table_alone_gives_the_entries_and_their_places() {
    let packed = Archive::new(vec![
        Entry::new("a.bin".into(), Cow::Owned(vec![1, 2, 3])),
        Entry::new("b.bin".into(), Cow::Owned(vec![4; 10])),
    ])
    .write()
    .unwrap();
    let len = table_len(&packed).unwrap();
    let data_len = packed.len() - len;
    let toc = read_toc(&packed[..len], data_len).unwrap();
    assert_eq!(toc.len(), 2);
    assert_eq!(
        (toc[0].name.as_str(), toc[0].offset, toc[0].size),
        ("a.bin", 0, 3)
    );
    assert_eq!(
        (toc[1].name.as_str(), toc[1].offset, toc[1].size),
        ("b.bin", 3, 10)
    );
    assert!(read_toc(&packed[..len], data_len - 1).is_err());
}
