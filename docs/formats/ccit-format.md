# Glyph table (.ccit) and font atlas format

## Overview

The `hunex-ccit` crate reads and writes the glyph tables of the Steam release of Witch on the Holy Night. In the install they are 16 entries of `data00100.hfa`, and their atlases are `FONT_*.mzp` entries of the same archive. Each table is a LenZu stream (see [lenzu-format.md](lenzu-format.md)). The format has no official specification.

Checked on sample files: all 16 tables rebuild byte for byte from their parsed form, and the atlas round trips are exact.

## Layout

### Table

All integers are little-endian uint32.

- A header of the glyph count N and the record size 8.
- N records of (code, index), 8 bytes each.
- A coordinate block, in one of two forms:
  - Coordinates present: N, the record size 20, and N records of (page, x, y, width, height).
  - Coordinates absent: one word of 0.
- The length is 16 + 28N with coordinates, and 12 + 8N without. Every sample parses to exactly its length.

A code is the glyph's UTF-8 bytes read as one big-endian integer. In all 16 files the records are in code order with no repeats, and the indices are 0 to N-1 with no repeats. The crate accepts any order when it parses, and keeps the order when it writes. Insertion and the import command require code order and dense indices.

### Atlas (bitmap type 0x01)

A font atlas is an MZP container with two entries.

- Entry 0 is 1041 bytes. Bytes 0 to 11 are six u16 fields: canvas width and height 1024, tile width and height 1024, across 1 and down 1. Byte 12 is the bitmap type 0x01. Byte 13 is 0x00, and byte 14 is 0x01. The sources call byte 14 the depth; this crate reads it as the crop, which has no effect on one tile. Byte 15 is 0. Bytes 16 to 1039 are the palette: 256 entries of R, G, B and a 7-bit alpha code, coded as for the background tiles. Byte 1040 is the transparency byte, which is 1 in the samples.
- Entry 1 is an MZX0 stream of 1,048,576 palette indices, row by row.

The palette is stored in swizzled order. Index i reads palette slot s(i), where s swaps bits 3 and 4 of the index: s(i) = (i & 0xE7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1). The map is its own inverse, so writing uses the same map. The blank index in the English atlases, 204, reads slot 212, whose alpha code is 0. Read linearly, slot 204 would give alpha 42, which shows in game as a translucent box behind each glyph.

Glyph coverage is the palette alpha. The palette colors are white, and blank cells are transparent.

### Grid

A table page holds 98 glyphs. Page p is index / 98, and it uses the atlas file for that page. Inside the atlas, the column is index % 14 and the row is (index % 98) / 14, so a page covers 7 rows of 14 cells. Each cell is 73 px square, and its origin is (column x 73, row x 73). This cell is the index cell of the glyph.

The stored x is not used for placement; see below.

## Encoding rules

- `hunex font` accepts either form of a table, detected by the LenZu magic.
- `hunex font render` draws each glyph from its index cell and at its stored width. This matches the game (see below).
- `hunex image from-png --like` keeps the original container byte for byte when the PNG is unchanged. An edited atlas is re-encoded with an exact palette, written in swizzled order, when it has 256 colors or fewer, which both English atlases do.
- Insertion needs an unused index whose cell is blank in the atlas. Insertion and the import command require code order and dense indices.

### Records at positions 98 to 105

The four 106-glyph tables have 8 records at positions 98 to 105 (indices 98 to 105, page 1). Their stored x values are assigned in code order, not index order, so for 6 of the 8 the stored x lands on another character's cell. The engine places these glyphs at their index cells, so the stored x does not affect them.

## What the game accepts

Verified in game (2026-10-13):

- The engine places each glyph at its index-derived cell and draws it at the stored width. The stored x is unused for placement. A box drawn across one cell showed only within the stored span.
- The line "à ä è é ê ï ó ÷" renders correctly, including the eight records whose stored x points at other cells.
- The palette is read in swizzled order. The swizzled alpha values are a smooth ramp, and the blank index 204 maps to alpha 0. Read linearly, the blank cells would show as translucent boxes.

An edited atlas glyph displays in the Steam release.

Open: a glyph table with inserted or changed records, and a table written with dense indices in code order. Whether the stored width is used as the pen advance, as opposed to the drawn width, is open.

## Known unknowns

- Which of byte 13 or byte 14 is the depth in atlas headers. The crate reads byte 14 as the crop.
- Whether the game accepts edited glyph tables.
- Whether the advance is the stored width.
- The atlas mapping for the Japanese, Chinese and Traditional Chinese tables is checked by count only.
- The bold set in rows 7 to 13 of each H00 atlas is not referenced by any table in the samples.
