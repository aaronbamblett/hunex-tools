//! CompressedBG images (`.cbg`): info, to-png, and from-png.

use std::fs;
use std::io::{BufReader, BufWriter};
use std::process::ExitCode;

use hunex_cbg::{decode, encode, encode_simple, Header, Image, STRIPE_HEIGHT};

use crate::cli::{finish, parse, usage, CliError};

/// Help text for the `cbg` (CompressedBG images) command.
pub const HELP: &str = "\
hunex cbg: CompressedBG images (.cbg)

Usage:
  hunex cbg info <file.cbg>
  hunex cbg to-png <file.cbg> -o <out.png>
  hunex cbg from-png <in.png> -o <out.cbg> [--like <original.cbg>] [--simple]
  hunex cbg help

to-png writes the image in its own color type: gray for 8 bits per pixel, RGB for 24, RGBA for 32.
from-png picks the depth from the PNG's color type (gray 8, RGB 24, RGBA 32). With --like, the
depth and stripe height come from the original, and the PNG must be the same size. --simple
writes literal runs with equal weights: larger, but decodes to the same pixels.
";

const INFO_USAGE: &str = "usage: hunex cbg info <file.cbg>\n";
const TO_PNG_USAGE: &str = "usage: hunex cbg to-png <file.cbg> -o <out.png>\n";
const FROM_PNG_USAGE: &str =
    "usage: hunex cbg from-png <in.png> -o <out.cbg> [--like <original.cbg>] [--simple]\n";

/// Runs the command with the arguments that follow its name. Returns the exit code.
pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            out!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("info") => info(&args[1..]),
        Some("to-png") => to_png(&args[1..]),
        Some("from-png") => from_png(&args[1..]),
        Some(other) => {
            eprintln!("hunex cbg: unknown command `{other}`");
            usage(HELP)
        }
    }
}

fn message(context: &str, error: impl std::fmt::Display) -> CliError {
    CliError::Message(format!("{context}: {error}"))
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
    let bytes = fs::read(path).map_err(|e| message(path, e))?;
    let image = decode(&bytes).map_err(|e| message(path, e))?;
    let header = image.header;
    let stripes = header.height.div_ceil(header.stripe_height);
    outln!("size:          {} x {}", header.width, header.height);
    outln!(
        "bits per pixel: {} ({})",
        header.bits_per_pixel,
        mode_name(&header)
    );
    outln!(
        "stripe height: {} ({stripes} stripes)",
        header.stripe_height
    );
    outln!("file size:     {} bytes", bytes.len());
    Ok(())
}

fn mode_name(header: &Header) -> &'static str {
    match header.bits_per_pixel {
        8 => "gray",
        24 => "RGB",
        _ => "RGBA",
    }
}

fn to_png(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o"], &[]) else {
        return usage(TO_PNG_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(TO_PNG_USAGE);
    };
    finish(to_png_file(input, output))
}

fn to_png_file(input: &str, output: &str) -> Result<(), CliError> {
    let bytes = fs::read(input).map_err(|e| message(input, e))?;
    let image = decode(&bytes).map_err(|e| message(input, e))?;
    let (color, data) = match image.header.bits_per_pixel {
        8 => (
            png::ColorType::Grayscale,
            image.rgba.chunks_exact(4).map(|p| p[0]).collect(),
        ),
        24 => (
            png::ColorType::Rgb,
            image
                .rgba
                .chunks_exact(4)
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect(),
        ),
        _ => (png::ColorType::Rgba, image.rgba.clone()),
    };
    write_png(
        output,
        image.header.width,
        image.header.height,
        color,
        &data,
    )
}

fn write_png(
    output: &str,
    width: usize,
    height: usize,
    color: png::ColorType,
    data: &[u8],
) -> Result<(), CliError> {
    let file = fs::File::create(output).map_err(|e| message(output, e))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| message(output, e))?;
    writer
        .write_image_data(data)
        .map_err(|e| message(output, e))
}

fn from_png(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o", "--like"], &["--simple"]) else {
        return usage(FROM_PNG_USAGE);
    };
    let (&[input], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(FROM_PNG_USAGE);
    };
    finish(from_png_file(
        input,
        output,
        parsed.value("--like"),
        parsed.flag("--simple"),
    ))
}

/// The pixels of a PNG as RGBA, and the bit depth its color type suggests.
fn read_png(input: &str) -> Result<(usize, usize, usize, Vec<u8>), CliError> {
    let file = fs::File::open(input).map_err(|e| message(input, e))?;
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| message(input, e))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| message(input, "image too large"))?;
    let mut buffer = vec![0; size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| message(input, e))?;
    let data = &buffer[..info.buffer_size()];
    let (bits, rgba) = match info.color_type {
        png::ColorType::Grayscale => (8, data.iter().flat_map(|&g| [g, g, g, 255]).collect()),
        png::ColorType::GrayscaleAlpha => (
            32,
            data.chunks_exact(2)
                .flat_map(|p| [p[0], p[0], p[0], p[1]])
                .collect(),
        ),
        png::ColorType::Rgb => (
            24,
            data.chunks_exact(3)
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
        ),
        png::ColorType::Rgba => (32, data.to_vec()),
        other => {
            return Err(message(
                input,
                format!("color type {other:?} is not supported"),
            ))
        }
    };
    Ok((info.width as usize, info.height as usize, bits, rgba))
}

fn from_png_file(
    input: &str,
    output: &str,
    like: Option<&str>,
    simple: bool,
) -> Result<(), CliError> {
    let (width, height, png_bits, rgba) = read_png(input)?;
    let header = match like {
        Some(path) => {
            let original = fs::read(path).map_err(|e| message(path, e))?;
            let header = hunex_cbg::parse_header(&original).map_err(|e| message(path, e))?;
            if (header.width, header.height) != (width, height) {
                return Err(message(
                    input,
                    format!(
                        "PNG is {width} x {height}, the original is {} x {}",
                        header.width, header.height
                    ),
                ));
            }
            header
        }
        None => Header {
            width,
            height,
            stripe_height: STRIPE_HEIGHT,
            bits_per_pixel: png_bits as u8,
        },
    };
    let image = Image { header, rgba };
    let packed = if simple {
        encode_simple(&image)
    } else {
        encode(&image)
    }
    .map_err(|e| message(input, e))?;
    fs::write(output, packed).map_err(|e| message(output, e))
}
