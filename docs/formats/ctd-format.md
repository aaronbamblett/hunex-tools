# Script text (.ctd) format

## Overview

The `hunex-ctd` crate reads and writes the script text bodies of the Steam release of Witch on the Holy Night. In the install they are four entries of `data00200.hfa`, one per language, each stored as a LenZu stream. The format has no official specification.

Checked on sample files: the 250 decompressed scripts refer to ids from `$000000` to `$024133` (23,879 distinct ids). The English text file has 24,134 lines.

## Layout

A stored entry is a LenZu stream (see [lenzu-format.md](lenzu-format.md)). Its decompressed bytes are the text body. `Text::from_bytes` accepts either form, detected by the LenZu magic. `Text::to_stored` writes the body as a LenZu stream.

The body has no header, no index, no length prefix, no padding and no trailer. It is a list of lines. Each line is UTF-8 text followed by CRLF (0x0D 0x0A). The body ends right after the last CRLF.

### Ids

A chs script refers to text with `$` followed by exactly six decimal digits. The id is the zero-based index of the line: line 0 is `$000000`. The largest id is one less than the line count. `format_id` and `parse_id` convert between an index and its id, and `lookup` returns the line for an id.

### Export format

`hunex ctd export` writes one line per index:

```text
$000000<TAB>text of line 0
$000001<TAB>text of line 1
```

Inside the text, a backslash is written as `\\`, a tab as `\t`, a CR as `\r` and a LF as `\n`. Other characters are written as they are, in UTF-8. The file ends with a newline.

## Encoding rules

- A line holds no CR and no LF. A bare CR, a bare LF, or text after the last CRLF is an error.
- The reader checks that the whole body is valid UTF-8 before splitting it.
- Line lengths are not stored, so a line can be any length, and a line's position does not depend on the lines before it.
- `hunex ctd import` reads the export format back. It needs one line per id, in order, starting at `$000000`, with no gaps and no repeats; a TAB after the id; only the escapes above; and text that does not decode to a CR or LF. Each error names the line.
- Changing a line's length is allowed by the format, because no offsets depend on it.
- Inserting a line shifts the ids of every later line, so every later reference would name a different line. Replace a line in place instead.
- Appending a line gives it the next id. No sample script refers to an id past the last line.
- All four languages should keep the same line count, because the ids are shared positions.

## What the game accepts

Verified in game (2026-10-13): an edited line of a different length displays in the Steam release of Witch on the Holy Night, with both a literal-only entry and a compressed entry (the full compressor, from `ctd import`).

Open: a body with a different line count.

## Known unknowns

- Open: whether the game accepts a body with a different line count. Changed line lengths are verified in game (see above).
- Whether the game reads ids past the last line.
- The meaning of the markup inside lines: `^`, the `@` codes and `|`. The format passes them through unchanged.
- Open: whether a stored entry can be written byte for byte as the original. The writer compresses the body, so its stored entry decodes to the same body but has different bytes.
