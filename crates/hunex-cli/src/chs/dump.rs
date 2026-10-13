use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use hunex_chs::{opcode_info, Instruction, Script};

use super::{asm, read_script};
use crate::cli::{finish, usage, CliError};

const USAGE: &str = "usage: hunex chs dump [--strings | --asm] <file.chs>\n";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Listing,
    Strings,
    Asm,
}

pub fn run(args: &[String]) -> ExitCode {
    let mut mode = None;
    let mut path = None;
    for argument in args {
        let next = match argument.as_str() {
            "--strings" => Mode::Strings,
            "--asm" => Mode::Asm,
            "-h" | "--help" => {
                out!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') || path.is_some() => return usage(USAGE),
            other => {
                path = Some(other);
                continue;
            }
        };
        if mode.replace(next).is_some() {
            return usage(USAGE);
        }
    }
    let Some(path) = path else {
        return usage(USAGE);
    };
    finish(dump(path, mode.unwrap_or(Mode::Listing)))
}

fn dump(path: &str, mode: Mode) -> Result<(), CliError> {
    let (_, script) = read_script(path)?;
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    match mode {
        Mode::Asm => {
            let program = script
                .lift()
                .map_err(|error| CliError::Message(format!("{path}: {error}")))?;
            out.write_all(asm::dump(&program).as_bytes())?;
        }
        Mode::Listing => write_listing(&script, false, &mut out, path)?,
        Mode::Strings => write_listing(&script, true, &mut out, path)?,
    }
    out.flush()?;
    Ok(())
}

fn write_listing(
    script: &Script,
    strings_only: bool,
    out: &mut impl Write,
    path: &str,
) -> Result<(), CliError> {
    for instruction in script.instructions() {
        let string_op = matches!(instruction.opcode(), 0x03 | 0x7F);
        if strings_only {
            if string_op {
                let text = script
                    .string(instruction)
                    .map_err(|error| CliError::Message(format!("{path}: {error}")))?;
                writeln!(out, "\"{}\"", escape(text))?;
            }
            continue;
        }
        write_instruction(script, instruction, out, path)?;
    }
    Ok(())
}

fn write_instruction(
    script: &Script,
    instruction: &Instruction,
    out: &mut impl Write,
    path: &str,
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
        line.push_str(&format!(" 0x{operand:08x}"));
    }
    if matches!(instruction.opcode(), 0x03 | 0x7F) {
        let text = script
            .string(instruction)
            .map_err(|error| CliError::Message(format!("{path}: {error}")))?;
        line.push_str(&format!("  \"{}\"", escape(text)));
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
                out.push_str(&format!("\\u{{{:x}}}", control as u32));
            }
            other => out.push(other),
        }
    }
    out
}
