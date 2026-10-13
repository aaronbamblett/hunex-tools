use std::process::ExitCode;

use hunex_chs_cli::asm;

const USAGE: &str = "usage: chs-asm <in.txt> -o <out.chs>";

fn main() -> ExitCode {
    let mut input = None;
    let mut output = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-o" => output = arguments.next(),
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') || input.is_some() => {
                eprintln!("chs-asm: unexpected argument {other}");
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            other => input = Some(other.to_string()),
        }
    }
    let (Some(input), Some(output)) = (input, output) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };

    let text = match std::fs::read(&input) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("chs-asm: {input}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let bytes = match asm::build(&text) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("chs-asm: {input}: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = std::fs::write(&output, &bytes) {
        eprintln!("chs-asm: {output}: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
