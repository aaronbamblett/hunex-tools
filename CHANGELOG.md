# Changelog

## 0.1.0

First release. It contains:

- hunex-chs: decoder for `.chs` compiled scripts (Hunex and Buriko headers), exact re-encoding, lifting to a symbolic form with labels and strings, lowering, and an editing API. The third operand of 0x7B is a string pointer, relocated by lifting and lowering.
- hunex-lenzu: LenZu decoder with checksum verification, and a literal-only encoder.
- hunex-hfa: reader and writer for HFA script archives in the packed layout; the table can be read without the data.
- hunex-ctd: reader and writer for script text (`.ctd`) bodies, with line lookup by id, and export and import one line per id.
- hunex-ogg: Ogg page reader and writer with CRC checks.
- hunex-hw: reader and writer for `.hw` audio (64-byte header and Ogg Vorbis payload).
- hunex-nopus: reader for Nintendo OPUS audio, and a remux to Ogg Opus (RFC 7845) that copies the packets unchanged.
- hunex-mzx: MZX0 decoder and encoder for the PC and Switch variants. The PC decoder applies the 4096-word window reset; the encoder writes back-references and literals only.
- hunex-mzp: MZP container reader and writer, with the PC and Switch size rules and 0xFF padding.
- hunex-image: background images. HEP palette tiles abut at full size into an RGBA canvas, and encode back to MZP. `from-png --like` keeps unchanged tiles byte for byte and re-encodes changed ones. Font atlases (bitmap type 0x01) use a swizzled palette order.
- hunex-ccit: glyph tables (`.ccit`) with both record forms, lookup by character, and insertion and replacement that keep the codes sorted and the indices dense.
- hunex-mrg: MRG archive sets (index, names, entries read by seek).
- hunex-nx: NXX wrappers (zlib and gzip), BNTX parsing, block-linear deswizzle, and BC7, BC4 and RGBA8 decoding. BC3 is reported as unsupported.
- hunex-switch-text: `script_text.mrg` offset tables and `$` line lookup.
- hunex-cli (`hunex`): commands for chs, hfa, lenzu, ctd, hw, nopus, mzp, image, font, mzx, mrg, nx, switch-text and switch-script. See the README.
- Format notes in `docs/`, and `THIRD_PARTY_LICENSES.md`, which is included in the release zips.
