//! Nintendo OPUS round trip: file -> Ogg Opus -> file must be byte-identical. Gated on
//! HUNEX_SWITCH_ROMFS_A (a RomFS folder with stream/). Checks the three named samples and every
//! 33rd file of the stream folder, about 500 files in all.

use std::path::PathBuf;

use hunex_nopus::{from_ogg, Stream};

#[test]
fn nopus_round_trips_through_ogg_opus() {
    let Some(romfs) = std::env::var_os("HUNEX_SWITCH_ROMFS_A").map(PathBuf::from) else {
        eprintln!("HUNEX_SWITCH_ROMFS_A is not set; skipping the Nintendo OPUS round trip");
        return;
    };
    let stream_dir = romfs.join("stream");
    let mut names: Vec<PathBuf> = std::fs::read_dir(&stream_dir)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "opus"))
        .collect();
    names.sort();
    let mut chosen: Vec<PathBuf> = [
        "A00_01_00_0000_0.opus",
        "A00_01_00_0001_0.opus",
        "15aEDSample.opus",
    ]
    .iter()
    .map(|name| stream_dir.join(name))
    .collect();
    chosen.extend(names.iter().step_by(33).cloned());
    let mut checked = 0usize;
    for path in &chosen {
        let original = std::fs::read(path).unwrap();
        let stream = Stream::parse(&original).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        assert_eq!(
            stream.to_bytes(),
            original,
            "{path:?}: Nintendo writer differs"
        );
        let ogg = stream.to_ogg().unwrap();
        let back = from_ogg(&ogg).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        assert!(back.fields_recorded, "{path:?}: fields not recorded");
        assert!(back.file == original, "{path:?}: round trip differs");
        checked += 1;
    }
    eprintln!("round-tripped {checked} Nintendo OPUS files");
}
