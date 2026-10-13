# CompressedBG image format (.cbg)

## Overview

The `hunex-cbg` crate reads and writes CompressedBG images, the `.cbg` files of the Steam release of Witch on the Holy Night. In the install they are entries of `data00000.hfa` and `data00010.hfa`: 537 files, 14 of them 8-bit grayscale, 16 RGB, and 507 RGBA. The format has no official specification.

Checked on sample files: all 537 decode, and the canonical encoder reproduces every sample byte for byte.

## Layout

All integers are little-endian.

| Offset | Size | Field |
|---|---|---|
| 0 | 16 | `CompressedBG_MT` and a zero byte |
| 16 | 4 | width in pixels |
| 20 | 4 | height in pixels |
| 24 | 4 | stripe height, 60 in every sample |
| 28 | 4 | bits per pixel: 8, 24 or 32 |
| 32 | 16 | zeros |
| 48 | 4 per stripe | byte offset of each stripe |

There are `ceil(height / stripe height)` stripes. The first offset is the end of the table, and each stripe ends where the next begins; the last ends at the end of the file. The offsets increase strictly. The file has no key and no checksum.

### Stripes

A stripe covers `stripe height` rows, and the last covers the rows that are left. It is stored in three parts:

1. A u32 `R`: the length of the run-length stream, in bytes.
2. The 256 weights, one per byte value in ascending order, as unsigned LEB128 integers.
3. The Huffman bitstream, which decodes `R` symbols. Its last byte is padded with zero bits, and it fills the rest of the stripe exactly.

### Huffman code

Leaves are the byte values, and a leaf of weight zero is never used. Repeatedly the two unjoined nodes of smallest weight are joined under a new node whose weight is their sum. Ties go to the earlier node. The smaller node becomes the 0-child. The build stops when a new node's weight reaches the total of all weights. The code of a symbol is the path from the root, where a 0 step goes to the 0-child. A stripe with one used value has an empty code.

Bits are taken from each byte least significant bit first, and the code bits of each symbol are read in path order.

### Run-length stream

The stream is a sequence of runs that alternate, starting with a literal run. Each run starts with an unsigned LEB128 count. A literal run is followed by that many raw bytes, and a zero run produces that many zero bytes. The stream ends exactly at `R`.

## Encoding rules

### Prediction

The bytes of a stripe, after the run-length stream is expanded, are residuals: the stored channel value minus a prediction, modulo 256. The predictions use pixels of the same stripe that are already known:

- The first pixel of the stripe predicts 0.
- A pixel in the first row predicts its left neighbor.
- A pixel in the first column predicts the pixel above it.
- Any other pixel predicts the floor of the average of the pixel above and the pixel to its left.

Each channel is predicted separately.

### Channels

Stored channels are B, G, R and, for 32 bits per pixel, A. For 8 bits per pixel there is one channel, which is gray. Decoding writes R, G, B, A. The 8-bit form decodes to gray with alpha 255, and 24-bit files decode with alpha 255.

### Encoders

The canonical encoder builds the run-length stream as above, counts each byte value in it, and uses the counts as the 256 weights. The weights are written as LEB128. The resulting bitstream reproduces every sample file byte for byte.

The simple form writes the whole residual stream as one literal run, with every weight 1, so every code is 8 bits long. It is larger, about 5.7 times the original size over all samples, and it decodes to the same pixels.

## What the game accepts

Verified in game (2026-10-13): a title logo repainted with `hunex cbg from-png --like` displays in the Steam release, alpha edges included. Images with other stripe heights or sizes are not tested.

## Known unknowns

- Open: whether the game accepts stripe heights or sizes other than those in the samples.
- Premultiplication and compositing of the alpha channel. The files are read as straight alpha.
- The Ethornell `CompressedBG___` variant, which is not in these files.
