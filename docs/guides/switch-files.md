# Reading Switch archives

The Switch release stores its files in the RomFS, its read-only file system. Its archives are sets of three files that share a base name, plus NXX-wrapped textures, MZP containers and a compiled script archive. This guide reads them, and it ends with the commands that write the script, script text and audio containers back.

The sets are named in the [Switch archive format note](../formats/switch-archives.md#game-sets). Set A is the Switch release of Tsukihime -A piece of blue glass moon-, and set B is the Switch release of Witch on the Holy Night.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `cp` only. Copy each file from your RomFS folder, and keep the copies outside it.

## Work on a copy of the parts set

The `parts` set in this guide has no `.nam` file, so its entries have no names. `mrg` shows them with a `-` in the name column and unpacks them as `entry-00000.bin` and so on.

Copy the set's `.hed` and `.mrg` files:

```sh
mkdir -p work/switch/parts
cp "<RomFS folder>/parts.hed" "<RomFS folder>/parts.mrg" work/switch/parts/
```

## List a set

Give `hunex mrg` the base path of the set, or the path of any one of its files:

```sh
hunex mrg list work/switch/parts/parts
```

Output:

```text
5 entries, 0 empty
     0       36679  nxx    -
     1      100340  nxx    -
     2       82080  nxx    -
     3       85180  nxx    -
     4      102515  nxx    -
```

Each row is the entry number, the size in bytes, the kind, and the name. `nxx` is an NXX-wrapped entry, and `mzp` is an MZP container. In a set that has a `.nam` file, an empty slot is named `NOFILE.EXISTS`, and some sets use small stand-in entries named `DUMMY.NXZ`.

Unpack the set to a folder:

```sh
hunex mrg unpack work/switch/parts/parts -o work/switch/parts/unpacked
```

Output:

```text
unpacked 5 entries to work/switch/parts/unpacked
```

## Decode a texture

An NXX entry holds a BNTX texture. `nx info` lists its textures:

```sh
hunex nx info work/switch/parts/unpacked/entry-00000.bin
```

Output:

```text
1 texture
  0  -                 0x2006   1920x1080     2949120 bytes  decodes
```

The format code `0x2006` is BC7. The decodable formats are BC7, BC4, BC3 and RGBA8. Other codes are reported as unsupported.

Decode one texture to PNG. `--index` picks the texture, and the default is 0:

```sh
hunex nx to-png work/switch/parts/unpacked/entry-00000.bin -o work/switch/parts/texture.png
```

The command prints nothing on success.

## Export script text

`script_text.mrg` is one MZP container with ten entries: an offset table and a text entry for each of five language pairs. Export one pair as lines. The default is pair 1, English. `--pair` picks another pair, from 0 to 4:

```sh
mkdir -p work/switch/text
cp "<RomFS folder>/script_text.mrg" work/switch/text/
hunex switch-text export work/switch/text/script_text.mrg -o work/switch/text/lines.txt --pair 1
```

Output:

```text
exported 43969 lines of pair 1 to work/switch/text/lines.txt
```

Each line has the same format as `hunex ctd export`: an id such as `$000000`, a tab, then the text.

## Dump the scripts

`allscr.mrg` holds the scripts as MZX0 streams. Dump them to a folder:

```sh
mkdir -p work/switch/scripts
cp "<RomFS folder>/allscr.mrg" work/switch/scripts/
hunex switch-script dump work/switch/scripts/allscr.mrg -o work/switch/scripts/dumped
```

Output:

```text
dumped 549 scripts to work/switch/scripts/dumped
```

The folder holds `scr0000.txt` to `scr0548.txt`, and `nam.bin`, the script name table.

## Switch editing

The writers rebuild the three Switch containers from their parts. The commands below edit one script, one line of script text, and one Nintendo OPUS clip. They need the GNU `sed` command, as in [Getting started](getting-started.md).

Edit one script. Dump the scripts, change one file, and pack the archive with `--base`. The packer copies every unchanged stream from the original, and encodes only the changed script:

```sh
mkdir -p work/switch/edit
cp "<RomFS folder>/allscr.mrg" work/switch/edit/
hunex switch-script dump work/switch/edit/allscr.mrg -o work/switch/edit/scripts
printf '\n' >> work/switch/edit/scripts/scr0100.txt
hunex switch-script pack work/switch/edit/scripts -o work/switch/edit/allscr-edited.mrg --base work/switch/edit/allscr.mrg
```

Output:

```text
packed 552 entries into work/switch/edit/allscr-edited.mrg: 548 unchanged, 1 encoded
```

Edit one line of script text. Export one pair, change one line, and import the file back with `--base`. The file must keep the same ids and line count:

```sh
cp "<RomFS folder>/script_text.mrg" work/switch/edit/
hunex switch-text export work/switch/edit/script_text.mrg -o work/switch/edit/lines.txt --pair 1
sed -i '2s/.*/$000001\tEDITED LINE FOR THE GUIDE/' work/switch/edit/lines.txt
hunex switch-text import work/switch/edit/lines.txt -o work/switch/edit/script_text-edited.mrg --base work/switch/edit/script_text.mrg --pair 1
```

Output:

```text
imported 43969 lines into pair 1 of work/switch/edit/script_text-edited.mrg
```

Convert a Nintendo OPUS clip. `to-ogg` writes the packets to Ogg Opus, and it stores the packet fields in a comment. `from-ogg` writes the same Nintendo file back:

```sh
mkdir -p work/switch/stream
cp "<RomFS folder>/stream/A00_01_00_0000_0.opus" work/switch/stream/
hunex nopus to-ogg work/switch/stream/A00_01_00_0000_0.opus -o work/switch/stream/clip.ogg
hunex nopus from-ogg work/switch/stream/clip.ogg -o work/switch/stream/clip-back.opus
cmp work/switch/stream/clip-back.opus work/switch/stream/A00_01_00_0000_0.opus && echo "clip round trip identical"
```

Output:

```text
wrote work/switch/stream/clip-back.opus: 65709 bytes
clip round trip identical
```

Checked on sample files: each command above reproduces the original where the original is unchanged. Open: the game has not loaded any file written this way.

## Check the decoding

The decoded scripts match script dumps made with an independent tool, deepLuna, which is licensed under BSD-2. The `HUNEX_SWITCH_DUMP_A` and `HUNEX_SWITCH_DUMP_B` gated tests check them. See [CONTRIBUTING](../../CONTRIBUTING.md).

Open: the game has not loaded any file that these commands write.

Format details: [Switch archive format note](../formats/switch-archives.md).
