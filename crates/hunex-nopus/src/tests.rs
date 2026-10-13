use super::*;
use alloc::vec;
use hunex_ogg::{pages, FLAG_BOS, FLAG_EOS};

const CELT_FB_20MS: u8 = 0xF8;
const CELT_NB_2_5MS: u8 = 0x80;

fn file(channels: u8, pre_skip: u16, packets: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&BASIC_CHUNK.to_le_bytes());
    out.extend_from_slice(&0x18u32.to_le_bytes());
    out.push(0);
    out.push(channels);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&48_000u32.to_le_bytes());
    out.extend_from_slice(&0x20u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&pre_skip.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    let body: usize = packets.iter().map(|packet| 8 + packet.len()).sum();
    out.extend_from_slice(&DATA_CHUNK.to_le_bytes());
    out.extend_from_slice(&(body as u32).to_le_bytes());
    for packet in packets {
        out.extend_from_slice(&(packet.len() as u32).to_be_bytes());
        out.extend_from_slice(&[1, 0, 0, 0]);
        out.extend_from_slice(packet);
    }
    out
}

fn three_packets() -> Vec<Vec<u8>> {
    vec![
        vec![CELT_FB_20MS, 1, 2, 3],
        vec![CELT_NB_2_5MS, 9],
        vec![CELT_FB_20MS | 3, 3, 0, 0],
    ]
}

#[test]
fn toc_samples_follow_the_rfc_frame_sizes() {
    assert_eq!(toc_samples(&[CELT_FB_20MS]), Some(960));
    assert_eq!(toc_samples(&[CELT_NB_2_5MS]), Some(120));
    assert_eq!(toc_samples(&[0x00]), Some(480));
    assert_eq!(toc_samples(&[0x01]), Some(960));
    assert_eq!(toc_samples(&[CELT_FB_20MS | 3, 3]), Some(2880));
    assert_eq!(toc_samples(&[CELT_FB_20MS | 3, 6]), Some(5760));
    assert_eq!(toc_samples(&[CELT_FB_20MS | 3, 7]), None);
    assert_eq!(toc_samples(&[CELT_FB_20MS | 3, 0]), None);
    assert_eq!(toc_samples(&[CELT_FB_20MS | 3]), None);
    assert_eq!(toc_samples(&[]), None);
}

#[test]
fn parses_packets_and_counts_samples() {
    let packets = three_packets();
    let refs: Vec<&[u8]> = packets.iter().map(Vec::as_slice).collect();
    let bytes = file(1, 120, &refs);
    let stream = Stream::parse(&bytes).unwrap();
    assert_eq!(stream.channels, 1);
    assert_eq!(stream.sample_rate, 48_000);
    assert_eq!(stream.pre_skip, 120);
    assert_eq!(stream.packets.len(), 3);
    assert_eq!(stream.total_samples(), 960 + 120 + 2880);
    assert_eq!(stream.decoded_samples(), 3840);
    assert_eq!(stream.packets[1].data, &[CELT_NB_2_5MS, 9]);
}

#[test]
fn remux_has_the_rfc_headers_granules_and_unchanged_packets() {
    let packets = three_packets();
    let refs: Vec<&[u8]> = packets.iter().map(Vec::as_slice).collect();
    let bytes = file(2, 120, &refs);
    let ogg = Stream::parse(&bytes).unwrap().to_ogg().unwrap();
    let pages = pages(&ogg).unwrap();
    assert_eq!(pages.len(), 5);
    assert_eq!(pages[0].header_type, FLAG_BOS);
    assert_eq!(pages[0].granule, 0);
    assert_eq!(pages[0].data.len(), 19);
    assert_eq!(&pages[0].data[..8], b"OpusHead");
    assert_eq!(pages[0].data[8], 1);
    assert_eq!(pages[0].data[9], 2);
    assert_eq!(
        u16::from_le_bytes([pages[0].data[10], pages[0].data[11]]),
        120
    );
    assert_eq!(
        u32::from_le_bytes(pages[0].data[12..16].try_into().unwrap()),
        48_000
    );
    assert_eq!(&pages[1].data[..8], b"OpusTags");
    assert_eq!(&pages[1].data[12..17], b"hunex");
    assert_eq!(pages[1].granule, 0);
    let granules: Vec<u64> = pages[2..].iter().map(|page| page.granule).collect();
    assert_eq!(granules, [960, 1080, 3960]);
    assert_eq!(pages[4].header_type, FLAG_EOS);
    for (page, packet) in pages[2..].iter().zip(&packets) {
        assert_eq!(page.data, packet.as_slice());
    }
}

