//! `.ccit` glyph tables: info, TSV export and import, and a text render from atlas pages.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::process::ExitCode;

use hunex_ccit::{cell_of, char_of, Cell, Coord, Glyph, Record, Table, CELL_SIZE, GLYPHS_PER_PAGE};
use hunex_image::{decode_mzp, Image};

use crate::cli::{finish, parse, usage, CliError};
use crate::image::write_png;

/// Help text for the `font` (glyph tables) command.
pub const HELP: &str = "\
hunex font: glyph tables (.ccit) of the HuneX engine

Usage:
  hunex font info <x.ccit>
  hunex font export <x.ccit> -o <glyphs.tsv>
  hunex font import <glyphs.tsv> -o <x.ccit>
  hunex font render <x.ccit> --atlas <page0.mzp> [--atlas <page1.mzp> ...] --text \"...\" -o <out.png>
  hunex font help

A .ccit file is the decompressed table, or the stored LenZu entry. The TSV has the header
char, code, index, page, x, y, w, h. The code is the decimal UTF-8 integer. The coordinate
columns are empty for a table without coordinates; import sorts rows by code and needs indices
0 to N-1 with no repeats.

Render draws each glyph from its index cell: page = index / 98, column = index % 14, row =
(index % 98) / 14, cells 73 px square. The stored x is not used for drawing; the 8 page-1
records of the 106-glyph tables store x values that land on other glyphs. The stored width
is the glyph's drawn width, and it is used as the pen advance (the advance is assumed, not verified in game). Missing characters are skipped with a message.
";

const INFO_USAGE: &str = "usage: hunex font info <x.ccit>\n";
const EXPORT_USAGE: &str = "usage: hunex font export <x.ccit> -o <glyphs.tsv>\n";
const IMPORT_USAGE: &str = "usage: hunex font import <glyphs.tsv> -o <x.ccit>\n";
const RENDER_USAGE: &str =
    "usage: hunex font render <x.ccit> --atlas <page0.mzp> [--atlas <page1.mzp> ...] --text \"...\" -o <out.png>\n";
const TSV_HEADER: &str = "char\tcode\tindex\tpage\tx\ty\tw\th";
const MARGIN: usize = 8;
/// The most pixels one render may have (64 Mpx).
const MAX_RENDER_PIXELS: usize = 1 << 26;

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("info") => info(&args[1..]),
        Some("export") => export(&args[1..]),
        Some("import") => import(&args[1..]),
        Some("render") => render(&args[1..]),
        Some(other) => {
            eprintln!("hunex font: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn info(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &[], &[]) else {
        return usage(INFO_USAGE);
    };
    let [path] = parsed.positional.as_slice() else {
        return usage(INFO_USAGE);
    };
    finish(info_file(path))
}

fn info_file(path: &str) -> Result<(), CliError> {
    let table = read_table(path)?;
    let records = table.records();
    outln!("glyphs:        {}", records.len());
    match table.coords() {
        Some(_) => outln!("coordinates:   present (page, x, y, width, height per glyph)"),
        None => outln!("coordinates:   absent"),
    }
    let codes = records.iter().map(|r| r.code);
    if let (Some(low), Some(high)) = (codes.clone().min(), codes.max()) {
        outln!(
            "code range:    {} .. {}",
            describe_code(low),
            describe_code(high)
        );
    }
    let indices = records.iter().map(|r| r.index);
    if let (Some(low), Some(high)) = (indices.clone().min(), indices.max()) {
        outln!("index range:   {low} .. {high}");
    }
    let sorted = records.windows(2).all(|pair| pair[0].code < pair[1].code);
    outln!(
        "code order:    {}",
        if sorted { "sorted" } else { "not sorted" }
    );
    let coords = table.coords().map(<[Coord]>::to_vec);
    let dense = Table::from_parts(records.to_vec(), coords).is_ok();
    outln!(
        "index check:   {}",
        if dense {
            "dense, 0 to N-1"
        } else {
            "not dense"
        }
    );
    let mut pages: BTreeMap<u32, usize> = BTreeMap::new();
    for record in records {
        *pages.entry(record.index / GLYPHS_PER_PAGE).or_insert(0) += 1;
    }
    for (page, count) in &pages {
        outln!("page {page}:       {count} glyphs");
    }
    Ok(())
}

