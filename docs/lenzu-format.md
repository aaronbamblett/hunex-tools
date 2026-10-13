# LenZu compressed stream format

The hunex-lenzu crate decodes the LenZu streams stored in HFA archives and writes a literal-only variant. The description is based on our own decoder and on the sample streams. It is not an official specification.

## Header

All numbers are unsigned 32-bit little-endian unless stated.

| Offset | Size | Contents |
|---|---|---|
| 0x00 | 16 | Magic `LenZuCompressor` and a zero byte |
| 0x10 | 4 | A constant (49 in the samples); the reader ignores it |
| 0x14 | 4 | A constant (48 in the samples); the reader ignores it |
| 0x18 | 8 | Zero |
| 0x20 | 4 | Decompressed size |
| 0x24 | 4 | Checksum, high 32 bits |
| 0x28 | 4 | Checksum, low 32 bits |
| 0x2C | 4 | Unknown; nonzero in the samples, ignored by the reader, zero in our writer |
| 0x30 | 6 | Option bytes |
| 0x36 | 1 | Table count |

The option bytes hold the symbol width (byte 0x31 and byte 0x32, both 7 in the samples; the reader takes the larger), a back-reference low-bit limit (0x33, unused), the number of low distance bits (0x34, 7 in the samples) and the distance base (0x35, 2 in the samples). Byte 0x30 is ignored. The reader accepts only symbol width 7 and at most 16 low bits, and returns an unsupported-variant error otherwise.

## Huffman table

The table count is at 0x36. A count of zero means 128 symbols.

- Dense form, used when the count is 103 or more: 128 weights of four bytes each, one per symbol 0 to 127, starting at 0x37.
- Sparse form, used otherwise: the count is the number of pairs. Each pair is one symbol byte and one four-byte weight. Symbols must be below 128 and may not repeat.

The code tree is built the way the sample streams need: repeatedly join the two lightest nodes that have no parent yet (ties go to the node created earlier), with the lighter node on the one-bit side. Stop when a joined weight covers every weight, or when one node is left. A table with one nonzero symbol gives a tree that reads no bits.

## Bitstream

Bits are read most significant first. The bitstream starts right after the table. Each token starts with one flag bit.

- Flag 0, a literal run: one symbol L from the tree, then L + 1 bytes of eight raw bits each.
- Flag 1, a back-reference: a symbol L (length = L + base), a symbol H, then the low distance bits as raw bits. Distance = (low bits | H shifted left by the low-bit count) + base. The copy goes byte by byte, so a distance shorter than the length repeats bytes.

Output stops at the decompressed size. A reference before the start of the output, or a stream that runs out of bits first, is an error.

## Checksum

Start with a 64-bit accumulator at zero. For each plain byte at index i, add the byte, then multiply the accumulator by a key chosen by i modulo 4: 233, 277, 311, 433 in that order. Keep the low 64 bits. The header stores the result, high half at 0x24 and low half at 0x28. The decoder compares it with the decoded bytes and fails on a mismatch.

## Literal-only writer

`compress_literal` writes the header with the option bytes above, a dense table with all 128 weights equal to one, and a body of literal runs of up to 128 bytes. With every weight equal to one, symbol s is coded as the seven-bit value 127 - s. The writer does not search for matches, so its output is larger than its input.

The format permits this stream, the reader decodes it, and the game accepts it: a literal-only stream was loaded and played in the Steam release on 2026-10-13 (one script).

## Unknowns

- Whether the game checks the checksum, and what it does with the field at 0x2C.
- Overlapping back-references: the reader handles them, but the samples contain none.
- Symbol widths other than 7, and table counts of zero.
