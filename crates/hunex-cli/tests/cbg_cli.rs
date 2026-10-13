//! The `hunex cbg` commands on synthetic PNGs (no game data needed).

use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cbg-cli");
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_png(path: &Path, width: u32, height: u32, color: png::ColorType, data: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(data)
        .unwrap();
}

fn read_rgba(path: &Path) -> (u32, u32, png::ColorType, Vec<u8>) {
    let mut decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(path).unwrap()));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buffer).unwrap();
    (
        info.width,
        info.height,
        info.color_type,
        buffer[..info.buffer_size()].to_vec(),
    )
}

fn hunex(args: &[&str]) {
    let status = Command::new(env!("CARGO_BIN_EXE_hunex"))
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "hunex {args:?} failed");
}

#[test]
fn from_png_picks_the_depth_from_the_color_type_and_round_trips() {
    let dir = scratch();
    let (width, height) = (70u32, 45u32);
    let gray: Vec<u8> = (0..width * height).map(|i| (i * 3 % 256) as u8).collect();
    let rgb: Vec<u8> = (0..width * height * 3)
        .map(|i| (i * 5 % 256) as u8)
        .collect();
    let rgba: Vec<u8> = (0..width * height * 4)
        .map(|i| (i * 7 % 256) as u8)
        .collect();
    let cases = [
        ("gray", png::ColorType::Grayscale, gray, 8u8),
        ("rgb", png::ColorType::Rgb, rgb, 24),
        ("rgba", png::ColorType::Rgba, rgba, 32),
    ];
    for (name, color, data, bits) in cases {
        let png_in = dir.join(format!("{name}.png"));
        let cbg = dir.join(format!("{name}.cbg"));
        let png_out = dir.join(format!("{name}.out.png"));
        write_png(&png_in, width, height, color, &data);
        hunex(&[
            "cbg",
            "from-png",
            png_in.to_str().unwrap(),
            "-o",
            cbg.to_str().unwrap(),
        ]);
        let info = fs::read(&cbg).unwrap();
        assert_eq!(
            &info[28..32],
            &u32::from(bits).to_le_bytes(),
            "{name}: depth"
        );
        hunex(&[
            "cbg",
            "to-png",
            cbg.to_str().unwrap(),
            "-o",
            png_out.to_str().unwrap(),
        ]);
        let (w, h, out_color, out) = read_rgba(&png_out);
        assert_eq!((w, h), (width, height));
        assert_eq!(out_color, color, "{name}: color type");
        assert_eq!(out, read_rgba(&png_in).3, "{name}: pixels");
        let again = dir.join(format!("{name}.again.cbg"));
        hunex(&[
            "cbg",
            "from-png",
            png_out.to_str().unwrap(),
            "-o",
            again.to_str().unwrap(),
            "--like",
            cbg.to_str().unwrap(),
        ]);
        assert_eq!(fs::read(&again).unwrap(), info, "{name}: like round trip");
    }
}
