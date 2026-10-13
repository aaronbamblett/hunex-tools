# Editing script text

The game's script text is in the archive `data00200.hfa`. It holds four entries, one per language, and each entry is a `.ctd` file. This guide exports the English text, changes one line, imports it, and rebuilds the archive. The other languages stay unchanged.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `sed` and `cmp`. The `sed -i` command uses GNU sed; on macOS, write `sed -i ''`.

## Rules for the text

- Each line has an id: `$000000` for the first line, then `$000001`, and so on. The ids run in order with no gaps.
- Keep the same number of lines. Do not insert or delete lines, because each reference names a line by its position.
- A line may change length.
- Escape a backslash as `\\`, a tab as `\t`, a CR as `\r` and an LF as `\n`.
- Markup such as `^` and `@` codes passes through unchanged. Copy it as it is.

## Get the archive

Copy the archive first, as [Getting started](getting-started.md) describes:

```sh
mkdir -p work/text
cp "<game folder>/data00200.hfa" work/text/
```

## Unpack the archive

Unpack without `--decompress`. The text entries then keep their stored form, which is the form `ctd import` writes:

```sh
hunex hfa unpack work/text/data00200.hfa -o work/text/unpacked
```

Output:

```text
unpacked 4 entries to work/text/unpacked
```

The folder holds `script_text_en.ctd`, `script_text_ja.ctd`, `script_text_zc.ctd` and `script_text_zt.ctd`, and a manifest file.

Do not add `--decompress` here. Pack compresses an entry that the manifest records as decompressed. If you unpack with `--decompress` and then import, pack refuses the file:

```text
hunex: work/text/unpacked/script_text_en.ctd: this entry is recorded as decompressed in hfa-manifest.txt, but the file already starts with the LenZu magic, so pack would compress it twice. Unpack the archive without --decompress and put the stored file in place, or replace it with the decompressed bytes
```

## Export the text

```sh
hunex ctd export work/text/unpacked/script_text_en.ctd -o work/text/en.txt
wc -l < work/text/en.txt
```

Output:

```text
24134
```

The export has one line per id, with a tab after the id. The English text has 24,134 lines, so its ids run from `$000000` to `$024133`.

To print one line by its id, use `ctd get`. The output starts with the id, and the text follows it. This command shows only the id:

```sh
hunex ctd get work/text/unpacked/script_text_en.ctd '$000001' | cut -f1
```

Output:

```text
$000001
```

## Edit one line

Replace line 2, which has the id `$000001`, with your own text. The `\t` in the command is a real tab, and `sed` needs GNU syntax here:

```sh
sed -i '2s/.*/$000001\tEDITED LINE FOR THE GUIDE/' work/text/en.txt
```

## Import the text

`ctd import` checks the ids and writes the file as a LenZu stream. It prints nothing on success:

```sh
hunex ctd import work/text/en.txt -o work/text/unpacked/script_text_en.ctd
```

An import stops at the first id out of order and names the line. To see this, delete line 5 of the export into a new file, and import that file:

```sh
sed '5d' work/text/en.txt > work/text/broken.txt
hunex ctd import work/text/broken.txt -o work/text/broken.ctd
```

Output:

```text
hunex: work/text/broken.txt: line 5: expected $000004, found $000005
```

## Pack the archive

Pack with `--base`, so the entries you did not change keep their original bytes:

```sh
hunex hfa pack work/text/unpacked -o work/text/data00200-edited.hfa --base work/text/data00200.hfa
hunex hfa list work/text/data00200-edited.hfa
```

Output:

```text
packed 4 entries into work/text/data00200-edited.hfa
4 entries
   0     740579  lenzu  script_text_en.ctd
   1     895880  lenzu  script_text_ja.ctd
   2     806325  lenzu  script_text_zc.ctd
   3     810364  lenzu  script_text_zt.ctd
```

The size of the English entry depends on your text. The other three entries keep their sizes.

## Check the result

Unpack the new archive and export the English text again. The export must match the file you edited:

```sh
hunex hfa unpack work/text/data00200-edited.hfa -o work/text/check
hunex ctd export work/text/check/script_text_en.ctd -o work/text/check.txt
cmp work/text/check.txt work/text/en.txt && echo "export matches the edited text"
```

Output:

```text
export matches the edited text
```

Verified in game: the Steam release of Witch on the Holy Night displayed an edited line of a different length, with both a literal-only entry and a compressed entry.

Format details: [script text format note](../formats/ctd-format.md).
