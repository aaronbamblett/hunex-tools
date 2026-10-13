use std::fmt::Write as _;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use hunex_chs::{opcode_info, Error, Instruction, Script};

const USAGE: &str = "usage: chs-dump [--strings] <file.chs>";

enum CliError {
    Io(io::Error),
    Chs(Error),
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError::Io(error)
    }
}

impl From<Error> for CliError {
    fn from(error: Error) -> Self {
        CliError::Chs(error)
    }
}

fn main() -> ExitCode {
    let mut strings_only = false;
    let mut path = None;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--strings" => strings_only = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') || path.is_some() => {
                eprintln!("chs-dump: unexpected argument {other}");
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            other => path = Some(other.to_string()),
        }
    }
    let Some(path) = path else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };

    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("chs-dump: {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let script = match Script::parse(&bytes) {
        Ok(script) => script,
        Err(error) => {
            eprintln!("chs-dump: {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let result =
        dump(&script, strings_only, &mut out).and_then(|()| out.flush().map_err(CliError::Io));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(CliError::Io(error)) => {
            eprintln!("chs-dump: {error}");
            ExitCode::FAILURE
        }
        Err(CliError::Chs(error)) => {
            eprintln!("chs-dump: {path}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn dump(script: &Script, strings_only: bool, out: &mut impl Write) -> Result<(), CliError> {
    for instruction in script.instructions() {
        let string_op = matches!(instruction.opcode(), 0x03 | 0x7F);
        if strings_only {
            if string_op {
                writeln!(out, "\"{}\"", escape(script.string(instruction)?))?;
            }
            continue;
        }
        write_instruction(script, instruction, out)?;
    }
    Ok(())
}

fn write_instruction(
    script: &Script,
    instruction: &Instruction,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let mnemonic = opcode_info(instruction.opcode())
        .and_then(|info| info.mnemonic)
        .unwrap_or("-");
    let mut line = format!(
        "0x{:06x}  0x{:03x}  {:<12}",
        instruction.offset(),
        instruction.opcode(),
        mnemonic
    );
    for operand in instruction.operands() {
        let _ = write!(line, " 0x{operand:08x}");
    }
    if matches!(instruction.opcode(), 0x03 | 0x7F) {
        let _ = write!(line, "  \"{}\"", escape(script.string(instruction)?));
    }
    writeln!(out, "{line}")?;
    Ok(())
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", control as u32);
            }
            other => out.push(other),
        }
    }
    out
}
