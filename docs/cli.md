# Command reference

The `hunex` command groups its commands by file format: `hunex <format> <command>`. Run `hunex help` for the list of formats, and `hunex <format> help` for the commands of one format. This page lists every command and option. The examples use placeholder file names. Replace them with files from your own copy of the game.

## Global

```text
hunex help
hunex --version
```

| Form | Purpose |
|---|---|
| `help`, `-h`, `--help` | Print the list of formats to standard output. |
| `-V`, `--version`, `version` | Print `hunex` and its version. |

Example: `hunex --version` prints `hunex 0.1.0` (the version of your build).

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command finished. |
| 1 | The command failed. It prints `hunex: <error>` to standard error. |
| 2 | Usage error: no command, an unknown command or format, or missing or unknown arguments. It prints the usage text to standard error. |

A closed output pipe (for example `hunex ... | head`) is not an error and exits with 0.

## Output files

Commands that take `-o <file>` write the file only when they succeed. Commands that take `-o <dir>` create the directory when it does not exist.

Write output to a folder in your work area, never into the game install. See the [getting-started guide](guides/getting-started.md).

## chs: compiled scripts (.chs)

Dumps, checks and assembles `.chs` scripts. See [chs-format.md](formats/chs-format.md).

### chs dump

Print the instructions of a script.

Usage: `hunex chs dump [--strings | --asm] <file.chs>`

| Option | Purpose |
|---|---|
| (none) | Print the instruction list. |
| `--strings` | Print only the string operands. |
| `--asm` | Print the text form, which `chs asm` reads back. |

Example: `hunex chs dump --asm script.chs > script.txt`

### chs roundtrip

Re-encode a script and compare the result with the file. Prints whether the bytes match.

Usage: `hunex chs roundtrip <file.chs>`

Example: `hunex chs roundtrip script.chs`

### chs asm

Assemble the text form into a script.

Usage: `hunex chs asm <in.txt> -o <out.chs>`

| Option | Purpose |
|---|---|
| `-o <out.chs>` | The script to write. Required. |

Example: `hunex chs asm script.txt -o script.chs`

### chs help

Usage: `hunex chs help`

Print the commands of `chs`.

## hfa: script archives (.hfa)

Lists, unpacks and packs `.hfa` archives. See [hfa-format.md](formats/hfa-format.md) and the [repacking guide](guides/repacking.md).

### hfa list

Print the number, size, kind and name of each entry in an archive.

Usage: `hunex hfa list <archive.hfa>`

Example: `hunex hfa list data00000.hfa`

### hfa unpack

Write each entry as a file, plus `hfa-manifest.txt`, which records the order, file names and archive names. An entry whose name is taken by an earlier entry (ignoring case) gets a numbered file name, such as `name(2).ext`.

Usage: `hunex hfa unpack <archive.hfa> -o <dir> [--decompress]`

| Option | Purpose |
|---|---|
| `-o <dir>` | The folder to write. Required. |
| `--decompress` | Write LenZu entries decompressed. The manifest records this. |

Example: `hunex hfa unpack data00000.hfa -o unpacked/data00000`

### hfa pack

Rebuild an archive from a folder written by `hfa unpack`.

Usage: `hunex hfa pack <dir> -o <archive.hfa> [--base <original.hfa>]`

| Option | Purpose |
|---|---|
| `-o <archive.hfa>` | The archive to write. Required. |
| `--base <original.hfa>` | Reuse the original's stored bytes for each entry whose content matches the original. Use it when you packed decompressed entries. |

Example: `hunex hfa pack unpacked/data00000 -o data00000.hfa --base data00000.hfa`

### hfa help

Usage: `hunex hfa help`

Print the commands of `hfa`.

## lenzu: LenZu compressed streams

Decompresses and compresses LenZu streams. See [lenzu-format.md](formats/lenzu-format.md).

### lenzu decompress

Decompress a LenZu stream to its plain bytes.

Usage: `hunex lenzu decompress <in> -o <out>`

Example: `hunex lenzu decompress entry.bin -o entry.dec`

### lenzu compress

Usage: `hunex lenzu compress <in> -o <out> [--literal]`

| Option | Purpose |
|---|---|
| `-o <out>` | The stream to write. Required. |
| `--literal` | Write literal runs only. The output is larger than the input. |

Example: `hunex lenzu compress entry.dec -o entry.bin`

### lenzu help

Usage: `hunex lenzu help`

Print the commands of `lenzu`.

## ctd: script text (.ctd)

Exports and imports script text, one line per id. Both the stored (LenZu) and the decompressed forms are accepted. See [ctd-format.md](formats/ctd-format.md) and the [text-editing guide](guides/editing-text.md).

