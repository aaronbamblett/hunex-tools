//! The `hunex` command. Each engine format is a module with its own commands.

#![warn(missing_docs)]

/// Writes a line to standard output. See [`out_write`].
#[macro_export]
#[doc(hidden)]
macro_rules! outln {
    ($($arg:tt)*) => {
        $crate::out_write(format_args!("{}\n", format_args!($($arg)*)))
    };
}

/// Writes to standard output. See [`out_write`].
#[macro_export]
#[doc(hidden)]
macro_rules! out {
    ($($arg:tt)*) => {
        $crate::out_write(format_args!($($arg)*))
    };
}

/// Writes formatted text to standard output. A closed pipe, such as `hunex ... | head`, ends the
/// command with status 0; any other write error is reported and ends it with status 1.
#[doc(hidden)]
pub fn out_write(args: core::fmt::Arguments<'_>) {
    use std::io::Write;
    if let Err(error) = std::io::stdout().lock().write_fmt(args) {
        if error.kind() == std::io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("hunex: {error}");
        std::process::exit(1);
    }
}
pub mod cbg;
pub mod chs;
pub mod cli;
pub mod ctd;
pub mod font;
pub mod hfa;
pub mod hw;
pub mod image;
pub mod lenzu;
pub mod mrg;
pub mod mzp;
pub mod mzx;
pub mod nopus;
pub mod nx;
pub mod switch_script;
pub mod switch_text;
