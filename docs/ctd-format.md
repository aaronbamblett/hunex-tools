# Script text (.ctd) format

The hunex-ctd crate reads and writes the script text files of the Steam release. In the install they are four entries of `data00200.hfa`, one per language. The description is based on our own reader and on the sample files. It is not an official specification.

## Stored and decompressed forms

A stored entry is a LenZu stream (see [lenzu-format.md](lenzu-format.md)). Its decompressed bytes are the text body described below. `Text::from_bytes` accepts either form, detected by the LenZu magic. `Text::to_stored` writes the body as a literal-only LenZu stream, which is larger than the body.

## Body

- The body has no header, no index, no length prefix, no padding and no trailer.
- It is a list of lines. Each line is UTF-8 text followed by CRLF (0x0D 0x0A). The body ends right after the last CRLF.
- A line holds no CR and no LF. A bare CR, a bare LF, or text after the last CRLF is an error.
- The reader checks that the whole body is valid UTF-8 before splitting it.

Line lengths are not stored, so a line can be any length, and a line's position does not depend on the lines before it.

## Ids

A chs script refers to text with `$` followed by exactly six decimal digits. The id is the zero-based index of the line. Line 0 is `$000000`, and so on. The largest id is therefore one less than the line count. `format_id` and `parse_id` convert between an index and its id, and `lookup` returns the line for an id.

In the samples, scanning the 250 decompressed scripts gives ids from 0 to 24133 (23,879 distinct), which is the same range as the four text files' line count. Whether the engine indexes lines this way has not been tested in the game.

## Export format

`hunex ctd export` writes one line per index:

```text
$000000<TAB>text of line 0
$000001<TAB>text of line 1
```

Inside the text, a backslash is written as `\\`, a tab as `\t`, a CR as `\r` and a LF as `\n`. Other characters are written as they are, in UTF-8. The file ends with a newline.

`hunex ctd import` reads that format back. It needs:

- one line per id, in order, starting at `$000000`, with no gaps and no repeats;
- a TAB after the id;
- only the escapes above;
- text that does not decode to a CR or LF.

Each error names the line. Import writes a literal-only LenZu stream.

## Editing rules

- Changing a line's length is allowed by the format, because no offsets depend on it. Whether the game accepts a different length has not been tested.
- Inserting a line shifts the ids of every later line, so every later reference would name a different line. Replace a line in place instead.
- Appending a line gives it the next id. No sample script refers to an id past the last line.
- All four languages should keep the same line count, because the ids are shared positions.

## Unknowns

- Whether the game accepts a body with a different line count or with changed line lengths.
- Whether the game reads ids past the last line.
- The meaning of the markup inside lines: `^`, the `@` codes and `|`. The format passes them through unchanged.
- Whether a byte-identical stored file can be made. The sample entries use back-references, and the writer here does not produce them. The decompressed bodies round-trip exactly.
