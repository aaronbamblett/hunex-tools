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

### Writing MRG sets

Checked on sample files: the seven RomFS sets and `jallpac` rebuild from their own index, names and entries, and the rebuilt index, name table and data match the originals byte for byte (`switch_mrg_write` gated test).

- Each index row is 8 bytes: the first sector (u32), the sectors on disk (u16) and the decompressed sectors (u16).
- Entries are placed back to back from sector 0. An entry's first sector is the previous entry's first sector plus its sectors on disk. Empty slots have zero rows.
- The index ends with two rows of `0xFF` bytes.
- The decompressed sector count is `ceil(decompressed size / 2048)` for an `NXCX` or `NXGX` entry, read from bytes 4 to 8 of the entry, and equal to the sectors on disk for every other entry.
- Each name row is 32 bytes: the name, zero padded to 30 bytes, then CR LF. The name table has one row per index row, then an all-zero row. Empty slots are named `NOFILE.EXISTS`.
- An entry's data is whole sectors. The bytes after the entry's own data, up to the sector end, are kept as they were in the original. In the sample sets they are zero in some entries and not zero in others, so a writer that pads with zeros does not reproduce every original.
- One set, `w_allpaccg`, has one name row more than it has index rows. The row has an empty name. The writer keeps it when the caller passes it.

### Writing `allscr.mrg`

Checked on sample files: dump, then pack with `--base`, reproduces `allscr.mrg` byte for byte (`switch_pack` gated test). Editing one script changes only that entry.

- The container uses the layout of the PC containers: each record holds the sector, the byte within the sector, the sector span of the entry, and the low 16 bits of its size. Each entry is followed by `0xFF` padding to the next eight-byte boundary, the last entry included.
- The stored span is the number of sectors the entry touches, so the Switch size rule gives the right size for every entry except three.
- The three entries at indices 143, 318 and 440 have a span of 32 and a size of 62,174, 63,976 and 62,405 bytes. The Switch size rule gives a size 65,536 bytes larger, which runs into the next entry. The stream itself ends where its last command ends, so the writer and `switch-script pack` use the stream's own length.
- Entries 1 and 2 are kept as stored. Entry 0 is the name table.

### Encoding Switch streams

Checked on sample files: the encoder output decodes to the same script for all 549 Tsukihime streams and all 249 set B dumps (`switch_mzx_envelope` gated test). The output is 1.104 times the size of the original streams, because the encoder does not use the ring.

Measured on the 549 Tsukihime streams:

- The original streams use 1,274,757 copies, 1,141,712 literal runs, 51,573 repeats and 1,002,814 ring recalls.
- No copy or repeat crosses a 4096-word window. No copy source lies before its window. The largest copy distance is 255 words, and distance 256 never occurs.
- No repeat starts a window. 992 non-literal commands start a window, all of them copies or ring recalls.
- A ring recall reads an entry written in an earlier window 46,206 times of 1,002,814.

The encoder writes literal runs of up to 64 words and copies of 2 to 64 words at distances up to 255 words, inside their window. It writes no repeats and no ring recalls, and it starts every window with a literal. These are the PC encoder's rules, which the Switch streams satisfy as well.

## What the game accepts

Open: the writers are checked against the sample files. The game has not loaded a rebuilt Switch archive or stream.

## Known unknowns

- The meaning of the `.hed` row fields beyond the layout above, and how the engine chooses between the `.nam` names and the index.
- A copy-back whose distance reaches before the start of output. No sample reads such a copy, and the decoder treats the read as `0xFFFF`.
- The game's size for the three `allscr` entries whose Switch size rule overruns. The game may read them by stream, as this crate does, or by the rule.
- The bytes after an MRG entry's data. Their values are not derived from the entry, so the writer keeps them only when they come from the original.
- Whether the game accepts streams written without ring recalls, or with the window-start literal rule.
