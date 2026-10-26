# FCSP

Source spelling: `_FCSP()`.

## Switch build

[code] T `0x8f6a0` parses stop duration in ms (default 0), truncates to 60 Hz frames (`0x8f6e4…0x8f704`), clears saved overlay type `+0x19a9`, and calls `0x83bc0(0,frames)` at `0x8f728`. Zero frames stops immediately through `0x83d08`; nonzero schedules a fade. It sets dirty state and returns zero; it does not wait for that fade or issue audio.

## Steam build

[code] Stock operand-free opcode `0x3a1`, S `0x140106670`, writes the overlay stop request with a fixed 400-ms fade. The task copies it at S `0x14014aa68/0x14014f8f0` and compares elapsed ms at `0x140151d88…0x140151e10`. The stop handlers return without a fade-completion wait. Recipes: `_FCSP:source`.

[data] Mahoyo: 1 call(s) (`raw/wik_q_a.chs`, assembly-dump line 8146).

## Differences

[code] Unpatched operand-free Steam waits 400 ms before the overlay disappears, while Switch clears at once. [inferred] For positive durations, Steam ms versus Switch frame quantization can still produce sub-frame boundary differences.

## Unverified

[inferred] Runtime fade timing and skip/load behavior remain unmeasured. In scr0189 after PGST(149), record FCLN followed by FCSP(2000), then an operand-free stop; compare two-second fade and immediate disappearance against Switch.