### ctd export

Usage: `hunex ctd export <file.ctd> -o <lines.txt>`

| Option | Purpose |
|---|---|
| `-o <lines.txt>` | The text file to write. Required. |

Example: `hunex ctd export text.ctd -o text.txt`

### ctd import

Usage: `hunex ctd import <lines.txt> -o <file.ctd>`

The ids must run `$000000`, `$000001`, and so on, in order.

| Option | Purpose |
|---|---|
| `-o <file.ctd>` | The script text to write. Required. |

Example: `hunex ctd import text.txt -o text.ctd`

### ctd get

Print the lines for the given ids.

Usage: `hunex ctd get <file.ctd> <$id>...`

Example: `hunex ctd get text.ctd '$000012' '$000013'`

### ctd help

Usage: `hunex ctd help`

Print the commands of `ctd`.

## hw: audio (.hw)

Reads and wraps `.hw` audio. See [hw-format.md](formats/hw-format.md) and the [audio guide](guides/editing-audio.md).

### hw info

Print the header fields and the Vorbis facts of a `.hw` file.

Usage: `hunex hw info <file.hw>`

Example: `hunex hw info sound.hw`

### hw unwrap

Write the Ogg Vorbis payload of a `.hw` file.

Usage: `hunex hw unwrap <file.hw> -o <out.ogg>`

| Option | Purpose |
|---|---|
| `-o <out.ogg>` | The Ogg file to write. Required. |

Example: `hunex hw unwrap sound.hw -o sound.ogg`

### hw wrap

Put a 64-byte header in front of an Ogg Vorbis file. The input must be Ogg Vorbis.

Usage: `hunex hw wrap <in.ogg> -o <out.hw>`

| Option | Purpose |
|---|---|
| `-o <out.hw>` | The `.hw` file to write. Required. |

Example: `hunex hw wrap edited.ogg -o sound.hw`

### hw help

Usage: `hunex hw help`

Print the commands of `hw`.

## font: glyph tables (.ccit)

Reads, exports, imports and renders glyph tables. Both the stored (LenZu) and the decompressed forms are accepted. See [ccit-format.md](formats/ccit-format.md) and the [font guide](guides/editing-fonts.md).

### font info

Print the glyph count, code range, index range, coordinate form and page counts of a table.

Usage: `hunex font info <x.ccit>`

Example: `hunex font info FONT_en.ccit`

### font export

Write the glyph table as a tab-separated file with the columns char, code, index, page, x, y, w, h. The code column is the character's code point in decimal. For a table without coordinates, the coordinate columns are empty.

Usage: `hunex font export <x.ccit> -o <glyphs.tsv>`

| Option | Purpose |
|---|---|
| `-o <glyphs.tsv>` | The file to write. Required. |

Example: `hunex font export FONT_en.ccit -o glyphs.tsv`

### font import

Build a table from a file written by `font export`. The import sorts the rows by code. The indices must run from 0 to N-1, with no repeats.

Usage: `hunex font import <glyphs.tsv> -o <x.ccit>`

| Option | Purpose |
|---|---|
| `-o <x.ccit>` | The table to write. Required. |

Example: `hunex font import glyphs.tsv -o FONT_en.ccit`

### font render

Draw a text line from the atlas pages of a table.

Usage: `hunex font render <x.ccit> --atlas <page0.mzp> [--atlas <page1.mzp> ...] --text "..." -o <out.png>`

| Option | Purpose |
|---|---|
| `--atlas <page.mzp>` | An atlas file for one page. Repeat it for each page the text needs. |
| `--text "..."` | The text to draw. Missing characters are skipped with a message. |
| `-o <out.png>` | The PNG to write. Required. |

Example: `hunex font render FONT_en.ccit --atlas FONT_en_H00.mzp --atlas FONT_en_H01.mzp --text "Hello" -o line.png`

### font help

Usage: `hunex font help`

Print the commands of `font`.

## cbg: CompressedBG images (.cbg)

Reads, converts and encodes `.cbg` images. See [cbg-format.md](formats/cbg-format.md) and the [image guide](guides/editing-images.md).

### cbg info

Print the size, bits per pixel, stripe height and file size of a `.cbg` image.

Usage: `hunex cbg info <file.cbg>`

Example: `hunex cbg info ui.cbg`

### cbg to-png

Write the image as PNG. The PNG uses the image's own color type: gray for 8 bits per pixel, RGB for 24, RGBA for 32.

Usage: `hunex cbg to-png <file.cbg> -o <out.png>`

| Option | Purpose |
|---|---|
| `-o <out.png>` | The PNG to write. Required. |

