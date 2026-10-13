use super::*;
use alloc::vec;

fn page_bytes(packets: &[&[u8]], granule: u64, header_type: u8) -> Vec<u8> {
    let mut out = Vec::new();
    write_page(&mut out, header_type, granule, 7, 3, packets).unwrap();
    out
}

#[test]
fn written_page_reads_back_with_its_fields() {
    let bytes = page_bytes(&[b"hello", b"world!"], 960, FLAG_BOS);
    let (page, end) = read_page(&bytes, 0).unwrap();
    assert_eq!(end, bytes.len());
    assert_eq!(page.header_type, FLAG_BOS);
    assert_eq!(page.granule, 960);
    assert_eq!(page.serial, 7);
    assert_eq!(page.sequence, 3);
    assert_eq!(page.segments, &[5, 6]);
    assert_eq!(page.data, b"helloworld!");
}

#[test]
fn lacing_handles_packets_that_are_multiples_of_255() {
    let exact = vec![0xAB; 255];
    let long = vec![0xCD; 600];
    let bytes = page_bytes(&[&exact, &long], 1, 0);
    let (page, _) = read_page(&bytes, 0).unwrap();
    assert_eq!(page.segments, &[255, 0, 255, 255, 90]);
    assert_eq!(first_packet(&bytes).unwrap(), Some(exact.clone()));
}

#[test]
fn first_packet_returns_a_packet_that_ends_on_the_page() {
    let bytes = page_bytes(&[b"id header", b"second"], 0, FLAG_BOS);
    assert_eq!(first_packet(&bytes).unwrap(), Some(b"id header".to_vec()));
}

#[test]
fn checksum_detects_a_changed_byte() {
    let mut bytes = page_bytes(&[b"payload"], 5, 0);
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    assert_eq!(read_page(&bytes, 0), Err(Error::BadChecksum { offset: 0 }));
}

#[test]
fn pages_must_fill_the_file() {
    let mut bytes = page_bytes(&[b"a"], 1, 0);
    bytes.extend_from_slice(&page_bytes(&[b"b"], 2, FLAG_EOS));
    assert_eq!(pages(&bytes).unwrap().len(), 2);
    bytes.push(0);
    assert!(pages(&bytes).is_err());
    assert_eq!(read_page(&[b'x'; 40], 0), Err(Error::NotOgg { offset: 0 }));
}

#[test]
fn oversized_page_is_rejected() {
    let packet = vec![0u8; 255 * 256];
    let mut out = Vec::new();
    assert_eq!(
        write_page(&mut out, 0, 0, 0, 0, &[&packet]),
        Err(Error::TooManySegments)
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
fn random_and_mutated_pages_never_panic() {
    let seed = page_bytes(&[b"alpha", &[1u8; 300]], 42, FLAG_BOS);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..20_000 {
        let mut random = b"OggS".to_vec();
        for _ in 0..(rng.next() % 300) {
            random.push(rng.next() as u8);
        }
        let _ = pages(&random);
        let _ = first_packet(&random);

        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        let _ = pages(&mutated);
        let _ = first_packet(&mutated);
        let cut = (rng.next() as usize) % mutated.len();
        let _ = pages(&mutated[..cut]);
    }
}

#[test]
fn a_packet_split_over_pages_reads_back_whole() {
    use crate::{packets, write_packet, write_page, FLAG_BOS, FLAG_EOS};
    let long: Vec<u8> = (0..70_000u32).map(|i| (i * 7 % 251) as u8).collect();
    let mut out = Vec::new();
    write_page(&mut out, FLAG_BOS, 0, 9, 0, &[b"head"]).unwrap();
    let next = write_packet(&mut out, 0, 0, 9, 1, &long);
    assert_eq!(next, 3, "a 70000-byte packet takes two pages");
    write_packet(&mut out, FLAG_EOS, 5, 9, next, b"tail");
    let parsed = packets(&out).unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0], b"head");
    assert_eq!(parsed[1], long);
    assert_eq!(parsed[2], b"tail");
}
