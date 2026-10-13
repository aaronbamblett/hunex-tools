use super::*;
use alloc::vec;

fn synthetic(width: usize, height: usize, colour: impl Fn(usize, usize) -> [u8; 4]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&colour(x, y));
        }
    }
    rgba
}

#[test]
fn alpha_mapping_matches_the_spec_rule() {
    assert_eq!(decode_alpha(0), 0);
    assert_eq!(decode_alpha(127), 255);
    assert_eq!(decode_alpha(0x80), 255);
    assert_eq!(decode_alpha(1), 2);
    assert_eq!(decode_alpha(64), 129);
    assert_eq!(ALPHA_CODES[0], 0);
    assert_eq!(ALPHA_CODES[255], 127);
    for code in 0..128u8 {
        let alpha = decode_alpha(code);
        assert_eq!(
            decode_alpha(ALPHA_CODES[usize::from(alpha)]),
            alpha,
            "code {code}"
        );
    }
}

#[test]
fn default_layout_covers_the_image_with_whole_tiles() {
    let layout = Layout::default_for(1000, 300);
    assert_eq!((layout.across, layout.down), (2, 2));
    assert_eq!((layout.canvas_width, layout.canvas_height), (1024, 512));
    assert_eq!(layout.image_size(), (1024, 512));
    assert_eq!(layout.crop, DEFAULT_CROP);
}

#[test]
fn default_layout_matches_the_sample_grids() {
    let layout = Layout::default_for(2560, 1280);
    assert_eq!((layout.across, layout.down), (5, 5));
    assert_eq!(layout.image_size(), (2560, 1280));
    let layout = Layout::default_for(3072, 1280);
    assert_eq!((layout.across, layout.down), (6, 5));
    assert_eq!(layout.image_size(), (3072, 1280));
}

#[test]
fn few_colours_round_trip_exactly() {
    let (width, height) = (300, 200);
    let rgba = synthetic(width, height, |x, y| match (x / 50 + y / 40) % 3 {
        0 => [10, 20, 30, 255],
        1 => [200, 100, 0, 0],
        _ => [0, 0, 0, 126],
    });
    let entries = encode(&rgba, width, height, None).unwrap();
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    let image = decode(&refs).unwrap();
    assert_eq!((image.width, image.height), (512, 256));
    for y in 0..height {
        for x in 0..width {
            let at = (y * image.width + x) * 4;
            let want = &rgba[(y * width + x) * 4..(y * width + x) * 4 + 4];
            assert_eq!(&image.rgba[at..at + 4], want, "pixel {x},{y}");
        }
    }
}

#[test]
fn alpha_127_is_stored_as_the_nearest_decodable_alpha() {
    assert_eq!(decode_alpha(ALPHA_CODES[127]), 126);
}

#[test]
fn template_keeps_the_header_and_requires_the_same_size() {
    let (width, height) = (700, 300);
    let rgba = synthetic(width, height, |x, y| [(x % 7) as u8, (y % 5) as u8, 9, 255]);
    let entries = encode(&rgba, width, height, None).unwrap();
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    let decoded = decode(&refs).unwrap();
    let again = encode(
        &decoded.rgba,
        decoded.width,
        decoded.height,
        Some(&decoded.layout),
    )
    .unwrap();
    assert_eq!(again[0][..16], entries[0][..16]);
    assert!(matches!(
        encode(&rgba, width, height, Some(&decoded.layout)),
        Err(Error::SizeMismatch { .. })
    ));
}

#[test]
fn many_colours_quantise_within_the_palette_and_stay_close() {
    let (width, height) = (512, 256);
    let rgba = synthetic(width, height, |x, y| {
        let v = (x * 31 + y * 17) % 997;
        [
            (v % 256) as u8,
            ((v * 3) % 256) as u8,
            ((x ^ y) % 256) as u8,
            255,
        ]
    });
    let entries = encode(&rgba, width, height, None).unwrap();
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    let image = decode(&refs).unwrap();
    let total: u64 = rgba
        .iter()
        .zip(&image.rgba)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    let mean = total as f64 / rgba.len() as f64;
    assert!(mean < 40.0, "mean error {mean}");
}

#[test]
fn bad_headers_and_tiles_are_rejected() {
    let (width, height) = (64, 64);
    let rgba = synthetic(width, height, |x, _| [x as u8, 0, 0, 255]);
    let entries = encode(&rgba, width, height, None).unwrap();

    let mut header = entries[0].clone();
    header[12] = 0x0B;
    let mut bad = entries.clone();
    bad[0] = header;
    let refs: Vec<&[u8]> = bad.iter().map(Vec::as_slice).collect();
    assert_eq!(
        decode(&refs),
        Err(Error::Unsupported {
            field: "bitmap type",
            value: 0x0B
        })
    );

    let mut header = entries[0].clone();
    header[13] = 0x11;
    let mut bad = entries.clone();
    bad[0] = header;
    let refs: Vec<&[u8]> = bad.iter().map(Vec::as_slice).collect();
    assert_eq!(
        decode(&refs),
        Err(Error::Unsupported {
            field: "depth",
            value: 0x11
        })
    );

    let mut bad = entries.clone();
    bad[0].pop();
    let refs: Vec<&[u8]> = bad.iter().map(Vec::as_slice).collect();
    assert_eq!(
        decode(&refs),
        Err(Error::BadHeader {
            field: "transparency count"
        })
    );

    let mut bad = entries.clone();
    bad[1] = hunex_mzx::compress(b"not a hep tile");
    let refs: Vec<&[u8]> = bad.iter().map(Vec::as_slice).collect();
    assert!(matches!(
        decode(&refs),
        Err(Error::BadTile { index: 0, .. })
    ));
}

