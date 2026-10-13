# Background images: MZP containers, MZX0 tiles and HEP palette tiles

## Overview

A background image is an MZP container. The `hunex-mzp` crate reads and writes the container, `hunex-mzx` decodes and encodes its MZX0 streams, and `hunex-image` assembles the tiles into an RGBA image and back. The format has no official specification.

Checked on sample files: `img0408.mzp` and `img1155.mzp` decode to the SHA-256 hashes listed below, and an unmodified container rebuilds byte for byte from its entries.

### Sample hashes

The two sample images decode to these SHA-256 hashes of their RGBA bytes:

- `img0408.mzp`, 2560 x 1280: `51a511b48599f67f2300fbb70c6647941830a6cdbc69caf8911d60b430c8c8a3`
- `img1155.mzp`, 3072 x 1280: `f9660925bd221764c12b2581d7656844e10b2e392d40b9c462ecb348c0748b1b`

## Layout

### Container (hunex-mzp)

- Magic `mrgd00`, then a u16 entry count N.
- N records of 8 bytes, each four u16 values: the sector index, the byte within that sector, the sector span of the entry, and the low 16 bits of its size.
- The data follows the records. An entry starts at `sector * 0x800 + byte` from the start of the data.
- The entry size is the one value that matches both the low 16 bits and the stored span. The span of `[offset, offset + size)` is the number of 0x800-byte sectors it touches. The reader fails if no size or more than one size fits.
- Each entry is padded with 0xFF to the next 8-byte boundary. The last entry is padded too, so the pad is 1 to 8 bytes.

### Image header (entry 0)

Integers are little-endian.

| Offset | Size | Contents |
|---|---|---|
| 0 | 2 each | Canvas width, canvas height, tile width, tile height, tiles across, tiles down |
| 12 | 1 | Bitmap type: 0x0C is the only one decoded |
| 13 | 1 | Depth: 0 is the only one decoded |
| 14 | 1 | Crop: kept as found, 17 in every sample. Its meaning is unknown |
| 15 | 1 | Flags: kept as found; 1 in the samples |
| 16 | 1 per tile | Transparency: 0 all transparent, 2 all opaque, 1 mixed |

The entry is exactly 16 plus one byte per tile. The image is the canvas: `width` by `height` from bytes 0 to 3.

### Tiles (entry i + 1 is tile i, row-major)

Each tile is an MZX0 stream of a HEP tile. The decompressed tile is:

- A 32-byte header of eight u32 fields: the magic `HEP\0`, the total size (32 + width x height + 1024), three constants (0x25300000, 6, 16 in the samples), the tile width, the tile height, and a transparency field (2 in the samples; its meaning is unknown).
- width x height palette index bytes, row by row.
- 256 palette entries of four bytes: red, green, blue and an alpha code.

The alpha code is seven bits. A code below 0x80 decodes to its value doubled, with bit 6 copied into bit 0; a code with bit 7 set means opaque. So the decodable alphas are the even values below 128 and the odd values from 129 up, plus 255. Alpha 127 is not decodable; the encoder stores the nearest decodable alpha (126).

Tiles abut at their full size. Tile column `c` and row `r` (tile `r * across + c`) cover canvas pixels from `(c * tile width, r * tile height)`, with no overlap and no crop. For the samples, `img0408` is 2560 by 1280 (5 by 5 tiles of 512 by 256) and `img1155` is 3072 by 1280 (6 by 5 tiles).

### MZX0 (hunex-mzx)

- Magic `MZX0`, then the decompressed size in bytes. The output is a sequence of 16-bit little-endian words.
- A command byte has the command in its low two bits and an argument in the high six bits.
  - Command 0 writes the last output word `argument + 1` times. Window rule: the output is divided into windows of 4096 words, and the repeat value is zero at the start of each window. The decoder uses this reading (`WindowRule::Command`). Two other readings, `WindowRule::Word` and `WindowRule::None`, are available for checks.
  - Command 1 reads one more byte `d` and copies `argument + 1` words from `d + 1` words back. The copy may overlap the words it writes.
  - Command 2 writes ring entry `argument`. The ring holds the last 64 literal words, and the index is the slot in the ring, not a position relative to the newest word. The samples decode to the reference hashes with this reading.
  - Command 3 writes `argument + 1` literal words, which follow the command byte.
- The output is cut to the decompressed size, and a final command may run past it. The decoder requires the stream to end exactly after the last command.

