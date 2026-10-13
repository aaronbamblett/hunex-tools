# Nintendo OPUS audio and Ogg Opus remux

## Overview

The Switch release stores its audio as Nintendo OPUS files in the RomFS `stream` folder. The `hunex-nopus` crate reads them and remuxes the packets to Ogg Opus (RFC 7845). The packets are not re-encoded. The format has no official specification.

Checked on sample files: all 16,601 files parse and remux (16,468 mono and 133 stereo), and ffmpeg decodes the remuxed sample files with exit status 0.

## Layout

### Basic chunk (0x20 bytes, little-endian)

| Offset | Size | Contents |
|---|---|---|
| 0x00 | 4 | Chunk id 0x80000001. |
| 0x04 | 4 | Chunk size, 0x18. Other sizes are rejected. |
| 0x08 | 1 | Version, 0. Other versions are rejected. |
| 0x09 | 1 | Channel count, 1 or 2. Zero and more than two are rejected. |
| 0x0A | 2 | Frame size, 0 (variable packets). Other values are rejected. |
| 0x0C | 4 | Sample rate. Stored in the Ogg Opus header; the samples use 48000 Hz. |
| 0x10 | 4 | Offset of the data chunk, which must be 0x20. |
| 0x14 | 4 | Must be zero. |
| 0x18 | 4 | Offset of a context chunk, which must be zero. Context chunks are not supported. |
| 0x1C | 2 | Pre-skip, in samples at 48 kHz. |
| 0x1E | 2 | Padding; not checked. |

### Data chunk (at 0x20)

- Chunk id 0x80000004.
- A size counted from offset 0x28 to the end of the file. Other sizes are rejected.

### Packets

The data chunk holds packets back to back. Each packet is a big-endian u32 length, a 4-byte field that this crate ignores, and the Opus packet, which is the length given above. A zero length, or a packet that runs past the end of the file, is an error.

## Encoding rules

### Samples per packet

The first byte of a packet is its TOC byte (RFC 6716). The crate reads the configuration (the top five bits) to get the frame size at 48 kHz, and the code (the low two bits) to get the number of frames: one for code 0, two for codes 1 and 2, and the count in the low six bits of the second byte for code 3. A packet must have 1 to 48 frames and no more than 120 ms in total. A byte that breaks these rules is an error.

### Ogg Opus remux

The output is an Ogg Opus stream with one packet per page, in this order:

1. An OpusHead page: version 1, the channel count, the pre-skip, the input sample rate, output gain 0, and mapping family 0. It is the only packet on its page and has the beginning-of-stream flag.
2. An OpusTags page with the vendor string `hunex` and no comments.
3. One page per audio packet, with its bytes copied unchanged. The granule position is the running total of samples from the first packet, including pre-skip. The last page has the end-of-stream flag.

Every page has a CRC-32 (polynomial 0x04C11DB7, no reflection, initial value 0, no final XOR). The stream uses a fixed serial number.

Only mono and stereo are accepted. Layouts this crate does not understand are reported as unsupported.

### Writing a Nintendo OPUS file

`hunex nopus from-ogg` writes the file from an Ogg Opus stream. Every value in the header is fixed in the sample files, and the writer writes it: version 0, frame size 0, data offset 0x20, zero reserved fields, sample rate and pre-skip from the Ogg Opus header, context chunk 0, padding 0. The data chunk size counts the bytes after its own header. Each packet is its big-endian length, its 4-byte field, and its bytes.

Checked on sample files: over 500 files, the structural fields have one value each, and the packets end exactly at the end of the file.

### The packet fields

The 4-byte field before each packet is not a checksum or a length of the packet. It is 1 in 21,408 of the 137,372 packets in 500 files, and it varies in the rest. Ogg Opus has no place for it, so `nopus to-ogg` stores the fields, little-endian and in hex, in an `OpusTags` comment named `HUNEX_NINTENDO_FIELDS`. The tags packet spans as many pages as the comment needs. `from-ogg` reads the comment back. An Ogg Opus file without the comment gets zero fields, and the command says so.

Checked on sample files: `to-ogg` then `from-ogg` reproduces 507 files byte for byte: the three named samples and every 33rd file of the stream folder (`nopus_roundtrip` gated test). The Ogg output also decodes with `ffmpeg`.

## What the game accepts

Verified indirectly (2026-10-13): a Nintendo OPUS clip, remuxed with `hunex nopus to-ogg` and converted to Vorbis, plays in the game through the `.hw` test. The Ogg Opus file itself was not played. Open: whether the game accepts a Nintendo OPUS file written by `from-ogg`.

## Known unknowns

- The meaning of the 4-byte field before each packet. It may be the encoder's final range, which would need a full decode of each packet to check. The field is carried in the Ogg comment until its meaning is known.
- Whether the game reads the 4-byte field at all. The reader in this crate ignores it.
- The meaning of header field 0x14. It is zero in every sample.
- The layout of a context (loop) chunk. No sample has one.
- Open: whether the decoded audio matches another decoder's output bit for bit. Only ffmpeg was used.
- Channel counts above two, and other mapping families.
