# Repacking an archive

An HFA archive is rebuilt from a folder with `hfa pack`. Unpack the archive, change the files you need, and pack the folder again. This guide shows why `--base` matters, and how to confirm the result.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `cmp`.

## Unpack and pack again

Unpack an archive with no changes, pack it again, and compare the result with the original. This example uses `data00300.hfa`, which holds raw `.hw` entries:

```sh
mkdir -p work/repacking
cp "<game folder>/data00300.hfa" work/repacking/
hunex hfa unpack work/repacking/data00300.hfa -o work/repacking/stored
hunex hfa pack work/repacking/stored -o work/repacking/data00300-repacked.hfa
cmp work/repacking/data00300-repacked.hfa work/repacking/data00300.hfa && echo "byte identical"
```

Output:

```text
packed 7 entries into work/repacking/data00300-repacked.hfa
byte identical
```

The `stored` folder holds the entries in their stored form. The same check works for `data01100.hfa`, which holds LenZu scripts, when you unpack it in the stored form:

```sh
cp "<game folder>/data01100.hfa" work/repacking/
hunex hfa unpack work/repacking/data01100.hfa -o work/repacking/scripts-stored
hunex hfa pack work/repacking/scripts-stored -o work/repacking/data01100-repacked.hfa
cmp work/repacking/data01100-repacked.hfa work/repacking/data01100.hfa && echo "byte identical"
```

Output:

```text
packed 138 entries into work/repacking/data01100-repacked.hfa
byte identical
```

## Why `--base`

`--base` names the original archive. An entry whose content has not changed reuses its stored bytes from that archive. Without `--base`, `pack` compresses every LenZu entry again.

Unpack the script archive in the decompressed form, then pack it both ways:

```sh
hunex hfa unpack work/repacking/data01100.hfa -o work/repacking/scripts-decompressed --decompress
hunex hfa pack work/repacking/scripts-decompressed -o work/repacking/with-base.hfa --base work/repacking/data01100.hfa
cmp work/repacking/with-base.hfa work/repacking/data01100.hfa && echo "with --base: byte identical"
hunex hfa pack work/repacking/scripts-decompressed -o work/repacking/without-base.hfa
cmp -s work/repacking/without-base.hfa work/repacking/data01100.hfa || echo "without --base: differs"
ls -l work/repacking/without-base.hfa | awk '{print $5}'
```

Output:

```text
with --base: byte identical
without --base: differs
3074293
```

The original archive is 3,173,765 bytes. Without `--base`, each LenZu entry is compressed again. The archive then decodes to the same files, but its bytes differ, and the size changes. With `--base`, the stored bytes are reused and the archive is identical.

Use `--base` for any archive that you unpacked with `--decompress`.

## Check an edited archive

This example replaces one `.hw` entry with a copy of another, in the stored form, and packs the archive with `--base`:

```sh
hunex hfa unpack work/repacking/data00300.hfa -o work/repacking/edit
cp work/repacking/edit/SSECancelled.hw work/repacking/edit/SSEUnable.hw
hunex hfa pack work/repacking/edit -o work/repacking/data00300-edited.hfa --base work/repacking/data00300.hfa
```

Output:

```text
packed 7 entries into work/repacking/data00300-edited.hfa
```

Check that the other entries are unchanged, and that the replaced entry now matches the copy:

```sh
hunex hfa unpack work/repacking/data00300-edited.hfa -o work/repacking/check
cmp work/repacking/check/SSECancelled.hw work/repacking/stored/SSECancelled.hw && echo "other entries unchanged"
cmp work/repacking/check/SSEUnable.hw work/repacking/stored/SSECancelled.hw && echo "replaced entry matches the copy"
```

Output:

```text
other entries unchanged
replaced entry matches the copy
```

Compare the stored form with the stored form. Decompressed entries need the same `--decompress` option on both sides.

Verified in game: five archives with changed entry sizes and offsets, rebuilt with `--base`, load in the Steam release of Witch on the Holy Night (data00000.hfa, data00200.hfa, data00300.hfa, data01100.hfa and data02001.hfa).

Format details: [HFA format note](../formats/hfa-format.md).