fn describe_code(code: u32) -> String {
    match char_of(code) {
        Some(c) => format!("U+{:04X} '{c}'", c as u32),
        None => format!("0x{code:X} (not one character)"),
    }
}

fn export(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(EXPORT_USAGE);
    };
    let (Some(output), [path]) = (parsed.value("-o"), parsed.positional.as_slice()) else {
        return usage(EXPORT_USAGE);
    };
    finish(export_file(path, output))
}

fn export_file(path: &str, output: &str) -> Result<(), CliError> {
    let table = read_table(path)?;
    let mut text = String::from(TSV_HEADER);
    text.push('\n');
    for glyph in table.glyphs() {
        let code = glyph.record.code;
        let c = char_of(code)
            .filter(|c| !c.is_control())
            .ok_or_else(|| message(path, format!("code 0x{code:X} has no printable character")))?;
        let mut columns = vec![
            c.to_string(),
            code.to_string(),
            glyph.record.index.to_string(),
        ];
        match glyph.coord {
            Some(k) => columns.extend(
                [k.page, k.x, k.y, k.width, k.height]
                    .into_iter()
                    .map(|value| value.to_string()),
            ),
            None => columns.extend((0..5).map(|_| String::new())),
        }
        text.push_str(&columns.join("\t"));
        text.push('\n');
    }
    fs::write(output, text).map_err(|e| message(output, e))
}

fn import(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(IMPORT_USAGE);
    };
    let (Some(output), [path]) = (parsed.value("-o"), parsed.positional.as_slice()) else {
        return usage(IMPORT_USAGE);
    };
    finish(import_file(path, output))
}

fn import_file(path: &str, output: &str) -> Result<(), CliError> {
    let text = fs::read_to_string(path).map_err(|e| message(path, e))?;
    let mut lines = text.lines();
    if lines.next() != Some(TSV_HEADER) {
        return Err(message(
            path,
            format!("the first line must be `{TSV_HEADER}`"),
        ));
    }
    let mut rows: Vec<(Record, Option<Coord>)> = Vec::new();
    let mut with_coords: Option<bool> = None;
    for (number, line) in lines.enumerate().map(|(n, line)| (n + 2, line)) {
        if line.is_empty() {
            continue;
        }
        let fail = |what: &str| message(path, format!("line {number}: {what}"));
        let fields: Vec<&str> = line.split('\t').collect();
        let [ch, code, index, page, x, y, width, height] = fields.as_slice() else {
            return Err(fail("expected 8 columns"));
        };
        let code: u32 = code.parse().map_err(|_| fail("code is not a number"))?;
        let index: u32 = index.parse().map_err(|_| fail("index is not a number"))?;
        if !ch.is_empty() && char_of(code) != ch.chars().next().filter(|_| ch.chars().count() == 1)
        {
            return Err(fail("the char column does not match the code"));
        }
        let coord_fields = [page, x, y, width, height];
        let filled = coord_fields
            .iter()
            .filter(|field| !field.is_empty())
            .count();
        let coord = match filled {
            0 => None,
            5 => {
                let mut values = [0u32; 5];
                for (value, field) in values.iter_mut().zip(coord_fields) {
                    *value = field
                        .parse()
                        .map_err(|_| fail("coordinate is not a number"))?;
                }
                let [page, x, y, width, height] = values;
                Some(Coord {
                    page,
                    x,
                    y,
                    width,
                    height,
                })
            }
            _ => {
                return Err(fail(
                    "the coordinate columns must all be filled or all empty",
                ))
            }
        };
        match with_coords {
            None => with_coords = Some(coord.is_some()),
            Some(expected) if expected != coord.is_some() => {
                return Err(fail("rows mix glyphs with and without coordinates"));
            }
            Some(_) => {}
        }
        rows.push((Record { code, index }, coord));
    }
    if rows.is_empty() {
        return Err(message(path, "no glyph rows"));
    }
    rows.sort_by_key(|(record, _)| record.code);
    let records: Vec<Record> = rows.iter().map(|(record, _)| *record).collect();
    let coords = match with_coords {
        Some(true) => {
            let mut coords = Vec::with_capacity(rows.len());
            for (record, coord) in &rows {
                coords
                    .push(coord.ok_or_else(|| message(path, format!("code 0x{:X}", record.code)))?);
            }
            Some(coords)
        }
        _ => None,
    };
    let table = Table::from_parts(records, coords).map_err(|e| message(path, e))?;
    fs::write(output, hunex_lenzu::compress(&table.to_bytes())).map_err(|e| message(output, e))
}

