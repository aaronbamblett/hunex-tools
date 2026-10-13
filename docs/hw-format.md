# .hw audio format

The hunex-hw crate reads and writes the `.hw` audio files of the Steam release. A `.hw` file is a 64-byte header followed by an Ogg Vorbis stream, unchanged. The description is based on our reader and on the sample files. It is not an official specification.

## Header

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

## Payload

The payload is a single Ogg Vorbis logical stream. The first packet is the Vorbis identification header: packet type 1, the word `vorbis`, version 0, channel count, sample rate, and a framing bit. The samples have 2 channels at 48000 Hz and block sizes given in the identification header. Every page is CRC-checked when it is read.

## Reading

`Hw::parse` requires:

- a header length of 0x40 and the exact tag;
- a payload length field equal to the bytes after the header;
- zero bytes in the padding;
- an Ogg Vorbis payload whose identification header is valid;
- header fields for sample frames, sample rate and channels that equal the values in the Ogg data.

Any other input is an error, and the error says which check failed. An Ogg stream that is not Vorbis is reported as not Vorbis.

## Writing

`wrap` reads the Ogg Vorbis file, takes the rate and channel count from its identification header and the frame count from its last granule, and writes the header in front of the unchanged file. The last granule must be a real position, not "no granule".

`unwrap` returns the payload.

## Checks on the samples

- The seven entries of `data00300` unwrap and wrap back to the same bytes.
- Their sample frames match the durations recorded in our notes. Each is 48000 Hz and stereo.
- A Vorbis file made with ffmpeg and libvorbis wraps and parses; its frame count matches ffprobe's duration.

## Unknowns

- Whether the game plays a rebuilt `.hw` file. This has not been tested in the game.
- Whether the game plays Opus in Ogg. The engine's strings point to Vorbis only, so Opus is not used.
- Mono files. All seven samples are stereo.