#[test]
fn atlas_round_trips_exactly_within_256_colours() {
    let template = Layout {
        canvas_width: 1024,
        canvas_height: 1024,
        tile_width: 1024,
        tile_height: 1024,
        across: 1,
        down: 1,
        bitmap_type: ATLAS_TYPE,
        depth: 0,
        crop: 1,
        flags: 0,
        transparency: vec![0],
    };
    let alphas = [0, decode_alpha(3), decode_alpha(60), 255];
    let rgba = synthetic(1024, 1024, |x, y| {
        [255, 255, 255, alphas[(x / 73 + 2 * (y / 73)) % 4]]
    });
    let entries = encode(&rgba, 1024, 1024, Some(&template)).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].len(), HEADER_BYTES + PALETTE_BYTES + 1);
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    let layout = parse_header(refs[0]).unwrap();
    assert_eq!(layout.bitmap_type, ATLAS_TYPE);
    assert_eq!(layout.transparency.len(), 1);
    let image = decode(&refs).unwrap();
    assert_eq!((image.width, image.height), (1024, 1024));
    assert!(
        image.rgba == rgba,
        "atlas pixels change after the round trip"
    );
}

fn window_hits(layout: &Layout, index: usize, rect: (usize, usize, usize, usize)) -> bool {
    let (pitch_x, pitch_y) = (layout.tile_width, layout.tile_height);
    let (column, row) = (index % layout.across, index / layout.across);
    let (x0, y0) = (column * pitch_x, row * pitch_y);
    let (x1, y1) = (x0 + layout.tile_width, y0 + layout.tile_height);
    let (rx0, ry0, rx1, ry1) = rect;
    x0 < rx1 && rx0 < x1 && y0 < ry1 && ry0 < y1
}

#[test]
fn like_keeps_unchanged_tiles_and_re_encodes_only_the_touched_ones() {
    let (width, height) = (300, 200);
    let rgba = synthetic(width, height, |x, y| match (x / 37 + y / 23) % 3 {
        0 => [10, 20, 30, 255],
        1 => [200, 100, 0, 0],
        _ => [90, 150, 210, decode_alpha(40)],
    });
    let entries = encode(&rgba, width, height, None).unwrap();
    let refs: Vec<&[u8]> = entries.iter().map(Vec::as_slice).collect();
    let layout = parse_header(refs[0]).unwrap();
    let image = decode(&refs).unwrap();

    let (same, reports) = encode_like(&image.rgba, image.width, image.height, &refs).unwrap();
    assert_eq!(
        same, entries,
        "an unchanged image must give back the same container"
    );
    assert!(reports.is_empty());

    let rect = (100, 40, 260, 120);
    let mut painted = image.rgba.clone();
    for y in rect.1..rect.3 {
        for x in rect.0..rect.2 {
            painted[(y * image.width + x) * 4..][..4].copy_from_slice(&[200, 30, 30, 255]);
        }
    }
    let (changed, reports) = encode_like(&painted, image.width, image.height, &refs).unwrap();
    let mut touched = Vec::new();
    for index in 0..layout.tiles() {
        if window_hits(&layout, index, rect) {
            touched.push(index);
        } else {
            assert_eq!(
                changed[index + 1],
                entries[index + 1],
                "tile {index} should be copied"
            );
        }
    }
    let reported: Vec<usize> = reports.iter().map(|r| r.index).collect();
    assert_eq!(reported, touched);
    assert!(reports.iter().all(|r| !r.quantised && r.max_error == 0));
    let refs: Vec<&[u8]> = changed.iter().map(Vec::as_slice).collect();
    let decoded = decode(&refs).unwrap();
    assert!(
        decoded.rgba == painted,
        "the repainted pixels must come back exactly"
    );
}

#[test]
fn an_oversized_canvas_is_rejected_before_its_tiles_are_read() {
    let mut header = Vec::new();
    for value in [65535u16, 65535, 512, 256, 128, 256] {
        header.extend_from_slice(&value.to_le_bytes());
    }
    header.extend_from_slice(&[BITMAP_TYPE, 0, 17, 1]);
    header.extend(vec![0u8; 128 * 256]);
    let mut entries: Vec<&[u8]> = vec![&header];
    entries.extend(core::iter::repeat_n(&[][..], 128 * 256));
    assert_eq!(
        decode(&entries),
        Err(Error::BadHeader {
            field: "canvas size"
        })
    );
}

#[test]
fn a_tile_stream_declaring_a_huge_output_is_rejected() {
    let layout = Layout::default_for(512, 256);
    let mut header = header_fields(&layout).unwrap();
    header.extend_from_slice(&[0]);
    let mut stream = hunex_mzx::MAGIC.to_vec();
    stream.extend_from_slice(&0x7FFF_FFFFu32.to_le_bytes());
    stream.extend_from_slice(&[0; 8]);
    let entries: Vec<&[u8]> = vec![&header, &stream];
    assert!(matches!(
        decode(&entries),
        Err(Error::Mzx(hunex_mzx::Error::Limit { .. }))
    ));
}

#[test]
fn a_header_field_beyond_u16_is_an_error_not_a_truncated_value() {
    let rgba = vec![0u8; 70_000 * 4];
    assert!(encode(&rgba, 70_000, 1, None).is_err());
}
