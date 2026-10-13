# hunex-tools

Tools for file formats of the HuneX engine. The first format is `.chs` compiled scripts, as used by the Steam release of *Witch on the Holy Night*. The project will add more formats as sibling crates.

- `hunex-chs` (library): decodes `.chs` scripts into instructions, strings and header facts; re-encodes them; converts to and from a symbolic form with labels and strings. It is `no_std`, has no dependencies, and builds for WebAssembly.
- `hunex-cli` (command `hunex`): lists, round-trips and assembles `.chs` files.

This is an unofficial fan project. It is not affiliated with HuneX or TYPE-MOON. No game data is included: you supply your own files from a copy of a game you own.

## Status

- Exact round-trip (decode, then encode) verified on all 250 Mahoyo scenes.
- Lift, lower and text dump/assemble round-trips verified on all 250 scenes.
- Relocation (inserting instructions and strings, then re-encoding) verified on the sample files in the test suite.
- Not yet verified inside the game. Runtime-computed pointers are not relocated (see below).

## Install

From a release: download the zip for your platform from the GitHub releases page, check it against `SHA256SUMS`, and unzip. The zip contains `hunex` (`hunex.exe` on Windows), this README and the licence.

From source, with Rust 1.96 or newer:

```sh
cargo install --path crates/hunex-cli
```

## Command line

```text
hunex chs dump <file.chs>             print the instruction list
hunex chs dump --strings <file.chs>   print only the string operands
hunex chs dump --asm <file.chs>       print the text form (re-assemblable)
hunex chs roundtrip <file.chs>        re-encode and compare with the file
hunex chs asm <in.txt> -o <out.chs>   assemble the text form into a script
hunex --version
hunex help
```

The text form is described in [docs/asm-format.md](docs/asm-format.md). The `.chs` format as implemented is described in [docs/chs-format.md](docs/chs-format.md). The opcode table is in [docs/opcodes.md](docs/opcodes.md).

## Library

```rust
use hunex_chs::Script;

let bytes = std::fs::read("script.chs")?;
let script = Script::parse(&bytes)?;
assert_eq!(script.encode(), bytes);

let program = script.lift()?;
let edited = program.lower()?.encode();
```

## Tests

```sh
cargo test --workspace
```

Tests that read real files are skipped unless `HUNEX_CHS_DIR` points at a directory with `.chs` files:

```sh
HUNEX_CHS_DIR=/path/to/chs-files cargo test --workspace -- --nocapture
```

Those tests expect 250 files. The test output prints the round-trip, lift/lower, asm and relocation counts.

## Known limits

- The code section has no explicit length. The string pool starts at the smallest string pointer, so a crafted file can make code look like pool data. Real files cannot be told apart from this case.
- Pointers computed at runtime, or stored in plain 0x00 dwords, are not recognised and are not moved when a script is lowered.

## Licence

MIT. See [LICENSE](LICENSE).
