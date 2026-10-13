# Editing scripts

The game's scripts are compiled `.chs` files, one for each scene. They are in the archive `data01100.hfa`, as LenZu entries. This guide dumps one script to text, edits a string, assembles the text back into a script, and rebuilds the archive.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `sed`, `grep` and `cmp`. The `sed -i` command uses GNU sed; on macOS, write `sed -i ''`.

## Rules for the text form

- The text form has one statement per line. A line that starts with `#` is a comment.
- A string is a `string` line in the pool, such as `string S_0062 "..."`. Change the text between the quotes. Keep the name.
- Labels look like `L_0000`. Inserting an instruction moves the labels after it, and `lower` recomputes the pointers.
- The code must end with a return.
- Each opcode has a mnemonic in the [opcode table](../formats/opcodes.md). An opcode with no mnemonic is written as its hex id, such as `0x0f7`. The assembler rejects a mnemonic it does not know.

## Get the archive

Copy the archive first, as [Getting started](getting-started.md) describes:

```sh
mkdir -p work/scripts
cp "<game folder>/data01100.hfa" work/scripts/
```

## Unpack the script archive

A script must be decompressed before `chs` can read it. Unpack with `--decompress`:

```sh
hunex hfa unpack work/scripts/data01100.hfa -o work/scripts/unpacked --decompress
```

Output:

```text
unpacked 138 entries to work/scripts/unpacked
```

The folder holds `1_0.chs`, `1_1.chs`, and so on. `chs` reads only the decompressed form. To see what happens with the stored form, unpack the same archive without `--decompress` and try to dump one script from it:

```sh
hunex hfa unpack work/scripts/data01100.hfa -o work/scripts/stored
hunex chs dump work/scripts/stored/1_0.chs
```

Output:

```text
hunex: work/scripts/stored/1_0.chs: not a HuneX or Buriko compiled script
```

## Dump the script

The `--asm` form is the text you edit:

```sh
hunex chs dump --asm work/scripts/unpacked/1_0.chs > work/scripts/1_0.txt
```

Output (the first lines, with string operands shortened to `...`):

```text
format hunex
header_size 21
header_padding 00 00 00 00 00 00 00 00 00 00 00 00 00
    PushDword 0x0
    0x0f7
    PushOffset L_0000
    JmpIf 0x1
    PushString S_0062
```

Check that the script re-encodes to the same bytes before you edit it:

```sh
hunex chs roundtrip work/scripts/unpacked/1_0.chs
```

Output:

```text
identical
```

## Edit a string

The pool contains a line `string S_0062 "..."` that gives the text for `S_0062`. Replace the text between the quotes with your own. This command uses placeholder text:

```sh
sed -i 's/^string S_0062 .*/string S_0062 "EDITED TEXT"/' work/scripts/1_0.txt
```

## Assemble the script

```sh
hunex chs asm work/scripts/1_0.txt -o work/scripts/unpacked/1_0.chs
hunex chs roundtrip work/scripts/unpacked/1_0.chs
hunex chs dump --strings work/scripts/unpacked/1_0.chs | grep -c "EDITED TEXT"
```

Output:

```text
identical
1
```

`identical` means the new script re-encodes to its own bytes. The count of 1 shows that the new string appears once in the script.

An error stops the assembler and names the line. A misspelled mnemonic gives:

```text
hunex: work/scripts/bad.txt: line 5: unknown instruction PushDwordX
```

## Pack the archive

The edited script is in the decompressed form, so `pack` compresses it. Pack with `--base`, so the entries you did not change keep their stored bytes:

```sh
hunex hfa pack work/scripts/unpacked -o work/scripts/data01100-edited.hfa --base work/scripts/data01100.hfa
hunex hfa list work/scripts/data01100-edited.hfa | sed -n 2p
```

Output:

```text
   0       6633  lenzu  1_0.chs
```

The size of `1_0.chs` depends on your edit. In this run the original was 6,729 bytes.

## Check the result

Unpack the new archive in the decompressed form, and compare an entry you did not change:

```sh
hunex hfa unpack work/scripts/data01100-edited.hfa -o work/scripts/check --decompress
cmp work/scripts/check/1_1.chs work/scripts/unpacked/1_1.chs && echo "1_1 unchanged"
```

Output:

```text
1_1 unchanged
```

Verified in game: an edited script with an inserted instruction, moved pointers and a changed image string played through in the Steam release of Witch on the Holy Night.

Format details: [compiled script format note](../formats/chs-format.md).