### Font atlases (bitmap type 0x01)

A font atlas has the same two-entry shape, but it is one 1024 by 1024 image of palette indices, not HEP tiles. The header entry is 16 bytes of header, the 256-entry palette, and one transparency byte. The pixel entry is one MZX0 stream of the indices. Atlas palettes are stored in swizzled order: index i reads palette slot (i & 0xE7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1). HEP tiles (type 0x0C) are not swizzled. The atlas layout is described in [ccit-format.md](ccit-format.md).

## Encoding rules

### Assembling the image

Decoding gives an RGBA image of `width x height x 4` bytes.

Encoding without a template works per tile. The image is padded with transparent pixels to whole 512 by 256 tiles, and each tile is the image window at `(c * tile width, r * tile height)`:

1. The tile's pixels have 256 colors or fewer: the palette is exactly those colors, in first-seen order. Otherwise a median cut reduces them to 256 colors, and each pixel takes the nearest palette color. This path is lossy.
2. Alpha is stored as the nearest decodable code.
3. The transparency byte is 0 if every alpha in the tile is 0, 2 if every one is 255, and 1 otherwise.
4. The HEP header is written with the observed constants and transparency 2. The palette is padded to 256 entries with zeros.
5. The tile is compressed with the greedy MZX encoder.

With `--like`, the encoder starts from the original archive, and the image must be the canvas size:

- A pixel that is unchanged keeps the original tile's own pixel at that tile position, so the original tile bytes are kept.
- A changed pixel takes the new value.
- A tile with no changed pixel is copied from the original container byte for byte, with its original transparency byte.
- A changed tile is encoded as above. Only changed tiles are quantized.

The header is rebuilt from the original header fields. Each re-encoded tile is reported with its changed-pixel count, color count, whether it was quantized, and its largest and mean per-channel error against the intended tile.

### MZX0 encoder

The encoder is greedy. It writes back-references of at least two words within 255 words back, and literals in runs of up to 64 words. A run of a repeated word is written as a back-reference at distance 1 that overlaps its own output, not as a repeat command. It does not use the ring command. A literal-only encoder is also available.

The original encoder stays inside an envelope, which the decoder's analyzer (`hunex_mzx::Envelope`) checks. On the sample tile streams, the font atlas streams, and the stream of the `img1227` background with a painted rectangle:

- Back-reference distance is at most 255 words (stored byte 254). Distance 256 (stored byte 255) does not occur.
- No repeat, back-reference or ring command crosses a 4096-word window boundary. Literal commands do cross them.
- No back-reference source lies before the start of its window.
- Ring references can point into an earlier window. The ring is not reset per window. The encoder emits no ring commands.

The encoder never lets a repeat, back-reference or ring command write the first word of a window. The first word of every window is written by a literal, so the window rule cannot change any word the encoder writes. The original streams do use repeats at window starts, so this rule is the encoder's and not part of the envelope. `Envelope::window_starts_not_literal` counts violations.

### Limits

The decoders refuse sizes beyond these limits, and check them before allocating: a canvas of more than 64 Mpx (`MAX_CANVAS_PIXELS`), a tile stream that declares more output than one tile needs, and an MZX0 stream declaring more than 256 MiB (`MAX_DECOMPRESSED`). The header fields of an edited image must fit in u16; a larger image is an error.

## What the game accepts

- Verified in game (2026-10-13): a static background, repainted with `hunex image from-png --like`, displays cleanly in the Steam release. Tile seams are continuous in game (the abutting-tile model is confirmed).
- Verified in game: literal and back-reference MZX0 streams load in the Steam release of Witch on the Holy Night. Streams with repeat commands showed one wrong row per 4096-word window, so the encoder writes no repeat commands.
- Open: images other than the static background test.
- Open: whether the game reads the seams of the other samples as continuous. Three samples have a seam whose mean difference is more than 2.5 times the mean difference inside tiles: `img0572` (horizontal), `img0624` (vertical) and `img0728` (vertical).

## Known unknowns

- What the transparency field in the HEP header, the crop byte at image offset 14, and the three constant HEP fields mean. The crop byte is 17 in every sample, and it is not an overlap.
- The zero padding of short palettes in the original files. The encoder writes zeros.
- Open: the mid-range alpha mapping. The samples use only 0 and 127.
- Open: the game's rule for repeat commands. The encoder writes none.
- Image types other than 0x0C, and depth 0x11. They are rejected.
