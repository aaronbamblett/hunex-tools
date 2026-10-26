# FADE

Source spelling: `_FADE(40,)`.

## Switch build

[code] T dispatch `0x78f64` calls `0x5edb0`. For the corpus types 40/41, case 40 at `0x5f088` saves message-state mode `+0x898=4`, operand1+127 at `+0x8c0`, and preset operand2 at `+0x8c2`; case 41 at `0x5f0c0` clears saved mode and selects global mode 5. Global mode/intensity live at `0x9e530c/0x9e5310`. Update `0x30a30` ramps intensity by 1/60 per update, and draw `0x30dc0` applies an animated sine distortion. These cases do not wait for the ramp.

[data] All 61 authored uses are types 40/41: 28 turn-on and 33 turn-off calls; first operands beyond type include negatives and presets (`dump/scr*.txt:1`). [inferred] Exact preset appearance is not established by the stored fields. The inspected effect is visual, not an audio fade.

## Steam build

[code] There is no FADE recipe/opcode in H `recipes_data.rs`' command inventory; FADS is separately compiled to `0x200` at `_FADS:0`. Prior comparison finds Mahoyo Switch FADE `0x3faf0` only handles types up to 35; it supplies no counterpart for Tsukihime types 40/41.

[data] No matching Steam opcode can be counted in raw CHS. The superficially similar FADS exists, e.g. `raw/1_0.chs@0x2fc`, but its handler S `0x1400ff5f0` creates a transition task rather than a persistent wave.

## Differences

[inferred] Replacing FADE with FADS would introduce unrelated transition behavior and a resource-dependent yield (S `0x1400ff8a1`). Omitting the wave changes the scene's visual meaning; timing alone cannot restore it (T `0x30dc0`).

## Unverified

[inferred] Compare scr0001's wave-on at statement 1332 and its following wave-off against Switch over 60 updates, observing distortion intensity and unchanged next-statement timing (T `0x30a30`, `0x30dc0`).
