//! Switch textures: info about a BNTX container and PNG output of one texture.

use std::fs;
use std::io::BufWriter;
use std::process::ExitCode;

use hunex_nx::{decode_texture, is_nxx, unwrap, Bntx};

use crate::cli::{finish, parse, usage, CliError};

pub const HELP: &str = "\
hunex nx: Switch textures (NXX-wrapped BNTX)

Usage:
  hunex nx info <in>                  list the textures of an NXX entry or a BNTX file
  hunex nx to-png <in> -o <out.png> [--index N]   decode one texture (default 0) to PNG
  hunex nx help

BC3 (format 0x1C06) textures are reported as unsupported.
";

const INFO_USAGE: &str = "usage: hunex nx info <in>\n";
const TO_PNG_USAGE: &str = "usage: hunex nx to-png <in> -o <out.png> [--index N]\n";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => usage(HELP),
        Some("help" | "-h" | "--help") => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("info") => info(&args[1..]),
        Some("to-png") => to_png(&args[1..]),
        Some(other) => {
            eprintln!("hunex nx: unknown command `{other}`");
            usage(HELP)
        }
    }
}

/// The BNTX bytes of a file: the file itself, or its unwrapped NXX body.
fn bntx_bytes(path: &str) -> Result<Vec<u8>, CliError> {
    let bytes = fs::read(path).map_err(|e| CliError::Message(format!("{path}: {e}")))?;
    if is_nxx(&bytes) {
        unwrap(&bytes).map_err(|e| CliError::Message(format!("{path}: {e}")))
    } else {
        Ok(bytes)
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
    let bytes = bntx_bytes(path)?;
    let container = Bntx::parse(&bytes).map_err(|e| CliError::Message(format!("{path}: {e}")))?;
    println!("{} textures", container.textures.len());
    for (index, texture) in container.textures.iter().enumerate() {
        let status = match decode_texture(&container, index) {
            Ok(_) => "decodes".to_string(),
            Err(error) => error.to_string(),
        };
        println!(
            "{index:>3}  {:<16}  0x{:04x}  {:>5}x{:<5}  {:>9} bytes  {status}",
            texture.name.as_deref().unwrap_or("-"),
            texture.format,
            texture.width,
            texture.height,
            texture.image_size
        );
    }
    Ok(())
}

fn to_png(args: &[String]) -> ExitCode {
    let Ok(parsed) = parse(args, &["-o", "--index"], &[]) else {
        return usage(TO_PNG_USAGE);
    };
    let (&[path], Some(output)) = (parsed.positional.as_slice(), parsed.value("-o")) else {
        return usage(TO_PNG_USAGE);
    };
    let index = match parsed.value("--index") {
        Some(value) => match value.parse::<usize>() {
            Ok(index) => index,
            Err(_) => return usage(TO_PNG_USAGE),
        },
        None => 0,
    };
    finish(to_png_file(path, output, index))
}

fn to_png_file(path: &str, output: &str, index: usize) -> Result<(), CliError> {
    let bytes = bntx_bytes(path)?;
    let container = Bntx::parse(&bytes).map_err(|e| CliError::Message(format!("{path}: {e}")))?;
    let image =
        decode_texture(&container, index).map_err(|e| CliError::Message(format!("{path}: {e}")))?;
    let file = fs::File::create(output).map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    let mut encoder = png::Encoder::new(
        BufWriter::new(file),
        image.width as u32,
        image.height as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    writer
        .write_image_data(&image.pixels)
        .map_err(|e| CliError::Message(format!("{output}: {e}")))?;
    Ok(())
}
