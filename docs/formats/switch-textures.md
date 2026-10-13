# Switch textures: NXX, BNTX and BC decoding

## Overview

The `hunex-nx` crate reads the textures of the Switch RomFS. It unwraps the NXX entry, reads the BNTX container, reverses the block-linear layout, and decodes the pixels to RGBA. This note describes what is implemented. The format has no official specification.

Recorded checks: background `IMG5012` from set A (1280 x 720, BC7, entry 2598 of `w_allpac`) and background `IMG0179` from set B (1400 x 840, BC7, entry 850 of `jallpac`) decode to the SHA-256 hashes listed below. The gated tests check both.

### Sample hashes

- `IMG5012` (set A), decoded RGBA: `ef58e090d43ed16f91c058d47feb16f1e53fe86f1ddac744d45d70530d879579`
- `IMG0179` (set B), decoded RGBA: `96de0854b88fa552ddcba7e1ee6ea3072f9bea4dc483b8e2889be5d70a3ce8b6`

## Layout

### BNTX container

Little-endian throughout, with a byte-order mark stored as the bytes `FF FE`.

- Magic `BNTX` at 0x00, the byte-order mark at 0x0C, and the tag `NX  ` at 0x20.
- The texture count (u32) at 0x24.
- The offset of the texture pointer table (u64) at 0x28. The table has one u64 per texture, giving that texture's BRTI record.
- The offset of the data block `BRTD` (u64) at 0x30, and of the dictionary (u64) at 0x38.
- An optional name table `_STRP` between the pointer table and the dictionary: magic, a u64 size at +8, a u64 count at +0x10, then names as a u16 length and bytes, from +0x18.

### BRTI records

Each record starts with the magic `BRTI`. The fields used are the format code (u32 at +0x1C), the width and height (u32 at +0x24 and +0x28), the layout word (+0x38), the image size (u32 at +0x50), and the alignment (u32 at +0x54).

Texture data starts at `BRTD + 0x10` for the first texture. Each later texture starts at the end of the previous one, rounded up to its alignment. The pointer fields inside the record are not offsets and are not used.

### Formats

| Code | Decoded as | Notes |
|---|---|---|
| 0x2006, 0x2001 | BC7, 16 bytes per 4 x 4 block | Values used as stored. |
| 0x1D01 | BC4, 8 bytes per block | Single channel, shown as gray with full alpha. |
| 0x0B06 | RGBA8, 4 bytes per pixel | |
| 0x1C06 | BC3 | Color from the color block, alpha from the alpha block (standard BC3). Confirmed on the two set B masks, which are white with the shape in alpha. |

Block decoding uses the `bcdec_rs` crate (MIT).

### Block-linear layout

Rows of units are stored in GOBs: 64 bytes wide by 8 rows, 512 bytes each. A unit is one pixel for RGBA8 and one 4 x 4 block for BC formats.

- The row width in bytes is the unit columns times the bytes per unit. The GOB column count is that width divided by 64, rounded up.
- The block height starts at 16. While it is above 1 and the unit rows are at most half the block height times 8, it halves.
- The stored size is the block rows times the block height times the GOB columns times 512. The decoder checks this against the image size and fails if they differ.

Within a GOB, the byte at column `x` (0 to 63) and row `y` (0 to 7) is at offset `(x & 15) | ((y & 1) << 4) | (((x >> 4) & 1) << 5) | (((y >> 1) & 3) << 6) | (((x >> 5) & 1) << 8)`. GOBs of a block are stacked by row, a block is the block height in GOBs tall, and blocks run across the GOB columns and then down.

## Encoding rules

- `hunex nx to-png` writes one texture as PNG (RGBA, 8 bits). `hunex nx info` lists the textures and says whether each one decodes.
- Textures above 16384 pixels on a side, or above 64 Mpx in total (`MAX_TEXTURE_SIDE`, `MAX_TEXTURE_PIXELS`), are an error. An NXX wrapper that unwraps to more than 256 MiB (`MAX_UNWRAPPED_BYTES`) is an error, and inflation stops at the declared size.
- Pointers and offsets are checked, so a crafted container is an error rather than a panic.

## What the game accepts

Not applicable in game: this format is read-only in this project. The tools decode textures and do not write BNTX.

## Known unknowns

- Formats other than the four in the table.
- The block-linear rules for block heights and sizes other than those in the samples.
