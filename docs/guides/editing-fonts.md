# Editing fonts

A font has two parts. A glyph table (`.ccit`) gives each character's code, index and position. Atlas images (`FONT_*.mzp`) hold the glyphs. The `font` commands read and write a table, and `font render` draws a line of text from the atlases.

The archive `data00100.hfa` holds the glyph tables and atlases for every language. This guide uses the English table, `Font000000.ccit`, and the English atlases, `FONT_en_H00.mzp` and `FONT_en_H01.mzp`.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `cmp`.

## Unpack the font archive

```sh
mkdir -p work/fonts
cp "<game folder>/data00100.hfa" work/fonts/
hunex hfa unpack work/fonts/data00100.hfa -o work/fonts/unpacked
```

The folder holds the tables (`Font000000.ccit` and others) and the atlases (`FONT_en_H00.mzp`, `FONT_en_H01.mzp`, and others).

## Read a table

```sh
hunex font info work/fonts/unpacked/Font000000.ccit
```

Output:

```text
glyphs:        106
coordinates:   present (page, x, y, width, height per glyph)
code range:    U+0020 ' ' .. U+00F7 '÷'
index range:   0 .. 105
code order:    sorted
index check:   dense, 0 to N-1
page 0:       98 glyphs
page 1:       8 glyphs
```

## Export and import the table

Export writes a tab-separated file with one row per glyph. The columns are `char`, `code`, `index`, `page`, `x`, `y`, `w` and `h`. The `code` column is the character's code point in decimal. For a table without coordinates, the coordinate columns are empty:

```sh
hunex font export work/fonts/unpacked/Font000000.ccit -o work/fonts/glyphs.tsv
wc -l < work/fonts/glyphs.tsv
```

Output:

```text
107
```

That is 106 glyphs and the header row.

Edit the file, then import it. The import sorts the rows by code. The indices must run from 0 to N-1, with no repeats:

```sh
hunex font import work/fonts/glyphs.tsv -o work/fonts/Font000000-edited.ccit
hunex lenzu decompress work/fonts/unpacked/Font000000.ccit -o work/fonts/original.dec
hunex lenzu decompress work/fonts/Font000000-edited.ccit -o work/fonts/edited.dec
cmp work/fonts/original.dec work/fonts/edited.dec && echo "decompressed tables identical"
```

Output:

```text
decompressed tables identical
```

The file size of the imported table depends on the LenZu compression, so it can differ from the original's. The decompressed tables match.

## Render a line

`font render` draws each glyph from its index cell, at the glyph's stored width. Pass one atlas for each page that the text uses. The cell is the 73-pixel square that the glyph's index selects in the atlas. The [glyph table format](../formats/ccit-format.md) describes the grid.

```sh
hunex font render work/fonts/unpacked/Font000000.ccit --atlas work/fonts/unpacked/FONT_en_H00.mzp --atlas work/fonts/unpacked/FONT_en_H01.mzp --text "Sample 123" -o work/fonts/sample.png
```

The command writes the PNG and prints nothing. A character with no glyph is skipped with a message. The character here is a CJK ideograph, which the English table does not contain:

```sh
hunex font render work/fonts/unpacked/Font000000.ccit --atlas work/fonts/unpacked/FONT_en_H00.mzp --atlas work/fonts/unpacked/FONT_en_H01.mzp --text "中" -o work/fonts/missing.png
```

Output:

```text
hunex font: no glyph for U+4E2D, skipped
```

## Edit an atlas

An atlas is an image of palette indices. Convert it to PNG with `hunex image to-png`, edit the PNG, and rebuild it with `hunex image from-png --like`, as [Editing images](editing-images.md) describes. The palette order is handled for you.

Verified in game: an edited atlas glyph displayed in the Steam release of Witch on the Holy Night, and accented glyphs rendered. Open: edited glyph tables have not been tested in game.

Format details: [glyph table format note](../formats/ccit-format.md).
