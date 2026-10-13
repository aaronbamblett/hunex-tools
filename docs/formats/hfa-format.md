# HFA script archive format

## Overview

HFA is the archive format of the game's `data*.hfa` files. The `hunex-hfa` crate reads and writes it. The format has no official specification.

Checked on sample files: the five archives data00000.hfa, data00100.hfa, data00200.hfa, data00300.hfa and data01100.hfa rewrite byte for byte from their unpacked entries. Repeated names are kept.

## Layout

All numbers are unsigned 32-bit little-endian.

| Offset | Size | Contents |
|---|---|---|
| 0x00 | 12 bytes | Magic `HUNEXGGEFA10`, no terminator |
| 0x0C | 4 | Entry count N |
| 0x10 | N x 0x80 | Entry records, in archive order |
| 0x10 + N x 0x80 | rest of file | Entry data, packed back to back |

The file ends exactly where the last entry's data ends.

### Entry record (0x80 bytes)

| Record offset | Size | Contents |
|---|---|---|
| 0x00 | 0x60 | Name: text bytes, then zero bytes to the end of the field |
| 0x60 | 4 | Data offset, relative to the start of the data region |
| 0x64 | 4 | Data size in bytes |
| 0x68 | 0x18 | Reserved; zero in every sample record |

## Encoding rules

The reader enforces these rules:

- The magic must match exactly.
- The record table must fit in the file. A count that runs past the end is an error.
- A name must end with a zero byte inside its 0x60-byte field, its bytes must be valid UTF-8, and nothing but zeros may follow the terminator.
- The reserved bytes must be zero. Writing them back as zeros is then exact.
- Entries are packed in record order: the first starts at offset zero, and each starts where the previous one ends. Gaps, overlaps and entries that run past the end of the file are errors. So is data after the last entry.

These rules are stricter than the format might need. They accept exactly the layout the sample archives use, so every archive they accept can be written back byte for byte.

The writer produces the same layout. It rejects an empty name, a name of 0x60 bytes or more, a name with a NUL, and sizes that do not fit 32-bit offsets. Repeated names are allowed: `data00000.hfa` has four (`face_on_ja.cbg`, `face2_on_ja.cbg`, `kusa1.cbg` and `kusa2.cbg`, each twice), and its second copies are byte-identical to the first.

## What the game accepts

- Verified in game (2026-10-13): five archives (data00000.hfa, data00200.hfa, data00300.hfa, data01100.hfa and data02001.hfa), each with changed entry sizes and offsets rebuilt with `pack --base`, load in the Steam release of Witch on the Holy Night. Offsets shift for every later entry, so changing one entry changes its successors' records.
- The same test included a literal-only LenZu entry packed with `hunex hfa pack --base`. See [lenzu-format.md](lenzu-format.md).

## Known unknowns

- Whether the game checks the reserved bytes, or looks up entries by name, by index, or both.
- Names outside ASCII. The sample names are ASCII.
- Open: archives with more entries or larger sizes than the samples.
