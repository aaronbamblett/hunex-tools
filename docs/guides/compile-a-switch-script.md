# Compiling a Switch script

A Steam scene script can be rebuilt from its Switch script text. This guide compiles one Switch script with `chs compile`, checks the output, and packs it into the script archive with `hfa pack --base`.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `cp`, `cmp` and `sed`.
- The Switch script text (`scr0001.txt`) and its `nam.bin` from your own Switch dump, plus the Steam archive they correspond to.

## Get the files

Copy the archive first, as [Getting started](getting-started.md) describes:

```sh
mkdir -p work/compile
cp "<game folder>/data01100.hfa" work/compile/
cp "<switch dump>/scr0001.txt" work/compile/
cp "<switch dump>/nam.bin" work/compile/
```

## Unpack the script archive

A script must be decompressed before `chs` can read it. Unpack with `--decompress`:

```sh
hunex hfa unpack work/compile/data01100.hfa -o work/compile/unpacked --decompress
```

Output:

```text
unpacked 138 entries to work/compile/unpacked
```

The folder holds `1_0.chs`, `1_1.chs`, and so on. The entries are LenZu streams; `pack --base` below compresses the edited entry back to one.

## Compile the script

```sh
hunex chs compile work/compile/scr0001.txt --names work/compile/nam.bin -o work/compile/1_0-compiled.chs
```

Output (to stderr):

```text
warning: 627 statements use zero-emission rules not proven on the corpus: _WKAD 75, _WKST 450, _WNTY 100, _ZZ 2
```

The warning is a per-command summary of the selections that are not proven on the corpus; the selection still compiles. For each warning with its position, pass `--warnings all`:

```sh
hunex chs compile work/compile/scr0001.txt --names work/compile/nam.bin -o work/compile/1_0-compiled.chs --warnings all
```

Output (the first lines):

```text
work/compile/scr0001.txt:1:1: _ZZ(): zero-emission predicate not proven on the corpus
work/compile/scr0001.txt:1:12: _ZZ(START): zero-emission predicate not proven on the corpus
```

Positions are 1-based `line:column`. Errors use the same form and exit non-zero.

## Check the output

Check that the compiled script re-encodes to its own bytes:

```sh
hunex chs roundtrip work/compile/1_0-compiled.chs
```

Output:

```text
identical
```

## Other scene layouts

`--names` covers the Mahoyo Switch layout. For any other game, pass a scene map instead: a UTF-8 text file with one `<number> <scene name>` per line. Blank lines and `#` comment lines are allowed. This script jumps to one scene, so its map is one line:

```text
# scene numbers for this script
201 1_1
```

```sh
hunex chs compile work/compile/scr0001.txt --scene-map work/compile/map.txt -o work/compile/1_0-map.chs
cmp work/compile/1_0-map.chs work/compile/1_0-compiled.chs && echo "same output"
```

Output:

```text
same output
```

The map needs an entry for every scene number the script jumps to. Without one, the command names the position:

```text
hunex: work/compile/scr0001.txt:1:19941: unknown scene number at statement 1177
```

## Pack the archive

Put the compiled script in place of the unpacked entry, then pack with `--base`, so the entries you did not change keep their stored bytes:

```sh
cp work/compile/1_0-compiled.chs work/compile/unpacked/1_0.chs
hunex hfa pack work/compile/unpacked -o work/compile/data01100-rebuilt.hfa --base work/compile/data01100.hfa
hunex hfa list work/compile/data01100-rebuilt.hfa | sed -n 2,3p
```

Output:

```text
packed 138 entries into work/compile/data01100-rebuilt.hfa
   0       4552  lenzu  1_0.chs
   1       9755  lenzu  1_1.chs
```

The sizes depend on the compile: the replaced entry is compressed again, while the rest reuse their stored bytes.

## Check the result

Unpack the new archive in the decompressed form, and compare the replaced entry with the compile plus an entry you did not change:

```sh
hunex hfa unpack work/compile/data01100-rebuilt.hfa -o work/compile/check --decompress
cmp work/compile/check/1_0.chs work/compile/1_0-compiled.chs && echo "rebuilt entry matches the compile"
cmp work/compile/check/1_1.chs work/compile/unpacked/1_1.chs && echo "other entries unchanged"
```

Output:

```text
rebuilt entry matches the compile
other entries unchanged
```

To test it, use the rebuilt archive in place of the original in a copy of the game files and play the scene.

Verified in game: a Mahoyo scene compiled from Switch text plays through in the Steam release of Witch on the Holy Night.

Format details: [compiled script format note](../formats/chs-format.md). Every option is in the [command reference](../cli.md).
