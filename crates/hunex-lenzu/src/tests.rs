use super::*;
use alloc::vec;

enum Token<'a> {
    Literal(&'a [u8]),
    Copy { length: usize, distance: usize },
}

fn code(symbol: u32) -> u32 {
    127 - symbol
}

fn dense_table(out: &mut Vec<u8>) {
    out.push(SYMBOLS as u8);
    for _ in 0..SYMBOLS {
        out.extend_from_slice(&1u32.to_le_bytes());
    }
}

/// Encodes tokens with the uniform table. Returns the stream and the plain bytes it holds.
fn encode(tokens: &[Token]) -> (Vec<u8>, Vec<u8>) {
    let mut plain = Vec::new();
    let mut writer = BitWriter::default();
    for token in tokens {
        match *token {
            Token::Literal(bytes) => {
                plain.extend_from_slice(bytes);
                for chunk in bytes.chunks(LITERAL_MAX) {
                    writer.push_bit(0);
                    writer.push_bits(code((chunk.len() - 1) as u32), 7);
                    for &byte in chunk {
                        writer.push_bits(u32::from(byte), 8);
                    }
                }
            }
            Token::Copy { length, distance } => {
                for _ in 0..length {
                    let byte = plain[plain.len() - distance];
                    plain.push(byte);
                }
                let distance_field = distance - 2;
                writer.push_bit(1);
                writer.push_bits(code((length - 2) as u32), 7);
                writer.push_bits(code((distance_field >> 7) as u32), 7);
                writer.push_bits((distance_field & 127) as u32, 7);
            }
        }
    }
    let mut out = Vec::new();
    write_fields(&mut out, &plain);
    dense_table(&mut out);
    out.extend_from_slice(&writer.finish());
    (out, plain)
}

#[test]
fn literal_round_trip_at_chunk_boundaries() {
    for length in [0usize, 1, 2, 127, 128, 129, 255, 256, 1000] {
        let plain: Vec<u8> = (0..length).map(|i| (i * 7 + 3) as u8).collect();
        let stream = compress_literal(&plain);
        assert_eq!(decompress(&stream), Ok(plain), "length {length}");
    }
}

#[test]
fn literal_round_trip_all_byte_values() {
    let plain: Vec<u8> = (0..=255u8).collect();
    assert_eq!(decompress(&compress_literal(&plain)), Ok(plain));
}

#[test]
fn literal_stream_is_larger_than_its_input() {
    let plain = vec![0x41u8; 1000];
    assert!(compress_literal(&plain).len() > plain.len());
}

#[test]
fn back_reference_copies_earlier_bytes() {
    let (stream, plain) = encode(&[
        Token::Literal(b"abcdef"),
        Token::Copy {
            length: 3,
            distance: 6,
        },
    ]);
    assert_eq!(plain, b"abcdefabc");
    assert_eq!(decompress(&stream), Ok(plain));
}

#[test]
fn overlapping_back_reference_repeats_bytes() {
    let (stream, plain) = encode(&[
        Token::Literal(b"ab"),
        Token::Copy {
            length: 6,
            distance: 2,
        },
    ]);
    assert_eq!(plain, b"abababab");
    assert_eq!(decompress(&stream), Ok(plain));
}

#[test]
fn long_back_reference_with_high_distance_bits() {
    let filler: Vec<u8> = (0..300u32).map(|i| (i * 13 % 251) as u8).collect();
    let (stream, plain) = encode(&[
        Token::Literal(&filler),
        Token::Copy {
            length: 129,
            distance: 300,
        },
    ]);
    assert_eq!(decompress(&stream), Ok(plain));
}

#[test]
fn back_reference_before_start_is_an_error() {
    let mut writer = BitWriter::default();
    writer.push_bit(1);
    writer.push_bits(code(0), 7);
    writer.push_bits(code(0), 7);
    writer.push_bits(0, 7);
    let mut out = Vec::new();
    write_fields(&mut out, b"ab");
    dense_table(&mut out);
    out.extend_from_slice(&writer.finish());
    assert_eq!(decompress(&out), Err(Error::BadDistance { offset: 0 }));
}