Example: `hunex cbg to-png ui.cbg -o ui.png`

### cbg from-png

Encode a PNG as a `.cbg` file. The color depth comes from the PNG's color type.

Usage: `hunex cbg from-png <in.png> -o <out.cbg> [--like <original.cbg>] [--simple]`

| Option | Purpose |
|---|---|
| `-o <out.cbg>` | The image to write. Required. |
| `--like <original.cbg>` | Take the depth and stripe height from the original. The PNG must be the same size. |
| `--simple` | Write literal runs with equal weights. The file is larger, and it decodes to the same pixels. |

Example: `hunex cbg from-png ui.png -o ui.cbg --like ui_original.cbg`

### cbg help

Usage: `hunex cbg help`

Print the commands of `cbg`.

## nopus: Nintendo OPUS audio

Reads Nintendo OPUS files and remuxes them to Ogg Opus. See [nintendo-opus-format.md](formats/nintendo-opus-format.md).

### nopus info

Print the channel count, sample rate, packet count and length.

Usage: `hunex nopus info <in.opus>`

Example: `hunex nopus info clip.opus`

### nopus to-ogg

Remux the file to Ogg Opus. The packets are copied unchanged.

Usage: `hunex nopus to-ogg <in.opus> -o <out.ogg>`

| Option | Purpose |
|---|---|
| `-o <out.ogg>` | The Ogg file to write. Required. |

Example: `hunex nopus to-ogg clip.opus -o clip.ogg`

### nopus from-ogg

Converts an Ogg Opus file back to a Nintendo OPUS file. The packets are copied unchanged. The packet fields come from the comment that `nopus to-ogg` writes. A file without that comment gets zero fields, and the command says so.

Usage: `hunex nopus from-ogg <in.ogg> -o <out.opus>`

| Option | Purpose |
|---|---|
| `-o <out.opus>` | The Nintendo OPUS file to write. Required. |

Example: `hunex nopus from-ogg work/audio/clip.ogg -o work/audio/clip.opus`

### nopus help

Usage: `hunex nopus help`

Print the commands of `nopus`.

## mzp: image containers (.mzp)

Lists, unpacks and packs MZP containers. See [mzp-image-format.md](formats/mzp-image-format.md).

### mzp list

Print the entries of an MZP container with their sizes.

Usage: `hunex mzp list <file.mzp>`

Example: `hunex mzp list img0408.mzp`

### mzp unpack

Write the entries as `entry-000.bin`, `entry-001.bin`, and so on.

Usage: `hunex mzp unpack <file.mzp> -o <dir>`

| Option | Purpose |
|---|---|
| `-o <dir>` | The folder to write. Required. |

Example: `hunex mzp unpack img0408.mzp -o img0408`

### mzp pack

Pack `entry-000.bin`, `entry-001.bin`, and so on, in order, into a container.

Usage: `hunex mzp pack <dir> -o <file.mzp>`

| Option | Purpose |
|---|---|
| `-o <file.mzp>` | The container to write. Required. |

Example: `hunex mzp pack img0408 -o img0408.mzp`

### mzp help

Usage: `hunex mzp help`

Print the commands of `mzp`.

## image: background images

Converts background images between PNG and MZP. See [mzp-image-format.md](formats/mzp-image-format.md) and the [image guide](guides/editing-images.md).

### image to-png

Convert a background image container to PNG.

Usage: `hunex image to-png <in.mzp> -o <out.png>`

| Option | Purpose |
|---|---|
| `-o <out.png>` | The PNG to write. Required. |

Example: `hunex image to-png img0408.mzp -o img0408.png`

### image from-png

Usage: `hunex image from-png <in.png> -o <out.mzp> [--like <original.mzp>]`

| Option | Purpose |
|---|---|
| `-o <out.mzp>` | The container to write. Required. |
| `--like <original.mzp>` | Take the canvas, tile grid, crop and flags from the original. The PNG must be the same size. Tiles that do not change keep their bytes. |

Without `--like`, the image is padded with transparent pixels to whole 512 by 256 tiles. Tiles whose colors do not fit in 256 are quantized, which is lossy.

Example: `hunex image from-png img0408.png -o img0408.mzp --like img0408_original.mzp`

### image help

Usage: `hunex image help`

Print the commands of `image`.

## mzx: PC MZX0 streams

Decompresses and compresses PC MZX0 streams, the tile format of background images. The Switch variant is read through `switch-script dump`. See [mzp-image-format.md](formats/mzp-image-format.md).

### mzx decompress

Usage: `hunex mzx decompress <in> -o <out>`

Example: `hunex mzx decompress tile.bin -o tile.dec`

