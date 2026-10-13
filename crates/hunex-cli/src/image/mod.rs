//! Background images: convert MZP image archives to PNG and back.

use std::fs;
use std::io::{BufReader, BufWriter};
use std::process::ExitCode;

use hunex_image::{decode_mzp, encode_like_mzp, encode_mzp};

use crate::cli::{finish, parse as parse_args, usage, CliError};

pub const HELP: &str = "\
hunex image: background images (MZP archives of palette tiles)

Usage:
  hunex image to-png <in.mzp> -o <out.png>
  hunex image from-png <in.png> -o <out.mzp> [--like <original.mzp>]

--like takes the canvas, tile grid, crop and flags from the original, and the PNG must be the
image's size. Without it, the image is padded with transparent pixels to whole 512x256 tiles.
Tiles whose colours do not fit in 256 are quantised (lossy).
  hunex image help
";

const TO_PNG_USAGE: &str = "usage: hunex image to-png <in.mzp> -o <out.png>\n";
const FROM_PNG_USAGE: &str =
    "usage: hunex image from-png <in.png> -o <out.mzp> [--like <original.mzp>]\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("to-png") => to_png(&args[1..]),
        Some("from-png") => from_png(&args[1..]),
        Some(other) => {
            eprintln!("hunex image: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn to_png(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse_args(args, &["-o"], &[]) else {
        return usage(TO_PNG_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(TO_PNG_USAGE);
    };
    finish(mzp_to_png(input, output))
}

fn mzp_to_png(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| message(input, e))?;
    let image = decode_mzp(&bytes).map_err(|e| message(input, e))?;
    write_png(output, image.width, image.height, &image.rgba)
}

fn from_png(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse_args(args, &["-o", "--like"], &[]) else {
        return usage(FROM_PNG_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(FROM_PNG_USAGE);
    };
    finish(png_to_mzp(input, output, parsed.value("--like")))
}

fn png_to_mzp(input: &str, output: &str, like: Option<&str>) -> Result<(), CliError> {
    let (width, height, rgba) = read_png(input)?;
    let packed =
        match like {
            Some(path) => {
                let original = fs::read(path).map_err(|e| message(path, e))?;
                let (packed, reports) = encode_like_mzp(&rgba, width, height, &original)
                    .map_err(|e| message(input, e))?;
                for report in reports {
                    eprintln!(
                    "tile {}: {} pixels changed, {} colours, {}, max error {}, mean error {:.3}",
                    report.index,
                    report.changed_pixels,
                    report.colours,
                    if report.quantised { "quantised" } else { "exact" },
                    report.max_error,
                    report.mean_error
                );
                }
                packed
            }
            None => encode_mzp(&rgba, width, height, None).map_err(|e| message(input, e))?,
        };
    fs::write(output, packed).map_err(|e| message(output, e))?;
    Ok(())
}

pub(crate) fn write_png(
    output: &str,
    width: usize,
    height: usize,
    rgba: &[u8],
) -> Result<(), CliError> {
    let file = fs::File::create(output).map_err(|e| message(output, e))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| message(output, e))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| message(output, e))?;
    Ok(())
}

fn read_png(input: &str) -> Result<(usize, usize, Vec<u8>), CliError> {
    let file = fs::File::open(input).map_err(|e| message(input, e))?;
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| message(input, e))?;
    let mut buffer = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or_else(|| message(input, "image too large"))?
    ];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| message(input, e))?;
    let width = info.width as usize;
    let height = info.height as usize;
    let data = &buffer[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => data
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        other => {
            return Err(message(
                input,
                format!("colour type {other:?} is not RGB or RGBA"),
            ))
        }
    };
    Ok((width, height, rgba))
}

fn message(context: &str, error: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
}
