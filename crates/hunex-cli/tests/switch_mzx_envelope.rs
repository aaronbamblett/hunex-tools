//! Measures the Switch script streams of the Tsukihime RomFS: which commands the original encoder
//! uses, and how its copies sit against the 4096-word windows. Gated on HUNEX_SWITCH_ROMFS_A.
//! Prints the summary; run with `--nocapture`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use hunex_mzx::{switch, Step, WINDOW_WORDS};

const WINDOW: usize = WINDOW_WORDS;

#[test]
#[ignore = "measurement tool; requires HUNEX_SWITCH_ROMFS_A and --nocapture"]
fn switch_script_streams_envelope() {
    let Some(dir) = std::env::var_os("HUNEX_SWITCH_ROMFS_A").map(PathBuf::from) else {
        eprintln!("HUNEX_SWITCH_ROMFS_A is not set; skipping the Switch MZX measurement");
        return;
    };
    let bytes = std::fs::read(dir.join("allscr.mrg")).unwrap();
    let container = hunex_mzp::parse_switch(&bytes).unwrap();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut streams = 0usize;
    let mut max_distance = 0usize;
    let mut max_copy = 0usize;
    let mut max_literal_run = 0usize;
    let mut max_size = 0usize;
    let mut crossing = 0usize;
    let mut source_before_window = 0usize;
    let mut window_start_non_literal = 0usize;
    let mut ring_uses = 0usize;
    let mut ring_from_earlier_window = 0usize;
    let mut distance_256 = 0usize;
    let mut repeat_at_window_start = 0usize;
    for (index, entry) in container.entries.iter().enumerate().skip(3) {
        if entry.is_empty() {
            continue;
        }
        let plain = switch::decompress(entry).unwrap_or_else(|e| panic!("entry {index}: {e}"));
        max_size = max_size.max(plain.len());
        let steps = switch::trace(entry).unwrap();
        streams += 1;
        for step in steps {
            match step {
                Step::Literal { words, .. } => {
                    *counts.entry("literal").or_default() += 1;
                    max_literal_run = max_literal_run.max(words);
                }
                Step::Repeat { start, words } => {
                    *counts.entry("repeat").or_default() += 1;
                    repeat_at_window_start += usize::from(start % WINDOW == 0);
                    window_start_non_literal += usize::from(start % WINDOW == 0);
                    crossing += usize::from(start / WINDOW != (start + words - 1) / WINDOW);
                }
                Step::BackReference {
                    start,
                    words,
                    distance,
                } => {
                    *counts.entry("copy").or_default() += 1;
                    max_distance = max_distance.max(distance);
                    max_copy = max_copy.max(words);
                    distance_256 += usize::from(distance == 256);
                    window_start_non_literal += usize::from(start % WINDOW == 0);
                    crossing += usize::from(start / WINDOW != (start + words - 1) / WINDOW);
                    let window = start / WINDOW * WINDOW;
                    source_before_window += usize::from(start - distance < window);
                }
                Step::Ring {
                    start, written_at, ..
                } => {
                    *counts.entry("ring").or_default() += 1;
                    ring_uses += 1;
                    window_start_non_literal += usize::from(start % WINDOW == 0);
                    ring_from_earlier_window += usize::from(written_at < start / WINDOW * WINDOW);
                }
            }
        }
    }
    eprintln!("Switch script streams: {streams}, largest decompressed size {max_size} bytes");
    eprintln!("commands: {counts:?}");
    eprintln!("copy distance: max {max_distance} words, distance 256 used {distance_256} times");
    eprintln!("longest copy {max_copy} words, longest literal run {max_literal_run} words");
    eprintln!("commands crossing a 4096-word boundary: {crossing}");
    eprintln!("copies whose source lies before the window start: {source_before_window}");
    eprintln!("non-literal commands that start a window: {window_start_non_literal}, repeats among them: {repeat_at_window_start}");
    eprintln!("ring recalls: {ring_uses}, of which read an entry written in an earlier window: {ring_from_earlier_window}");
}

/// Checks the encoder on one plain script: the decode is identical, and the stream stays inside
/// the measured envelope. Returns the encoded size.
fn check_encoder(label: &str, plain: &[u8]) -> usize {
    let encoded = switch::compress(plain);
    assert_eq!(
        switch::decompress(&encoded).unwrap(),
        plain,
        "{label}: decode differs"
    );
    for step in switch::trace(&encoded).unwrap() {
        match step {
            Step::Repeat { .. } => panic!("{label}: encoder wrote a repeat"),
            Step::Ring { .. } => panic!("{label}: encoder wrote a ring recall"),
            Step::Literal { .. } => {}
            Step::BackReference {
                start,
                words,
                distance,
            } => {
                assert!(distance <= 255, "{label}: distance {distance}");
                assert_eq!(
                    start / WINDOW,
                    (start + words - 1) / WINDOW,
                    "{label}: copy crosses a window"
                );
                assert!(
                    start - distance >= start / WINDOW * WINDOW,
                    "{label}: copy source before window"
                );
                assert!(start % WINDOW != 0, "{label}: copy starts a window");
            }
        }
    }
    encoded.len()
}

#[test]
fn switch_encoder_round_trips_and_stays_inside_the_envelope() {
    let mut total_original = 0usize;
    let mut total_encoded = 0usize;
    if let Some(dir) = std::env::var_os("HUNEX_SWITCH_ROMFS_A").map(PathBuf::from) {
        let bytes = std::fs::read(dir.join("allscr.mrg")).unwrap();
        let container = hunex_mzp::parse_switch(&bytes).unwrap();
        for (index, entry) in container.entries.iter().enumerate().skip(3) {
            if entry.is_empty() {
                continue;
            }
            let plain = switch::decompress(entry).unwrap();
            total_original += entry.len();
            total_encoded += check_encoder(&format!("allscr entry {index}"), &plain);
        }
    } else {
        eprintln!("HUNEX_SWITCH_ROMFS_A is not set; skipping the Tsukihime encoder checks");
    }
    if let Some(dir) = std::env::var_os("HUNEX_SWITCH_DUMP_B").map(PathBuf::from) {
        let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
            .collect();
        names.sort();
        for path in &names {
            let plain = std::fs::read(path).unwrap();
            check_encoder(&format!("{path:?}"), &plain);
        }
        eprintln!("round-tripped {} set B script dumps", names.len());
    }
    if total_original > 0 {
        eprintln!(
            "encoder size: {total_encoded} bytes against {total_original} original stream bytes ({:.3}x)",
            total_encoded as f64 / total_original as f64
        );
    }
}
