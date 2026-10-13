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
