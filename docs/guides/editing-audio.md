# Editing audio

The game's voice and sound effects are `.hw` files. A `.hw` file is a 64-byte header followed by an Ogg Vorbis stream. The `hw` commands read the header and convert between `.hw` and Ogg. This guide replaces a stock sound in `data00300.hfa` with a voice clip from the Switch release. The clip goes through Nintendo OPUS, then Ogg Opus, then Ogg Vorbis, then `.hw`.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- `ffmpeg` built with the libvorbis encoder, on your `PATH`. Check it with `ffmpeg -encoders | grep libvorbis`.
- `sed` and `cmp`. The commands here do not need `sed -i`.

## Get the sound archive and a Switch clip

The stock sounds are in `data00300.hfa`:

```sh
mkdir -p work/audio
cp "<game folder>/data00300.hfa" work/audio/
hunex hfa unpack work/audio/data00300.hfa -o work/audio/unpacked
```

Output:

```text
unpacked 7 entries to work/audio/unpacked
```

The voice clip is a Nintendo OPUS file in the Switch release's RomFS, in its `stream` folder. Copy the clip from your RomFS folder:

```sh
cp "<RomFS folder>/stream/A00_01_00_0000_0.opus" work/audio/
```

## Read a `.hw` file

```sh
hunex hw info work/audio/unpacked/SSEDecided.hw
```

Output:

```text
file size:      32611 bytes
payload:       32547 bytes (Ogg Vorbis)
sample rate:   48000 Hz
channels:      2
sample frames: 77712 (1.619000 s)
```

Unwrap the payload to an Ogg Vorbis file:

```sh
hunex hw unwrap work/audio/unpacked/SSEDecided.hw -o work/audio/stock.ogg
```

The command prints nothing on success.

## Convert the Switch clip

Inspect the clip first:

```sh
hunex nopus info work/audio/A00_01_00_0000_0.opus
```

Output:

```text
file size:      65709 bytes
channels:       1
input rate:     48000 Hz
pre-skip:       120 samples
packets:        297
total samples:  285120 (48 kHz, with pre-skip)
decoded length: 285000 samples (5.937500 s)
```

Remux it to Ogg Opus. The audio packets are copied as they are:

```sh
hunex nopus to-ogg work/audio/A00_01_00_0000_0.opus -o work/audio/clip.ogg
```

Convert the Ogg Opus file to Ogg Vorbis at 48 kHz and stereo, the format `.hw` files use:

```sh
ffmpeg -hide_banner -loglevel error -y -i work/audio/clip.ogg -ar 48000 -ac 2 -c:a libvorbis work/audio/clip-vorbis.ogg
```

Wrap the Vorbis file in a `.hw` header:

```sh
hunex hw wrap work/audio/clip-vorbis.ogg -o work/audio/clip.hw
hunex hw info work/audio/clip.hw
```

Output:

```text
file size:      57696 bytes
payload:       57632 bytes (Ogg Vorbis)
sample rate:   48000 Hz
channels:      2
sample frames: 285000 (5.937500 s)
```

`hw wrap` accepts Ogg Vorbis only. Giving it the Opus file fails:

```sh
hunex hw wrap work/audio/A00_01_00_0000_0.opus -o work/audio/bad.hw
```

Output:

```text
hunex: work/audio/A00_01_00_0000_0.opus: no Ogg page at 0x0
```

## Replace the stock sound

Copy the new file over the stock name in the unpacked folder, and pack the archive with `--base`:

```sh
cp work/audio/clip.hw work/audio/unpacked/SSEDecided.hw
hunex hfa pack work/audio/unpacked -o work/audio/data00300-edited.hfa --base work/audio/data00300.hfa
hunex hfa list work/audio/data00300-edited.hfa | head -3
```

Output:

```text
packed 7 entries into work/audio/data00300-edited.hfa
7 entries
   0      57696  raw    SSEDecided.hw
   1       7565  raw    SSECancelled.hw
```

The other six sounds keep their original bytes. A replaced `.hw` can be longer than the original.

Check that they did:

```sh
hunex hfa unpack work/audio/data00300-edited.hfa -o work/audio/check
cmp work/audio/check/SSECancelled.hw work/audio/unpacked/SSECancelled.hw && echo "other sounds unchanged"
```

Output:

```text
other sounds unchanged
```

Verified in game: a converted voice clip played in place of the stock sound in the Steam release of Witch on the Holy Night.

Format details: [audio format note](../formats/hw-format.md).
