use super::*;
use alloc::vec;

fn row(start: u32, on_disk: u16, decompressed: u16) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(&start.to_le_bytes());
    bytes[4..6].copy_from_slice(&on_disk.to_le_bytes());
    bytes[6..].copy_from_slice(&decompressed.to_le_bytes());
    bytes
}

fn name_row(name: &str) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[..name.len()].copy_from_slice(name.as_bytes());
    bytes[30] = b'\r';
    bytes[31] = b'\n';
    bytes
}

#[test]
fn index_rows_stop_at_the_terminator() {
    let mut hed = Vec::new();
    hed.extend_from_slice(&row(0, 1, 5));
    hed.extend_from_slice(&row(0, 0, 0));
    hed.extend_from_slice(&row(3, 2, 2));
    hed.extend_from_slice(&[0xFF; 16]);
    let rows = parse_index(&hed).unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows[1].is_empty());
    assert_eq!(rows[2].start_sector, 3);
    assert_eq!(parse_index(&hed[..24]), Err(Error::NoTerminator));
    assert_eq!(
        parse_index(&hed[..20]),
        Err(Error::Truncated { offset: 16 })
    );
}

#[test]
fn names_stop_at_the_zero_row_and_keep_empty_rows() {
    let mut nam = Vec::new();
    nam.extend_from_slice(&name_row("A.NXZ"));
    nam.extend_from_slice(&name_row(""));
    nam.extend_from_slice(&name_row("B.JPG"));
    nam.extend_from_slice(&[0u8; 32]);
    nam.extend_from_slice(&name_row("NEVER.REACHED"));
    assert_eq!(parse_names(&nam), ["A.NXZ", "", "B.JPG"]);
}

#[test]
fn opens_a_set_and_slices_sectors() {
    let mut hed = Vec::new();
    hed.extend_from_slice(&row(0, 1, 1));
    hed.extend_from_slice(&row(0, 0, 0));
    hed.extend_from_slice(&row(1, 1, 1));
    hed.extend_from_slice(&[0xFF; 8]);
    let mut mrg = vec![7u8; 2 * SECTOR];
    mrg[SECTOR] = 9;
    let mut nam = Vec::new();
    nam.extend_from_slice(&name_row("IMG.NXZ"));
    nam.extend_from_slice(&name_row(PLACEHOLDER));
    nam.extend_from_slice(&name_row("RAW.JPG"));
    let set = open(&hed, &mrg, Some(&nam)).unwrap();
    assert_eq!(set.entries.len(), 3);
    assert_eq!(set.entries[0].data.len(), SECTOR);
    assert!(set.entries[1].data.is_empty());
    assert_eq!(set.entries[2].data[0], 9);
    assert_eq!(set.entries[2].name.as_deref(), Some("RAW.JPG"));
    assert!(!set.entries[0].is_placeholder());
    assert_eq!(kind(b"NXCXabcd"), Kind::Nxx);
    assert_eq!(kind(b"NXGX...."), Kind::Nxx);
    assert_eq!(kind(b"mrgd00"), Kind::Mzp);
    assert_eq!(kind(b"\xFF\xD8\xFF"), Kind::Raw);
}

#[test]
fn entries_past_the_data_are_errors() {
    let mut hed = Vec::new();
    hed.extend_from_slice(&row(5, 1, 1));
    hed.extend_from_slice(&[0xFF; 8]);
    assert_eq!(
        open(&hed, &[0u8; 100], None),
        Err(Error::EntryPastEnd { index: 0 })
    );
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
fn random_indexes_and_data_never_panic() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..5_000 {
        let length = (rng.next() % 300) as usize;
        let hed: Vec<u8> = (0..length).map(|_| rng.next() as u8).collect();
        let mrg: Vec<u8> = (0..length * 3).map(|_| rng.next() as u8).collect();
        let nam: Vec<u8> = (0..length).map(|_| rng.next() as u8).collect();
        let _ = open(&hed, &mrg, Some(&nam));
    }
}

#[test]
fn index_gives_rows_and_names_without_data() {
    let mut hed = Vec::new();
    hed.extend_from_slice(&row(4, 2, 3));
    hed.extend_from_slice(&[0xFF; 8]);
    let mut nam = Vec::new();
    nam.extend_from_slice(&name_row("A.NXZ"));
    let metas = index(&hed, Some(&nam)).unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].row.byte_start(), 4 * SECTOR as u64);
    assert_eq!(metas[0].row.byte_span(), 2 * SECTOR as u64);
    assert_eq!(metas[0].name.as_deref(), Some("A.NXZ"));
}
