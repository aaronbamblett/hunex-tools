//! Commands that print to standard output exit 0 when the reader closes the pipe, as `head` does.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn hunex() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hunex"))
}

fn work_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("hunex-pipe-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs the command with its standard output pipe closed before it writes anything.
fn run_with_closed_pipe(args: &[&str]) -> (bool, String) {
    let mut child = hunex()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start hunex");
    drop(child.stdout.take());
    let output = child.wait_with_output().expect("wait for hunex");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn assert_quiet_exit(args: &[&str]) {
    let (success, stderr) = run_with_closed_pipe(args);
    assert!(success, "{args:?} did not exit 0: {stderr}");
    assert!(!stderr.contains("panicked"), "{args:?} panicked: {stderr}");
    assert!(!stderr.contains("Broken pipe"), "{args:?}: {stderr}");
}

#[test]
fn help_and_version_into_a_closed_pipe_exit_cleanly() {
    assert_quiet_exit(&["help"]);
    assert_quiet_exit(&["--version"]);
    for format in [
        "cbg",
        "chs",
        "ctd",
        "font",
        "hfa",
        "hw",
        "image",
        "lenzu",
        "mrg",
        "mzp",
        "mzx",
        "nopus",
        "nx",
        "switch-script",
        "switch-text",
    ] {
        assert_quiet_exit(&[format, "help"]);
    }
}

#[test]
fn large_dump_into_a_closed_pipe_exits_cleanly() {
    let dir = work_dir("dump");
    let mut source = String::from("format hunex\nheader_size 8\n");
    for _ in 0..20_000 {
        source.push_str("    PushDword 0x5\n");
    }
    source.push_str("    Return\n");
    let text = dir.join("big.txt");
    let script = dir.join("big.chs");
    fs::write(&text, source).unwrap();

    let status = hunex()
        .args(["chs", "asm"])
        .arg(&text)
        .arg("-o")
        .arg(&script)
        .status()
        .expect("assemble");
    assert!(status.success());

    let script = script.to_str().unwrap();
    let result = std::panic::catch_unwind(|| {
        assert_quiet_exit(&["chs", "dump", script]);
        assert_quiet_exit(&["chs", "dump", "--asm", script]);
        assert_quiet_exit(&["chs", "dump", "--strings", script]);
    });
    let _ = fs::remove_dir_all(&dir);
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
