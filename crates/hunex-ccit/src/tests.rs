use super::*;
use alloc::vec;

fn word_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

fn sample_without_coords() -> Vec<u8> {
    let a = code_of('A');
    let b = code_of('B');
    word_bytes(&[2, 8, a, 0, b, 1, 0])
}

fn sample_with_coords() -> Vec<u8> {
    let a = code_of('A');
    let b = code_of('B');
    word_bytes(&[
        2, 8, a, 0, b, 1, //
        2, 20, 0, 1, 2, 3, 4, //
        0, 0, 10, 20, 30, //
    ])
}

#[test]
fn both_record_forms_parse_and_rebuild_exactly() {
    for bytes in [sample_without_coords(), sample_with_coords()] {
        let table = Table::parse(&bytes).unwrap();
        assert_eq!(table.to_bytes(), bytes);
    }
    let table = Table::parse(&sample_with_coords()).unwrap();
    assert_eq!(table.records().len(), 2);
    let coords = table.coords().unwrap();
    assert_eq!(
        coords[1],
        Coord {
            page: 0,
            x: 0,
            y: 10,
            width: 20,
            height: 30
        }
    );
    assert!(Table::parse(&sample_without_coords())
        .unwrap()
        .coords()
        .is_none());
}

#[test]
fn empty_tables_of_both_forms_round_trip() {
    let with = word_bytes(&[0, 8, 0, 20]);
    let without = word_bytes(&[0, 8, 0]);
    assert_eq!(Table::parse(&with).unwrap().to_bytes(), with);
    assert_eq!(Table::parse(&without).unwrap().to_bytes(), without);
}

#[test]
fn malformed_tables_are_rejected() {
    assert_eq!(
        Table::parse(&word_bytes(&[1, 4, 0, 0, 0])),
        Err(Error::RecordSize { found: 4 })
    );
    let mut truncated = sample_without_coords();
    truncated.truncate(13);
    assert!(matches!(
        Table::parse(&truncated),
        Err(Error::Truncated { .. })
    ));
    let mut long = sample_without_coords();
    long.push(0);
    assert!(Table::parse(&long).is_err());
    assert_eq!(
        Table::parse(&word_bytes(&[2, 8, 0x41, 0, 0x42, 1, 3, 20])),
        Err(Error::CoordCount {
            glyphs: 2,
            coords: 3
        })
    );
    assert_eq!(
        Table::parse(&word_bytes(&[2, 8, 0x41, 0, 0x42, 1, 2, 16])),
        Err(Error::CoordSize { found: 16 })
    );
    assert_eq!(
        Table::parse(&word_bytes(&[0, 8, 5])),
        Err(Error::CoordCount {
            glyphs: 0,
            coords: 5
        })
    );
}

#[test]
fn codes_follow_the_utf8_big_endian_rule() {
    assert_eq!(code_of('A'), 0x41);
    assert_eq!(code_of('à'), 0xC3A0);
    assert_eq!(code_of('÷'), 0xC3B7);
    assert_eq!(code_of('\u{FF5E}'), 0xEFBD9E);
    assert_eq!(char_of(0xC3A0), Some('à'));
    assert_eq!(char_of(0xEFBD9E), Some('\u{FF5E}'));
    assert_eq!(char_of(0), None);
    assert_eq!(char_of(0x80), None);
    assert_eq!(char_of(0xC3), None);
    assert_eq!(char_of(0x4142), None);
}

#[test]
fn glyph_lookup_and_slot_model() {
    let table = Table::parse(&sample_with_coords()).unwrap();
    let glyph = table.find('B').unwrap();
    assert_eq!(
        glyph.record,
        Record {
            code: 0x42,
            index: 1
        }
    );
    assert_eq!(glyph.coord.unwrap().y, 10);
    assert_eq!(table.find('C'), None);
    assert_eq!(
        cell_of(0),
        Cell {
            page: 0,
            x: 0,
            y: 0
        }
    );
    assert_eq!(
        cell_of(33),
        Cell {
            page: 0,
            x: 5 * 73,
            y: 2 * 73
        }
    );
    assert_eq!(
        cell_of(98),
        Cell {
            page: 1,
            x: 0,
            y: 0
        }
    );
    assert_eq!(
        cell_of(106),
        Cell {
            page: 1,
            x: 8 * 73,
            y: 0
        }
    );
}

#[test]
fn insert_keeps_code_order_and_dense_indices() {
    let mut table = Table::parse(&sample_with_coords()).unwrap();
    let coord = Coord {
        page: 0,
        x: 1,
        y: 2,
        width: 3,
        height: 73,
    };
    let index = table.insert(code_of('@'), Some(coord)).unwrap();
    assert_eq!(index, 2);
    let codes: Vec<u32> = table.records().iter().map(|r| r.code).collect();
    assert_eq!(codes, vec![0x40, 0x41, 0x42]);
    assert_eq!(table.coords().unwrap()[0], coord);
    let rebuilt = Table::parse(&table.to_bytes()).unwrap();
    assert_eq!(rebuilt, table);
    assert_eq!(
        table.insert(code_of('A'), Some(coord)),
        Err(Error::DuplicateCode { code: 0x41 })
    );
    assert_eq!(table.insert(code_of('Z'), None), Err(Error::CoordsMismatch));
}

#[test]
fn replace_keeps_the_record_and_index() {
    let mut table = Table::parse(&sample_with_coords()).unwrap();
    let coord = Coord {
        page: 1,
        x: 9,
        y: 8,
        width: 7,
        height: 73,
    };
    table.replace(code_of('B'), Some(coord)).unwrap();
    let glyph = table.find('B').unwrap();
    assert_eq!(glyph.record.index, 1);
    assert_eq!(glyph.coord, Some(coord));
    assert_eq!(
        table.replace(code_of('Q'), Some(coord)),
        Err(Error::NoSuchCode { code: 0x51 })
    );
}

#[test]
fn tables_without_coordinates_take_no_coordinate_entries() {
    let mut table = Table::parse(&sample_without_coords()).unwrap();
    assert_eq!(table.insert(code_of('C'), None), Ok(2));
    assert_eq!(
        table.to_bytes(),
        word_bytes(&[3, 8, 0x41, 0, 0x42, 1, 0x43, 2, 0])
    );
}

#[test]
fn from_parts_checks_the_invariants() {
    let records = vec![
        Record {
            code: 0x42,
            index: 0,
        },
        Record {
            code: 0x41,
            index: 1,
        },
    ];
    assert_eq!(Table::from_parts(records, None), Err(Error::Unsorted));
    let repeated = vec![
        Record {
            code: 0x41,
            index: 0,
        },
        Record {
            code: 0x41,
            index: 1,
        },
    ];
    assert_eq!(
        Table::from_parts(repeated, None),
        Err(Error::DuplicateCode { code: 0x41 })
    );
    let gap = vec![
        Record {
            code: 0x41,
            index: 0,
        },
        Record {
            code: 0x42,
            index: 2,
        },
    ];
    assert_eq!(Table::from_parts(gap, None), Err(Error::IndexNotDense));
    let ok = vec![
        Record {
            code: 0x41,
            index: 1,
        },
        Record {
            code: 0x42,
            index: 0,
        },
    ];
    assert!(Table::from_parts(ok, None).is_ok());
}

#[test]
fn a_coordinate_count_too_large_for_the_length_is_an_error_not_an_overflow() {
    let bytes = word_bytes(&[u32::MAX, 8, 0x41, 0, u32::MAX, 20]);
    assert!(Table::parse(&bytes).is_err());
}
