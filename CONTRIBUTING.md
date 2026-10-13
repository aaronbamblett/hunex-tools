# Contributing

Thank you for helping with `hunex`. This page covers building, testing, the environment variables that enable tests on real game files, the clean-room policy, and commit messages.

## Build

You need the Rust toolchain named in `rust-toolchain.toml`. The workspace uses edition 2021 and a minimum Rust version of 1.96.

```text
cargo build --workspace
cargo build --release -p hunex-cli
```

The release binary is `target/release/hunex`.

Library crates are `no_std` with `alloc`, and most have no dependencies. The exceptions are `hunex-nx`, which uses `bcdec_rs` and `miniz_oxide`, and `hunex-cli`, which uses `png`.

To check the library crates for WebAssembly, build each one for `wasm32-unknown-unknown` and `wasm32-wasip2`. The CI workflow in `.github/workflows/ci.yml` lists the crate groups.

## Test

Run the default tests. They need no game files.

```text
cargo test --workspace --no-fail-fast
```

Use `--no-fail-fast` so that one failing test binary does not hide the others.

Before a commit, run the same gates as CI:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Each crate enables `#![warn(missing_docs)]`, and the documentation gate turns warnings into errors, so every public item needs a doc comment.

### Tests that use real files

Some tests read real game files. Each one is skipped unless its environment variable is set. Set only the variables for the tests you want to run. Point each variable at a copy of the files, never at the game install. These copies are the sample files that the notes in [docs/formats](docs/formats/README.md) refer to. See [getting started](docs/guides/getting-started.md).

Run gated tests with the variables set in the same command:

```text
HUNEX_CHS_DIR=/path/to/chs HUNEX_WORK=/var/tmp/hunex-work cargo test -p hunex-chs --test real_files
```

| Variable | Points at | Tests that read it | Unset behavior |
|---|---|---|---|
| `HUNEX_CHS_DIR` | A folder of `.chs` script files. | `hunex-chs` `real_files`, `relocation`; `hunex-cli` `chs_asm_round_trip` | Skipped with a message. |
| `HUNEX_HFA_DIR` | A folder of `data*.hfa` archives (`data00100.hfa`, `data00300.hfa`, `data01000.hfa` to `data01400.hfa`). | `hunex-cli` `hfa_real`, `font_real`, `audio_real`, `lenzu_compress` | Skipped with a message. |
| `HUNEX_LENZU_DIR` | A folder of stored LenZu streams. | `hunex-cli` `lenzu_compress`, `lenzu_envelope` | Skipped with a message. |
| `HUNEX_CTD_DIR` | A folder with the four stored `.ctd` entries. | `hunex-ctd` `real_ctd`; `hunex-cli` `ctd_real` | Skipped with a message. |
| `HUNEX_CBG_DIR` | A folder of `.cbg` files. | `hunex-cbg` `samples` | Skipped with a message. |
| `HUNEX_MZP_DIR` | A folder of `.mzp` files. | `hunex-image` `samples` | Skipped with a message. |
| `HUNEX_IMG1227` | One `img1227` `.mzp` file, for the painted-rectangle checks. | `hunex-image` `samples` (two tests) | Those checks are skipped. |
| `HUNEX_NOPUS_DIR` | The Switch `stream` folder of Nintendo OPUS files. | `hunex-cli` `audio_real` | Skipped with a message. |
| `HUNEX_NOPUS_SURVEY` | Set to `1` to add a survey of every file in `HUNEX_NOPUS_DIR`. | `hunex-cli` `audio_real` | No survey. |
| `HUNEX_FFMPEG` | Set to `1` to decode the remuxed Ogg files with `ffmpeg`, which must be on `PATH`. | `hunex-cli` `audio_real` | Decode step is skipped. |
| `HUNEX_SWITCH_ROMFS_A` | The Switch RomFS folder of game set A. | `hunex-cli` `switch_real`, `switch_mrg_write`, `switch_mzx_envelope`, `switch_pack`, `switch_text_import`, `nopus_roundtrip` | Skipped with a message. |
| `HUNEX_SWITCH_ROMFS_B` | The Switch RomFS folder of game set B. | `hunex-cli` `switch_real` | Skipped with a message. |
| `HUNEX_SWITCH_DUMP_A` | A folder of decompressed script dumps for game set A. | `hunex-cli` `switch_real` | Skipped with a message. |
| `HUNEX_SWITCH_DUMP_B` | A folder of decompressed script dumps for game set B. | `hunex-cli` `switch_real`, `switch_mzx_envelope` | Skipped with a message. |
| `HUNEX_SWITCH_SET_B` | A folder with the set B Switch sets (`jallpac.hed`, `.mrg`, `.nam`). | `hunex-cli` `switch_mrg_write` | Skipped with a message. |
| `HUNEX_BC3_DIR` | A folder of MRG sets, scanned for BC3 textures. | `hunex-cli` `bc3_scan` | Skipped with a message. |
| `HUNEX_WORK` | A scratch folder for test output. Use a folder under `/var/tmp`. | `hunex-cli` `hfa_real`, `font_real`, `audio_real`, `ctd_real`, `switch_real`, `bc3_scan` | Checks that write output are skipped with a message. |
| `HUNEX_UPDATE_DOCS` | Set to `1` to rewrite `docs/formats/opcodes.md` from the opcode table. | `hunex-chs` `opcodes_doc` | The test fails if the file is out of date. |

## Clean-room policy

Formats are learned from the game files themselves and from facts: byte layouts, constants and counts. Nobody ports or translates another tool's code.

If you study another tool to learn a format, write down the facts first, then implement from those facts. Do not copy that tool's code or its structure.

Third-party code appears only as normal crate dependencies. Their licenses are recorded in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

Do not commit game data, dumps, keys or extracted assets. Test fixtures are synthetic bytes written inside the test.

The format notes in `docs/formats/` label every claim as checked on sample files, verified in game, or open. When you change a format, update its note and its labels. The labels are defined in [docs/formats/README.md](docs/formats/README.md).

## Commit style

- Subject line in the imperative mood, sentence case, no full stop, about 72 characters or fewer.
- Name the crate when the change is confined to one, for example `HFA: keep repeated names`.
- Describe the reason in the body when the subject does not show it.
- Do not add assistant attribution or review notes to the message.
