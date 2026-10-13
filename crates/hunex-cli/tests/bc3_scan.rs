//! BC3 (format 0x1C06) textures in a folder of MRG sets, gated on HUNEX_BC3_DIR. Each set is
//! read one entry at a time. Every BC3 texture is decoded and written as a PNG to HUNEX_WORK/bc3
//! (the PNG step is skipped when HUNEX_WORK is not set).

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufWriter, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use hunex_nx::{decode_texture, Bntx};

const BC3: u32 = 0x1C06;

fn sets(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let name = e.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".hed").map(str::to_string)
        })
        .collect();
    names.sort();
    names
}

/// One BC3 texture found in a set.
struct Found {
    set: String,
    entry: usize,
    entry_name: String,
    texture: usize,
    bytes: Vec<u8>,
}

fn write_png(path: &Path, width: usize, height: usize, rgba: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

fn stem(name: &str) -> String {
    name.rsplit_once('.')
        .map_or(name, |(stem, _)| stem)
        .to_string()
}

#[test]
fn bc3_textures_decode() {
    let Some(dir) = std::env::var_os("HUNEX_BC3_DIR").map(PathBuf::from) else {
        eprintln!("HUNEX_BC3_DIR is not set; skipping");
        return;
    };
    let mut found: Vec<Found> = Vec::new();
    let mut total = 0usize;
    for set in sets(&dir) {
        let hed = fs::read(dir.join(format!("{set}.hed"))).unwrap();
        let nam = fs::read(dir.join(format!("{set}.nam"))).ok();
        let index = hunex_mrg::index(&hed, nam.as_deref()).unwrap();
        let mut file = fs::File::open(dir.join(format!("{set}.mrg"))).unwrap();
        for meta in &index {
            if meta.row.is_empty() {
                continue;
            }
            let mut buffer = vec![0u8; meta.row.byte_span() as usize];
            file.seek(SeekFrom::Start(meta.row.byte_start())).unwrap();
            let read = file.read(&mut buffer).unwrap();
            buffer.truncate(read);
            let bntx = if buffer.starts_with(b"BNTX") {
                buffer
            } else if hunex_nx::is_nxx(&buffer) {
                match hunex_nx::unwrap(&buffer) {
                    Ok(bytes) => bytes,
                    Err(_) => continue,
                }
            } else {
                continue;
            };
            let Ok(parsed) = Bntx::parse(&bntx) else {
                continue;
            };
            for (texture, info) in parsed.textures.iter().enumerate() {
                total += 1;
                if info.format == BC3 {
                    found.push(Found {
                        set: set.clone(),
                        entry: meta.index,
                        entry_name: meta.name.clone().unwrap_or_default(),
                        texture,
                        bytes: bntx.clone(),
                    });
                }
            }
        }
    }
    println!("{}: {total} textures, {} BC3", dir.display(), found.len());

    let Some(work) = std::env::var_os("HUNEX_WORK").map(PathBuf::from) else {
        eprintln!("HUNEX_WORK is not set; not writing the PNGs");
        return;
    };
    let out = work.join("bc3");
    fs::create_dir_all(&out).unwrap();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in &found {
        *counts.entry(stem(&item.entry_name)).or_insert(0) += 1;
    }
    for item in &found {
        let parsed = Bntx::parse(&item.bytes).unwrap();
        let info = &parsed.textures[item.texture];
        let mut name = stem(&item.entry_name);
        if counts[&name] > 1 {
            name = format!("{name}_{}", item.entry);
        }
        if parsed.textures.len() > 1 {
            name = format!("{name}_t{}", item.texture);
        }
        let image = decode_texture(&parsed, item.texture).unwrap();
        assert_eq!((image.width, image.height), (info.width, info.height));
        assert_eq!(image.pixels.len(), info.width * info.height * 4);
        let path = out.join(format!("{name}.png"));
        write_png(&path, image.width, image.height, &image.pixels);
        println!(
            "{} entry {} {}: {}x{} -> {}",
            item.set,
            item.entry,
            item.entry_name,
            image.width,
            image.height,
            path.display()
        );
    }
}
