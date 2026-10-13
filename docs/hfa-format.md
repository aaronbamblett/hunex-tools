# HFA script archive format

The hunex-hfa crate reads and writes HFA archives as the game's Steam release stores them. The description is based on our own analysis of five archives and on our reader. It is not an official specification.

## Layout

All numbers are unsigned 32-bit little-endian.

| Offset | Size | Contents |
|---|---|---|
| 0x00 | 12 bytes | Magic `HUNEXGGEFA10`, no terminator |
| 0x0C | 4 | Entry count N |
| 0x10 | N x 0x80 | Entry records, in archive order |
| 0x10 + N x 0x80 | rest of file | Entry data, packed back to back |

The file ends exactly where the last entry's data ends.

## Entry record (0x80 bytes)

| Record offset | Size | Contents |
|---|---|---|
| 0x00 | 0x60 | Name: text bytes, then zero bytes to the end of the field |
| 0x60 | 4 | Data offset, relative to the start of the data region |
| 0x64 | 4 | Data size in bytes |
| 0x68 | 0x18 | Reserved; zero in every sample record |

## Rules the reader enforces

- The magic must match exactly.
- The record table must fit in the file. A count that runs past the end is an error.
- A name must end with a zero byte inside its 0x60-byte field, its bytes must be valid UTF-8, and nothing but zeros may follow the terminator.
- The reserved bytes must be zero. Writing them back as zeros is then exact.
- Entries are packed in record order: the first starts at offset zero, and each starts where the previous one ends. Gaps, overlaps and entries that run past the end of the file are errors. So is data after the last entry.

These rules are stricter than the format might need. They accept exactly the layout the sample archives use, so that every archive they accept can be written back byte for byte.

## Writing

The writer produces the same layout. It rejects an empty name, a name of 0x60 bytes or more, a name with a NUL, a repeated name, and sizes that do not fit 32-bit offsets. Record fields are written in the order above, with the reserved bytes zeroed.

## Unknowns

- Whether the game accepts an archive whose entry sizes differ from the originals. Offsets shift for every later entry, so changing one entry changes its successors' records.
- Whether the game checks the reserved bytes, or looks up entries by name, by index, or both.
- Names outside ASCII. The sample names are ASCII.
