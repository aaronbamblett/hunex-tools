# Changelog

## Unreleased

- Fix ordered comparison opcodes (lt/le/gt/ge were mapped to the wrong Steam handlers)
- `hunex chs compile`: compile Switch script text to a Steam `.chs` file. It holds a parser for `_NAME(arguments);` statements, the engine recipe table for 92 commands, and a lowering with labels, conditionals, jumps and messages. It emits no Debug records and interns strings in first-use order.
- `hunex-chs` APIs: `source` parses Switch script text, `normalize` reduces a program to the Debug-free form used for comparison, `compile` lowers statements with the recipe table, and `compare` classifies a scene as exact, shape or authored.
- Verified in game: a Mahoyo scene compiled from Switch text plays through in the Steam release of Witch on the Holy Night.
- `hunex chs compile`: `_SBAS(child,parent)` lowers to opcode 0x25C, which only a patched executable registers. 0x25C is now in the opcode table.
- `hunex chs compile`: `_TOMP(n)` with one integer lowers to opcode 0x1C2 (play movie by index), which only a patched executable registers; `_TOMP()` still lowers to 0x1C0. 0x1C2 is now in the opcode table.
- `hunex chs compile`: `_FCSP(duration)` lowers to opcode 0x25E, which only a patched executable registers. 0x25E is now in the opcode table.
- Corpus counts, cumulative over 249 Mahoyo scenes: 162 match exactly, 216 including retuned literals (shape), 242 including reviewed release edits (authored).

## 0.1.0

First release. It contains:

- hunex-chs: decoder for `.chs` compiled scripts (Hunex and Buriko headers), exact re-encoding, lifting to a symbolic form with labels and strings, lowering, and an editing API. The third operand of 0x7B is a string pointer, relocated by lifting and lowering.
- hunex-lenzu: LenZu decoder with checksum verification, a compressor (greedy LZ77 with a Huffman table, limited to the envelope measured on the original streams), and a literal-only mode.
- hunex-hfa: reader and writer for HFA script archives in the packed layout; the table can be read without the data. Repeated names are kept.
- hunex-cli hfa: `unpack` gives repeated or reserved names a numbered file name, and the manifest records each entry's file and archive name, so `pack` restores the original names. `data00000.hfa` unpacks and packs back byte for byte.
- hunex-ctd: reader and writer for script text (`.ctd`) bodies, with line lookup by id, and export and import one line per id.
- hunex-ogg: Ogg page reader and writer with CRC checks.
- hunex-hw: reader and writer for `.hw` audio (64-byte header and Ogg Vorbis payload).
- hunex-nopus: reader for Nintendo OPUS audio, and a remux to Ogg Opus (RFC 7845) that copies the packets unchanged.
- Switch writers: the MRG set writer (index, names and layout), the Switch MZX0 encoder, the `allscr.mrg` writer (`switch-script pack`), the script text writer (`switch-text import`), and the Nintendo OPUS writer (`nopus from-ogg`). Each one is checked on the sample files.
- hunex-mzx: MZX0 decoder and encoder for the PC and Switch variants. The PC decoder resets its repeat value at each 4096-word window; the encoder writes literals and back-references only.
- hunex-mzp: MZP container reader and writer, with the PC and Switch size rules and 0xFF padding.
- hunex-image: background images. HEP palette tiles abut at full size into an RGBA canvas, and encode back to MZP. `from-png --like` keeps unchanged tiles byte for byte and re-encodes changed ones. Font atlases (bitmap type 0x01) use a swizzled palette order.
- hunex-cbg: CompressedBG images (`.cbg`). Decodes all 537 sample files to RGBA. The canonical encoder reproduces each sample byte for byte, and a simple encoding with literal runs and equal weights is also available. The `cbg info`, `cbg to-png` and `cbg from-png` commands (with `--like` and `--simple`) use it.
- hunex-ccit: glyph tables (`.ccit`) with both record forms, lookup by character, and insertion and replacement that keep the codes sorted and the indices dense.
- hunex-mrg: MRG archive sets (index, names, entries read by seek).
- hunex-nx: NXX wrappers (zlib and gzip), BNTX parsing, block-linear deswizzle, and BC7, BC4, BC3 and RGBA8 decoding. BC3 takes color from the color block and alpha from the alpha block, as the standard format does. Checked on two masks.
- hunex-switch-text: `script_text.mrg` offset tables and `$` line lookup.
- hunex-cli (`hunex`): commands for chs, hfa, lenzu, ctd, hw, nopus, mzp, image, font, mzx, mrg, nx, switch-text and switch-script. See the [README](README.md) and [docs/cli.md](docs/cli.md).
- Format notes in `docs/formats/`, and `THIRD_PARTY_LICENSES.md`, which is included in the release zips.
