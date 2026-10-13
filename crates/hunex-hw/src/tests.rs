use super::*;
use alloc::vec;
use hunex_ogg::{write_page, FLAG_BOS, FLAG_EOS};

fn identification(channels: u8, rate: u32, version: u32, framing: u8) -> Vec<u8> {
    let mut packet = vec![1];
    packet.extend_from_slice(b"vorbis");
    packet.extend_from_slice(&version.to_le_bytes());
    packet.push(channels);
    packet.extend_from_slice(&rate.to_le_bytes());
    packet.extend_from_slice(&0u32.to_le_bytes());
    packet.extend_from_slice(&192_000u32.to_le_bytes());
    packet.extend_from_slice(&0u32.to_le_bytes());
    packet.push(0xB8);
    packet.push(framing);
    packet
}

fn ogg_with_first(first: &[u8], granule: u64) -> Vec<u8> {
    let mut ogg = Vec::new();
    write_page(&mut ogg, FLAG_BOS, 0, 9, 0, &[first]).unwrap();
    let comment = [
        3u8, b'v', b'o', b'r', b'b', b'i', b's', 0, 0, 0, 0, 0, 0, 0, 0, 1,
    ];
    write_page(&mut ogg, 0, 0, 9, 1, &[&comment]).unwrap();
    write_page(&mut ogg, FLAG_EOS, granule, 9, 2, &[&[0xAA; 40]]).unwrap();
    ogg
}

fn vorbis(frames: u64) -> Vec<u8> {
    ogg_with_first(&identification(2, 48_000, 0, 1), frames)
}

#[test]
fn wrap_then_parse_gives_the_header_facts_and_the_original_payload() {
    let ogg = vorbis(96_000);
    let hw = wrap(&ogg).unwrap();
    assert_eq!(&hw[..4], &0x40u32.to_le_bytes());
    assert_eq!(&hw[4..8], b"hw  ");
    assert_eq!(
        u32::from_le_bytes(hw[8..12].try_into().unwrap()) as usize,
        ogg.len()
    );
    let parsed = Hw::parse(&hw).unwrap();
    assert_eq!(parsed.sample_frames, 96_000);
    assert_eq!(parsed.sample_rate, 48_000);
    assert_eq!(parsed.channels, 2);
    assert_eq!(parsed.payload(), ogg.as_slice());
    assert_eq!(unwrap(&hw).unwrap(), ogg);
}

#[test]
fn non_vorbis_audio_is_rejected_with_a_clear_error() {
    let opus = ogg_with_first(b"OpusHead\x01\x02", 960);
    assert_eq!(wrap(&opus), Err(Error::NotVorbis));
    assert_eq!(
        wrap(b"not ogg at all, definitely not"),
        Err(Error::Ogg(hunex_ogg::Error::NotOgg { offset: 0 }))
    );
}

#[test]
fn bad_identification_is_rejected() {
    assert_eq!(
        wrap(&ogg_with_first(&identification(2, 48_000, 1, 1), 1)),
        Err(Error::BadIdentification)
    );
    assert_eq!(
        wrap(&ogg_with_first(&identification(0, 48_000, 0, 1), 1)),
        Err(Error::BadIdentification)
    );
    assert_eq!(
        wrap(&ogg_with_first(&identification(2, 48_000, 0, 0), 1)),
        Err(Error::BadIdentification)
    );
}

#[test]
fn a_last_page_without_granule_is_rejected() {
    assert_eq!(
        wrap(&ogg_with_first(
            &identification(2, 48_000, 0, 1),
            hunex_ogg::NO_GRANULE
        )),
        Err(Error::BadGranule)
    );
}

fn wrapped() -> Vec<u8> {
    wrap(&vorbis(5_000)).unwrap()
}

#[test]
fn each_header_field_is_checked() {
    let mut bytes = wrapped();
    bytes[0] = 0x41;
    assert_eq!(
        Hw::parse(&bytes),
        Err(Error::BadHeaderLength { found: 0x41 })
    );

    let mut bytes = wrapped();
    bytes[4] = b'x';
    assert_eq!(Hw::parse(&bytes), Err(Error::BadTag));

    let mut bytes = wrapped();
    bytes[8] ^= 1;
    assert!(matches!(
        Hw::parse(&bytes),
        Err(Error::BadPayloadLength { .. })
    ));

    let mut bytes = wrapped();
    bytes[0x20] = 1;
    assert_eq!(
        Hw::parse(&bytes),
        Err(Error::NonZeroPadding { offset: 0x20 })
    );

    let mut bytes = wrapped();
    bytes[0x0C] ^= 1;
    assert_eq!(
        Hw::parse(&bytes),
        Err(Error::HeaderMismatch {
            field: "sample frames"
        })
    );

    let mut bytes = wrapped();
    bytes[0x10] ^= 1;
    assert_eq!(
        Hw::parse(&bytes),
        Err(Error::HeaderMismatch {
            field: "sample rate"
        })
    );

    let mut bytes = wrapped();
    bytes[0x14] = 1;
    assert_eq!(
        Hw::parse(&bytes),
        Err(Error::HeaderMismatch { field: "channels" })
    );

    assert_eq!(
        Hw::parse(&bytes[..10]),
        Err(Error::Truncated { offset: 10 })
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
fn random_and_mutated_files_never_panic() {
    let seed = wrapped();
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..5_000 {
        let mut random = seed[..0x40].to_vec();
        for _ in 0..(rng.next() % 200) {
            random.push(rng.next() as u8);
        }
        let _ = Hw::parse(&random);
        let _ = wrap(&random);

        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        let _ = Hw::parse(&mutated);
        let cut = (rng.next() as usize) % mutated.len();
        let _ = Hw::parse(&mutated[..cut]);
    }
}
