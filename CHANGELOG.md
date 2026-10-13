# Changelog

## 0.1.0

First release.

- hunex-chs: decoder for `.chs` compiled scripts (Hunex and Buriko headers), with exact re-encoding, lifting to a symbolic form with labels and strings, lowering, and an editing API. Relocation was checked on all 250 sample scripts.
- hunex-lenzu: decoder for LenZu compressed streams with checksum verification, and a literal-only encoder. The literal-only output is valid for the format but larger than its input, and not yet confirmed to be accepted by the game.
- hunex-hfa: reader and writer for HFA script archives. The reader accepts only the packed layout the originals use; the writer produces it.
- hunex-cli (`hunex`): `chs dump` (listing, `--strings`, `--asm`), `chs roundtrip`, `chs asm`, `hfa list`, `hfa unpack` (with `--decompress`), `hfa pack` (with `--base`), `lenzu decompress`, `lenzu compress`, `--version` and `help`.
- Documentation: the `.chs`, HFA and LenZu formats, the text form, and the opcode table, which is checked against the code by a test.
- Known limits: the `.chs` pool start is inferred from the smallest string pointer, and pointers computed at runtime are not relocated. Edited archives and scripts have not been run in the game.
