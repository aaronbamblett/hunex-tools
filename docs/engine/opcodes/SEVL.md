# SEVL

Source spelling: `_SEVL(0,,,,`001:3000,,`017:30)`.

## Switch build

[code] Sets SE channel volume, with tag 001 ramp duration and tag 017 target. Handler T `0x6a9c0` searches tags; tag 068 queues action 3 through `0x65fe0` (`0x6ac70–0x6acf8`). Immediate duration is signed-16-bit, target clamped to 0..100, channel reduced modulo 20 (`0x6ad20–0x6adb4`). A queued-start record gets its volume byte updated at `0x6add4–0x6ae00`; otherwise character-category settings can force gain 0 (`0x6ae04–0x6ae40`).

[code] Reads remembered channel volume at `0x1354e58+0x8c+2*channel`, computes ramp rate and calls `0x9a8c0`, then stores the target (`0x6ae60–0x6aeb4`). It returns 0, with no playback or ramp-completion wait (`0x6aec0–0x6aed4`). Audible intent is an overlapping SE gain ramp.

## Steam build

[code] Opcode `0x196`, S `0x140108060`; H `_SEVL:0` emits six reverse-order fields `(delay, duration, target, unused=0, unused=0, channel)`. Handler discards the two unused fields, takes target/duration/delay, clamps nonpositive duration to 1, bounds channel against the SE vector, and marks pending gain state: active `+0`, flag `+0x80`, sentinel `+0x84=-1`, target `+0x88`, duration `+0x8c`, delay `+0x90`. Returns 0 without attaching a task (`0x140108078–0x140108144`).

[data] Mahoyo has 385 calls; `raw/1_1.chs` file `0x1870` selects channel 5, target 40, duration 2000, delay 0.

## Differences

[code] Switch applies remembered category configuration inherited from SEPL (T `0x6ae04–0x6ae40`); Steam SEPL has no category operand. Switch wraps channels/modifies queued starts, while Steam bounds a vector and writes pending state (T `0x6ad98–0x6ae00`; S `0x140108101–0x140108132`). [inferred] An SEVL following category-070 playback can therefore differ under player settings. Signed-16-bit duration narrowing versus dword storage also limits any claim of exactness (T `0x6ad2c`; S `0x14010812b`). It cannot serve as a voice/SE completion wait.

## Unverified

[inferred] Compare the cited three-second SE ramp and a delayed-start/ramp pair while changing ordinary SE and character-category gains independently. Record onset, target gain, interruption and whether the marker after SEVL runs immediately. This tests the queued-state and settings differences at T `0x6add4–0x6aeac` versus S `0x140108112–0x140108132`.