### mzx compress

Usage: `hunex mzx compress <in> -o <out>`

| Option | Purpose |
|---|---|
| `-o <out>` | The stream to write. Required. |

Example: `hunex mzx compress tile.dec -o tile.bin`

### mzx help

Usage: `hunex mzx help`

Print the commands of `mzx`.

## mrg: Switch archive sets (.hed, .mrg, .nam)

Lists and unpacks Switch archive sets. See [switch-archives.md](formats/switch-archives.md).

### mrg list

Usage: `hunex mrg list <set>`

`<set>` is the base path of the set, or the path of any one of its three files.

Example: `hunex mrg list allpachdml`

### mrg unpack

Write the entries to a folder, named from the `.nam` file. Unpacked NXX entries keep their wrapper, and other entries are written as stored, including sector padding.

Usage: `hunex mrg unpack <set> -o <dir>`

| Option | Purpose |
|---|---|
| `-o <dir>` | The folder to write. Required. |

Example: `hunex mrg unpack allpachdml -o allpachdml-unpacked`

### mrg help

Usage: `hunex mrg help`

Print the commands of `mrg`.

## nx: Switch textures

Lists and decodes Switch textures. See [switch-textures.md](formats/switch-textures.md).

### nx info

List the textures of an NXX entry or a BNTX file.

Usage: `hunex nx info <in>`

Example: `hunex nx info texture.nxx`

### nx to-png

Decode one texture to PNG.

Usage: `hunex nx to-png <in> -o <out.png> [--index N]`

| Option | Purpose |
|---|---|
| `-o <out.png>` | The PNG to write. Required. |
| `--index N` | The texture to decode. The default is 0. |

Example: `hunex nx to-png texture.nxx -o texture.png --index 2`

### nx help

Usage: `hunex nx help`

Print the commands of `nx`.

## switch-text: Switch script text

Exports the script text of a Switch `script_text.mrg`. See [switch-script-text.md](formats/switch-script-text.md).

### switch-text export

Write one line per script id: `$000000`, a TAB, then the text.

Usage: `hunex switch-text export <script_text.mrg> -o <lines.txt> [--pair N]`

| Option | Purpose |
|---|---|
| `-o <lines.txt>` | The text file to write. Required. |
| `--pair N` | The language pair, 0 to 4. The default is 1 (English). |

Example: `hunex switch-text export script_text.mrg -o lines.txt --pair 1`

### switch-text import

Rebuilds `script_text.mrg` from a file written by `switch-text export`. The file replaces one language pair. The other pairs are copied from the original. The file must keep the ids in order and the same number of lines as the pair.

Usage: `hunex switch-text import <lines.txt> -o <script_text.mrg> --base <original.mrg> [--pair N]`

| Option | Purpose |
|---|---|
| `-o <script_text.mrg>` | The container to write. Required. |
| `--base <original.mrg>` | The original container. Required. |
| `--pair N` | The language pair to replace, 0 to 4. The default is 1. |

Example: `hunex switch-text import work/switch/text/lines.txt -o work/switch/text/script_text-edited.mrg --base work/switch/text/script_text.mrg --pair 1`

### switch-text help

Usage: `hunex switch-text help`

Print the commands of `switch-text`.

## switch-script: Switch scripts (allscr.mrg)

Dumps the scripts of a Switch `allscr.mrg`. See [switch-archives.md](formats/switch-archives.md).

### switch-script dump

Write `scr0000.txt`, `scr0001.txt`, and so on, and `nam.bin`. The scripts are written decompressed, from entry 3 onward; `nam.bin` is entry 0.

Usage: `hunex switch-script dump <allscr.mrg> -o <dir>`

| Option | Purpose |
|---|---|
| `-o <dir>` | The folder to write. Required. |

Example: `hunex switch-script dump allscr.mrg -o scripts`

### switch-script pack

Rebuilds `allscr.mrg` from a folder of script files written by `switch-script dump`. A file whose text matches the original script keeps the original stream byte for byte. A changed file is encoded again. Entries 1 and 2 and the name table come from the original, unless the folder has a `nam.bin`.

Usage: `hunex switch-script pack <dir> -o <allscr.mrg> --base <original.mrg>`

| Option | Purpose |
|---|---|
| `-o <allscr.mrg>` | The archive to write. Required. |
| `--base <original.mrg>` | The original archive. Required. Unchanged entries are copied from it. |

Example: `hunex switch-script pack work/switch/scripts/dumped -o work/switch/allscr-edited.mrg --base work/switch/scripts/allscr.mrg`

### switch-script help

Usage: `hunex switch-script help`

Print the commands of `switch-script`.
