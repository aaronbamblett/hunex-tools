# hunex

`hunex` is a command-line tool and a set of Rust library crates for the file formats of the HuneX engine, as used in games such as *Tsukihime -A piece of blue glass moon-* and *Witch on the Holy Night* (Mahoyo). You use `hunex` to read, edit and rebuild the files in your own copies of those games.

This is an unofficial fan project. It is not affiliated with HuneX.

## What it is and is not

`hunex` is:

- a command-line tool, `hunex`, that lists, converts, edits and rebuilds game files;
- a set of library crates that read and write the same formats. Most have no dependencies, and CI builds the library crates for WebAssembly.

`hunex` is not:

- a game, a patch, or a translation. It does not run the game.
- a source of game data. The repository contains no game files, dumps, keys or extracted assets. You supply the files from a copy of a game you own.

Work on a copy of the game's files, and do not edit the game's folder. The [guides](docs/guides/README.md) show each task step by step.

## Format status

- **Read** means the crate parses the file.
- **Write** means the crate writes files that its own reader accepts and that match the layout of the sample files.
- **Verified in game** means the result was seen in the Steam release of Witch on the Holy Night on 2026-10-13.
- **Checked on sample files** means the result comes from running the crate on the sample files.
- **Open** means the behavior is not yet tested in game.

Each row links to its format note, which gives the details and the open questions.

| Format | Crate | Read | Write | Result |
|---|---|---|---|---|
| [Compiled scripts (`.chs`)](docs/formats/chs-format.md) | `hunex-chs` | Yes | Yes, with pointer relocation | Verified in game: edited scripts, with moved pointers and changed strings, play through. |
| [Script archives (HFA)](docs/formats/hfa-format.md) | `hunex-hfa` | Yes | Yes | Verified in game: five archives with changed entry sizes and offsets, rebuilt with `pack --base`, load. |
| [LenZu streams](docs/formats/lenzu-format.md) | `hunex-lenzu` | Yes | Yes | Verified in game: literal-only and compressed output both load. |
| [Script text (`.ctd`)](docs/formats/ctd-format.md) | `hunex-ctd` | Yes | Yes | Verified in game: an edited line of a different length displays, with a literal-only entry and a compressed one. |
| [Background images (MZP, MZX0, HEP)](docs/formats/mzp-image-format.md) | `hunex-mzp`, `hunex-mzx`, `hunex-image` | Yes | Yes | Verified in game: a repainted static background displays cleanly. |
| [Glyph tables and font atlases (`.ccit`)](docs/formats/ccit-format.md) | `hunex-ccit` | Yes | Yes | Verified in game: an edited atlas glyph displays and accented glyphs render. Open: edited glyph tables. |
| [Audio (`.hw`, Ogg Vorbis)](docs/formats/hw-format.md) | `hunex-hw` | Yes | Yes | Verified in game: a converted Switch voice clip plays in place of a stock sound. |
| [CompressedBG images (`.cbg`)](docs/formats/cbg-format.md) | `hunex-cbg` | Yes | Yes | Verified in game: a repainted title logo displays, alpha included. |
| [Nintendo OPUS audio (Switch)](docs/formats/nintendo-opus-format.md) | `hunex-nopus` | Yes | Ogg Opus remux | Verified in game indirectly, through the `.hw` test. |
| [Switch archive sets](docs/formats/switch-archives.md) | `hunex-mrg` | Yes | No | Checked on sample files. Not applicable in game. |
| [Switch textures (NXX, BNTX)](docs/formats/switch-textures.md) | `hunex-nx` | Yes, decode only | No | Checked on sample files. Not applicable in game. |
| [Switch script text](docs/formats/switch-script-text.md) | `hunex-switch-text` | Yes | No | Checked on sample files. Not applicable in game. |

## Install

From a release: download the zip for your platform from the [releases page](https://github.com/aaronbamblett/hunex-tools/releases). Check it against `SHA256SUMS`, which is on the same page. On Linux, run `sha256sum -c SHA256SUMS --ignore-missing` in the folder with the zip. The zip holds `hunex` (`hunex.exe` on Windows), `README.md`, `LICENSE` and `THIRD_PARTY_LICENSES.md`. Unzip it, and put the `hunex` file on your `PATH`.

From source, with Rust 1.96 or newer:

```sh
git clone https://github.com/aaronbamblett/hunex-tools
cd hunex-tools
cargo install --path crates/hunex-cli
```

Cargo puts `hunex` in its bin folder, which is `~/.cargo/bin` by default. Add that folder to your `PATH` if the command is not found.

## Quickstart

These commands use placeholder names. Replace them with files from your own copy of the game.

```sh
hunex --version
hunex hfa list archive.hfa                           # list the entries
hunex hfa unpack archive.hfa -o work --decompress    # write them to work/
hunex chs dump --asm work/script.chs > script.txt    # the script as text
# edit script.txt, for example the text of one string line
hunex chs asm script.txt -o work/script.chs          # assemble it again
hunex hfa pack work -o edited.hfa --base archive.hfa # rebuild the archive
```

`hfa pack --base` keeps the stored bytes of each entry that did not change, so only the edited entry differs from the original.

## Documentation

- [Guides](docs/guides/README.md): one task each, with the commands and their output.
- [Command reference](docs/cli.md): every command and option.
- [Format notes](docs/formats/README.md): the layout and rules of each file format, with the open questions.
- [CONTRIBUTING.md](CONTRIBUTING.md): building, testing, the environment variables for tests on real files, and the clean-room policy.
- [CHANGELOG.md](CHANGELOG.md): what changed in each release.
- [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md): third-party code in the binary.

## License

MIT. See [LICENSE](LICENSE).
