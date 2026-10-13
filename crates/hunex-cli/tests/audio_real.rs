//! Real audio, gated on HUNEX_HFA_DIR (data00300.hfa) and HUNEX_NOPUS_DIR (the Switch stream
//! folder). HUNEX_NOPUS_SURVEY=1 adds a survey of every file in that folder. HUNEX_FFMPEG=1
//! decodes the remuxed files with ffmpeg (ffmpeg must be on PATH). HUNEX_WORK picks the scratch
//! folder; it is removed afterwards.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use hunex_hfa::Archive;
use hunex_hw::Hw;
use hunex_nopus::Stream;
use hunex_ogg::pages;

/// Sample frames from the spec's table (last Ogg granule of each entry).
const FRAMES: [(&str, u32); 7] = [
    ("SSECancelled.hw", 5_480),
    ("SSEChoiced.hw", 11_275),
    ("SSEBookChoiced.hw", 9_010),
    ("SSEBookDecided.hw", 19_200),
    ("SSETurnedPage.hw", 35_269),
    ("SSEUnable.hw", 44_460),
    ("SSEDecided.hw", 77_712),
];

fn env_dir(name: &str) -> Option<PathBuf> {
    match std::env::var_os(name) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("{name} is not set; skipping");
            None
        }
    }
}

fn work_root() -> PathBuf {
    match std::env::var_os("HUNEX_WORK") {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir(),
    }
    .join(format!("hunex-audio-e2e-{}", std::process::id()))
}

#[test]
fn data00300_entries_rebuild_byte_for_byte_with_the_spec_header_facts() {
    let Some(dir) = env_dir("HUNEX_HFA_DIR") else {
        return;
    };
    let bytes = fs::read(dir.join("data00300.hfa")).unwrap();
    let archive = Archive::parse(&bytes).unwrap();
    assert_eq!(archive.entries().len(), 7);
    for entry in archive.entries() {
        let original = entry.data();
        let payload =
            hunex_hw::unwrap(original).unwrap_or_else(|e| panic!("{}: {e}", entry.name()));
        assert_eq!(
            hunex_hw::wrap(&payload).unwrap(),
            original,
            "{}: unwrap then wrap differs",
            entry.name()
        );
        let hw = Hw::parse(original).unwrap();
        assert_eq!(hw.sample_rate, 48_000, "{}", entry.name());
        assert_eq!(hw.channels, 2, "{}", entry.name());
        let expected = FRAMES
            .iter()
            .find(|(name, _)| *name == entry.name())
            .unwrap_or_else(|| panic!("unexpected entry {}", entry.name()));
        assert_eq!(
            hw.sample_frames,
            expected.1,
            "{}: sample frames",
            entry.name()
        );
    }
    println!("hw: 7 of 7 entries unwrap and wrap byte-for-byte; header facts match the spec");
}

#[test]
fn switch_samples_remux_to_the_spec_packet_counts_and_durations() {
    let Some(dir) = env_dir("HUNEX_NOPUS_DIR") else {
        return;
    };
    let work = work_root();
    fs::create_dir_all(&work).unwrap();
    let cases = [
        ("A00_01_00_0000_0", 297usize, 285_000u64, 1u8),
        ("A00_01_00_0001_0", 223, 213_960, 1),
        ("15aEDSample", 10_903, 10_466_760, 2),
    ];
    for (name, packets, frames, channels) in cases {
        let source = fs::read(dir.join(format!("{name}.opus"))).unwrap();
        let stream = Stream::parse(&source).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(stream.packets.len(), packets, "{name}: packets");
        assert_eq!(stream.decoded_samples(), frames, "{name}: decoded frames");
        assert_eq!(stream.channels, channels, "{name}: channels");
        let ogg = stream.to_ogg().unwrap();
        let ogg_pages = pages(&ogg).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(ogg_pages.len(), packets + 2, "{name}: page count");
        assert_eq!(ogg_pages.last().unwrap().granule, stream.total_samples());
        assert_eq!(
            ogg_pages.last().unwrap().header_type & hunex_ogg::FLAG_EOS,
            hunex_ogg::FLAG_EOS
        );
        let audio: Vec<&[u8]> = ogg_pages[2..].iter().map(|page| page.data).collect();
        let source_packets: Vec<&[u8]> = stream.packets.iter().map(|packet| packet.data).collect();
        assert!(audio == source_packets, "{name}: audio packets differ");

        if std::env::var_os("HUNEX_FFMPEG").is_some() {
            let path = work.join(format!("{name}.ogg"));
            fs::write(&path, &ogg).unwrap();
            let status = Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(&path)
                .args(["-f", "null", "-"])
                .status()
                .expect("run ffmpeg (HUNEX_FFMPEG is set)");
            assert!(
                status.success(),
                "{name}: ffmpeg could not decode the remux"
            );
        }
    }
    if std::env::var_os("HUNEX_FFMPEG").is_some() {
        println!("ffmpeg decodes all three remuxed files with exit 0");
    }
    println!("nopus: three named samples remux with the spec packet counts and exact durations");
    fs::remove_dir_all(&work).ok();
}

#[test]
fn every_switch_sample_parses_and_remuxes() {
    let Some(dir) = env_dir("HUNEX_NOPUS_DIR") else {
        return;
    };
    if std::env::var_os("HUNEX_NOPUS_SURVEY").is_none() {
        eprintln!("HUNEX_NOPUS_SURVEY is not set; skipping the full survey");
        return;
    }
    let mut mono = 0usize;
    let mut stereo = 0usize;
    let mut total = 0usize;
    let mut failures = Vec::new();
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "opus"))
        .collect();
    files.sort();
    for path in &files {
        total += 1;
        let source = fs::read(path).unwrap();
        match Stream::parse(&source).and_then(|stream| {
            let channels = stream.channels;
            stream.to_ogg().map(|ogg| (channels, ogg))
        }) {
            Ok((channels, ogg)) => {
                if pages(&ogg).is_err() {
                    failures.push(path.display().to_string());
                } else if channels == 1 {
                    mono += 1;
                } else {
                    stereo += 1;
                }
            }
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }
    println!(
        "survey: {total} files, {mono} mono, {stereo} stereo, {} failures",
        failures.len()
    );
    assert!(failures.is_empty(), "failures: {failures:?}");
    assert_eq!(total, 16_601);
    assert_eq!((mono, stereo), (16_468, 133));
}
