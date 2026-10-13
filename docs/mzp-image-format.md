# Background images: MZP archives, MZX0 tiles and HEP palette tiles

The hunex-mzp, hunex-mzx and hunex-image crates read and write the background images of the engine. This document describes what they implement. It is based on our reader, our encoder and the sample files; it is not an official specification.

## Container (hunex-mzp)

- Magic `mrgd00`, then a u16 entry count N.
- N records of 8 bytes, each four u16 values: the sector index, the byte within that sector, the sector span of the entry, and the low 16 bits of its size.
- The data follows the records. An entry starts at `sector * 0x800 + byte` from the start of the data.
- The entry size is the one value that matches both the low 16 bits and the stored span. The span of `[offset, offset + size)` is the number of 0x800-byte sectors it touches. The reader fails if no size or more than one size fits.
- Each entry is padded with 0xFF to the next 8-byte boundary. The last entry is padded too, so the pad is 1 to 8 bytes.

Rebuilding from the entries gives the original bytes for all 52 sample containers.

## Image header (entry 0)

Integers are little-endian.

| Offset | Size | Contents |
|---|---|---|
| 0 | 2 each | Canvas width, canvas height, tile width, tile height, tiles across, tiles down |
| 12 | 1 | Bitmap type: 0x0C is the only one decoded |
| 13 | 1 | Depth: 0 is the only one decoded |
| 14 | 1 | Crop: kept as found, 17 in every sample. Not an overlap; its meaning is unknown |
| 15 | 1 | Flags: kept as found; 1 in the samples |
| 16 | 1 per tile | Transparency: 0 all transparent, 2 all opaque, 1 mixed |

The entry is exactly 16 plus one byte per tile.

The image is the canvas: `width` by `height` from bytes 0 to 3. Tiles abut at their full size, so the tile grid covers the canvas. For the samples, `img0408` is 2560 by 1280 (5 by 5 tiles of 512 by 256) and `img1155` is 3072 by 1280 (6 by 5 tiles).

## Tiles (entry i + 1 is tile i, row-major)

Each tile is an MZX0 stream of a HEP tile:

- A 32-byte header of eight u32 fields: the magic `HEP\0`, the total size (32 + width x height + 1024), three constants (0x25300000, 6, 16 in the samples), the tile width, the tile height, and a transparency field (2 in the samples; its meaning is unknown).
- width x height palette index bytes, row by row.
- 256 palette entries of four bytes: red, green, blue and an alpha code.

The alpha code is seven bits. A code below 0x80 decodes to its value doubled, with bit 6 copied into bit 0; a code with bit 7 set means opaque. So the decodable alphas are the even values below 128 and the odd values from 129 up, plus 255. Alpha 127 is not decodable; the encoder stores the nearest decodable alpha (126).

## MZX0 (hunex-mzx)

- Magic `MZX0`, then the decompressed size in bytes. The output is a sequence of 16-bit little-endian words.
- A command byte has the command in its low two bits and an argument in the high six bits.
  - Command 0 writes the last output word `argument + 1` times. Window rule: the output is divided into windows of 4096 words, and the repeat value is zero at the start of each window. A repeat that starts on a window boundary therefore writes zeros; a repeat that starts before a boundary keeps its value across it. The decoder uses this reading (`WindowRule::Command`). The other reading, where a repeat that crosses a boundary writes zeros after it (`WindowRule::Word`), and no reset at all (`WindowRule::None`), are available for checks. On all 1737 tile streams of the 52 samples and on all 155 font atlas streams, the first two readings decode identically. Without the reset, 31 of the tile streams decode differently, so the reset is needed.
  - Command 1 reads one more byte `d` and copies `argument + 1` words from `d + 1` words back. The copy may overlap the words it writes.
  - Command 2 writes ring entry `argument`. The ring holds the last 64 literal words, and the index is the slot in the ring, not a position relative to the newest word. The samples decode to the spec's hashes with this reading.
  - Command 3 writes `argument + 1` literal words, which follow the command byte.
- The output is cut to the decompressed size, and a final command may run past it. The decoder requires the stream to end exactly after the last command.

The encoder is greedy. It writes back-references of at least two words within 255 words back, and literals in runs of up to 64 words. A run of a repeated word is written as a back-reference at distance 1 that overlaps its own output, not as a repeat command. It does not use the ring command. A literal-only encoder is also available.

Repeat semantics in the game are not fully known. The window reading above is what the reference suggests, and the originals decode identically under either reading, so the samples cannot settle it. The encoder writes no repeat commands. Each run of up to 64 words is a distance-1 back-reference, which takes two bytes where a repeat would take one.

The original encoder stays inside an envelope, and the decoder's analyser (`hunex_mzx::Envelope`) checks the same limits. Measured on the 1737 sample tile streams, the 25 img1227 streams and the 155 font atlas streams:

- Back-reference distance is at most 255 words (stored byte 254). Distance 256 (stored byte 255) never occurs.
- No repeat, back-reference or ring command crosses a 4096-word window boundary. Literal commands do cross them.
- No back-reference source lies before the start of its window.
- Ring references can point into an earlier window: 212,899 of 6,961,730 ring uses in the samples do. The ring is not reset per window. The encoder emits no ring commands.

The encoder's output on the samples, on the img1227 edit and on the font atlases stays inside the envelope.

The encoder also never lets a repeat, back-reference or ring command write the first word of a window (position 0 included). The first word of every window is written by a literal, which may start before the boundary and cross it, or start on it. The window rule therefore cannot change any word the encoder writes, whatever value the game resets to. The original streams do use repeats at window starts (4,062 times in the 20 samples checked), so this rule is the encoder's and not part of the envelope. `Envelope::window_starts_not_literal` counts violations. The encoder output decodes identically under all three readings.

