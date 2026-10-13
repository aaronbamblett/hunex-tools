use super::*;

fn assert_round_trip(plain: &[u8]) {
    assert_eq!(decompress(&compress(plain)).unwrap(), plain, "greedy");
    assert_eq!(
        decompress(&compress_literal(plain)).unwrap(),
        plain,
        "literal"
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
fn round_trips_empty_odd_and_long_inputs() {
    assert_round_trip(b"");
    assert_round_trip(b"x");
    assert_round_trip(b"xy");
    assert_round_trip(b"abc");
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for length in [2usize, 3, 63, 64, 65, 127, 128, 129, 1000, 4097] {
        let random: Vec<u8> = (0..length).map(|_| rng.next() as u8).collect();
        assert_round_trip(&random);
    }
}

#[test]
fn runs_and_repeats_compress_well() {
    let plain: Vec<u8> = (0..4096).flat_map(|_| [0u8, 0, 0, 0]).collect();
    let packed = compress(&plain);
    // Runs are distance-1 back-references (two bytes per 64 words), since the encoder writes no
    // repeat commands.
    assert!(packed.len() < 300, "{}", packed.len());
    assert_round_trip(&plain);
}

#[test]
fn repeated_patterns_use_back_references() {
    let pattern: Vec<u8> = (0..200u32)
        .flat_map(|i| (i as u16 * 37).to_le_bytes())
        .collect();
    let plain: Vec<u8> = pattern
        .iter()
        .cycle()
        .take(pattern.len() * 20)
        .copied()
        .collect();
    let packed = compress(&plain);
    assert!(
        packed.len() * 4 < plain.len(),
        "{} vs {}",
        packed.len(),
        plain.len()
    );
    assert_round_trip(&plain);
}

#[test]
fn literal_only_stream_uses_no_other_command() {
    let plain: Vec<u8> = (0..300u32).map(|i| i as u8).collect();
    let packed = compress_literal(&plain);
    let mut position = HEADER;
    while position < packed.len() {
        assert_eq!(packed[position] & 3, 3);
        let words = usize::from(packed[position] >> 2) + 1;
        position += 1 + 2 * words;
    }
    assert_eq!(position, packed.len());
}

#[test]
fn decodes_a_hand_written_stream() {
    let stream = [
        b'M',
        b'Z',
        b'X',
        b'0',
        8,
        0,
        0,
        0,
        0x03 | (1 << 2),
        0x34,
        0x12,
        0x78,
        0x56,
        0x01 | (1 << 2),
        1,
    ];
    assert_eq!(
        decompress(&stream).unwrap(),
        [0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x78, 0x56]
    );
    let repeat = [b'M', b'Z', b'X', b'0', 6, 0, 0, 0, 0x03, 0x34, 0x12, 1 << 2];
    assert_eq!(
        decompress(&repeat).unwrap(),
        [0x34, 0x12, 0x34, 0x12, 0x34, 0x12]
    );
}

#[test]
fn rejects_bad_streams() {
    assert_eq!(decompress(b"NOPE\0\0\0\0"), Err(Error::BadMagic));
    assert!(matches!(
        decompress(b"MZX0\x04\0\0\0"),
        Err(Error::Truncated { .. })
    ));
    let before_start = [b'M', b'Z', b'X', b'0', 2, 0, 0, 0, 0x01, 0];
    assert_eq!(
        decompress(&before_start),
        Err(Error::BadReference { offset: 9 })
    );
    let extra = [b'M', b'Z', b'X', b'0', 0, 0, 0, 0, 0x00];
    assert_eq!(decompress(&extra), Err(Error::TrailingBytes { offset: 8 }));
}

#[test]
fn odd_sizes_are_cut_to_the_size() {
    let plain = b"abcde";
    assert_eq!(decompress(&compress(plain)).unwrap(), plain);
}

#[test]
fn random_and_mutated_streams_never_panic() {
    let plain: Vec<u8> = (0..700u32).map(|i| (i % 13) as u8).collect();
    let seed = compress(&plain);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..20_000 {
        let mut random = MAGIC.to_vec();
        random.extend_from_slice(&(rng.next() as u32 % 4096).to_le_bytes());
        for _ in 0..(rng.next() % 200) {
            random.push(rng.next() as u8);
        }
        let _ = decompress(&random);

        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 4 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        let _ = decompress(&mutated);
        let cut = (rng.next() as usize) % mutated.len();
        let _ = decompress(&mutated[..cut]);
    }
}

#[test]
fn switch_streams_round_trip_and_xor_literals() {
    for length in [0usize, 1, 2, 3, 127, 128, 129, 255, 1000] {
        let plain: Vec<u8> = (0..length).map(|i| (i * 31 + 7) as u8).collect();
        let packed = switch::compress_literal(&plain);
        assert_eq!(
            switch::decompress(&packed).unwrap(),
            plain,
            "length {length}"
        );
    }
    let stored = switch::compress_literal(&[0x12, 0x34]);
    assert_eq!(&stored[8..], &[3, 0xED, 0xCB]);
    assert_eq!(switch::decompress(&stored), Ok(vec![0x12, 0x34]));
}

#[test]
fn switch_initial_last_word_is_ffff() {
    let stream = [b'M', b'Z', b'X', b'0', 4, 0, 0, 0, 0x04];
    assert_eq!(
        switch::decompress(&stream).unwrap(),
        [0xFF, 0xFF, 0xFF, 0xFF]
    );
    assert_eq!(decompress(&stream), Ok(vec![0, 0, 0, 0]));
}

/// Literal commands of at most 64 words each, then the stream header.
fn literal_stream(words: &[u16], tail: &[u8], size_words: usize) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&((size_words * 2) as u32).to_le_bytes());
    for chunk in words.chunks(64) {
        out.push((((chunk.len() - 1) as u8) << 2) | 3);
        for word in chunk {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out.extend_from_slice(tail);
    out
}

fn words_of(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect()
}

#[test]
fn a_repeat_starting_on_a_window_boundary_writes_zero() {
    let mut words: Vec<u16> = (1..=4096u16).collect();
    words[4095] = 0x1234;
    let stream = literal_stream(&words[..4096], &[((2 - 1) << 2) as u8], 4098);
    let command = words_of(&decompress_with(&stream, WindowRule::Command).unwrap());
    let word = words_of(&decompress_with(&stream, WindowRule::Word).unwrap());
    let none = words_of(&decompress_with(&stream, WindowRule::None).unwrap());
    assert_eq!(&command[4096..4098], &[0, 0]);
    assert_eq!(&word[4096..4098], &[0, 0]);
    assert_eq!(&none[4096..4098], &[0x1234, 0x1234]);
}

#[test]
fn a_repeat_crossing_a_boundary_keeps_its_value_under_the_command_reading_only() {
    let words: Vec<u16> = (1..=4095u16).collect();
    let stream = literal_stream(&words, &[((3 - 1) << 2) as u8], 4098);
    let command = words_of(&decompress_with(&stream, WindowRule::Command).unwrap());
    let word = words_of(&decompress_with(&stream, WindowRule::Word).unwrap());
    let none = words_of(&decompress_with(&stream, WindowRule::None).unwrap());
    assert_eq!(&command[4095..4098], &[0x0FFF, 0x0FFF, 0x0FFF]);
    assert_eq!(&word[4095..4098], &[0x0FFF, 0, 0]);
    assert_eq!(&none[4095..4098], &[0x0FFF, 0x0FFF, 0x0FFF]);
}

#[test]
fn encoder_keeps_round_trips_across_window_boundaries() {
    let mut words: Vec<u16> = (0..4090u16).map(|w| w.wrapping_mul(7) | 1).collect();
    words.extend(core::iter::repeat_n(0x1234u16, 12));
    words.extend(core::iter::repeat_n(0u16, 9));
    words.extend((0..5000u16).map(|w| w / 3));
    let plain: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    let stream = compress(&plain);
    assert_eq!(decompress(&stream).unwrap(), plain);
    assert_eq!(decompress_with(&stream, WindowRule::Word).unwrap(), plain);
}

#[test]
fn encoder_output_decodes_the_same_under_both_readings_for_repetitive_data() {
    let mut seed = 0x2545_F491u32;
    let mut words = Vec::new();
    while words.len() < 20_000 {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let value = if seed.is_multiple_of(3) {
            0
        } else {
            (seed >> 8) as u16 % 5
        };
        let run = (seed >> 16) as usize % 40 + 1;
        words.extend(core::iter::repeat_n(value, run));
    }
    let plain: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    let stream = compress(&plain);
    assert_eq!(
        decompress_with(&stream, WindowRule::Command).unwrap(),
        plain
    );
    assert_eq!(decompress_with(&stream, WindowRule::Word).unwrap(), plain);
}

#[test]
fn encoder_never_writes_a_window_start_with_a_non_literal() {
    for start in [0usize, 4096, 8192] {
        let mut words: Vec<u16> = (0..start)
            .map(|w| (w as u16).wrapping_mul(13) | 1)
            .collect();
        words.extend(core::iter::repeat_n(0u16, 300));
        words.extend(core::iter::repeat_n(0x0ABC, 40));
        let plain: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        let stream = compress(&plain);
        let mut envelope = Envelope::default();
        envelope.add(&stream).unwrap();
        assert_eq!(envelope.window_starts_not_literal, 0, "start {start}");
        assert_eq!(decompress_with(&stream, WindowRule::None).unwrap(), plain);
        assert_eq!(decompress_with(&stream, WindowRule::Word).unwrap(), plain);
    }
}

#[test]
fn a_declared_output_beyond_the_limit_is_rejected_before_decoding() {
    let mut stream = MAGIC.to_vec();
    stream.extend_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
    stream.extend_from_slice(&[3, 0, 0, 0, 0]);
    assert!(matches!(decompress(&stream), Err(Error::Limit { .. })));
    assert!(matches!(
        decompress_within(&stream, 1 << 20),
        Err(Error::Limit { .. })
    ));
}
