# MPLY

Source spelling: `_MPLY(52,,`017:100)`.

## Switch build

[code] Plays BGM from source track id. Handler T `0x647c0` searches tags by code: 017 volume (default 100), 049 loop/mode (default 1), 001 fade-in duration, 011 secondary fade duration, 069 start sample frame (default 0). Volume/mode/durations are read through signed-16-bit conversion; offset uses the 32-bit reader `0x55d80` (`0x64a80–0x64c44`). Negative track stops music (`0x64ad8–0x64b08`); `0x6f0` bounds the archive lookup to ids through 131 and reads table `0x21812c`.

[code] Current track is consulted before starting (`0x64c70–0x64c78`); mode is sent to `0x9a180`, immediate start to `0x99c90`, fade-start to `0x99ef0`. Remembered volume/track are written to `0x1354e70`/`0x1354e74`, with heard-track bits at `0x64d00–0x64d2c`. The handler returns 0 at `0x64d44`: playback/fades are asynchronous, not waits. Tag 069 reaches the SDK as type 2, a raw sample-frame position (T `0xebc54–0xebc6c`, `0x120c88–0x120cc8`).

## Steam build

[code] Opcode `0x180`, handler S `0x140106d80`, pops track/slot, volume, mode, fade-in, secondary duration from five reverse-order pushes. It uses slot table `0x1405806d0`, routing negative or >=66 ids to the pending stop-command path at `0x140106f51–0x140106f5a`, `0x14010720c`. It writes a pending BGM command in state `+0x300` (0xa8-byte records), records the current music channel through `0x1405aaf28`, and returns 0 (`0x14010720c–0x140107224`). H `_MPLY:0` supplies ordinary defaults `(secondary=0, fade=0, mode=1, volume=100, slot)`.

[data] Mahoyo has 325 `0x180` calls; `raw/1_0.chs` file `0xa54` uses slot 20, volume 100, mode 1, both durations 0.

## Differences

[code] Switch ids and Steam slots have different domains; direct numeric pass-through is wrong. The carrier accepts offsets only 0..0x1fffffff, rejecting negatives/duplicates, versus Switch's 32-bit read (H `compile::compile`; T `0x64c18`). Marked same-track requests bypass Steam reuse to seek again; unmarked requests retain native comparisons. [inferred] Same-track offset behavior deserves explicit Switch comparison because T `0x64c78` can bypass a restart. Fade envelopes and out-of-stream seeks remain unverified; neither command is a playback barrier (T `0x64d44`; S `0x140107224`).

## Unverified

[inferred] Compare scr0004 page 43's track-8 cue at frame 502272 = 10.464 s, including a repeat while the same track is active, fade-in, and simultaneous voice/SE. Confirm the musical event and uninterrupted SE/voice against Switch.
