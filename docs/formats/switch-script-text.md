# Switch script text

## Overview

The `hunex-switch-text` crate reads `script_text.mrg`, an MZP container with ten entries in five language pairs. Entry `2p` is a pair's offset table and entry `2p + 1` is its text. The layout is the same in both games. The format has no official specification.

## Layout

### Offset table

The table is a flat array of big-endian u32 values, `N + 2` of them. The first `N` values are the start of each slot in the text. Value `N` is the length of the text, and value `N + 1` is 0xFFFFFFFF. Slot `N - 1` is a zero-length terminator, so the pair has `N - 1` lines.

The pairs, by content, are: pair 0 Japanese, pair 1 English, pair 2 Chinese, pair 3 a second CJK set, and pair 4 the null language, where every line is an ideographic space. Pair 1 is the default for `hunex switch-text export`.

Line counts: 43,969 lines in each pair for set A; 24,134 for set B.

### Lines

Line `i` is the bytes from slot `i`'s start to slot `i + 1`'s start. Each line ends with CR LF, which is removed when the line is read. The text is UTF-8.

## Encoding rules

- Starts never decrease, and the length value must equal the text entry's size. The reader checks both.
- A line that is not UTF-8 is an error.
- A script refers to a line with `$` and exactly six decimal digits. The number is the zero-based line index in the pair the engine loads. The crate's `ref_index` and `format_ref` convert between the two forms.
- `hunex switch-text export` writes one line per index, `$000000` then a TAB, then the text. Backslash, tab, CR and LF are written as `\\`, `\t`, `\r` and `\n`, as in the `.ctd` export format.

- Writing: `switch-text import` replaces one pair. The table has one start per line, then the text length, then `0xFFFFFFFF`, all big-endian. Each line is written with CR LF after it. The import keeps the line count of the pair, because the ids are shared positions.
- Checked on sample files: exporting the Tsukihime English pair and importing it into the original container reproduces `script_text.mrg` byte for byte. An edited line survives a second export, and no other line changes (`switch_text_import` gated test). The Mahoyo text is not in the samples, so only the Tsukihime container is checked.

## What the game accepts

Open: the game has not loaded a rebuilt `script_text.mrg`.

## Known unknowns

- Which pair the engine loads, and which pair is used for a given id when more than one language is present, is not established.
- Open: whether the game accepts a line of a different length in this container.
