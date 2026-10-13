# Switch script text

The hunex-switch-text crate reads `script_text.mrg`: an MZP archive of ten entries in five language pairs. Entry `2p` is a pair's offset table and entry `2p + 1` is its text. The layout is the same in both games.

## Offset table

The table is a flat array of big-endian u32 values, `N + 2` of them. The first `N` values are the start of each slot in the text. Value `N` is the length of the text, and value `N + 1` is 0xFFFFFFFF. Slot `N - 1` is a zero-length terminator, so the pair has `N - 1` lines.

Starts never decrease, and the length value must equal the text entry's size. The reader checks both.

## Lines

Line `i` is the bytes from slot `i`'s start to slot `i + 1`'s start. Each line ends with CR LF, which is removed when the line is read. The text is UTF-8; a line that is not UTF-8 is an error.

The pairs, by content, are: pair 0 Japanese, pair 1 English, pair 2 Chinese, pair 3 a second CJK set, and pair 4 the null language, where every line is an ideographic space. Pair 1 is the default for `hunex switch-text export`.

Line counts: Tsukihime 43,969 lines in each pair; Mahoyo 24,134.

## References

A script refers to a line with `$` and exactly six decimal digits. The number is the zero-based line index in the pair the engine loads. The crate's `ref_index` and `format_ref` convert between the two forms.

Which pair the engine loads, and which pair is used for a given reference when more than one language is present, is not established.

## Export format

`hunex switch-text export` writes one line per index, `$000000` then a TAB, then the text. Backslash, tab, CR and LF are written as `\\`, `\t`, `\r` and `\n`, as in the `.ctd` export format.