#[test]
fn an_empty_stream_remuxes_to_headers_only() {
    let bytes = file(1, 120, &[]);
    let ogg = Stream::parse(&bytes).unwrap().to_ogg().unwrap();
    let pages = pages(&ogg).unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[1].header_type, FLAG_EOS);
}

#[test]
fn rejects_unsupported_and_malformed_layouts() {
    let one = [vec![CELT_FB_20MS, 1]];
    let refs: Vec<&[u8]> = one.iter().map(Vec::as_slice).collect();

    assert_eq!(
        Stream::parse(&file(3, 0, &refs)),
        Err(Error::UnsupportedChannels(3))
    );
    assert_eq!(Stream::parse(&file(0, 0, &refs)), Err(Error::NoChannels));

    let mut bytes = file(1, 0, &refs);
    bytes[0] = 0;
    assert!(matches!(
        Stream::parse(&bytes),
        Err(Error::BadChunk { offset: 0, .. })
    ));

    let mut bytes = file(1, 0, &refs);
    bytes[8] = 1;
    assert_eq!(Stream::parse(&bytes), Err(Error::UnsupportedVersion(1)));

    let mut bytes = file(1, 0, &refs);
    bytes[0x0A] = 4;
    assert_eq!(
        Stream::parse(&bytes),
        Err(Error::UnsupportedLayout {
            field: "fixed frame size"
        })
    );

    let mut bytes = file(1, 0, &refs);
    bytes[0x10] = 0x30;
    assert_eq!(
        Stream::parse(&bytes),
        Err(Error::UnsupportedLayout {
            field: "data chunk offset"
        })
    );

    let mut bytes = file(1, 0, &refs);
    bytes[0x18] = 1;
    assert_eq!(
        Stream::parse(&bytes),
        Err(Error::UnsupportedLayout {
            field: "context chunk"
        })
    );

    let mut bytes = file(1, 0, &refs);
    bytes[0x20] = 0;
    assert!(matches!(
        Stream::parse(&bytes),
        Err(Error::BadChunk { offset: 0x20, .. })
    ));

    let mut bytes = file(1, 0, &refs);
    bytes[0x24] ^= 1;
    assert!(matches!(
        Stream::parse(&bytes),
        Err(Error::BadDataSize { .. })
    ));

    let mut bytes = file(1, 0, &refs);
    bytes[0x28..0x2C].copy_from_slice(&200u32.to_be_bytes());
    let fixed = bytes.len();
    bytes.truncate(fixed);
    assert!(matches!(
        Stream::parse(&bytes),
        Err(Error::TruncatedPacket { offset: 0x28 })
    ));

    let mut bytes = file(1, 0, &refs);
    bytes[0x28..0x2C].copy_from_slice(&0u32.to_be_bytes());
    assert_eq!(
        Stream::parse(&bytes),
        Err(Error::EmptyPacket { offset: 0x28 })
    );

    let bad_toc = [vec![CELT_FB_20MS | 3, 0]];
    let refs: Vec<&[u8]> = bad_toc.iter().map(Vec::as_slice).collect();
    assert_eq!(
        Stream::parse(&file(1, 0, &refs)),
        Err(Error::BadToc { offset: 0x28 })
    );
    assert!(matches!(
        Stream::parse(&file(1, 0, &[])[..10]),
        Err(Error::Truncated { .. })
    ));
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
fn random_and_mutated_streams_never_panic() {
    let packets = three_packets();
    let refs: Vec<&[u8]> = packets.iter().map(Vec::as_slice).collect();
    let seed = file(2, 120, &refs);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..5_000 {
        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        if let Ok(stream) = Stream::parse(&mutated) {
            let _ = stream.to_ogg();
        }
        let cut = (rng.next() as usize) % seed.len();
        let _ = Stream::parse(&seed[..cut]);
    }
}
