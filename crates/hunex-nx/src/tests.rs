use super::*;
use crate::decode::stored_size;
use crate::deswizzle::GOB_BYTES;
use alloc::vec;
use alloc::vec::Vec;

#[test]
fn block_height_follows_the_halving_rule() {
    assert_eq!(block_height(180), 16);
    assert_eq!(block_height(64), 8);
    assert_eq!(block_height(8), 1);
    assert_eq!(block_height(1), 1);
}

#[test]
fn gob_offsets_are_a_permutation_of_the_gob() {
    let mut seen = [false; 512];
    for x in 0..64 {
        for y in 0..8 {
            let offset = gob_offset(x, y);
            assert!(!seen[offset], "duplicate offset {offset}");
            seen[offset] = true;
        }
    }
    assert!(seen.iter().all(|&s| s));
}

#[test]
fn deswizzle_reads_each_byte_from_its_gob_position() {
    let source: Vec<u8> = (0..512).map(|i| i as u8).collect();
    let out = deswizzle(&source, 64, 8, 1, 1);
    for row in 0..8 {
        for column in 0..64 {
            assert_eq!(out[row * 64 + column], source[gob_offset(column, row)]);
        }
    }
}

fn zlib_wrapped(magic: &[u8; 4], plain: &[u8], body: &[u8]) -> Vec<u8> {
    let mut out = magic.to_vec();
    out.extend_from_slice(&(plain.len() as u32).to_le_bytes());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(body);
    out
}

#[test]
fn nxx_wrappers_unwrap_zlib_and_gzip() {
    let plain: Vec<u8> = (0..3000u32)
        .flat_map(|i| (i % 97) as u8..(i % 97) as u8 + 1)
        .collect();
    let zlib = miniz_oxide::deflate::compress_to_vec_zlib(&plain, 6);
    assert_eq!(
        unwrap(&zlib_wrapped(b"NXCX", &plain, &zlib)).unwrap(),
        plain
    );

    let deflate = miniz_oxide::deflate::compress_to_vec(&plain, 6);
    let mut gzip = vec![0x1F, 0x8B, 8, 0x08, 0, 0, 0, 0, 0, 0xFF];
    gzip.extend_from_slice(b"name.bin\0");
    gzip.extend_from_slice(&deflate);
    gzip.extend_from_slice(&[0; 8]);
    assert_eq!(
        unwrap(&zlib_wrapped(b"NXGX", &plain, &gzip)).unwrap(),
        plain
    );

    let wrong = zlib_wrapped(b"NXCX", &plain[..10], &zlib);
    // The declared size is a hard limit, so an output that is too long stops inflation early.
    assert!(matches!(
        unwrap(&wrong),
        Err(Error::SizeMismatch { .. }) | Err(Error::Inflate { .. })
    ));
    assert_eq!(
        unwrap(b"ABCD\0\0\0\0\0\0\0\0\0\0\0\0"),
        Err(Error::BadMagic { what: "NXX" })
    );
}

/// Builds a minimal BNTX with one texture whose data is given as stored bytes.
fn bntx(format: u32, width: u32, height: u32, image_size: u32, data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 0x200];
    out[..4].copy_from_slice(b"BNTX");
    out[0x0C..0x0E].copy_from_slice(&[0xFF, 0xFE]);
    out[0x20..0x24].copy_from_slice(b"NX  ");
    out[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());
    out[0x28..0x30].copy_from_slice(&0x100u64.to_le_bytes());
    out[0x30..0x38].copy_from_slice(&0x180u64.to_le_bytes());
    out[0x38..0x40].copy_from_slice(&0x1F0u64.to_le_bytes());
    out[0x100..0x108].copy_from_slice(&0x60u64.to_le_bytes());
    let record = 0x60;
    out[record..record + 4].copy_from_slice(b"BRTI");
    out[record + 0x1C..record + 0x20].copy_from_slice(&format.to_le_bytes());
    out[record + 0x24..record + 0x28].copy_from_slice(&width.to_le_bytes());
    out[record + 0x28..record + 0x2C].copy_from_slice(&height.to_le_bytes());
    out[record + 0x34..record + 0x38].copy_from_slice(&0u32.to_le_bytes());
    out[record + 0x50..record + 0x54].copy_from_slice(&image_size.to_le_bytes());
    out[record + 0x54..record + 0x58].copy_from_slice(&512u32.to_le_bytes());
    out[0x180..0x184].copy_from_slice(b"BRTD");
    out.truncate(0x190);
    out.extend_from_slice(data);
    out
}

#[test]
fn rgba8_texture_is_deswizzled_and_decoded() {
    let mut data = vec![0u8; 512];
    for y in 0..2 {
        for x in 0..2 {
            for channel in 0..4 {
                data[gob_offset(x * 4 + channel, y)] = (x * 40 + y * 100 + channel * 7) as u8;
            }
        }
    }
    let file = bntx(0x0B06, 2, 2, 512, &data);
    let container = Bntx::parse(&file).unwrap();
    let image = decode_texture(&container, 0).unwrap();
    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(&image.pixels[4..8], &[40, 47, 54, 61]);
    assert_eq!(container.textures[0].data_offset, 0x190);
}