#[test]
fn checksum_mismatch_is_reported() {
    let mut stream = compress_literal(b"hello");
    stream[SIZE_AT - 1] ^= 1;
    stream[CHECKSUM_LOW_AT] ^= 1;
    assert!(matches!(
        decompress(&stream),
        Err(Error::ChecksumMismatch { .. })
    ));
}

#[test]
fn sparse_table_decodes() {
    let plain = b"AB";
    let mut out = Vec::new();
    write_fields(&mut out, plain);
    out.push(2);
    for (symbol, weight) in [(1u8, 1u32), (127, 1)] {
        out.push(symbol);
        out.extend_from_slice(&weight.to_le_bytes());
    }
    let mut writer = BitWriter::default();
    writer.push_bit(0);
    writer.push_bit(1);
    writer.push_bits(u32::from(b'A'), 8);
    writer.push_bits(u32::from(b'B'), 8);
    out.extend_from_slice(&writer.finish());
    assert_eq!(decompress(&out), Ok(plain.to_vec()));
}

#[test]
fn single_symbol_table_reads_no_tree_bits() {
    let plain = b"123456";
    let mut out = Vec::new();
    write_fields(&mut out, plain);
    out.push(1);
    out.push(5);
    out.extend_from_slice(&3u32.to_le_bytes());
    let mut writer = BitWriter::default();
    writer.push_bit(0);
    for &byte in plain {
        writer.push_bits(u32::from(byte), 8);
    }
    out.extend_from_slice(&writer.finish());
    assert_eq!(decompress(&out), Ok(plain.to_vec()));
}

#[test]
fn duplicate_or_out_of_range_symbols_are_bad_tables() {
    let mut out = Vec::new();
    write_fields(&mut out, b"");
    out.push(2);
    out.extend_from_slice(&[9, 1, 0, 0, 0, 9, 1, 0, 0, 0]);
    assert_eq!(decompress(&out), Err(Error::BadTable { offset: 0x3C }));

    let mut out = Vec::new();
    write_fields(&mut out, b"");
    out.push(1);
    out.extend_from_slice(&[200, 1, 0, 0, 0]);
    assert!(matches!(decompress(&out), Err(Error::BadTable { .. })));
}

#[test]
fn bad_magic_truncation_and_variant_are_reported() {
    assert_eq!(decompress(b"not lenzu"), Err(Error::BadMagic));
    let stream = compress_literal(b"hello world");
    assert!(matches!(
        decompress(&stream[..20]),
        Err(Error::Truncated { .. })
    ));
    let mut wide = stream.clone();
    wide[WIDTH_RAW_AT] = 8;
    wide[WIDTH_MIN_AT] = 8;
    assert_eq!(decompress(&wide), Err(Error::Unsupported));
    let cut = &stream[..stream.len() - 3];
    assert!(matches!(decompress(cut), Err(Error::Truncated { .. })));
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
    let (seed, _) = encode(&[
        Token::Literal(b"abcdefgh"),
        Token::Copy {
            length: 4,
            distance: 8,
        },
        Token::Literal(b"xyz"),
    ]);
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for round in 0..3_000 {
        let mut random = MAGIC.to_vec();
        for _ in 0..(rng.next() % 200) {
            random.push(rng.next() as u8);
        }
        let _ = decompress(&random);

        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        if round % 3 == 0 {
            let cut = (rng.next() as usize) % mutated.len();
            mutated.truncate(cut);
        }
        let _ = decompress(&mutated);
    }
}

fn round_trip(plain: &[u8]) {
    let packed = super::compress(plain);
    assert_eq!(
        decompress(&packed).unwrap(),
        plain,
        "round trip of {} bytes",
        plain.len()
    );
}

#[test]
fn compressor_round_trips_edge_cases() {
    round_trip(b"");
    round_trip(b"x");
    round_trip(&[0u8; 1000]);
    round_trip(&[0xFFu8; 129]);
    round_trip(&(0..=255u8).collect::<Vec<u8>>());
    let mut seed = 0x1234_5678u32;
    let random: Vec<u8> = (0..5000)
        .map(|_| {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (seed >> 16) as u8
        })
        .collect();
    round_trip(&random);
    let text: Vec<u8> = b"the quick brown fox jumps over the lazy dog. ".repeat(300);
    round_trip(&text);
}
