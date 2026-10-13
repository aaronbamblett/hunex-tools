# hunex-chs

A Rust decoder for HuneX compiled script (`.chs`) files, as used by the Steam release of *Witch on the Holy Night*. It reads a script and returns its header facts, the instruction list (file offset, opcode id, inline operands), and the string pool. The library has no dependencies and builds as `no_std`, so it also compiles to `wasm32-unknown-unknown` and `wasm32-wasip2`. The `chs-dump` command prints a script as an instruction list.

The project does not include any game files. Supply your own copy of the scripts from a game you own.

## Build and test

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p hunex-chs --target wasm32-unknown-unknown
cargo build -p hunex-chs --target wasm32-wasip2
```

The unit tests use synthetic bytes. The real-file checks in `crates/hunex-chs/tests/real_files.rs` run only when `HUNEX_CHS_DIR` points at a directory holding 250 `.chs` files. Without it they print a note and skip:

```sh
HUNEX_CHS_DIR=/path/to/decompressed/chs cargo test --workspace -- --nocapture
```

## Command line

```sh
cargo run -p hunex-chs-cli -- path/to/script.chs
cargo run -p hunex-chs-cli -- --strings path/to/script.chs
```

`--roundtrip` re-encodes the parsed script and prints `identical`, or the first differing offset and exits non-zero.

Each instruction prints as offset, opcode id, mnemonic (where known), and operands. Opcodes 0x03 and 0x7F also print their resolved string. `--strings` prints only the resolved strings, in code order.

## Known limit

The code section has no explicit length in the file. The string pool is taken to start at the smallest string pointer, and the walk must end on a return exactly there. A crafted file whose smallest pointer targets a return in the middle of the code is accepted, and the code after that return is treated as pool. Real files cannot be told apart from that case, and the exe has no explicit code length either.

## Lift, lower and text form

`Script::lift()` turns a parsed script into a `Program`: typed items (`Dword`, `Label` for 0x01 code pointers, `Str` for the string operand of 0x03 and 0x7F), labels on instruction positions, and a string table. `Program::lower()` recomputes every code pointer and string pointer and returns a `Script`, which `encode()` turns into bytes. Editing: `insert`, `remove`, `add_label`, `move_label`, `add_string`, `set_string`. Removing an instruction that has a label on it is an error; move the label first.

Pool rule: lifted strings keep the file's order, including entries no code references. On lowering, the pool starts at the first original string that code references, because the decoder takes the smallest string pointer as the pool start. Original strings follow in their original order. New or changed strings come after them, in table order, if code references them. Unreferenced new strings are dropped. Tail bytes after the last NUL are written last. Unchanged programs lower byte-identically (250 of 250 files).

Text form: `chs-dump --asm file.chs` prints it, and `chs-asm in.txt -o out.chs` assembles it. Directives: `format hunex|buriko`, `header_size N`, `include "name"`, `header_padding HEX...`, `tail "text"`, `string S_0000 "text"` (original pool entry, in pool order) and `string new S_0000 "text"` (new). Labels are `L_0000:`. Instructions take a mnemonic or a hex id (`0x0f7`), with operands as numbers, `L_` labels, `S_` strings, or quoted strings (which become new strings). Quoted text escapes `\n \r \t \\ \"` and `\xNN`; valid UTF-8 is written as is.

Known risk: a pointer computed at runtime, or stored in a plain 0x00 dword, is not a label or string operand, so lowering does not move it. The 250 files show none detected, but that is not proven until a relocated script runs in the game.
