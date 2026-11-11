# Animation tables

## Overview

The engine's animation evaluator reads a table of sprite animations. Each animation has a duration, a frame range into a shared keyframe array, and loop and easing flags. A keyframe is 64 bytes and drives the sprite's attributes over time. The Switch copy lives in the main NSO; the Mahoyo copy is in `WoH.exe`. Both store the same layout. The `hunex` crates do not read this table yet; this note records the engine behaviour for future tools.

Addresses use T for an offset in the Switch main module (Tsukihime) and S for a virtual address in Mahoyo's `WoH.exe`.

## Layout

Little-endian throughout.

- One i32 duration per animation.
- A frame range: one i32 end per animation, cumulative and increasing. Animation 1 starts at frame 0, and each later animation starts at the previous end.
- A loop flag and an easing flag per animation.
- Frames are 64 bytes: an i32 time value, then 30 i16 fields. The frame is 32 i16 slots; slots 0 and 1 form the time, and field n for n from 1 to 30 starts at slot n+1. Field 1 is the first i16 after the time.

## Sentinels

| i16 value | 32-bit value | Meaning |
|---|---|---|
| -32767 (`0x8001`) | -100001 | Unspecified |
| -32766 (`0x8002`) | -100000 | No key for this field |

[code] An unspecified field interpolates from the last explicit value. The walk back passes over -32767, and the segment's start time stays at the frame that held -32767.

[code] A field with -32766 has no key. Both engines skip that frame when they look for the previous and next values of the field, so the value comes from the surrounding keys. The 32-bit fields, time and rotation, use -100001 for unspecified and -100000 for no key.

## Easing and interpolation mode

[code] Field 1 is the easing type and field 2 is the interpolation mode: 0 linear, 1 cubic, 2 hold.

[code] Switch selects easing at T `0x950b8`-`0x950fc`. It walks back from the next key and tests each candidate in two steps: it skips a candidate whose evaluated field is -32766, then skips a frame whose field 1 is -32767 or -32766, and uses the first value that survives. Mahoyo selects easing at S `0x1400a4149`-`0x1400a4199` and skips only frames whose field 1 is a sentinel. When a key has no value for the animated field, Mahoyo applies that key's own easing to the skipped field, so the value moves mid-segment. The Switch rule is the one to reproduce.

[code] Interpolation mode at T `0x951b4`-`0x951e0` and S `0x1400a42b1`-`0x1400a42da` walks back over -32767 and -32766 in field 2 only. The two walks are identical.

[code] Switch forces the interpolation mode to hold for fields 15 and 17 through 27. Mahoyo's bitmap at S `0x1400a427f` (`0x73e8000`) covers 15, 17 through 21 and 24 through 26, so fields 22, 23 and 27 differ. The visible effect of the difference is unverified.

## Field use

[data] Tsukihime's table has 24,192 animations. 1,515 of them use -32766, across 6,057 keyed fields.

[code] Fields 4 and 5 hold the i16 x and y indices, field 6 is opacity, and fields 12 and 13 are zoom X and Y.

## What the game accepts

Not applicable: no tool writes this table.

## Known unknowns

- The visible effect of the hold-mask difference in fields 22, 23 and 27.
- The meaning of the remaining fields.