#[test]
fn bc4_block_decodes_to_gray() {
    let mut block = [0u8; 8];
    block[0] = 255;
    block[1] = 0;
    let mut data = vec![0u8; 512];
    for (column, byte) in block.iter().enumerate() {
        data[gob_offset(column, 0)] = *byte;
    }
    let file = bntx(0x1D01, 4, 4, 512, &data);
    let image = decode_texture(&Bntx::parse(&file).unwrap(), 0).unwrap();
    assert!(image
        .pixels
        .chunks(4)
        .all(|pixel| pixel == [255, 255, 255, 255]));
}

#[test]
fn wait_icon_strip_uses_the_block_height_from_the_header() {
    let (width_units, rows, cols) = (1024, 10, 128);
    let size = stored_size(width_units * 8, rows, 1).unwrap();
    assert_eq!(size, 2 * cols * GOB_BYTES);
    let mut data = vec![0u8; size];
    let block_row_8 = cols * GOB_BYTES;
    for column in 0..8 {
        data[block_row_8 + gob_offset(column, 0)] = if column == 0 { 255 } else { 0 };
    }
    let file = bntx(0x1D01, 4095, 39, size as u32, &data);
    let image = decode_texture(&Bntx::parse(&file).unwrap(), 0).unwrap();
    assert_eq!((image.width, image.height), (4095, 39));
    let pixel = |x: usize, y: usize| image.pixels[(y * 4095 + x) * 4];
    assert_eq!(pixel(0, 32), 255);
    assert_eq!(pixel(3, 35), 255);
    assert_eq!(pixel(0, 0), 0);
    assert_eq!(pixel(4, 32), 0);
}

#[test]
fn unsupported_and_mismatched_textures_are_errors() {
    let file = bntx(0x7777, 4, 4, 512, &vec![0u8; 512]);
    assert!(matches!(
        decode_texture(&Bntx::parse(&file).unwrap(), 0),
        Err(Error::UnsupportedFormat { code: 0x7777, .. })
    ));
    let file = bntx(0x0B06, 4, 4, 1024, &vec![0u8; 512]);
    assert!(matches!(
        decode_texture(&Bntx::parse(&file).unwrap(), 0),
        Err(Error::SizeMismatch { .. })
    ));
    let mut file = bntx(0x0B06, 4, 4, 512, &vec![0u8; 512]);
    file[0x60] = b'X';
    assert!(matches!(
        Bntx::parse(&file),
        Err(Error::BadTexture { index: 0, .. })
    ));
    assert!(matches!(Bntx::parse(b"nope"), Err(Error::BadMagic { .. })));
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
fn random_and_mutated_textures_never_panic() {
    let seed = bntx(0x2006, 8, 8, 512, &vec![0xAB; 512]);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..3_000 {
        let mut mutated = seed.clone();
        for _ in 0..1 + rng.next() % 6 {
            let index = (rng.next() as usize) % mutated.len();
            mutated[index] = rng.next() as u8;
        }
        if let Ok(container) = Bntx::parse(&mutated) {
            for index in 0..container.textures.len() {
                let _ = decode_texture(&container, index);
            }
        }
        let _ = unwrap(&mutated);
    }
}

#[test]
fn a_huge_texture_is_an_error_not_a_panic() {
    let bytes = bntx(0x0B06, 0x8000_0000, 0x8000_0000, 0, &[]);
    let parsed = Bntx::parse(&bytes).unwrap();
    assert!(decode_texture(&parsed, 0).is_err());
}

#[test]
fn pointers_at_the_top_of_the_address_range_are_an_error_not_a_panic() {
    let mut bytes = bntx(0x0B06, 4, 4, 64, &[0; 64]);
    bytes[0x28..0x30].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(Bntx::parse(&bytes).is_err());
}

#[test]
fn an_nxx_wrapper_declaring_too_much_output_is_rejected() {
    let mut wrapped = b"NXCX".to_vec();
    wrapped.extend_from_slice(&0x2000_0000u32.to_le_bytes());
    wrapped.extend_from_slice(&4u32.to_le_bytes());
    wrapped.extend_from_slice(&[0; 4]);
    wrapped.extend_from_slice(&[0x78, 0x9C, 0, 0]);
    assert!(matches!(unwrap(&wrapped), Err(Error::Limit { .. })));
}

#[test]
fn bc3_block_reads_color_from_the_color_block_and_alpha_from_the_alpha_block() {
    let mut block = vec![0u8; 512];
    block[0] = 0;
    block[1] = 255;
    block[8..10].copy_from_slice(&0xFFFFu16.to_le_bytes());
    let file = bntx(0x1C06, 4, 4, 512, &block);
    let parsed = Bntx::parse(&file).unwrap();
    let image = decode_texture(&parsed, 0).unwrap();
    assert_eq!((image.width, image.height), (4, 4));
    for pixel in image.pixels.chunks_exact(4) {
        assert_eq!(pixel, [255, 255, 255, 0]);
    }
}
