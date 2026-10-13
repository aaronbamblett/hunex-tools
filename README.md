# hunex-tools

Tools for file formats of the HuneX engine, as used by the Steam release of *Witch on the Holy Night*. The project has four crates:

- `hunex-chs`: decodes `.chs` compiled scripts into instructions, strings and header facts; re-encodes them exactly; converts to and from a symbolic form with labels and strings.
- `hunex-hfa`: reads and writes `.hfa` script archives.
- `hunex-lenzu`: decodes LenZu compressed streams, and writes literal-only streams.
- `hunex-cli`: the `hunex` command that ties them together.

The library crates are `no_std`, have no dependencies, and build for WebAssembly.

This is an unofficial fan project. It is not affiliated with HuneX or TYPE-MOON. No game data is included: you supply your own files from a copy of a game you own.

## Status

- `.chs` scripts: exact decode and re-encode verified on all 250 Mahoyo scenes. Lift, lower and the text form also round-trip all 250.
- Editing: insertion of instructions and a new string, then re-encoding, verified on all 250 scenes in the test suite.
- HFA archives: all five archives rewrite byte for byte, and unpack then pack reproduces them.
- LenZu: all 250 entries decompress to the reference scripts. The literal-only encoder round-trips through the decoder, but the stream is larger than its input and it is not yet confirmed that the game accepts it.
- Not verified inside the game. Edited scripts and archives have not been run in the game.

## Install

From a release: download the zip for your platform from the GitHub releases page, check it against `SHA256SUMS`, and unzip. The zip holds `hunex` (`hunex.exe` on Windows), this README and the licence.

From source, with Rust 1.96 or newer:

```sh
cargo install --path crates/hunex-cli
```

## From install to an edited script and back

These steps use generic names. Replace `archive.hfa` with an archive from your copy of the game.

```sh
hunex hfa list archive.hfa                              # see the entries
hunex hfa unpack archive.hfa -o work --decompress       # files in work/, plus hfa-manifest.txt
hunex chs dump --asm work/script.chs > script.txt       # the script as editable text
# edit script.txt: change the text of a line such as: string S_0003 "some text"
hunex chs asm script.txt -o work/script.chs             # assemble the edited script
hunex hfa pack work -o edited.hfa --base archive.hfa    # unchanged entries keep their bytes
```

`hfa pack --base` reuses the base archive's stored bytes for every entry whose content did not change, so only the edited entry differs. An edited entry is stored as a literal-only LenZu stream, which is larger than the original.

## Command line

```text
hunex chs dump <file.chs>             print the instruction list
hunex chs dump --strings <file.chs>   print only the string operands
hunex chs dump --asm <file.chs>       print the text form (re-assemblable)
hunex chs roundtrip <file.chs>        re-encode and compare with the file
hunex chs asm <in.txt> -o <out.chs>   assemble the text form into a script
hunex hfa list <archive.hfa>
hunex hfa unpack <archive.hfa> -o <dir> [--decompress]
hunex hfa pack <dir> -o <archive.hfa> [--base <original.hfa>]
hunex lenzu decompress <in> -o <out>
hunex lenzu compress <in> -o <out>    literal only
hunex --version
hunex help
```

Format documentation: [chs-format.md](docs/chs-format.md), [asm-format.md](docs/asm-format.md), [opcodes.md](docs/opcodes.md), [hfa-format.md](docs/hfa-format.md), [lenzu-format.md](docs/lenzu-format.md).

## Library

```rust
use hunex_chs::Script;

let bytes = std::fs::read("script.chs")?;
let script = Script::parse(&bytes)?;
assert_eq!(script.encode(), bytes);

let program = script.lift()?;
let edited = program.lower()?.encode();
```

HFA and LenZu have the same shape: `hunex_hfa::Archive::parse` and `Archive::write`, and `hunex_lenzu::decompress` and `compress_literal`.

## Tests

```sh
cargo test --workspace
```

Some tests read real files. They are skipped unless the variables are set:

- `HUNEX_CHS_DIR`: a folder of decompressed `.chs` files (250 expected).
- `HUNEX_HFA_DIR`: the folder with the five `.hfa` archives. The stored entries are read from the sibling `../chs` folder, and the decompressed references from `../raw`.

```sh
HUNEX_CHS_DIR=/path/to/raw HUNEX_HFA_DIR=/path/to/hfa cargo test --workspace -- --nocapture
```

The end-to-end test uses `HUNEX_WORK` as its scratch folder when set, and removes that folder when it finishes.

## Known limits

- The `.chs` code section has no explicit length. The string pool starts at the smallest string pointer, so a crafted file can make code look like pool data. Real files cannot be told apart from this case.
- Pointers computed at runtime, or stored in plain 0x00 dwords, are not recognised and are not moved when a script is lowered.
- The HFA reader accepts only the packed layout the originals use.

## Licence

MIT. See [LICENSE](LICENSE).
