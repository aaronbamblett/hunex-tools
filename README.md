# hunex-tools

Tools for file formats of the HuneX engine, as used by the Steam release of *Witch on the Holy Night*. The project has these crates:

- `hunex-chs`: decodes `.chs` compiled scripts into instructions, strings and header facts; re-encodes them exactly; converts to and from a symbolic form with labels and strings.
- `hunex-hfa`: reads and writes `.hfa` script archives.
- `hunex-lenzu`: decodes LenZu compressed streams, and writes literal-only streams.
- `hunex-ctd`: reads and writes the script text (`.ctd`) files, one line per script id.
- `hunex-hw`: reads and writes `.hw` audio files (a 64-byte header and an Ogg Vorbis stream).
- `hunex-ccit`: reads and writes the glyph tables (`.ccit`) of the fonts, and looks up a glyph by character.
- `hunex-nopus`: reads the Switch's Nintendo OPUS audio and remuxes it to Ogg Opus.
- `hunex-ogg`: shared Ogg page reader and writer with CRC checks.
- `hunex-mzx`, `hunex-mzp`, `hunex-image`: background images. MZX0 tile streams, the MZP container, and the RGBA canvas assembled from palette tiles, with an encoder back to MZP.
- `hunex-mrg`, `hunex-nx`, `hunex-switch-text`: the Switch RomFS. MRG archive sets, NXX-wrapped BNTX textures (BC4, BC7 and RGBA8), and the script text of `script_text.mrg`.
- `hunex-cli`: the `hunex` command that ties them together.

The library crates are `no_std`, have no dependencies, and build for WebAssembly.

This is an unofficial fan project. It is not affiliated with HuneX or TYPE-MOON. No game data is included: you supply your own files from a copy of a game you own.

## Status

