//! Real glyph tables and atlases, gated on HUNEX_HFA_DIR (the folder with data00100.hfa).
//! HUNEX_WORK is the scratch folder for the render and the export round trip; those checks are
//! skipped when it is not set.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hunex_ccit::{cell_of, code_of, Table, CELL_SIZE, GLYPHS_PER_PAGE};
use hunex_hfa::Archive;
use hunex_image::{decode_mzp, encode_mzp};
use hunex_lenzu as lenzu;

const TABLES: [(&str, usize, bool, u32, u32); 16] = [
    ("Font000000", 106, true, 0x20, 0xF7),
    ("Font000001", 2694, false, 0x2010, 0xFFE5),
    ("Font000100", 260, true, 0x20, 0xFF01),
    ("Font000200", 947, true, 0x20, 0xFF5E),
    ("Font010000", 106, true, 0x20, 0xF7),
    ("Font010001", 7, false, 0x2014, 0x25A0),
    ("Font010100", 55, true, 0x20, 0x7C),
    ("Font010200", 68, true, 0x20, 0x2026),
    ("Font020000", 106, true, 0x20, 0xF7),
    ("Font020001", 3351, false, 0x2014, 0xFF5E),
    ("Font020100", 246, true, 0x20, 0xFF0C),
    ("Font020200", 1273, true, 0x20, 0xFF5E),
    ("Font030000", 106, true, 0x20, 0xF7),
    ("Font030001", 3358, false, 0x2015, 0xFF5E),
    ("Font030100", 251, true, 0x20, 0xFF0C),
    ("Font030200", 1269, true, 0x20, 0xFF5E),
];

fn archive_bytes() -> Option<Vec<u8>> {
    let Some(dir) = std::env::var_os("HUNEX_HFA_DIR").map(PathBuf::from) else {
        eprintln!("HUNEX_HFA_DIR is not set; skipping glyph table checks");
        return None;
    };
    Some(fs::read(dir.join("data00100.hfa")).expect("data00100.hfa"))
}

fn work_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("HUNEX_WORK").map(PathBuf::from);
    if dir.is_none() {
        eprintln!("HUNEX_WORK is not set; skipping the render and export checks");
    }
    dir
}

fn entry_data<'a>(archive: &'a Archive, name: &str) -> &'a [u8] {
    archive
        .entries()
        .iter()
        .find(|entry| entry.name() == name)
        .unwrap_or_else(|| panic!("{name} is not in the archive"))
        .data()
}

fn payload(archive: &Archive, name: &str) -> Vec<u8> {
    let data = entry_data(archive, name);
    if lenzu::is_lenzu(data) {
        lenzu::decompress(data).expect("decompress")
    } else {
        data.to_vec()
    }
}

fn hunex(args: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_hunex"))
        .args(args)
        .output()
        .expect("run hunex");
    assert!(
        output.status.success(),
        "hunex {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn all_sixteen_tables_rebuild_byte_for_byte() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    for (name, glyphs, coords, _, _) in TABLES {
        let body = payload(&archive, &format!("{name}.ccit"));
        let table = Table::parse(&body).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(table.records().len(), glyphs, "{name}: glyph count");
        assert_eq!(table.coords().is_some(), coords, "{name}: coordinates");
        assert_eq!(table.to_bytes(), body, "{name}: rebuild differs");
        println!("{name}: {glyphs} glyphs, {} bytes, byte-exact", body.len());
    }
}

#[test]
fn glyph_counts_code_ranges_and_indices_match_the_spec() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    for (name, glyphs, _, low, high) in TABLES {
        let table = Table::parse(&payload(&archive, &format!("{name}.ccit"))).unwrap();
        let records = table.records();
        assert_eq!(records.len(), glyphs, "{name}");
        let codes = records.iter().map(|r| r.code);
        let point = |code: Option<u32>| code.and_then(hunex_ccit::char_of).map(|c| c as u32);
        assert_eq!(
            point(codes.clone().min()),
            Some(low),
            "{name}: first code point"
        );
        assert_eq!(
            point(codes.clone().max()),
            Some(high),
            "{name}: last code point"
        );
        assert!(
            records.windows(2).all(|pair| pair[0].code < pair[1].code),
            "{name}: codes are not strictly increasing"
        );
        let coord_list = table.coords().map(<[_]>::to_vec);
        assert!(
            Table::from_parts(records.to_vec(), coord_list).is_ok(),
            "{name}: indices are not dense"
        );
    }
}

