# Switch RomFS archives

## Overview

The Switch RomFS stores its assets in three archive kinds: MRG sets, NXX wrapped entries, and MZP containers. The `hunex-mrg`, `hunex-mzp` and `hunex-mzx` crates read them. This note describes what they implement, checked on sample files.

## Layout

### Game sets

Two Switch releases appear in the sample files. Set A is the Switch release of Tsukihime -A piece of blue glass moon-, with 43,969 lines of script text in each language pair. Set B is the Switch release of Witch on the Holy Night, with 24,134 lines.

## MRG sets

A set is three files with one base name:

- `.hed`: the index. Rows are 8 bytes, little-endian: the start sector (u32), the on-disk length in sectors (u16) and the decompressed length in sectors (u16). A sector is 2048 bytes. A zero row is an empty slot. A row whose start is 0xFFFFFFFF ends the index.
- `.mrg`: the data. Entry `i` starts at `start_sector * 2048`. Its length on disk is the sector count, so a raw entry can end in padding. NXX entries carry their own length (below).
- `.nam`: the names. Rows are 32 bytes: the name, zero padded, then CR LF. Row `i` names index row `i`. A zero row ends the list. Names can repeat, and a placeholder `NOFILE.EXISTS` marks empty slots.

A set can also have no `.nam` file. Entry kinds are read from the first bytes: `NXCX` and `NXGX` (texture or data wrapped by zlib or gzip), `mrgd00` (an MZP container), and anything else (JPG, PNG, and small binary files).

Game set A's sets and their entry counts are: `allpachd` 14,765 rows (14,705 empty), `allpachdml` 67, `w_allpac` 29,267, `w_allpaccg` 1,521, `w_allpacml` 344, `w_allui` 48, `parts` 5. Game set B's sets are `allpachdml` 89, `allpacml` 167, `allui` 70, `jallpac` 22,156, `jallpachd` 11,414 and `parts` 2.

### NXX wrapper

A 16-byte header: the four-byte magic, the decompressed size (u32), the compressed size (u32), and four zero bytes. The body follows and is the compressed size long. `NXCX` is a zlib stream and `NXGX` is a gzip member. The decompressed output must match the declared size.

### MZP containers

The magic is `mrgd00`, then the entry count (u16). Eight-byte records follow: the sector, the byte within it, the sector span, and the low 16 bits of the size. The data starts after the records, and an entry starts at `sector * 2048 + byte` from there.

The Switch size rule is: take the span times 2048, clear its low 16 bits, and OR in the stored low 16 bits. The PC MZP files use a different rule with a stored span, so the two are read by separate functions in `hunex-mzp`. Switch entries are not padded, so the reader does not require padding.

### MZX0 on the Switch

The Switch script streams use the same MZX0 command set as the PC image tiles, with three differences:

- The last word and every ring slot start as 0xFFFF, not 0.
- Literal bytes are stored XORed with 0xFF.
- The decoder stops at the declared size and does not require the input to end exactly there.

Command semantics are otherwise the same: repeat writes the last word, copy-back takes `d + 1` words back, ring recall writes ring slot `n`, and literal writes `n + 1` words.

### Scripts in allscr.mrg

Entry 0 is the script name table. Entry 1 and entry 2 are small containers of labels. Entry `3 + i` is script `i`, an MZX0 stream.

## Encoding rules

- `hunex_mzp::switch_spans` reads the table alone, so a large container does not have to be loaded to find its entries.
- Bulk commands read one entry at a time with a seek, and drop each buffer before the next, so they do not load a whole data file.
- The largest files the commands read whole are the script text archives, at about 12 MB.
- `hunex switch-script dump` writes `scr0000.txt` onward and `nam.bin` (entry 0). The decoded scripts match script dumps made with an independent tool, deepLuna (BSD-2 license). The gated tests `HUNEX_SWITCH_DUMP_A` and `HUNEX_SWITCH_DUMP_B` check them (see CONTRIBUTING).
- The Switch decoder reproduces the 549 set A and 249 set B script streams.

## What the game accepts

Not applicable in game: the Switch formats in this document are read-only in this project. They are checked against known dumps.

## Known unknowns

- The meaning of the `.hed` row fields beyond the layout above, and how the engine chooses between the `.nam` names and the index.
- A copy-back whose distance reaches before the start of output. No sample reads such a copy, and the decoder treats the read as `0xFFFF`.