fn render(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["--atlas", "--text", "-o"], &[]) else {
        return usage(RENDER_USAGE);
    };
    let atlases = parsed.values("--atlas");
    let (Some(text), Some(output), [path], false) = (
        parsed.value("--text"),
        parsed.value("-o"),
        parsed.positional.as_slice(),
        atlases.is_empty(),
    ) else {
        return usage(RENDER_USAGE);
    };
    finish(render_file(path, &atlases, text, output))
}

struct Placed {
    pen: usize,
    coverage: Vec<u8>,
    left: Option<usize>,
}

fn render_file(path: &str, atlas_paths: &[&str], text: &str, output: &str) -> Result<(), CliError> {
    let table = read_table(path)?;
    let atlases = atlas_paths
        .iter()
        .map(|atlas| {
            let bytes = fs::read(atlas).map_err(|e| message(atlas, e))?;
            decode_mzp(&bytes).map_err(|e| message(atlas, e))
        })
        .collect::<Result<Vec<Image>, CliError>>()?;
    let mut placed = Vec::new();
    let mut pen = 0usize;
    for c in text.chars() {
        let Some(Glyph { record, coord }) = table.find(c) else {
            eprintln!("hunex font: no glyph for U+{:04X}, skipped", c as u32);
            continue;
        };
        let cell = cell_of(record.index);
        let atlas = atlases.get(cell.page as usize).ok_or_else(|| {
            message(
                path,
                format!(
                    "U+{:04X} is on atlas page {}; add that page with --atlas",
                    c as u32, cell.page
                ),
            )
        })?;
        if cell.x + CELL_SIZE > atlas.width as u32 || cell.y + CELL_SIZE > atlas.height as u32 {
            return Err(message(
                path,
                format!("index {} is outside its atlas", record.index),
            ));
        }
        let advance = coord.map_or(CELL_SIZE, |k| k.width);
        if advance > CELL_SIZE {
            return Err(message(
                path,
                format!("U+{:04X}: width {advance} is wider than its cell", c as u32),
            ));
        }
        let advance = advance as usize;
        let coverage = cell_coverage(atlas, cell);
        let left = (0..CELL_SIZE as usize).find(|&cx| {
            (0..CELL_SIZE as usize).any(|cy| coverage[cy * CELL_SIZE as usize + cx] > 0)
        });
        placed.push(Placed {
            pen,
            coverage,
            left,
        });
        pen += advance;
    }
    let cell = CELL_SIZE as usize;
    let width = pen + cell + 2 * MARGIN;
    let height = cell + 2 * MARGIN;
    if width
        .checked_mul(height)
        .is_none_or(|pixels| pixels > MAX_RENDER_PIXELS)
    {
        return Err(message(path, "the text is too long to render"));
    }
    let mut ink = vec![255u8; width * height];
    for piece in &placed {
        let Some(left) = piece.left else {
            continue;
        };
        for cy in 0..cell {
            for cx in left..cell {
                let coverage = piece.coverage[cy * cell + cx];
                if coverage == 0 {
                    continue;
                }
                let x = MARGIN + piece.pen + cx - left;
                let y = MARGIN + cy;
                let at = y * width + x;
                ink[at] = ink[at].min(255 - coverage);
            }
        }
    }
    let rgba: Vec<u8> = ink.iter().flat_map(|&v| [v, v, v, 255]).collect();
    write_png(output, width, height, &rgba)
}

/// The glyph coverage of one atlas cell: the palette alpha of each pixel.
fn cell_coverage(atlas: &Image, cell: Cell) -> Vec<u8> {
    let size = CELL_SIZE as usize;
    let mut coverage = Vec::with_capacity(size * size);
    for cy in 0..size {
        for cx in 0..size {
            coverage.push(
                atlas.rgba[((cell.y as usize + cy) * atlas.width + cell.x as usize + cx) * 4 + 3],
            );
        }
    }
    coverage
}

fn read_table(path: &str) -> Result<Table, CliError> {
    let bytes = fs::read(path).map_err(|e| message(path, e))?;
    let payload = if hunex_lenzu::is_lenzu(&bytes) {
        hunex_lenzu::decompress(&bytes).map_err(|e| message(path, e))?
    } else {
        bytes
    };
    Table::parse(&payload).map_err(|e| message(path, e))
}

fn message(context: &str, error: impl Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
}