#[test]
fn stored_coordinates_follow_the_slot_model_with_no_mismatch() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    let mut checked = 0usize;
    let mut mismatches = 0usize;
    for (name, _, coords, _, _) in TABLES {
        if !coords {
            continue;
        }
        let table = Table::parse(&payload(&archive, &format!("{name}.ccit"))).unwrap();
        for glyph in table.glyphs() {
            let cell = cell_of(glyph.record.index);
            let coord = glyph.coord.expect("coordinates");
            checked += 1;
            if coord.page != glyph.record.index / GLYPHS_PER_PAGE
                || coord.y != cell.y
                || coord.height != CELL_SIZE
            {
                mismatches += 1;
            }
        }
    }
    assert_eq!(checked, 4793, "entries in the 12 coordinate tables");
    assert_eq!(
        mismatches, 0,
        "{mismatches} of {checked} coordinate entries differ from the slot model"
    );
    println!("coordinate entries checked: {checked}, mismatches: {mismatches}");
}

#[test]
fn the_eight_page_one_records_are_where_the_spec_says() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    for name in ["Font000000", "Font010000", "Font020000", "Font030000"] {
        let table = Table::parse(&payload(&archive, &format!("{name}.ccit"))).unwrap();
        let mut page_one: Vec<u32> = table
            .records()
            .iter()
            .filter(|r| r.index / GLYPHS_PER_PAGE == 1)
            .map(|r| r.index)
            .collect();
        page_one.sort_unstable();
        assert_eq!(page_one, (98..106).collect::<Vec<u32>>(), "{name}");
        let accented: Vec<u32> = "àäèéêïó÷".chars().map(code_of).collect();
        for code in accented {
            assert!(
                table.find_code(code).is_some(),
                "{name}: missing accented code 0x{code:X}"
            );
        }
    }
}

#[test]
fn both_atlas_pages_round_trip_exactly_within_256_colours() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    for name in ["FONT_en_H00.mzp", "FONT_en_H01.mzp"] {
        let data = entry_data(&archive, name);
        let image = decode_mzp(data).unwrap();
        assert_eq!((image.width, image.height), (1024, 1024), "{name}");
        let mut colours: Vec<[u8; 4]> = image
            .rgba
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2], p[3]])
            .collect();
        colours.sort_unstable();
        colours.dedup();
        assert!(colours.len() <= 256, "{name}: {} colours", colours.len());
        let rebuilt =
            encode_mzp(&image.rgba, image.width, image.height, Some(&image.layout)).unwrap();
        let again = decode_mzp(&rebuilt).unwrap();
        assert!(
            again.rgba == image.rgba,
            "{name}: pixels change after the round trip"
        );
        println!(
            "{name}: {} colours, rebuilt {} bytes (original {})",
            colours.len(),
            rebuilt.len(),
            data.len()
        );
    }
}

#[test]
fn every_table_exports_and_imports_back_byte_for_byte() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let Some(work) = work_dir() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    fs::create_dir_all(&work).unwrap();
    for (name, _, _, _, _) in TABLES {
        let body = payload(&archive, &format!("{name}.ccit"));
        let input = work.join(format!("{name}.ccit"));
        let tsv = work.join(format!("{name}.tsv"));
        let rebuilt = work.join(format!("{name}.rebuilt.ccit"));
        fs::write(&input, &body).unwrap();
        hunex(&["font", "export", path(&input), "-o", path(&tsv)]);
        hunex(&["font", "import", path(&tsv), "-o", path(&rebuilt)]);
        assert_eq!(
            fs::read(&rebuilt).unwrap(),
            body,
            "{name}: export and import differ"
        );
    }
}

