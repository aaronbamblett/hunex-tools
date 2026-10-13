# .hw audio format

## Overview

The `hunex-hw` crate reads and writes the `.hw` audio files of the Steam release of Witch on the Holy Night. A `.hw` file is a 64-byte header followed by an Ogg Vorbis stream, unchanged. The format has no official specification.

Every `.hw` payload in the sample files is Ogg Vorbis. This note does not describe Opus payloads.

## Layout

Integers are little-endian.

| Offset | Size | Contents |
|---|---|---|
| 0x00 | 4 | Header length. Always 0x40; this is where the payload starts. |
| 0x04 | 4 | The tag `hw` followed by two spaces. |
| 0x08 | 4 | Payload length: the file size minus 0x40. |
| 0x0C | 4 | Total sample frames. This equals the granule position of the last Ogg page. |
| 0x10 | 4 | Sample rate. |
| 0x14 | 4 | Channel count. |
| 0x18 | 0x28 | Zero. |

The payload is a single Ogg Vorbis logical stream. The first packet is the Vorbis identification header: packet type 1, the word `vorbis`, version 0, channel count, sample rate, and a framing bit. Every Ogg page is CRC-checked when it is read.

## Encoding rules

`Hw::parse` requires:

- a header length of 0x40 and the exact tag;
- a payload length field equal to the bytes after the header;
- zero bytes in the padding;
- an Ogg Vorbis payload whose identification header is valid;
- header fields for sample frames, sample rate and channels that equal the values in the Ogg data.

Any other input is an error, and the error says which check failed. An Ogg stream that is not Vorbis is reported as not Vorbis.

`wrap` reads the Ogg Vorbis file, takes the rate and channel count from its identification header and the frame count from its last granule, and writes the header in front of the unchanged file. The last granule must be a real position. `unwrap` returns the payload.

Checked on sample files: the seven entries of `data00300` unwrap and wrap back to the same bytes. A Vorbis file made with ffmpeg and libvorbis wraps and parses, and its frame count matches ffprobe's duration.

## What the game accepts

Verified in game (2026-10-13): a converted Switch voice clip plays in place of a stock sound in the Steam release. The clip went through `hunex nopus to-ogg`, `ffmpeg -ar 48000 -ac 2 -c:a libvorbis`, and `hunex hw wrap`, then `hfa pack --base`. A replaced `.hw` can be longer than the original.

## Known unknowns

- Mono files. All seven samples are stereo.
