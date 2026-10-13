# Glyph table (.ccit) and font atlas format

The hunex-ccit crate reads and writes the glyph tables of the Steam release. In the install they are 16 entries of `data00100.hfa`, and their atlases are `FONT_*.mzp` entries of the same archive. The description comes from our own reader and the sample files. It is not an official specification, and nothing here has been checked in the running game.

## Stored and decompressed forms

A stored entry is a LenZu stream (see [lenzu-format.md](lenzu-format.md)). Its decompressed bytes are the table. `hunex font` accepts either form, detected by the LenZu magic.

## Table

All integers are little-endian uint32.

- A header of the glyph count N and the record size 8.
- N records of (code, index), 8 bytes each.
- A coordinate block, in one of two forms:
  - Coordinates present: N, the record size 20, and N records of (page, x, y, width, height).
  - Coordinates absent: one word of 0.
- The length is 16 + 28N with coordinates, and 12 + 8N without. Every sample parses to exactly its length.

A code is the glyph's UTF-8 bytes read as one big-endian integer. In all 16 files the records are in code order with no repeats, and the indices are 0 to N-1 with no repeats. The crate accepts any order when it parses, and keeps the order when it writes. Insertion and the import command require code order and dense indices.

## Results on the samples

- All 16 tables rebuild byte for byte from their parsed form. The count, record size and length match the formulas above in every file.
- The per-file glyph counts and code ranges match the table in the notes we keep for the project.
- The 12 coordinate tables hold 4,793 coordinate entries. For each, the page is index / 98, the y is the cell row times 73, and the height is 73. There are no mismatches.

## Atlas grid

An atlas page has 14 columns and 7 rows of 73 px cells. For an index, the page is index / 98, the column is index % 14, and the row is (index % 98) / 14. The cell origin is (column x 73, row x 73).

The stored x is not the cell position for every record. In the four 106-glyph tables, the 8 records with indices 98 to 105 store x values that land on other glyphs' cells. The renderer draws from the index cell, and the stored x is not used for drawing. Rendering the 8 accented page-1 characters followed by "Tsu" this way gives the same characters as the input. The stored width is used as the pen advance. That is the reference's behaviour as far as we know, and it is not verified in the game.

## Atlas (bitmap type 0x01)

A font atlas is an MZP archive with two entries.

- Entry 0 is 1041 bytes. Bytes 0 to 11 are six u16 fields: canvas width and height 1024, tile width and height 1024, across 1 and down 1. Byte 12 is the bitmap type 0x01. Byte 13 is 0x00, and byte 14 is 0x01. The spec calls byte 14 the depth; we read it as the crop, which has no effect on one tile. Byte 15 is 0. Bytes 16 to 1039 are the palette: 256 entries of R, G, B and a 7-bit alpha code, coded as for the background tiles. Byte 1040 is the transparency byte, which is 1 in the samples.
- Entry 1 is an MZX0 stream of 1,048,576 palette indices, row by row.

The palette is stored in swizzled order. Index i reads palette slot s(i), where s swaps bits 3 and 4 of the index (s(i) = (i & 0xE7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1)). The map is its own inverse, so writing uses the same map. The blank index in the English atlases, 204, reads slot 212, whose alpha code is 0. Read linearly, slot 204 would give alpha 42, which is wrong. Read linearly, the blank index would give alpha 42 instead of 0.

Glyph coverage is the palette alpha. The palette colours are white, and blank cells are transparent. The renderer uses the alpha as it is.

`hunex image to-png` writes the atlas as RGBA. `hunex image from-png --like` keeps the original container byte for byte when the PNG is unchanged. An edited atlas is re-encoded with an exact palette, written in swizzled order, when it has 256 colours or fewer, which both English atlases do.

## Open questions

- Which of byte 13 or byte 14 is the depth in atlas headers.
- Whether the game draws the stored x or the index cell for the page-1 records.
- Whether the advance is the stored width.
- Whether the game accepts edited tables, edited atlases, or a table written in code order with dense indices.
- The atlas mapping for the Japanese, Chinese and Traditional Chinese tables is only checked by count.