- `.chs` scripts: exact decode and re-encode verified on all 250 Mahoyo scenes. Lift, lower and the text form also round-trip all 250.
- Editing: insertion of instructions and a new string, then re-encoding, verified on all 250 scenes in the test suite.
- HFA archives: all five archives rewrite byte for byte, and unpack then pack reproduces them.
- LenZu: all 250 entries decompress to the reference scripts. The literal-only encoder round-trips through the decoder and the game accepts its streams, but the stream is larger than its input and does not shrink data.
- In the game: on 2026-10-13 one edited script was assembled, packed with `hunex hfa pack --base` as a literal-only LenZu entry, and then loaded and played in the Steam release. The script had one inserted instruction at the start, which moves every code and string pointer, and a changed image string. This confirms that literal-only LenZu streams are accepted, that archives with changed entry sizes and offsets are accepted, and that relocation works for that script. It is one script; runtime-computed pointers remain a known limit.

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
hunex ctd export <file.ctd> -o <lines.txt>   one line per script id: $000000, TAB, text
hunex font info <x.ccit>                     counts, code range, pages, coordinate form
hunex font export <x.ccit> -o <glyphs.tsv>   one row per glyph: char, code, index, page, x, y, w, h
hunex font import <glyphs.tsv> -o <x.ccit>   rows sorted by code; indices must be 0 to N-1
hunex font render <x.ccit> --atlas <page0.mzp> [--atlas <page1.mzp>] --text "..." -o <out.png>
hunex ctd import <lines.txt> -o <file.ctd>   ids must run in order from $000000
hunex ctd get <file.ctd> <$id>...
hunex hw info <file.hw>
hunex hw unwrap <file.hw> -o <out.ogg>
hunex hw wrap <in.ogg> -o <out.hw>
hunex nopus info <in.opus>
hunex nopus to-ogg <in.opus> -o <out.ogg>
hunex mzp list <file.mzp>
hunex mzp unpack <file.mzp> -o <dir>
hunex mzp pack <dir> -o <file.mzp>
hunex image to-png <in.mzp> -o <out.png>
hunex image from-png <in.png> -o <out.mzp> [--like <original.mzp>]
hunex mzx decompress <in> -o <out>
hunex mzx compress <in> -o <out>
hunex --version
hunex help
```

## Fonts

A `.ccit` table maps characters to glyph indices, and its coordinates place each glyph in an atlas page (`.mzp`). `hunex font` reads the tables, exports and imports them as TSV, and renders text from the atlas pages. `hunex image to-png` and `from-png --like` also read and write the font atlases (bitmap type 0x01).

```sh
hunex lenzu decompress Font010000.ccit -o Font010000.payload     # or pass the stored entry
hunex font info Font010000.payload
hunex font export Font010000.payload -o font.tsv
hunex font import font.tsv -o Font010000.new.payload
hunex font render Font010000.payload --atlas FONT_en_H00.mzp --text "Tsukihime 123" -o line.png
```

Glyphs are drawn from their index cell, and the stored x is not used for drawing. For 8 page-1 records in the 106-glyph tables, the stored x lands on other glyphs. The stored width is the pen advance, which is not verified in the game. Atlas coverage is the palette alpha, read through the atlas palette's swizzled order.

The tables rebuild byte for byte, and the atlas round trips are exact. Whether the game reads an edited table or atlas has not been tested. Format notes: [ccit-format.md](docs/ccit-format.md).

## Switch RomFS

The Switch RomFS holds the same kinds of data as the Steam release, in other containers. `hunex` reads the Tsukihime and Mahoyo Switch dumps without other tools:

```sh
hunex mrg list romfs/w_allpac                       # entries of a set, by base name
hunex mrg unpack romfs/w_allpac -o work/w_allpac    # every entry, named from the .nam file
hunex nx info work/w_allpac/IMG5012.NXGZ            # textures of an NXX entry (or a BNTX file)
hunex nx to-png work/w_allpac/IMG5012.NXGZ -o img.png
hunex switch-script dump romfs/allscr.mrg -o work/scripts   # scr0000.txt onward, plus nam.bin
hunex switch-text export romfs/script_text.mrg -o lines.txt --pair 1   # $000000 TAB text
```

Textures are decoded from BC7, BC4 and RGBA8. BC3 textures (format 0x1C06, two in Mahoyo) are reported as unsupported, because the channel reading is not settled. Switch script streams use a variant of the MZX0 codec; `hunex-mzx` reads both.

Bulk commands read one entry at a time with a seek, so memory stays small. Measured on the largest set (`w_allpac`, 29,267 entries, 16.75 GB data file) with `/usr/bin/time -v`: `mrg list` peaks at about 9 MB resident, and `mrg unpack` writes all of it with a peak of 44 MB, in about a minute.

The formats are described in [switch-archives.md](docs/switch-archives.md), [switch-textures.md](docs/switch-textures.md) and [switch-script-text.md](docs/switch-script-text.md).

Third-party code in the binary is listed in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## Background images

Each background is an MZP archive: a header entry with the canvas and tile grid, then one MZX0-compressed palette tile per grid cell. `hunex image to-png` assembles the tiles into one RGBA PNG, and `hunex image from-png` writes a new archive from a PNG.

```sh
hunex image to-png bg.mzp -o bg.png                       # the full background as PNG
# repaint bg.png in an image editor, keeping its size
hunex image from-png bg.png -o bg-new.mzp --like bg.mzp   # keeps the original canvas and tile grid
```

With `--like`, the PNG must be the same size as the original image, and the encoder starts from the original file. Pixels you did not change keep the original tile bytes, so untouched tiles are copied byte for byte, and only the tiles with changed pixels are re-encoded. A changed tile with more than 256 colours is quantised to 256 by a median cut, which is lossy; the tool prints the error of each re-encoded tile. An unmodified PNG gives back the original file for all 52 samples.

Tiles abut at their full size, and the image is the canvas. Without `--like`, the image is padded with transparent pixels to whole 512 by 256 tiles; a tile with more than 256 colours is quantised to 256 by a median cut. The sample `img1155` round-trips exactly.

The PNG is read and written with the `png` crate, which only the command-line tool uses. The libraries do not depend on it.

The rebuilt archives are not yet tested in the game, and the greedy MZX encoder produces streams of a different size from the originals. The docs list what is not known: see [mzp-image-format.md](docs/mzp-image-format.md).

Format documentation: [chs-format.md](docs/chs-format.md), [asm-format.md](docs/asm-format.md), [opcodes.md](docs/opcodes.md), [hfa-format.md](docs/hfa-format.md), [lenzu-format.md](docs/lenzu-format.md), [ctd-format.md](docs/ctd-format.md), [hw-format.md](docs/hw-format.md), [nintendo-opus-format.md](docs/nintendo-opus-format.md), [ccit-format.md](docs/ccit-format.md).

## Audio

The game's `.hw` files are Ogg Vorbis inside a 64-byte header. The engine's strings point to Vorbis; they show no sign of Opus support, so Opus-in-Ogg is not used.

Unwrap and wrap a `.hw` file:

```sh
hunex hw unwrap sound.hw -o sound.ogg     # the Ogg Vorbis stream inside
hunex hw wrap sound.ogg -o sound.hw       # the input must be Ogg Vorbis
hunex hw info sound.hw                    # sample rate, channels, length
```

The Switch files are Nintendo OPUS. The path to a `.hw` file runs through two steps that `hunex` does not do:

```sh
hunex nopus to-ogg clip.opus -o clip.ogg                             # remux to Ogg Opus, packets unchanged
ffmpeg -i clip.ogg -ar 48000 -ac 2 -c:a libvorbis clip-vorbis.ogg    # external tool, not bundled
hunex hw wrap clip-vorbis.ogg -o clip.hw
```

`ffmpeg` is not part of this project; install it separately. Decoding the Switch audio uses ffmpeg here, and its output has not been compared with the reference decoder.

The paths above have not been run in the game. A rebuilt `.hw` file has not been tested in the game.

## Changing script text

The script text is in the `data00200` archive. Export a language's lines, edit them, and pack them back. Each line is `$000000`, a TAB, and the text; keep the ids in order and do not add or remove lines.

```sh
hunex hfa unpack data00200.hfa -o text --decompress      # script_text_*.ctd files in text/
hunex ctd export text/script_text_en.ctd -o lines.txt
# edit lines.txt: change the text after the TAB on a line, for example $000000
hunex ctd import lines.txt -o text/script_text_en.ctd
hunex hfa pack text -o data00200.edited.hfa --base data00200.hfa
```

Whether the game accepts text whose lines are longer or shorter than the originals has not been tested. Use `hunex ctd get <file.ctd> <$id>...` to print the current text for an id.

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
- `HUNEX_HFA_DIR`: the folder with the `.hfa` archives (`data01000` to `data01400`, and `data00200`). The stored entries are read from the sibling `../chs` folder, and the decompressed references from `../raw`.
- `HUNEX_CTD_DIR`: the `chs` folder with the four stored `.ctd` entries.
- `HUNEX_TSUKI_ROMFS`, `HUNEX_MAHOYO_ROMFS`: the Switch RomFS folders. `HUNEX_TSUKI_DUMP` and `HUNEX_MAHOYO_DUMP`: the existing script dumps they are checked against. The gated Switch tests run in release mode; set `HUNEX_WORK` to a folder outside `/tmp`.
- `HUNEX_HFA_DIR` also enables the glyph table checks. They read `data00100.hfa` (the 16 `.ccit` tables and the English atlases) from that folder.
- `HUNEX_WORK` with `HUNEX_HFA_DIR`: the font render and export checks write their files there, and are skipped without it.
- `HUNEX_MZP_DIR`: a folder of `.mzp` image archives. The gated image tests decode every file in it, check the two named samples (`img0408`, `img1155`) against their hashes, and report compression sizes.
- `HUNEX_NOPUS_DIR`: the Switch `stream` folder with `.opus` files. `HUNEX_NOPUS_SURVEY=1` adds a survey of every file in it (slow in debug builds; use `--release`). `HUNEX_FFMPEG=1` decodes the remuxed samples with ffmpeg, which must be on PATH.

```sh
HUNEX_CHS_DIR=/path/to/raw HUNEX_HFA_DIR=/path/to/hfa cargo test --workspace -- --nocapture
```

The end-to-end test uses `HUNEX_WORK` as its scratch folder when set, and removes that folder when it finishes.

## Known limits

- The `.chs` code section has no explicit length. The string pool starts at the smallest string pointer, so a crafted file can make code look like pool data. Real files cannot be told apart from this case.
- Pointers computed at runtime, or stored in plain 0x00 dwords, are not recognised and are not moved when a script is lowered. The third operand of 0x7B is handled as a string pointer, as are the first operands of 0x03 and 0x7F.
- The HFA reader accepts only the packed layout the originals use.
- Font edits are not tested in the game beyond the atlas palette order. The renderer's advance uses the stored width.

## Licence

MIT. See [LICENSE](LICENSE).