## Assembling and encoding (hunex-image)

Tiles abut. Tile column `c` and row `r` (tile `r * across + c`) cover canvas pixels from `(c * tile width, r * tile height)`, at full size, with no overlap and no crop. The image is the canvas. The crop byte is read and written unchanged, and its meaning is unknown.

Decoding gives an RGBA image of `width x height x 4` bytes. The two sample images decode to these SHA-256 hashes of their RGBA bytes:

- `img0408.mzp`, 2560 x 1280: `51a511b48599f67f2300fbb70c6647941830a6cdbc69caf8911d60b430c8c8a3`
- `img1155.mzp`, 3072 x 1280: `f9660925bd221764c12b2581d7656844e10b2e392d40b9c462ecb348c0748b1b`

Encoding without a template works per tile. The image is padded with transparent pixels to whole 512 by 256 tiles, and each tile is the image window at `(c * tile width, r * tile height)`:

1. The tile's 512 by 256 pixels have 256 colours or fewer: the palette is exactly those colours, in first-seen order. Otherwise a median cut reduces them to 256 colours, and each pixel takes the nearest palette colour. This path is lossy.
2. Alpha is stored as the nearest decodable code.
3. The transparency byte is 0 if every alpha in the tile is 0, 2 if every one is 255, and 1 otherwise.
4. The HEP header is written with the observed constants and transparency 2. The palette is padded to 256 entries with zeros.
5. The tile is compressed with the greedy MZX encoder.

With `--like`, the encoder starts from the original archive, and the image must be the canvas size. It decodes the original image, and compares each pixel of the new image with that decoded image:

- A pixel that is unchanged keeps the original tile's own pixel at that tile position, so the original tile bytes are kept.
- A changed pixel takes the new value.
- A tile with no changed pixel is copied from the original container byte for byte, with its original transparency byte.
- A changed tile is encoded as above. Its palette is exact when it has 256 colours or fewer, and it is quantised otherwise. Only changed tiles are quantised.

The header is rebuilt from the original header fields. Each re-encoded tile is reported with its changed-pixel count, colour count, whether it was quantised, and its largest and mean per-channel error against the intended tile.

## Measured results

- Both sample images decode to the hashes above, at the canvas sizes.
- With `--like`, an unmodified PNG gives back the original container byte for byte, for all 52 samples.
- With `--like`, the red rectangle repaint (x 150 to 1349, y 350 to 699, filled with 200, 30, 30) re-encodes only the tiles it overlaps: 6 tiles on `img0408` (19 copied), and 6 tiles on `img1155` (24 copied). All 12 are exact, with max error 0.
- Without `--like`, `img1155` round-trips exactly (every pixel, max error 0), and so does `img0408` through the repaint test. The rebuilt containers are not byte-identical to the originals: `img0408` 64,928 bytes against 33,952, and `img1155` 2,917,512 against 2,674,144.
- Seam continuity over all 52 samples: the mean difference across a tile seam (last column against the next first column, or last row against the next first row) is compared with the mean difference between adjacent pixels inside tiles. `img0408` is 0.000 against 0.025 horizontally and 0.000 against 0.041 vertically. `img1155` is 0.876 against 0.527 (1.66x) and 0.672 against 0.485 (1.39x). Forty-nine samples are within 2.5x on both axes; the worst of those is `img0208` at 2.07x. Three are over: `img0572` horizontal 2.60x, `img0624` vertical 4.60x, and `img0728` vertical 3.51x. Their seams are sharp edges in the picture; a row search found no better alignment, so they are not shown to be wrong. Whether the game shows them as continuous is not known.

## Unknowns

- Whether the game accepts rebuilt images, and literal-only MZX streams. Nothing here has been run in the game.
- What the transparency field in the HEP header, byte 15 of the image header, and the three constant HEP fields mean.
- The meaning of the crop byte (17 in every sample). It is not an overlap.
- Whether the game reads the seams of the three samples listed above as continuous.
- The zero padding of short palettes in the original files. The encoder writes zeros.
- The mid-range alpha mapping. The samples use only 0 and 127.
- Which of the two window readings the game uses. They differ only for a repeat that crosses a boundary, and no sample has one. The encoder never writes such a repeat.
- Image types other than 0x0C, and depth 0x11. They are rejected.

## Limits

The decoders refuse sizes beyond these limits, and check them before allocating: a canvas of more than 64 Mpx (`MAX_CANVAS_PIXELS`), a tile stream that declares more output than one tile needs, and an MZX0 stream declaring more than 256 MiB (`MAX_DECOMPRESSED`). The header fields of an edited image must fit in u16; a larger image is an error. The crop byte is kept as read and has no meaning here.

## Font atlases (bitmap type 0x01)

A font atlas has the same two-entry shape as the other image archives, but it is one 1024 by 1024 image of palette indices, not HEP tiles. The header entry is 16 bytes of header, the 256-entry palette, and one transparency byte. The header's bitmap type is 0x01, and the pixel entry is one MZX0 stream of the indices. See [ccit-format.md](ccit-format.md) for the layout and its open questions. `from-png` needs `--like` for an atlas, since the template fixes the size.

Atlas palettes are stored in swizzled order: index i reads palette slot (i & 0xE7) | ((i & 0x08) << 1) | ((i & 0x10) >> 1). The map swaps bits 3 and 4 and is its own inverse. HEP tiles (type 0x0C) are not swizzled. An unchanged atlas is written back as the original container; an edited one gets a new palette, written in swizzled order.
