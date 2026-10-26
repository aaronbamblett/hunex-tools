# SEFD

Source spelling: `_SEFD(1,,,`001:2000)`.

## Switch build

[code] Fades/stops SE selected by operand 0; tag 001 supplies signed-16-bit fade duration. Handler T `0x6aee0` searches tags (001 at `0x6b200`) and reads only channel by position at `0x6b23c`. Channel is reduced modulo 20; a negative result selects all channels 0..19 (`0x6b248–0x6b2c0`). It cancels a queued start where appropriate, invokes fade `0x9a830(channel,duration)` for positive duration or stop `0x9a7a0` otherwise, clears remembered id/volume state, then returns 0 (`0x6b2ec–0x6b350`, `0x6b29c–0x6b2b4`).

[code] Tag 068 routes to deferred action 2 via `0x65fe0` (`0x6b1a0–0x6b1fc`). This queues a channel operation; neither deferred execution nor fading yields the VM. [inferred] Audible intent is fade to silence and stop, not wait for a sound's natural end (T `0x6b33c–0x6b34c`).

## Steam build

[code] Opcode `0x195`, S `0x140107fb0`; H `_SEFD:0` emits reverse pushes `(delay=0, duration, unused=0, channel)` or a delay-bearing variant. The handler discards the unused field, reads channel/duration/delay, bounds a single channel against state `+0x330/+0x338`, and marks record `+0/+0x70/+0x74=1`, `+0x78=duration`, `+0x7c=delay`; returns 0 (`0x140107fb0–0x140108053`). Unlike SESP's handler, this handler has no negative-channel all-record loop.

[data] Mahoyo has 3,071 calls; `raw/1_0.chs` file `0x62a8` uses channel 0, duration 5000, delay 0.

## Differences

[code] Switch negative-channel SEFD iterates channels; Steam `0x195` tests its zero-extended channel against vector length at S `0x140108024–0x140108027`, skips writes on failure, and returns. Before `sefd-all-channels`, a source SEFD(-1...) could leave all SE playing (T `0x6b27c–0x6b2b8`; H `_SEFD:0`; S `0x140108024–0x140108053`). This differs from SESP(-1), which has an explicit all-channel path. [inferred] Later scene transitions can retain loops, even though no SEFD wait itself can hang. Delay ordering and signed-16-bit versus dword durations are additional fidelity risks (T `0x6b230`; S `0x140108049`).

## Unverified

[inferred] Start two looping SE, execute an authored negative-channel SEFD with a two-second fade, then a marker. Compare both channels' gain and stop behavior with Switch, including a delayed variant.
