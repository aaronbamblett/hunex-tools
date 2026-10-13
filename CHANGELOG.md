# Changelog

## 0.1.0

First release.

- hunex-chs: decoder for `.chs` compiled scripts (Hunex and Buriko headers), with exact re-encoding, lifting to a symbolic form with labels and strings, lowering, and an editing API.
- hunex-cli: the `hunex` command with `chs dump` (listing, `--strings`, `--asm`), `chs roundtrip`, `chs asm`, `--version` and `help`.
- Documentation: the `.chs` format, the text form, and the opcode table, which is checked against the code by a test.
- Known limits: the pool start is inferred from the smallest string pointer, and pointers computed at runtime are not relocated.