#[test]
fn tsukihime_123_renders_with_the_english_atlas() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let Some(work) = work_dir() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    fs::create_dir_all(&work).unwrap();
    let table_path = work.join("render-Font010000.ccit");
    let atlas_path = work.join("render-FONT_en_H00.mzp");
    fs::write(&table_path, payload(&archive, "Font010000.ccit")).unwrap();
    fs::write(&atlas_path, entry_data(&archive, "FONT_en_H00.mzp")).unwrap();
    let png_path = work.join("font-render-tsukihime-123.png");
    let text = "Tsukihime 123";
    hunex(&[
        "font",
        "render",
        path(&table_path),
        "--atlas",
        path(&atlas_path),
        "--text",
        text,
        "-o",
        path(&png_path),
    ]);

    let decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(&png_path).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buffer).unwrap();
    let (width, height) = (info.width as usize, info.height as usize);
    assert!(width > 0 && height > 0);
    let dark = |x: usize| (0..height).any(|y| buffer[(y * width + x) * 4] < 128);
    let inked_columns: Vec<usize> = (0..width).filter(|&x| dark(x)).collect();
    assert!(!inked_columns.is_empty(), "the render has no ink");

    let table = Table::parse(&payload(&archive, "Font010000.ccit")).unwrap();
    let margin = 8usize;
    let mut pen = margin;
    let mut boxes = Vec::new();
    for c in text.chars() {
        let glyph = table.find(c).expect("glyph");
        let width = glyph.coord.expect("coordinates").width as usize;
        boxes.push((c, pen, pen + width));
        pen += width;
    }
    let ink_end = pen + 2;
    assert!(
        inked_columns.iter().all(|&x| x >= margin && x < ink_end),
        "ink outside the glyph boxes"
    );
    for (c, start, end) in boxes {
        if c == ' ' {
            continue;
        }
        assert!(
            (start..end).any(dark),
            "no ink in the box of {c:?} at {start}..{end}"
        );
    }
    println!(
        "render: {} by {} px, kept at {}",
        width,
        height,
        png_path.display()
    );
}

fn path(path: &Path) -> &str {
    path.to_str().expect("utf-8 path")
}

#[test]
fn atlas_to_png_and_back_gives_the_same_png() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let Some(work) = work_dir() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    fs::create_dir_all(&work).unwrap();
    for name in ["FONT_en_H00", "FONT_en_H01"] {
        let atlas = work.join(format!("roundtrip-{name}.mzp"));
        let first = work.join(format!("roundtrip-{name}.png"));
        let rebuilt = work.join(format!("roundtrip-{name}.rebuilt.mzp"));
        let second = work.join(format!("roundtrip-{name}.rebuilt.png"));
        fs::write(&atlas, entry_data(&archive, &format!("{name}.mzp"))).unwrap();
        hunex(&["image", "to-png", path(&atlas), "-o", path(&first)]);
        hunex(&[
            "image",
            "from-png",
            path(&first),
            "-o",
            path(&rebuilt),
            "--like",
            path(&atlas),
        ]);
        hunex(&["image", "to-png", path(&rebuilt), "-o", path(&second)]);
        assert_eq!(
            fs::read(&first).unwrap(),
            fs::read(&second).unwrap(),
            "{name}: PNG differs"
        );
    }
}

#[test]
fn font_atlas_streams_decode_the_same_under_both_window_readings() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    let mut atlases = 0usize;
    let mut differing = Vec::new();
    for entry in archive.entries() {
        if !entry.name().starts_with("FONT_") || !entry.name().ends_with(".mzp") {
            continue;
        }
        let container = hunex_mzp::parse(entry.data()).unwrap();
        let Some(stream) = container.entries.get(1) else {
            continue;
        };
        if !hunex_mzx::is_mzx0(stream) {
            continue;
        }
        atlases += 1;
        let command = hunex_mzx::decompress_with(stream, hunex_mzx::WindowRule::Command).unwrap();
        let word = hunex_mzx::decompress_with(stream, hunex_mzx::WindowRule::Word).unwrap();
        if command != word {
            differing.push(entry.name().to_string());
        }
    }
    println!("{atlases} font atlas streams; readings that differ: {differing:?}");
    assert!(differing.is_empty(), "{differing:?}");
}

#[test]
fn font_atlas_streams_stay_inside_the_original_envelope() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    let mut envelope = hunex_mzx::Envelope::default();
    for entry in archive.entries() {
        if !entry.name().starts_with("FONT_") || !entry.name().ends_with(".mzp") {
            continue;
        }
        let container = hunex_mzp::parse(entry.data()).unwrap();
        for stream in container.entries.iter().skip(1) {
            if hunex_mzx::is_mzx0(stream) {
                envelope.add(stream).unwrap();
            }
        }
    }
    println!("font atlases: {envelope:?}");
    assert!(envelope.within_original_envelope(), "{envelope:?}");
}

