use super::*;
use alloc::vec;

fn image(width: usize, height: usize, bits: u8, rgba: Vec<u8>) -> Image {
    Image {
        header: Header {
            width,
            height,
            stripe_height: STRIPE_HEIGHT,
            bits_per_pixel: bits,
        },
        rgba,
    }
}

fn gradient(width: usize, height: usize, bits: u8) -> Image {
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let v = ((x * 7 + y * 13) % 256) as u8;
            let a = if bits == 32 {
                ((x * 3 + y) % 256) as u8
            } else {
                255
            };
            if bits == 8 {
                rgba.extend_from_slice(&[v, v, v, 255]);
            } else {
                rgba.extend_from_slice(&[v, v.wrapping_mul(3), 255 - v, a]);
            }
        }
    }
    image(width, height, bits, rgba)
}

#[test]
fn leb128_round_trips_and_rejects_overflow() {
    for value in [0u64, 1, 127, 128, 300, u64::from(u32::MAX), u64::MAX] {
        let mut out = Vec::new();
        write_leb(&mut out, value);
        assert_eq!(read_leb(&out, 0).unwrap(), (value, out.len()));
    }
    assert!(read_leb(&[0xFF; 11], 0).is_err());
    assert!(read_leb(&[0x80], 0).is_err());
}

#[test]
fn every_depth_round_trips_canonical_and_simple() {
    for bits in [8u8, 24, 32] {
        for (width, height) in [(1usize, 1usize), (3, 2), (17, 61), (64, 130)] {
            let original = gradient(width, height, bits);
            for encoded in [
                encode(&original).unwrap(),
                encode_simple(&original).unwrap(),
            ] {
                assert_eq!(
                    decode(&encoded).unwrap(),
                    original,
                    "{bits} bpp {width}x{height}"
                );
            }
        }
    }
}

#[test]
fn canonical_encoding_of_its_own_output_is_identical() {
    let original = gradient(70, 70, 32);
    let once = encode(&original).unwrap();
    let again = encode(&decode(&once).unwrap()).unwrap();
    assert_eq!(once, again);
}

#[test]
fn a_single_value_stripe_round_trips() {
    let flat = image(5, 4, 32, [9u8, 9, 9, 255].repeat(20));
    assert_eq!(decode(&encode(&flat).unwrap()).unwrap(), flat);
    let black = image(3, 3, 8, [0u8, 0, 0, 255].repeat(9));
    assert_eq!(decode(&encode(&black).unwrap()).unwrap(), black);
}

#[test]
fn encoder_refuses_pixels_the_depth_cannot_hold() {
    let color = image(1, 1, 8, vec![1, 2, 3, 255]);
    assert!(matches!(encode(&color), Err(Error::Unsupported { .. })));
    let translucent = image(1, 1, 24, vec![1, 2, 3, 128]);
    assert!(matches!(
        encode(&translucent),
        Err(Error::Unsupported { .. })
    ));
    let short = image(2, 2, 32, vec![0; 4]);
    assert!(matches!(encode(&short), Err(Error::SizeMismatch { .. })));
}

#[test]
fn headers_are_checked_before_anything_is_allocated() {
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&65535u32.to_le_bytes());
    bytes.extend_from_slice(&65535u32.to_le_bytes());
    bytes.extend_from_slice(&60u32.to_le_bytes());
    bytes.extend_from_slice(&32u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]);
    assert!(matches!(decode(&bytes), Err(Error::Limit { .. })));
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes[20..24].copy_from_slice(&1u32.to_le_bytes());
    bytes[28..32].copy_from_slice(&16u32.to_le_bytes());
    assert!(matches!(
        decode(&bytes),
        Err(Error::BadHeader {
            field: "bits per pixel"
        })
    ));
    assert!(matches!(decode(b"NotACbgFile.."), Err(Error::BadMagic)));
}

#[test]
fn damaged_streams_are_errors_not_panics() {
    let good = encode(&gradient(20, 70, 32)).unwrap();
    for cut in [0, 10, 47, 48, 60, good.len() / 2, good.len() - 1] {
        let _ = decode(&good[..cut]);
    }
    let mut seed = 0x9E37_79B9u32;
    for _ in 0..2000 {
        let mut bytes = good.clone();
        for _ in 0..4 {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let at = (seed as usize >> 4) % bytes.len();
            bytes[at] ^= (seed >> 16) as u8 | 1;
        }
        let _ = decode(&bytes);
    }
}
