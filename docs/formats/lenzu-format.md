# LenZu compressed stream format

## Overview

LenZu is the compression wrapper around the game's HFA entries, script text bodies and glyph tables. The `hunex-lenzu` crate decodes it and can write it. This note describes what the crate reads and writes, checked on the sample files. The format has no official specification.

The reader verifies the checksum in the header. The decoder rejects a stream whose checksum does not match.

Checked on sample files: all 250 script streams decode, and their checksums match. The compressor output for the 250 scripts, the 4 script text files and the 16 glyph tables decodes back to the same bytes.

## Layout

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

Option bytes: byte 0x30 is ignored. Bytes 0x31 and 0x32 hold the symbol width (7 in the samples; the reader takes the larger). Byte 0x33 is unused. Byte 0x34 is the number of low distance bits (7 in the samples). Byte 0x35 is the distance base (2 in the samples). The reader accepts only symbol width 7 and at most 16 low bits, and returns an unsupported-variant error otherwise.

### Huffman table

The table count at 0x36 selects the form:

- Dense form, when the count is 103 or more: 128 weights of four bytes each, one per symbol 0 to 127, starting at 0x37.
- Sparse form, otherwise: the count is the number of pairs. Each pair is one symbol byte and one four-byte weight. Symbols must be below 128 and may not repeat.

A count of zero is read as 128 symbols. Open: no sample uses a count of zero, so this reading is not checked on the files.

### Bitstream

Bits are read most significant first, starting right after the table. Each token starts with one flag bit.

- Flag 0, a literal run: one symbol L from the tree, then L + 1 bytes of eight raw bits each.
- Flag 1, a back-reference: a symbol L (length = L + base), a symbol H, then the low distance bits as raw bits. Distance = (low bits | H shifted left by the low-bit count) + base.

### Checksum

Start with a 64-bit accumulator at zero. For each plain byte at index i, add the byte, then multiply the accumulator by a key chosen by i modulo 4: 233, 277, 311, 433 in that order. Keep the low 64 bits. The header stores the result, high half at 0x24 and low half at 0x28.

## Encoding rules

### Decoding

- The code tree is built by repeatedly joining the two lightest nodes that have no parent yet. Ties go to the node created earlier, and the lighter node goes on the one-bit side. The build stops when a joined weight covers every weight, or when one node is left. A table with one nonzero symbol gives a tree that reads no bits.
- Each back-reference copies byte by byte, so a distance shorter than the length repeats bytes.
- Output stops at the decompressed size. A reference before the start of the output, or a stream that runs out of bits first, is an error.

### Compressor

`compress` runs a greedy LZ77 parse with hash chains over the last 16385 bytes. It takes the longest match of at least 2 bytes, at most 129, and never longer than its distance. Literal runs are at most 54 bytes. The table counts the symbols the parse produced, in the form above. If a code would be deeper than 13, the weights are halved, keeping each used symbol at least 1, until it is not. The writer sets field 0x2C to zero.

Compressed size, as a fraction of the stored originals: scripts 0.971, script text 0.953, glyph tables 0.917. The largest script text body (2.0 MB) compresses in about 0.14 s.

### Literal-only writer

`compress_literal` writes the header, a dense table with all 128 weights equal to one, and a body of literal runs of up to 128 bytes. With every weight equal to one, symbol s is coded as the seven-bit value 127 - s. The writer does not search for matches, so its output is larger than its input.

### Envelope of the original streams

Measured on the 250 script streams, the 4 script text streams and the 16 glyph tables:

- Header options (bytes 0x30 to 0x35) are 06 07 07 0E 07 02 in every stream.
- Field 0x2C is nonzero in every stream. It is not the stored length, the plain length, or the sum of the table weights.
- The table form follows the declared count as described above. The declared count is the number of symbols with a nonzero weight in every stream.
- The table weights equal the number of times each symbol is read from the bitstream, in every stream.
- Code depth reaches 12 in scripts, 9 in script text and 13 in glyph tables.
- Matches run from 2 to 129 bytes, at distances from 2 to 16385. No match copies bytes it has just written.
- Literal runs reach 54 bytes in scripts, 43 in script text and 7 in glyph tables.
- The unread bits of the last byte are zero in every stream.

`Envelope` in the crate reports these counts for a set of streams.

## What the game accepts

- Verified in game (2026-10-13): both literal-only streams and output from the full compressor load in the Steam release of Witch on the Holy Night. Compressed scripts and script text were both shown in game.

## Known unknowns

- Whether the game checks the checksum, and what it does with the field at 0x2C.
- Whether the game uses the constants at 0x10 and 0x14, the field at 0x30, or the unused byte at 0x33.
- Open: overlapping back-references (length greater than distance). The reader handles them, but no sample contains one.
- Open: symbol widths other than 7, and table counts of zero. No sample uses either.
- Open: the byte layout a literal-only stream from another writer would use. The layout above is the one this crate writes and the game loads.