#[test]
fn font_atlas_encoder_output_stays_inside_the_envelope() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    let mut envelope = hunex_mzx::Envelope::default();
    for name in ["FONT_en_H00.mzp", "FONT_en_H01.mzp"] {
        let original = entry_data(&archive, name);
        let image = hunex_image::decode_mzp(original).unwrap();
        let layout =
            hunex_image::parse_header(hunex_mzp::parse(original).unwrap().entries[0]).unwrap();
        let packed =
            hunex_image::encode_mzp(&image.rgba, image.width, image.height, Some(&layout)).unwrap();
        for stream in hunex_mzp::parse(&packed).unwrap().entries.iter().skip(1) {
            envelope.add(stream).unwrap();
        }
    }
    println!("font atlas encoder output: {envelope:?}");
    assert!(envelope.within_original_envelope(), "{envelope:?}");
    assert_eq!(
        envelope.repeats, 0,
        "encoder output contains repeat commands"
    );
}

#[test]
fn font_atlas_encoder_streams_start_every_window_with_a_literal() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    for name in ["FONT_en_H00.mzp", "FONT_en_H01.mzp"] {
        let original = entry_data(&archive, name);
        let image = hunex_image::decode_mzp(original).unwrap();
        let layout =
            hunex_image::parse_header(hunex_mzp::parse(original).unwrap().entries[0]).unwrap();
        let packed =
            hunex_image::encode_mzp(&image.rgba, image.width, image.height, Some(&layout)).unwrap();
        for (k, stream) in hunex_mzp::parse(&packed)
            .unwrap()
            .entries
            .iter()
            .enumerate()
            .skip(1)
        {
            let mut envelope = hunex_mzx::Envelope::default();
            envelope.add(stream).unwrap();
            assert_eq!(envelope.window_starts_not_literal, 0, "{name} entry {k}");
            assert!(
                envelope.within_original_envelope(),
                "{name} entry {k}: {envelope:?}"
            );
            let none = hunex_mzx::decompress_with(stream, hunex_mzx::WindowRule::None).unwrap();
            let command =
                hunex_mzx::decompress_with(stream, hunex_mzx::WindowRule::Command).unwrap();
            let word = hunex_mzx::decompress_with(stream, hunex_mzx::WindowRule::Word).unwrap();
            assert!(
                none == command && command == word,
                "{name} entry {k}: readings differ"
            );
        }
    }
}

#[test]
fn every_font_atlas_has_a_transparent_blank_and_round_trips_unchanged() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    let mut atlases = 0usize;
    for entry in archive.entries() {
        if !entry.name().starts_with("FONT_") || !entry.name().ends_with(".mzp") {
            continue;
        }
        let container = hunex_mzp::parse(entry.data()).unwrap();
        let header = container.entries[0];
        if header[12] != hunex_image::ATLAS_TYPE {
            continue;
        }
        atlases += 1;
        let image = hunex_image::decode_mzp(entry.data()).unwrap();
        let mut counts: std::collections::HashMap<[u8; 4], usize> =
            std::collections::HashMap::new();
        for px in image.rgba.chunks_exact(4) {
            *counts.entry([px[0], px[1], px[2], px[3]]).or_insert(0) += 1;
        }
        let (dominant, _) = counts.iter().max_by_key(|(_, n)| **n).unwrap();
        assert_eq!(
            dominant[3],
            0,
            "{}: the most common pixel is not transparent",
            entry.name()
        );
        let (packed, reports) =
            hunex_image::encode_like_mzp(&image.rgba, image.width, image.height, entry.data())
                .unwrap();
        assert!(reports.is_empty());
        assert_eq!(
            packed,
            entry.data(),
            "{}: unchanged atlas is not byte-identical",
            entry.name()
        );
    }
    println!(
        "{atlases} font atlases: blank is transparent and unchanged atlases are byte-identical"
    );
    assert!(atlases >= 2);
}

#[test]
fn every_real_glyph_width_fits_its_cell() {
    let Some(bytes) = archive_bytes() else {
        return;
    };
    let archive = Archive::parse(&bytes).unwrap();
    let mut widest = 0u32;
    for (name, _, coords, _, _) in TABLES {
        if !coords {
            continue;
        }
        let table = Table::parse(&payload(&archive, &format!("{name}.ccit"))).unwrap();
        for glyph in table.glyphs() {
            widest = widest.max(glyph.coord.unwrap().width);
        }
    }
    println!("widest glyph in the coordinate tables: {widest} px");
    assert!(widest <= CELL_SIZE, "{widest} px");
}
